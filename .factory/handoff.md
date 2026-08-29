# Repair 8 handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-repair-8`
Base verifier report: [verification-12.md](verification-12.md)
Repair commits: `d24ff915eae5bdcae3f308fc79b9919a00cbb9d7` and `4ee36ffaa1496e94ebdb0f0b0fc17b907f824372`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status: PASS

QA12-01 is repaired, deployed, and verified on the public production URL. The public build is the exact repair commit and it runs one healthy replica with the two required Azure Files mounts. No real clinic, patient, payment, or provider credential was used during repair or verification.

## Repair

The candidate was deployed by an image-only Container Apps update. That replaced the revision template with `maxReplicas: 3` and no volumes, so the production mount guard safely rejected startup.

- Added `scripts/containerapp-topology.mjs`. It validates the checked-in one-replica topology and composes its two Azure Files volumes and `/durable` plus `/backups` mounts into every image rollout.
- Added `npm run deploy:container -- --image <registry/image:tag>`. It PATCHes only the fully composed revision template, retains factory-managed app settings, waits for the exact image to become healthy, and fails on timeout.
- Strengthened `npm run verify:deployment`: it evaluates the sole traffic-bearing revision instead of inactive zero-traffic history, checks the exact expected image tag when `EXPECTED_BUILD_SHA` is set, validates both mounts and the live build identity, then proves the sixth same-client demo creation returns `429` with `Retry-After`.
- Added `@regression:qa12-01`. Starting from the verifier’s broken template shape (`maxReplicas: 3`, no volumes, candidate image), it asserts the generated ARM patch restores exactly one replica, both named Azure Files shares, both mounts, and preserves the container environment/resources. The regression also verifies `.dockerignore` excludes `.git`.

## Deployment evidence

- ACR build `ch128` built a `.git`-excluded 185.653 KB source archive and pushed `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:4ee36ffaa1496e94ebdb0f0b0fc17b907f824372` (digest `sha256:0d9534efd2e4eb5e47b54caf899810c0317ec9340375ff45cb8b75c4cbcfd44e`).
- The guarded deploy created healthy revision `sf-clinic-reminder-proof--0000040`.
- `EXPECTED_BUILD_SHA=4ee36ffaa1496e94ebdb0f0b0fc17b907f824372 npm run verify:deployment` passed: one replica; both Azure Files mounts; live `/health` build identity `4ee36ffaa1496e94ebdb0f0b0fc17b907f824372`; rate-limit sequence `200, 200, 200, 200, 200, 429`; `Retry-After: 3599`.

## Verification

- Clean install: `npm ci` installed 87 locked packages; audit reported 0 vulnerabilities.
- Complete local suite: `npm test` passed 11 Vitest contracts, 34 Rust tests, and 40 Chromium tests.
- All 31 commands in `.factory/claims.json` passed independently. The final topology claim ran its declared browser assertion and the live deployment verifier against revision `0000040`.
- `npm run check` passed with Svelte 0 errors/0 warnings, rustfmt clean, and Clippy warnings denied.
- `npm run build` emitted `dist/` and `target/release/reminder-proof-api`. Public entry JS is 28.63 KB gzip and CSS is 5.54 KB gzip.
- Release-binary smoke: with only `PORT=4811`, `/health` returned `200 {"status":"ok","build_sha":"dev"}` and startup generated its local key without exposing it.
- Container build: ACR `ch128` passed the multi-stage, non-root Dockerfile using `rust:1-slim` and no Git metadata.
- Browser: `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e` passed all 40 desktop, 390 px mobile, keyboard, reduced-motion, privacy, offline-read, routing, response-policy, and API recovery tests.
- Accessibility: the Playwright Axe audit passed seven public routes with zero serious or critical findings. `/opt/fleet/lib/verify-url.sh` passed live with title, `lang=en`, one H1, main landmark, alt coverage, labels, and zero console errors.
- Privacy and identity: live demo requests remained same-origin; no analytics or third-party runtime loaded. The live sign-in button reached the required Sociobot Entra `/oauth2/v2.0/authorize` authority with client `25c704f4-465a-47af-80ab-2c489466b697`, PKCE, and redirect URI `https://clinic-reminder-proof.sociobot.in/auth/callback`.
- Live headers include CSP response-header `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP.
- Live mobile Lighthouse: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.4 s, LCP 1.4 s, TBT 40 ms, CLS 0.001.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build

# Build the commit image with the factory ACR workflow, then:
npm run deploy:container -- --image sociobotregistry.azurecr.io/sf-clinic-reminder-proof:<full-commit>
EXPECTED_BUILD_SHA=<full-commit> npm run verify:deployment
```

## Known limits and next steps

- Reminder Proof handles operational reminder data, not clinical notes. Continue normal BAA/DPA, consent, jurisdictional messaging, retention, and provider-credential review before onboarding real clinics.
- This product is a web service, not a package, CLI, or PWA. Package-consumer and service-worker update checks do not apply; the browser offline-read path is covered by the executable browser suite.
