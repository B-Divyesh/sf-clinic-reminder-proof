# Repair handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-repair-13`
Verifier report: `f4cd35314c2f641235e121a0133d38c1ec6ccb51` / `verification-17.md`
Reported candidate: `3a341b7d34f6e734791ec37596295cda193374ed`
Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: repaired, deployed, and verified

Both release blockers are closed. Production serves the exact commit that
contains this handoff. The public health response and frontend footer match
that full `git rev-parse HEAD` identity. Azure has one healthy traffic-bearing
revision at 100%, exactly one live replica, and separate Azure Files mounts at
`/durable` and `/backups`.

## Reproduced findings and root causes

- **QA17-01:** `npm run verify:deployment:current` failed before repair with
  `deployment topology must set minReplicas and maxReplicas to 1`. Azure showed
  unhealthy revision `0000051` at declared 100% traffic with `maxReplicas: 3`
  and no volumes or mounts. Its short image tag was `3a341b7d34f6`.
- **QA17-02:** public `/health` returned
  `2ec97d29b07279e15efb5e82caf002ffe63765e1`; Azure was falling back to healthy
  revision `0000050` because the requested candidate could not start.
- The immediate cause was an image-only factory rollout that replaced the safe
  revision template. A second identity gap let deployment verification derive
  the newest selected source-file commit instead of the exact candidate HEAD.
  A later handoff commit could therefore become the verifier candidate while
  the repository still accepted an older runtime identity.

## Repairs

- Candidate identity now resolves from exact `HEAD`, including handoff changes.
- The rollout refuses uncommitted tracked changes and requires exact HEAD to be
  published at `origin/main` before it can touch Azure.
- The image tag must still be the same full 40-character candidate SHA.
- The existing rollout continues to replace unsafe scale, volume, and mount
  fields from `deployment/containerapp.json`; it waits for the exact healthy
  revision, sole 100% traffic, one replica, and matching public identities.
- The production runtime still refuses to start without both durable mounts.
- README, operations guidance, and the plain-language copy audit now document
  the commit → push → full-tag build → topology-aware deploy order.
- The researched brief, UI, API, storage model, demo sandbox, claims, visual
  system, privacy behavior, authentication, and container deployment class are
  unchanged.

## Exact regression coverage

- `@regression:qa17-01` asserts that deployment rejects a dirty final handoff
  and a candidate not yet published to `origin/main`.
- `@regression:qa17-02` recreates verifier revision `0000051`: short tag,
  unhealthy state, 100% declared traffic, three-replica ceiling, no mounts, and
  old public identity. It proves this state is rejected and that the checked-in
  rollout restores one replica and both Azure Files mounts.
- Existing QA12–V16 coverage still proves topology replacement, sole-traffic
  convergence, full tags, public health/footer identity, current-candidate
  derivation, and mismatched-image rejection.

## Verification evidence

| Check | Result |
| --- | --- |
| Clean install | `npm ci` — 87 packages installed; 0 vulnerabilities |
| Full suite | `npm test` — 18 Vitest, 34 Rust, and 40 Chromium tests passed |
| Claim gate | `CI=1 npm run test:e2e -- --grep @claim:` — 31 passed; all three named Rust claim commands passed |
| Type / format / lint | `npm run check` — Svelte 0 errors/0 warnings; rustfmt and Clippy with warnings denied passed |
| Production build | `npm run build` produced `dist/` and `target/release/reminder-proof-api` |
| Bundle budget | Entry JS 28.63 KB gzip; lazy auth JS 68.23 KB gzip; CSS 5.54 KB gzip; fonts 86.0 KB emitted |
| Default runtime | Release binary started with only `PORT=18081`, generated its local key, returned health and metrics, and shut down cleanly |
| Load smoke | 100 concurrent `/health` requests returned 100 × 200 |
| Container build | Factory ACR remote build passed from the Git-free Docker context using the exact full HEAD for all three build identity arguments |
| Deployment | `npm run deploy:container` passed for the full HEAD image; only the matching healthy revision has traffic, with one replica and both mounts |
| Deployment claim | `npm run verify:deployment:current` passed; public health/footer matched HEAD and demo creation returned `200,200,200,200,200,429` with positive `Retry-After` |
| Live browser | 40/40 Chromium tests passed against production, covering desktop, 390 px, 200% text, keyboard, reduced motion, deep links, 404, errors, and offline read-only behavior |
| Accessibility | Factory `verify-url.sh` passed; Playwright Axe found no serious/critical issue on every public route; title, `lang`, one H1, main, names, alt text, focus, and 44 px targets passed |
| Privacy / policy | Same-origin/no-tracking flow, self-hosted fonts, CSP/HSTS/nosniff/referrer/permissions/COOP, body/content-type limits, request IDs, auth challenges, rate limits, and cache policy passed |
| Live identity | The sign-in redirect used the required Sociobot CIAM authority, tenant, client, authorization-code flow, PKCE S256, scopes, and production callback |
| Lighthouse mobile | Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1,351 ms; LCP 1,411 ms; TBT 35 ms; CLS 0.0011 |

This product is a web service, not a package or CLI, so consumer-package checks
do not apply. It intentionally has no service worker or offline-reload claim.
The tested offline behavior is a timestamped read-only ledger with writes
disabled.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

For later releases, finish and commit the handoff first, push `main`, build an
image tagged with the full HEAD SHA, and run:

```sh
npm run deploy:container -- --image sociobotregistry.azurecr.io/sf-clinic-reminder-proof:<full-HEAD-SHA>
```

## Known gaps / operator action

- Docker is unavailable in this worker, so the container was built by Azure
  Container Registry. That is the same Git-free remote build path used for
  production.
- No real clinic account or payment was used. Fixture coverage passed, and the
  public pre-auth redirect was checked without completing sign-in.
- Confirm that
  `https://clinic-reminder-proof.sociobot.in/auth/callback` remains registered
  on the shared Sociobot Entra SPA before inviting clinics.
