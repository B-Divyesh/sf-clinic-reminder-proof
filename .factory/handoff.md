# Verification 14 handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-14`
Candidate: `e16e61c4c300fe88b9b2705e890127566f89ca28`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL

The candidate must not be released. Its local build and full automated suite pass, but the mandatory live deployment verification fails: Azure reports the candidate traffic revision with `maxReplicas: 3` and no `/durable` or `/backups` Azure Files mount. Public `/health` and the footer still identify prior build `c2b1ace…`, not candidate `e16e61c…`.

See [verification-14.md](verification-14.md) for exact evidence, passing checks, defects, and the required recheck command.

## Prior repair record (superseded by Verification 14)

QA13-01 is repaired, deployed, and verified. Production revision `sf-clinic-reminder-proof--0000043` is healthy, runs the exact `c2b1ace…` image, owns 100% of traffic, has one replica, and mounts both required Azure Files shares. No real patient data, messaging-provider credential, payment, or clinic account was used.

## Finding reproduction and repair

Before repair, `EXPECTED_BUILD_SHA=4ee36… npm run verify:deployment` failed with `maximum replica count; expected 1, got 3`. Azure reported healthy durable revision `0000040` at 0% traffic and unhealthy image-only revision `0000041` at 100%. The deploy command stopped after finding a healthy revision, so it could report success without proving traffic convergence.

- Added `inspectRollout`, which requires the exact image to be Azure's latest healthy revision and its only active 100% traffic target.
- Changed the deploy command to require single-revision mode and wait for Azure's automatic promotion before reporting success.
- The deploy command now validates the checked-in topology against both the app template and the promoted revision.
- The production verifier now checks the serving revision's 100% weight, image, replica count, volumes, and mounts instead of relying only on the current app template.
- Added `@regression:qa13-01` with the exact verifier state: healthy candidate at 0% and unhealthy non-durable revision at 100%. It fails convergence until traffic moves to the candidate.
- Removed the duplicate `deploy:container` package script and updated deployment documentation and the copy audit.

## Deployment evidence

- ACR build `ch13n` built the `.git`-excluded source and pushed `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:c2b1aced3ed7e5585d9db4eb73ffff495d1874e0`.
- Image digest: `sha256:eddce92ec58ae0735627d9b0aa43be6bd12dbeddbc2c2b193ba768d519259e08`.
- The guarded rollout created healthy revision `sf-clinic-reminder-proof--0000043`.
- `EXPECTED_BUILD_SHA=c2b1aced3ed7e5585d9db4eb73ffff495d1874e0 npm run verify:deployment` passed: one traffic-bearing revision at 100%, one replica, both Azure Files mounts, exact live build identity, and demo-create statuses `200, 200, 200, 200, 200, 429` with `Retry-After: 3599`.

## Verification

- Clean install: `npm ci` installed 87 locked packages; audit reported 0 vulnerabilities.
- Complete local suite: `npm test` passed 12 Vitest contracts, 34 Rust tests, and 40 Chromium tests.
- Claims: all 31 literal commands in `.factory/claims.json` passed independently; the topology claim included the live deployment verifier.
- Type and lint: `npm run check` passed Svelte diagnostics with 0 errors and 0 warnings, rustfmt, and Clippy with warnings denied.
- Production build: `npm run build` emitted `dist/` and `target/release/reminder-proof-api`; initial JS was 28.63 KB gzip and CSS was 5.54 KB gzip.
- Runtime default: the release binary started with only `PORT=18081`; `/health` returned `200` and startup logged a generated local key without printing it.
- Container: ACR build `ch13n` passed the multi-stage, non-root Dockerfile using `rust:1-slim` without Git metadata.
- Live browser: `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e` passed all 40 desktop, 390 px mobile, 200% text, keyboard, reduced-motion, privacy, offline-read, routing, response-policy, and recovery journeys.
- Accessibility and console: the Playwright Axe audit found zero serious or critical issues across seven public routes and both themes. `verify-url.sh` found `lang=en`, one H1, a main landmark, complete alt/button names, and zero console errors. Evidence is in [`qa-artifacts/repair-9/live`](qa-artifacts/repair-9/live/verify.json).
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.4 s, LCP 1.5 s, TBT 80 ms, CLS 0.001.
- Privacy: live landing and demo journeys made only same-origin runtime requests. No analytics, CDN font, messaging-provider, or clinic connection loaded in the public flow.
- Identity: the live sign-in action reached the Sociobot CIAM tenant and correct client ID with authorization code, S256 PKCE, and `https://clinic-reminder-proof.sociobot.in/auth/callback`. OIDC discovery returned the tenant issuer and JWKS URI. Anonymous clinic access returned `401`, `WWW-Authenticate: Bearer`, and a request ID.
- Response policy: live headers include CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP. Hashed assets are immutable.
- Load smoke: 100 concurrent `/health` requests returned 100×200 in 0.332 seconds (301 requests/second observed).
- Package/consumer and service-worker update checks do not apply: this remains a containerized web service, not a package or installable PWA. Its read-only offline state is covered by the browser suite.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
EXPECTED_BUILD_SHA=c2b1aced3ed7e5585d9db4eb73ffff495d1874e0 npm run verify:deployment
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

## Known limits and operator notes

No QA13 release blocker remains. Continue normal BAA/DPA, retention, consent, jurisdictional messaging, and provider-credential review before accepting real clinic data. The CIAM entry point and redirect were verified without using a real clinic identity.
