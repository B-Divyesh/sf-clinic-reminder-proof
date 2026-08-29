# Independent product QA 7 — FAIL

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-7`

Candidate: `a3ec1d2b5a24d9e7a58b53046a1c12b84769d51d`

Live URL: `https://clinic-reminder-proof.sociobot.in`

Result: **FAIL — do not release until the claims contract is repaired.**

The previous deployment-only failure is fixed. Fresh production evidence shows
the exact candidate image, one healthy replica, and both durable Azure Files
mounts. All 21 declared claim commands, the complete local suite, checks,
production build, and 24 live Playwright checks pass. Independent boundary and
copy review nevertheless found release-blocking claims defects.

## Release-blocking findings

### QA7-01 — High — The `request-protection` claim is broader than the product and its test

`.factory/claims.json` promises: “API writes enforce JSON and 16 KB body limits
with structured errors.” Its test covers malformed JSON and an oversized body
only on a demo route. Fresh live boundary probes found:

- Valid JSON sent as `Content-Type: text/plain` returns `415` with
  `text/plain; charset=utf-8` and the unstructured body
  `Expected request with Content-Type: application/json`.
- A 17 KB valid JSON body sent to the protected
  `/api/v1/billing/checkout` write route is accepted by the extractor and
  reaches authentication, returning `401`, not `413`.
- Source confirms the mismatch: demo routes use a 16 KB body layer, clinic and
  billing routes use a 5 MB layer, and error normalization excludes `415`.

The declared command passes, but the observable claim does not. Either scope
the claim explicitly to demo writes or enforce the promised policy on all API
writes. Normalize `415` into the JSON problem shape and extend the claim test
to cover wrong content type and a protected write.

### QA7-02 — High — Public promises are missing from `claims.json`

The claims contract says any public or README claim without a manifest entry
fails review. At least these distinct promises are unlisted:

- Landing page: “Reminder Proof … sends no marketing campaigns.”
- Privacy page: “Signed-in clinics can export or delete their workspace.”
- README: “clinic export/delete” is included as shipped functionality.

No claim ID asserts that managed dispatch cannot send marketing content, or
that an authenticated user can complete export and deletion. Add focused
observable claim tests or remove/narrow those promises.

## Other findings

### QA7-03 — Medium — Error responses promise a request ID that is absent or unusable

The live `401` clinic responses contain
`"request_id":"available-in-response-header"` but have no `X-Request-Id`
header. Malformed demo JSON does include the header, but its value is the
constant `local-request`. This prevents staff from correlating failures with
structured service logs.

### QA7-04 — Low — The implemented theme control does not match the visual thesis

`.factory/design.md` says after-hours mode follows the system and “can be
chosen explicitly.” System light/dark behavior works, but there is no explicit
theme control in the UI.

### QA7-05 — Low — Every rendered route has two description meta elements

The static shell supplies one `<meta name="description">`, and Svelte adds a
second. This occurred on `/`, `/demo`, `/start`, `/app`, `/privacy`, `/terms`,
`/404`, and an unknown route. Titles, canonical URLs, Open Graph fields, one
`h1`, and landmarks are otherwise correct.

## Mandatory first checks

### Claims run

The literal first invocation was made before installing dependencies and all
browser commands failed to start because `@playwright/test` was not yet in
`node_modules`. The Rust half of `managed-auth-storage` passed. After the
required clean install (`npm ci`: 87 packages, 0 vulnerabilities), every exact
command from `.factory/claims.json` was rerun independently.

| Claim ID | Result |
| --- | --- |
| `demo-isolation` | PASS |
| `sample-outcome-coverage` | PASS |
| `consent-channel-guard` | PASS |
| `fallback-order` | PASS |
| `delivery-timeline` | PASS |
| `exception-ownership` | PASS |
| `demo-reset` | PASS |
| `minimal-reminder-content` | PASS |
| `public-price` | PASS |
| `demo-cookie-lifetime` | PASS |
| `demo-replica-continuity` | PASS |
| `no-tracking` | PASS |
| `request-protection` | PASS as written; independent boundary check fails the claim (QA7-01) |
| `rate-limit-policy` | PASS |
| `security-headers` | PASS |
| `build-identity` | PASS |
| `managed-auth-storage` | PASS |
| `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS |
| `managed-storage-recovery` | PASS |
| `single-replica-durable-topology` | PASS |

Exact rerun output: `/tmp/reminder-proof-qa/claims-after-install.log`.

### Cold first-read

PASS. At 1440×900 the first screen says:

- What it does: “See every reminder outcome,” with delivery proof and a clear
  next step when reminders fail.
- Who it is for: independent clinics.
- What to click first: “Try it with sample data,” beside an explanation that
  it opens a sample clinic and touches no real clinic data.

The action opens `/demo` in one click with five realistic appointments and the
persistent sample-data banner. Screenshot:
`/tmp/reminder-proof-qa/first-read-cold.png`.

## Local verification

Environment: Node 22.23.2, npm 10.9.8, rustc/cargo 1.98.0, Playwright 1.58.2.

| Gate | Result |
| --- | --- |
| Clean candidate | PASS — detached exact candidate, initially clean |
| `npm ci` | PASS — 87 packages; 0 vulnerabilities |
| Every claim command | PASS — 21/21 after install |
| `npm test` | PASS — 6 Vitest, 24 Rust, 28 Chromium |
| `npm run check` | PASS — Svelte 0 errors/warnings, rustfmt, clippy `-D warnings` |
| `npm run build` | PASS — `dist/` and release API binary produced |
| Production dependency audit | PASS — 0 known vulnerabilities |

Docker was not installed in the verifier container, so a separate local
`docker build` was unavailable. The exact package production build passed, and
the deployed ACR image tag and live health identity match this candidate.

## End-to-end product evidence

The live demo was exercised in a fresh context, not only through builder tests:

1. One click from `/` created an isolated sample clinic with five appointments.
2. Advancing the four due reminders produced 3 delivered and 1 exception.
3. Jordan L.'s timeline showed `TEMPLATE_REJECTED` on WhatsApp followed by an
   accepted email fallback.
4. Sofia R.'s consent block had no provider attempt. Assignment to Sam Rivera,
   resolution, reload persistence, keyboard focus on Undo, undo, and reset all
   worked.
5. Invalid owner, resolve-before-owner, and missing-reminder requests returned
   actionable JSON `422`, `409`, and `404` responses respectively.
6. Malformed JSON returned structured `400`; 17 KB on the demo endpoint
   returned structured `413`. The wrong-content-type and protected-write
   boundaries fail as described in QA7-01.

Screenshots and captured state:
`/tmp/reminder-proof-qa/live-demo-desktop.png`,
`/tmp/reminder-proof-qa/live-demo-mobile-390-dark-reduced.png`, and
`/tmp/reminder-proof-qa/live-independent-browser.json`.

## Privacy, accessibility, and browser behavior

- The recorded landing-to-demo flow made 17 requests, all to the product
  origin. No analytics, messaging, billing, or account request occurred.
- No browser console errors, page errors, or failed requests occurred in that
  flow. The full live route crawl also passed.
- Live axe checks found zero serious or critical issues on seven public routes
  in both light and dark system themes.
- At 390×844 in dark/reduced-motion mode: no horizontal overflow, no tested
  button/link target below 44 px, Tab exposed a 3 px visible focus ring, Enter
  moved focus to `<main>`, and reduced-motion media matched.
- 200% text reflow, deep links, browser Back, offline demo reads, post-resolution
  focus, and the styled HTTP 404 all passed live.
- All tested routes have `lang=en`, one `h1`, one `main`, header/footer
  landmarks, route-specific titles, and canonical URLs.

## Live backend, deployment, auth, and billing

- `/health` returned
  `a3ec1d2b5a24d9e7a58b53046a1c12b84769d51d`.
- Local/live SHA-256 hashes match for the entry JS and CSS.
- Azure Container Apps reports active revision
  `sf-clinic-reminder-proof--0000025`, image tag `a3ec1d2b5a24`, one healthy
  replica receiving 100% traffic, `minReplicas=1`, `maxReplicas=1`, and Azure
  Files mounts at `/durable` and `/backups`. A fresh container exec reported
  UID/GID 999 and visible writable-style mount permissions.
- Startup logs identify the exact SHA, port 8080, generated local data key, and
  managed storage readiness without printing secrets.
- Live rate limits: 18 concurrent demo creates yielded exactly 5×200 and
  13×429 with `Retry-After: 3599`. A 60-request `/metrics` burst yielded
  40×200 and 20×429 with `Retry-After: 1`. A 60-request protected billing
  burst yielded 42×401 and 18×429 with `Retry-After: 1`. Health is the documented
  exemption.
- Unauthenticated clinic and export routes return 401 with
  `WWW-Authenticate: Bearer`.
- Sign-in redirects only to the required
  `sociobotcustomers.ciamlogin.com` tenant, exact tenant/client IDs, PKCE, and
  the production callback. The page offers “No account? Create one.”
- The Sociobot product checkout returns 303 to hosted Dodo checkout; the page
  displays “Reminder Proof — $79.00 / Month.” No purchase was submitted.
- Authenticated clinic/provider dispatch could not be exercised against real
  patient/provider accounts without clinic credentials and consent. The
  isolated Rust integration tests cover that boundary with signed fixtures.

## Headers, caching, and performance

- Root and API responses include CSP, HSTS, `nosniff`, referrer policy,
  permissions policy, COOP, and `Cache-Control: no-cache`.
- Hashed assets use `public, max-age=31536000, immutable`.
- Initial entry JS: 80.13 KB raw / 27.92 KB gzip. Lazy MSAL chunk: 271.99 KB
  raw / 68.23 KB gzip. CSS: 24.80 KB raw / 5.38 KB gzip. Fonts remain under
  the 120 KB budget.
- Fresh mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 1.2 s, LCP 1.2 s, TBT 80 ms, CLS 0, Speed Index 1.2 s,
  total transfer 58 KiB.

## Acceptance decision

**FAIL.** The deployment repair is proven and the usable product paths are in
good shape, but QA7-01 and QA7-02 violate the explicit claims acceptance
contract. No product source code was changed during this verification.
