# Repair 10 handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-repair-10`
Base verifier report: [verification-14.md](verification-14.md)
Rejected candidate: `e16e61c4c300fe88b9b2705e890127566f89ca28`
Repair source commits: `58b60f33d40fb7b6355ab036d49b7b6e863244a7`, `e8311677822d4a60183b9efcd5aab8980fc2b200`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status: PASS

QA14-01 and QA14-02 are repaired and deployed. Production revision
`sf-clinic-reminder-proof--0000046` is healthy, is the only 100% traffic
target, runs one replica, has both required Azure Files mounts, and serves the
exact full `e8311677822d4a60183b9efcd5aab8980fc2b200` build identity through
both `/health` and the public application bundle.

## Reproduction and root cause

At the start of this repair, Azure reported the verifier's image-only
revision `sf-clinic-reminder-proof--0000044` at 100% traffic with
`maxReplicas: 3` and no volumes or mounts. It was unhealthy. Public `/health`
and the rendered footer still reported the earlier healthy `c2b1ace…` build.

The checked-in topology was correct. The failure was an image-only Container
Apps update that bypassed `npm run deploy:container`, replacing the revision
template and leaving Azure traffic metadata different from the running build.

## Repair

- Added immutable deployment identity helpers. Every guarded image rollout now
  requires a full 40-character commit tag; a short tag such as the verifier's
  `e16e61c4c300` is rejected before Azure is changed.
- The deploy command still composes the checked-in single-replica topology,
  then waits for one healthy 100% Azure revision and matching public health
  and client-bundle identities before succeeding.
- The production verifier now requires `EXPECTED_BUILD_SHA`, validates that
  exact image tag, `/health`, and the client bundle which renders the footer
  identity, in addition to topology, traffic, replica, and rate-limit checks.
- Added exact regression tests:
  - `@regression:qa14-01` rejects the verifier's short image tag.
  - `@regression:qa14-02` reproduces stale `c2b1ace…` health/footer identity
    against the verifier's `e16e61c…` candidate and rejects either mismatch.
- Updated the topology claim so its live verifier uses the deployed immutable
  repair SHA. Updated deployment and recovery documentation and its copy audit.

## Deployment evidence

- ACR build `ch14u` built a `.git`-excluded source archive and published
  `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:e8311677822d4a60183b9efcd5aab8980fc2b200`.
- Image digest: `sha256:f5f0263f9457095491467e45c93516cb8fc98ecf619e7b1d43805c0846552596`.
- `npm run deploy:container -- --image …:e831167…` created healthy revision
  `sf-clinic-reminder-proof--0000046` and waited for it to own 100% traffic.
- `EXPECTED_BUILD_SHA=e8311677822d4a60183b9efcd5aab8980fc2b200 npm run verify:deployment`
  passed: exact image, exact public build identity, one replica, both mounts,
  and demo-create statuses `200, 200, 200, 200, 200, 429` with
  `Retry-After: 3599`.

## Verification

- Clean install: `npm ci` installed 87 locked packages; audit reported zero
  vulnerabilities.
- Complete local suite: `npm test` passed 14 Vitest contracts, 34 Rust tests,
  and 40 Chromium browser journeys.
- Claims: all 31 literal commands in `.factory/claims.json` were executed in
  manifest order. The final topology claim passed against revision `0000046`.
- Type, format, and lint: `npm run check` passed Svelte diagnostics with zero
  errors/warnings, rustfmt, and Clippy with warnings denied.
- Production build: `npm run build` produced `dist/` and
  `target/release/reminder-proof-api`. Public entry JS was 28,235 bytes gzip;
  CSS was 5,553 bytes gzip.
- Container: ACR's multi-stage Docker build completed with `rust:1-slim`, no
  Git metadata, and the non-root runtime image.
- Live browser: `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e`
  passed all 40 desktop, 390 px mobile, 200% text, keyboard, reduced-motion,
  offline-read, privacy, route, response-policy, recovery, and CIAM journeys.
- Accessibility: the in-suite Axe sweep passed on public routes and the
  standalone `@axe-core/cli` scan of `/`, `/demo`, `/privacy`, and `/terms`
  reported zero violations. The browser routes have one H1, language, main,
  landmarks, named controls, alt text, skip-link focus, and no console errors.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.4 s, LCP 1.4 s, TBT 70 ms, CLS 0.001.
- Privacy: the browser suite recorded only same-origin runtime requests in the
  landing/demo flow. No analytics, tracking, CDN font, messaging-provider, or
  clinic connection is loaded publicly.
- Identity and policy: live auth config names only the Sociobot CIAM tenant;
  anonymous `/api/v1/clinic` returns `401`, `WWW-Authenticate: Bearer`, and a
  request ID. CSP includes `frame-ancestors 'none'`; HSTS, nosniff, strict
  referrer policy, permissions policy, and COOP are present.
- Load smoke: 100 concurrent public `/health` requests returned 100×200 in
  2.316 seconds (43.2 requests/second observed).
- Package/consumer and service-worker update checks do not apply: this is a
  containerized web service, not a package or installable PWA. The browser
  suite verifies its supported read-only offline state.

## Run and verify

```sh
npm ci
npm test
npm run check
BUILD_SHA=e8311677822d4a60183b9efcd5aab8980fc2b200 npm run build
EXPECTED_BUILD_SHA=e8311677822d4a60183b9efcd5aab8980fc2b200 npm run verify:deployment
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

## Known limits and operator notes

No QA14 release blocker remains. No real clinic identity, patient data,
messaging-provider credential, or payment was used during repair verification.
Continue normal BAA/DPA, consent, retention, jurisdictional messaging, and
provider-credential review before accepting real clinic data.
