# Repair handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-repair-12`

Verifier report repaired: `1e46bd6dd5e6c0af4c28c690cc30875db16c02de` / `.factory/verification-16.md`

Original candidate: `4a23898a51958f530909bfe9b3e86678403ad86d`

Repair source: `2ec97d29b07279e15efb5e82caf002ffe63765e1`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: deployed and verified

Both V16 release blockers are repaired. The live revision is
`sf-clinic-reminder-proof--0000050`, the only active traffic target at 100%,
with one replica and both required Azure Files mounts. Public `/health` and
the rendered footer identify the repair source `2ec97d29…`.

| Deployment evidence | Result |
| --- | --- |
| ACR build | `ch170` succeeded from the Git-free source archive |
| Image | `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:2ec97d29b07279e15efb5e82caf002ffe63765e1` |
| Digest | `sha256:3d7df720940fa8348ef26569ff67e3251ff606812e0b3af649091cf5559ffac1` |
| Revision / traffic | `sf-clinic-reminder-proof--0000050`, healthy, 100%, one replica |
| Durable topology | `minReplicas: 1`, `maxReplicas: 1`; `clinic-data` at `/durable`; `clinic-backups` at `/backups` |
| Public identity | `/health` returns `2ec97d29b07279e15efb5e82caf002ffe63765e1`; production verifier also checked the public footer bundle |

## What changed

- Repaired the failed deployment by building the repair with a full immutable
  40-character tag and deploying it through `scripts/deploy-containerapp.mjs`.
  The prior unhealthy short-tag update had bypassed the topology-aware release
  path, replacing the revision template with `maxReplicas: 3` and no volumes.
- Expanded the release identity inputs in `scripts/source-commit.mjs` to cover
  the Azure topology and deployment/verification safety scripts, while still
  excluding documentation-only `.factory` updates.
- Made `scripts/deploy-containerapp.mjs` reject an image tag unless its full
  SHA matches the checked-out release identity. A different full historical
  image can no longer be promoted as the current candidate.
- Added `@regression:v16-01` coverage for that mismatch and extended the
  QA15 identity coverage to include the deployment template and command.
- Preserved the product behavior, researched brief, visual system, data model,
  demo, privacy posture, and deployment class.

## Verification evidence

| Check | Result |
| --- | --- |
| Clean install | `npm ci` — 87 packages, 0 vulnerabilities |
| Unit/integration/browser suite | `npm test` — 16 Vitest, 34 Rust, 40 Chromium tests passed |
| Type / format / lint | `npm run check` — Svelte 0 errors/0 warnings; rustfmt and Clippy with warnings denied passed |
| Production build | `npm run build` produced `dist/` and `target/release/reminder-proof-api` |
| Regression | `@regression:v16-01` passed; it rejects a non-candidate full image tag before rollout |
| Claim contract | All 31 literal `.factory/claims.json` commands passed from the clean checkout |
| Deployment verifier | `npm run verify:deployment:current` passed: exact source/image/footer identity; sole healthy 100% revision; both mounts; one replica; demo creation `200,200,200,200,200,429`; `Retry-After: 3599` |
| Deployment dry run | Composed template contains both Azure Files volumes/mounts and `minReplicas=maxReplicas=1` |
| Live browser suite | `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e` — 40/40 Chromium tests passed |
| Accessibility | Factory `verify-url.sh` passed: 200, title, `lang=en`, one H1, main, no missing image alt, no unnamed buttons, zero console/page errors. Playwright Axe found no serious/critical violations on all public routes. Standalone Axe CLI could not locate a system Chrome in this container, so the project’s Playwright Axe integration was used as the documented alternative. |
| Desktop / mobile / keyboard | Live suite covered desktop and 390 px, 200% text, visible skip-link focus, keyboard resolution focus, 44 px targets, deep links, 404, reduced motion, and offline read-only behavior |
| Privacy / response policy | Live suite covered same-origin/no-tracking requests, self-hosted fonts, JSON body/content-type limits, error request IDs, `WWW-Authenticate`, rate limits, CSP/HSTS/nosniff/referrer/permissions/COOP headers, and immutable hashed assets |
| Live identity | The real sign-in redirect used `sociobotcustomers.ciamlogin.com`, tenant `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client `25c704f4-465a-47af-80ab-2c489466b697`, authorization code + PKCE `S256`, scopes `openid profile email`, and `https://clinic-reminder-proof.sociobot.in/auth/callback` |
| Performance | Lighthouse live: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1,431 ms; CLS 0.000742 |
| Bundle budget | Public entry JS 28,358 bytes gzip; lazy auth chunk 67,778 bytes gzip; CSS 5,535 bytes gzip; local WOFF2 total 66,539 bytes gzip |

This is a web service, not a library or CLI, so package-consumer checks do not
apply. It intentionally has no service worker or offline-reload claim; its
supported offline behavior is the tested read-only ledger state with writes
disabled.

## How to run and verify

```sh
npm ci
npm test
npm run check
npm run build
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

For a release, build the full 40-character source SHA into ACR with
`BUILD_SHA`, `GIT_SHA`, and `SOURCE_COMMIT` set to that SHA, then run:

```sh
npm run deploy:container -- --image sociobotregistry.azurecr.io/sf-clinic-reminder-proof:<full-source-sha>
```

The command refuses a mismatched or short tag and waits for the matching
healthy, single-traffic revision before succeeding.

## Known gaps / next steps

- No release-blocking product or deployment gaps remain.
- A real clinic account, payment, and completed Entra login were not performed
  because this worker has no clinic identity or payment authority. Fixture
  coverage and the live pre-auth CIAM redirect passed.
- Before inviting clinics, an operator should confirm that
  `https://clinic-reminder-proof.sociobot.in/auth/callback` remains registered
  on the shared Sociobot Entra SPA.
