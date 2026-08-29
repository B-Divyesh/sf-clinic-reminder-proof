# Independent product verification 18 — FAIL

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-18`
Candidate: `36d39a8d57aa77e3d8131b5e0359d22d9519883e`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release or accept real clinic data.** Candidate code is locally
healthy and the public fallback currently serves that exact candidate, but the
Azure Container App's declared 100% traffic revision is unhealthy. Its short
image tag, three-replica ceiling, and missing durable/backup mounts violate the
backend topology and make the mandatory deployment claim fail.

This is fresh evidence. It confirms a deployment-only release blocker rather
than a functional defect in the candidate's locally tested core workflow.

## Mandatory first gates

### Claims gate: FAIL

`.factory/claims.json` exists and contains 31 claims. After `npm ci`, I ran each
entry's exact `test` command separately from this clean candidate checkout.
Thirty passed. The final composite claim passed its Playwright portion, then
failed in `npm run verify:deployment:current`:

```text
Error: deployment topology must set minReplicas and maxReplicas to 1
```

| Claim ID | Exact manifest test result |
| --- | --- |
| `demo-isolation` | PASS |
| `sample-outcome-coverage` | PASS |
| `consent-channel-guard` | PASS |
| `fallback-order` | PASS |
| `delivery-timeline` | PASS |
| `exception-ownership` | PASS |
| `sample-exception-visibility` | PASS |
| `demo-reset` | PASS |
| `minimal-reminder-content` | PASS |
| `public-price` | PASS |
| `demo-cookie-lifetime` | PASS |
| `demo-replica-continuity` | PASS |
| `no-tracking` | PASS |
| `explicit-theme-choice` | PASS |
| `request-protection` | PASS |
| `rate-limit-policy` | PASS |
| `security-headers` | PASS |
| `build-identity` | PASS |
| `managed-auth-storage` | PASS, including its named Rust test |
| `signed-calendar-intake` | PASS |
| `approved-whatsapp-dispatch` | PASS |
| `twilio-receipt-verification` | PASS |
| `resend-receipt-verification` | PASS |
| `managed-secret-encryption` | PASS |
| `managed-data-minimisation` | PASS |
| `no-marketing-campaigns` | PASS, including its named Rust test |
| `signed-in-export-delete` | PASS, including its named Rust test |
| `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS |
| `managed-storage-recovery` | PASS |
| `single-replica-durable-topology` | **FAIL**: local topology assertion passed; required live deployment verification failed |

A required claim failure is release-blocking regardless of the other results.

### First-read and one-click demo: PASS

Cold desktop and 390 px mobile visits answer all three required questions on
the first screen:

- What it does: **“See every reminder outcome.”**
- Who it is for: **independent clinics** needing delivery proof and a next step
  when reminders fail.
- What to click first: **“Try it with sample data.”** The adjacent sentence says
  it opens a sample clinic and does not touch real clinic data.

One click opened `/demo`, immediately showed five fictional appointments, and
kept the “Demo — sample data, nothing is saved” banner, Reset demo, and Start
for real visible. This gate passes.

Evidence:

- `qa-artifacts/verification-18-live-cold-desktop.png`
- `qa-artifacts/verification-18-live-cold-mobile.png`
- `qa-artifacts/verification-18-live-demo-desktop.png`
- `qa-artifacts/verification-18-live-demo-mobile.png`

## Defects by severity

| Severity | ID | Finding |
| --- | --- | --- |
| Critical / release blocker | QA18-01 | Azure declares unhealthy revision `sf-clinic-reminder-proof--0000053` at 100% traffic. It uses short tag `36d39a8d57aa`, has `maxReplicas: 3`, and has no `/durable` or `/backups` volumes or mounts. `npm run verify:deployment:current` and the required topology claim fail. Public requests work only because Azure falls back to healthy revision `0000052`. |
| Medium | QA18-02 | The live Playwright suite is not repeatable within the one-hour demo-create window. `@claim:rate-limit-policy` derives predictable `198.18.*` client keys, so a later clean browser run reused an exhausted server bucket and received six immediate 429s instead of five 200s then one 429. A newly randomized client independently produced the documented `200,200,200,200,200,429` result. The production behavior is correct; the live-test client identity is not isolated between runs. |

No other independently reproducible product defect was found.

## Deployment and build identity evidence

Public health currently returns the requested candidate:

```json
{"status":"ok","build_sha":"36d39a8d57aa77e3d8131b5e0359d22d9519883e"}
```

The footer renders `Build 36d39a8`. Building the web app with the full candidate
in `BUILD_SHA` produced `index-BVczyEci.js`, and that file matched the live JS
byte-for-byte.

Read-only Azure inspection found:

| Revision | Image | Health / declared traffic | Topology |
| --- | --- | --- | --- |
| `sf-clinic-reminder-proof--0000052` | full candidate SHA | Healthy / 0% | one replica; both Azure Files volumes mounted at `/durable` and `/backups` |
| `sf-clinic-reminder-proof--0000053` | short `36d39a8d57aa` tag | **Unhealthy / 100%** | `minReplicas: 1`, **`maxReplicas: 3`**, no volumes, no mounts |

The live body matches the candidate, but the control plane does not satisfy the
candidate's deployment contract. A fallback is not an acceptable release.

## Local candidate gates

| Check | Result | Evidence |
| --- | --- | --- |
| Checkout identity | PASS | `HEAD` and `origin/main` both equal the full candidate SHA. The checkout was clean before QA artifacts were created. |
| Install | PASS | `npm ci`: 87 packages; 0 vulnerabilities. Production `npm audit --omit=dev`: 0 vulnerabilities. |
| Full tests | PASS | `npm test`: 18 Vitest, 34 Rust, and 40 Chromium tests passed. |
| Type / lint / format | PASS | `npm run check`: Svelte 0 errors/0 warnings; rustfmt and Clippy with warnings denied passed. |
| Production build | PASS | `npm run build` emitted `dist/` and `target/release/reminder-proof-api`. |
| Default service start | PASS | Release binary started with only `PORT=18081`, generated its local key, served health/metrics, and shut down cleanly. |
| Concurrency smoke | PASS | 100 concurrent local `/health` requests returned 100 × 200. |
| Container build | NOT RUN | Docker CLI is unavailable in this worker. Dockerfile inspection confirms multi-stage builds, `rust:1-slim`, build args, non-root runtime, and port 8080. |

## Core workflow and recovery

The smallest useful product works through the public sample and fixture-backed
managed tests:

- Five realistic fictional appointments load from one click with no account or
  setup.
- Advancing due reminders records delivery, a rejected WhatsApp attempt followed
  by email fallback, and a consent-blocked exception without a provider call.
- Assigning Sofia R. to Sam Rivera, resolving as Called patient, reloading,
  undoing, and resetting all produced the expected persistent and recovery
  states.
- Timelines expose source, consent, channel, time, simulated provider result,
  outcome, response, and staff ownership.
- Invalid JSON, wrong content type, unknown clinical fields, missing auth,
  invalid signatures, replayed callbacks, and oversized bodies have tested
  structured recovery paths. Exactly 16,384 bytes reached JSON validation with
  400; 16,385 bytes returned 413 with a matching UUID request ID.
- Rust and browser claims cover encrypted tenant storage, restart recovery,
  source idempotency, tenant isolation, signed Twilio/Resend receipts, fallback,
  export/delete ownership, and Sociobot-hosted billing return.

No real clinic identity, messaging-provider credential, or payment was used.
Those external integrations were exercised with the repository's signed fixture
adapters, while the public pre-auth path was verified live.

## Browser, accessibility, privacy, and security

- The live browser suite completed 39/40 tests; only QA18-02 failed. All core,
  dark/light Axe, keyboard, mobile, 200% text, reduced-motion, routing, 404,
  offline read-only, headers, and console checks passed.
- Playwright Axe reported zero serious or critical findings across `/`, `/demo`,
  `/start`, `/app`, `/privacy`, `/terms`, and the 404 route in light and dark
  treatments.
- Factory `verify-url.sh` passed: HTTPS 200, title, `lang=en`, one `h1`, main
  landmark, no missing alts, no unnamed buttons, and no console/page errors.
- At 390 px there was no horizontal overflow. First Tab exposed the skip link
  with a `3px` `#005fcc` focus ring; Enter moved focus to `<main>`. Touch targets
  and 200% text checks passed. Reduced-motion emulation left no active animation.
- A cold landing and complete demo workflow made only same-origin requests.
  Fonts came from the product origin; no tracker or third-party script loaded.
- Live HTML/API responses include CSP with header-delivered
  `frame-ancestors 'none'`, HSTS, nosniff, strict-origin referrer policy,
  permissions policy, and COOP. HTML uses `no-cache`; hashed JS, CSS, and fonts
  use one-year immutable caching.
- Every discovered local and external site link returned 200. Unknown routes
  return a styled HTTP 404.
- The live sign-in redirect uses only
  `sociobotcustomers.ciamlogin.com`, tenant
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, authorization code + PKCE S256,
  and `https://clinic-reminder-proof.sociobot.in/auth/callback`.

## Rate limits and backend boundaries

- Demo workspace creation: five requests per client per hour. A fresh live key
  returned five 200s, then 429 with `Retry-After: 3599`.
- General API: configured as 20 requests/second with burst 40. A fully concurrent
  local probe produced 40 allowed and 20 rate-limited responses. A live 60-request
  metrics probe allowed 44 while tokens refilled and rejected 16 with
  `Retry-After: 1`; a protected-route probe produced 40 × 401 then 20 × 429.
- Health is intentionally exempt. Metrics and all API groups are under the
  shared limiter.
- Demo state survived reload and repeated reads. Managed persistence, encrypted
  key/database pairing, 30-day recovery pruning, restart restoration, and
  tenant boundaries passed their named tests.

## Performance and budgets

The full-SHA production build emitted:

- initial JS: 82.68 KB raw / 28.67 KB gzip;
- lazy auth JS: 271.99 KB raw / 68.23 KB gzip;
- CSS: 25.92 KB raw / 5.54 KB gzip;
- emitted fonts: 85.97 KB total.

Fresh mobile Lighthouse evidence at
`qa-artifacts/verification-18-lighthouse-live.json`:

| Category / metric | Result |
| --- | ---: |
| Performance | 98 |
| Accessibility | 100 |
| Best Practices | 100 |
| SEO | 100 |
| FCP | 1,351 ms |
| LCP | 1,452 ms |
| TBT | 151 ms |
| CLS | 0.00074 |
| Total transfer | 90.7 KB |

All stated bundle and Lighthouse-class budgets pass. INP is not available from
a no-interaction synthetic run; TBT is within the 200 ms interaction proxy.

## Applicability and remaining operator checks

This is a web service, not a library/CLI, so pack-and-consumer checks do not
apply. It is not a PWA and makes no offline-reload claim; its tested offline
behavior is a read-only already-loaded ledger with writes disabled. The brief
does not benefit from an AI feature, so no model gateway is expected.

Before inviting clinics, confirm the production callback remains registered on
the shared Sociobot Entra SPA. A later acceptance pass should use a test clinic
identity and registered Sociobot subscription product if those credentials are
available.

## Required remediation and recheck

Deploy the exact full-SHA candidate through the checked-in topology-aware
rollout. Require one healthy revision, sole 100% declared traffic,
`minReplicas=maxReplicas=1`, both Azure Files volumes and mounts, and the exact
full image tag. Randomize the live rate-limit test's client key per run.

Then rerun:

```sh
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

Acceptance requires both commands to pass from a fresh checkout.
