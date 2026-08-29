# Independent product verification 19 — FAIL

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-verify-19`

Candidate: `9791736e1428961621a50ef8e9e1785c365e76b4`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**FAIL — do not release or accept real clinic data.** The candidate passes its
local product, browser, accessibility, security, and build gates. The live
public fallback also serves byte-matching web code from the candidate. However,
the Azure Container App declares an unhealthy, unsafe revision at 100% traffic.
That revision has no durable or backup volume mounts, allows three replicas,
and fails startup. The mandatory `single-replica-durable-topology` claim fails.

This is fresh evidence, not reliance on the builder's deployment report.

## Mandatory first gates

### Claims gate: FAIL

`.factory/claims.json` exists and contains 31 claims. I invoked every manifest
`test` entry separately. The initial pre-install invocation confirmed the clean
clone had no `node_modules`; after the required `npm ci`, I reran all entries.
Thirty passed. The final composite claim passed its Playwright assertion, then
failed in `npm run verify:deployment:current`:

```text
Error: deployment topology must set minReplicas and maxReplicas to 1
```

| Claim | Result |
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
| `single-replica-durable-topology` | **FAIL** — local template test passed; required live deployment verification failed |

Each claim ID occurs exactly once as an `@claim:<id>` browser tag. A required
claim failure is release-blocking regardless of the other results.

### Cold first-read and one-click demo: PASS

The desktop and 390 px first screens answer the three required questions:

- What it does: **“See every reminder outcome.”**
- Who it serves: **independent clinics** needing delivery proof and a clear
  next step when reminders fail.
- What to do first: **“Try it with sample data.”** The adjacent copy says it
  opens a sample clinic and does not touch real clinic data.

One click opened `/demo` with five fictional appointments already populated.
The persistent banner said “Demo — sample data, nothing is saved to your
clinic” and exposed both Reset demo and Start for real.

## Defects by severity

| Severity | ID | Finding |
| --- | --- | --- |
| Critical / release blocker | QA19-01 | Azure declares revision `sf-clinic-reminder-proof--0000056` at 100% traffic, but it is `Unhealthy`, `ActivationFailed`, and `0/1 replicas ready`. It uses short image tag `9791736e1428`, has `minReplicas: 1`, `maxReplicas: 3`, and has no Azure Files volumes or `/durable` and `/backups` mounts. The process repeatedly exits with `required durable storage mounts are missing: /durable, /backups; refusing unsafe production storage`. The public site works only through healthy fallback revision `0000055`, which Azure reports at 0% declared traffic. |

No other independently reproducible product defect was found.

## Deployment and candidate identity

Fresh read-only Azure inspection found:

| Revision | State / traffic | Image | Topology |
| --- | --- | --- | --- |
| `0000055` | Healthy, running, declared 0% | full candidate SHA | one replica; both Azure Files shares mounted |
| `0000056` | **Unhealthy, ActivationFailed, declared 100%** | short `9791736e1428` tag | **max three replicas; no volumes or mounts** |

`latestReadyRevisionName` is `0000055`, while `latestRevisionName` is
`0000056`. The failed revision reports “Deployment Progress Deadline Exceeded.
0/1 replicas ready.” Its container log records the durable-storage guard panic.

The healthy fallback still matches the requested source:

- `/health` returns the full candidate SHA.
- Every checked route footer shows `Build 9791736`.
- Building with `BUILD_SHA=9791736e1428961621a50ef8e9e1785c365e76b4`
  produced `index-D7BxUH40.js`; its SHA-256
  `40702a4fb33c0dccabe44315ce211fb47036f02321515f2e5f4441f2eb644c61`
  matches the live asset byte-for-byte.

Source identity therefore matches, but the required serving topology does not.

## Local candidate gates

| Check | Result | Evidence |
| --- | --- | --- |
| Checkout | PASS | Clean checkout; `HEAD` and `origin/main` were the exact candidate before report edits. |
| Install | PASS | `npm ci`: 87 packages, 0 vulnerabilities. `npm audit --omit=dev`: 0 vulnerabilities. |
| Full tests | PASS | `npm test`: 19 Vitest, 34 Rust API, and 40 Playwright tests passed. |
| Type/lint/format | PASS | `npm run check`: Svelte 0 errors/0 warnings; rustfmt and Clippy with warnings denied passed. |
| Production build | PASS | `npm run build` emitted `dist/` and `target/release/reminder-proof-api`. |
| Default backend start | PASS | Release binary started with only `PORT=18081`, generated its local key, served health, and shut down cleanly. |
| Concurrency smoke | PASS | 100 concurrent local `/health` requests returned 100 × 200. |
| Container build | NOT RUN | No Docker executable is installed in this worker. Dockerfile inspection confirms multi-stage builds, `rust:1-slim`, build args, non-root runtime, and port 8080. |

## End-to-end workflow and recovery

The public sample completed the smallest useful job:

- advancing due reminders produced delivery proof, a simulated rejected
  WhatsApp attempt followed by consented email fallback, and a consent-blocked
  staff exception without a provider attempt;
- the timeline showed channel, time, provider result, outcome, and “Simulated”;
- assigning Sofia R. to Sam Rivera, resolving as Called patient, reloading,
  undoing, and resetting restored the expected states;
- reloads and 30 repeated reads kept the same isolated demo workspace;
- the demo cookie was HttpOnly, Secure, SameSite=Lax, scoped to
  `/api/v1/demo`, and had an 86,400-second lifetime;
- a bad owner returned 422; malformed JSON returned 400; text input returned
  415; 16,384 malformed bytes reached JSON validation; 16,385 bytes returned
  413; missing authentication returned 401 with `WWW-Authenticate: Bearer`.
  Every error carried a matching UUID request ID and a recovery instruction.

Rust and browser coverage also passed tenant isolation, encrypted credentials
and destinations, signed/idempotent calendar intake, signed Twilio and Resend
receipts, replay handling, consented provider fallback, export/delete ownership,
30-day recovery pairing, and Sociobot-hosted subscription return. No real
patient record, messaging credential, payment, or clinic identity was used.

## Browser, accessibility, privacy, and security

- `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run
  test:e2e`: **40/40 passed**.
- Playwright Axe found zero serious or critical issues across `/`, `/demo`,
  `/start`, `/app`, `/privacy`, `/terms`, and the 404 route in light and dark.
- `verify-url.sh`: PASS in 720 ms; correct title, `lang=en`, one `<h1>`, one
  main landmark, no missing alt text, no unnamed buttons, and no console errors.
- At 390 px there was no horizontal overflow, including at 200% root text.
  Visible controls met 44 × 44 px. First Tab exposed the skip link with a 3 px
  `#005fcc` ring; Enter moved focus to `<main>`. Settled reduced-motion pages
  had no active animations.
- The cold landing plus full demo flow made 19 requests, all same-origin.
  There were no console errors, page errors, failed requests, trackers, CDN
  fonts, or provider/billing calls.
- All discovered links resolved. Unknown routes returned a styled HTTP 404.
- HTML and API responses include CSP with header-delivered
  `frame-ancestors 'none'`, HSTS, nosniff, strict-origin referrer policy,
  permissions policy, and COOP. HTML is `no-cache`; hashed JS/CSS use
  `public, max-age=31536000, immutable`.
- Sign-in fetched CIAM discovery and redirected with authorization code + PKCE
  S256 to `sociobotcustomers.ciamlogin.com`, tenant
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, and the expected production callback.

## Rate limits and backend boundaries

- Demo creation: **5 requests/client/hour**. An 18-request concurrent live
  probe produced exactly 5 × 200 and 13 × 429; every 429 had
  `Retry-After: 3599`.
- General API: response headers advertise a burst allowance of 40. Fresh
  100-request concurrent probes produced 45 × 200 / 55 × 429 on auth config,
  41 × 200 / 59 × 429 on metrics, and 40 × 401 / 60 × 429 on a protected
  clinic route. Refill during scheduling explains the small variation; every
  429 had a positive `Retry-After`.
- Protected checkout writes also rate-limited: 60 concurrent requests produced
  42 × 401 and 18 × 429 with positive `Retry-After`.
- `/health` is intentionally exempt. All tested API groups enforce a limit.
- Managed persistence and recovery pass locally, but the unhealthy selected
  production revision has no storage mounts. That deployment defect prevents
  acceptance of the persistence boundary.

## Performance and budgets

The full-SHA production web build emitted:

- initial JS: 82.68 KB raw / 28.67 KB gzip;
- lazy authentication JS: 271.99 KB raw / 68.23 KB gzip;
- CSS: 25.92 KB raw / 5.54 KB gzip;
- fonts: 85.97 KB raw total.

Fresh mobile Lighthouse against the live root:

| Measure | Result |
| --- | ---: |
| Performance | 98 |
| Accessibility | 100 |
| Best Practices | 100 |
| SEO | 100 |
| FCP | 1.4 s |
| LCP | 1.5 s |
| TBT | 150 ms |
| CLS | 0.001 |
| Initial transfer | 90.6 KB |

All stated performance budgets pass. INP is unavailable from a no-interaction
synthetic run; TBT is within the 200 ms interaction proxy.

## Applicability and remaining checks

This is not a library or CLI, so pack/consumer checks do not apply. It is not a
PWA and makes no offline-reload claim; its tested offline state keeps an already
loaded ledger readable and disables writes. The brief does not call for an AI
feature.

A real Entra user, real provider credentials, and a paid Sociobot subscription
were unavailable, so those external flows were limited to the correct live
CIAM redirect, protected live boundaries, and repository fixture adapters.
Confirm the production callback registration before inviting clinics.

## Required remediation

Deploy the exact full-SHA candidate through the checked-in topology-aware
rollout. Require one healthy revision at the sole declared 100% traffic target,
`minReplicas=maxReplicas=1`, the full 40-character image tag, and both Azure
Files volumes mounted at `/durable` and `/backups`. Then rerun:

```sh
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

Acceptance requires both commands to pass from a fresh checkout.
