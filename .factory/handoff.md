# Repair handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-repair-11`

Verifier report repaired: `0323da2045e6a159001a43f656cde5a4c78aa861` / `verification-15.md`

## Status: deployed and verified

The requested verifier candidate `ae20862641e90b0a265fc75ab76e5273159e7bef`
was not available from the supplied clone or GitHub remote. The repair is the
published source commit `659284a4195b954267ae7f243c129633963a24f6`, deployed
as the same full immutable image tag.

Production now serves:

- URL: <https://clinic-reminder-proof.sociobot.in>
- Revision: `sf-clinic-reminder-proof--0000048`
- Image: `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:659284a4195b954267ae7f243c129633963a24f6`
- ACR build: `ch15x`; digest `sha256:7bfebb685b06940df26cc065ab825073cc647e107735a70e07a46d691dc1eb7e`
- Public `/health` build SHA: `659284a4195b954267ae7f243c129633963a24f6`
- Traffic: 100% to revision `0000048`; one live replica
- Storage: Azure Files mounted at `/durable` and `/backups`

## What changed

- Replaced the stale `e831167…` deployment-claim value in `.factory/claims.json`.
  The claim now runs `npm run verify:deployment:current`.
- Added `scripts/verify-current-deployment.mjs` and `scripts/source-commit.mjs`.
  They derive the full immutable source revision from the newest Docker build
  input, then run the existing production verifier. An inherited expected SHA
  cannot mask a later runtime change.
- Preserved the checked-in one-replica Azure Files topology. The repair
  rollout used `node scripts/deploy-containerapp.mjs --image <full-SHA-tag>`;
  its dry run proved that it replaces the unsafe image-only template with
  `minReplicas: 1`, `maxReplicas: 1`, and both required mounts.
- Added `@regression:qa15-04` coverage for the stale-SHA failure and for
  resolving the checked-out Docker build input. Existing QA12–14 regressions
  continue to cover mount preservation, traffic convergence, full image tags,
  and public build identity.

## Verification evidence

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 87 packages; 0 vulnerabilities |
| `npm test` | PASS — 15 Vitest, 34 Rust, 40 Chromium tests |
| `npm run check` | PASS — Svelte 0 errors/0 warnings; rustfmt and Clippy clean |
| `npm run build` | PASS — `dist/` and `target/release/reminder-proof-api` produced |
| Container build | PASS — ACR build `ch15x`, `.git` excluded, immutable full-SHA tag/digest above |
| All manifest commands | PASS — all 31 literal claim commands; final command derived `659284a…` and returned 200,200,200,200,200,429 with `Retry-After: 3599` |
| Explicit deployment verifier | PASS — `EXPECTED_BUILD_SHA=659284a4195b954267ae7f243c129633963a24f6 npm run verify:deployment` |
| Live browser suite | PASS — 40/40 Chromium tests against `https://clinic-reminder-proof.sociobot.in` |
| Accessibility | PASS — worker `verify-url.sh`: title/lang/H1/main/alt/button-name and zero console errors; standalone axe-core 4.11.4: 0 violations; Playwright axe light/dark routes: no serious/critical findings |
| Responsive and keyboard | PASS — live 390 px, 200% text, reduced motion, skip link, main focus, touch targets, drawer/undo focus, deep links and 404 all covered by the 40-test suite |
| Privacy and response policy | PASS — same-origin/no-tracking claim; CSP, HSTS, nosniff, referrer, permissions and COOP headers observed; malformed/oversize/content-type/auth/rate-limit response contracts passed |
| Offline/update | PASS where applicable — live read-only offline state and disabled writes covered. This is not a PWA and ships no service worker, so update/offline-reload is not a product claim. |
| Live identity | PASS — Entra authorize request used `sociobotcustomers.ciamlogin.com`, client `25c704f4-465a-47af-80ab-2c489466b697`, code flow, PKCE `S256`, and the expected callback URL |
| Load smoke | PASS — 100 concurrent public `/health` requests returned 100 × 200 |
| Lighthouse mobile | PASS — Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1,448.5 ms; CLS 0.000742 |

The release binary was also launched with only `PORT=18081` among its app
settings. It generated its local configuration, served `/health` and
Prometheus `/metrics`, and did not log a secret value.

## How to run and verify

```sh
npm ci
npm test
npm run check
npm run build
EXPECTED_BUILD_SHA=659284a4195b954267ae7f243c129633963a24f6 npm run verify:deployment
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

For a future image rollout, build and tag the image with the full source
commit, then use `npm run deploy:container -- --image <registry/image:full-commit>`.
The deployment command restores the required topology before accepting the
revision. The manifest claim derives the latest Docker build-input commit, so
handoff-only files excluded by `.dockerignore` do not create a false runtime
identity mismatch.

## Known gaps and next steps

- No release-blocking gaps remain.
- A real clinic sign-up and payment were not performed because this repair
  worker has no clinic identity or payment authority. Fixture integration
  coverage passed; the public demo remains isolated and one-click.
- Before inviting clinics, the factory operator should confirm that
  `https://clinic-reminder-proof.sociobot.in/auth/callback` remains registered
  on the shared Sociobot Entra SPA. The live authorization request used that
  exact callback.
