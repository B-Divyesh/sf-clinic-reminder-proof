# Independent verification 20 — Reminder Proof

Date: 2026-08-30 UTC

Work order: `clinic-reminder-proof-verify-20`

Candidate: `ab685c2435e65a5b3332db785e2bf037d7a3a07a`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict: FAIL

The candidate is not releasable. One mandatory claim fails against the active
Azure deployment. Azure selects revision
`sf-clinic-reminder-proof--0000059` for 100% traffic, but that revision is
unhealthy and cannot start because it has no durable or backup mounts. Its
template also permits three replicas. The public URL remains available only
through healthy fallback revision `0000058`.

This conclusion comes from fresh claim execution and read-only production
inspection during this work order. It does not rely on an earlier deployment
report.

## Mandatory first gates

### Claims gate: FAIL

`.factory/claims.json` exists and contains 31 entries. After the clean-clone
`npm ci`, I ran every entry's exact `test` command separately. Thirty passed.
The final composite command passed its Playwright topology assertion, then
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
| `single-replica-durable-topology` | **FAIL** — source template test passed; required active-deployment verification failed |

Each manifest ID occurs exactly once as an `@claim:<id>` test tag. The one
claim failure is release-blocking under the acceptance contract.

### Cold first-read and one-click demo: PASS

The first screen answers all three questions in plain words on desktop and at
390 px:

- What it does: **“See every reminder outcome.”**
- Who it is for: **independent clinics** that need delivery proof and a clear
  next step when reminders fail.
- What to click: **“Try it with sample data.”** Adjacent copy says it opens a
  sample clinic without touching real clinic data.

One click opened `/demo` with five fictional appointments. The persistent
banner says “Demo — sample data, nothing is saved to your clinic” and includes
Reset demo and Start for real.

## Defects by severity

| Severity | ID | Finding |
| --- | --- | --- |
| Critical / release blocker | QA20-01 | The selected Azure revision `0000059` is `Unhealthy` / `ActivationFailed` at declared 100% traffic. Its image uses the short tag `ab685c2435e6`, scale is `minReplicas: 1`, `maxReplicas: 3`, and it has no Azure Files volumes or `/durable` and `/backups` mounts. The process repeatedly exits because required durable mounts are missing. |

No other independently reproducible product defect was found.

## Deployment and candidate identity

Fresh Azure CLI inspection found:

| Revision | State / declared traffic | Image | Topology |
| --- | --- | --- | --- |
| `sf-clinic-reminder-proof--0000059` | **Unhealthy**, `ActivationFailed`, 100% | short tag `ab685c2435e6` | max three replicas; no volumes or mounts |
| `sf-clinic-reminder-proof--0000058` | Healthy, `RunningAtMaxScale`, 0% | full candidate SHA | one replica; both Azure Files shares and mounts |

The application reports `latestRevisionName=0000059` and
`latestReadyRevisionName=0000058`. Revision `0000059` logs:

```text
initialize durable clinic store: "required durable storage mounts are missing:
/durable, /backups; refusing unsafe production storage"
```

The fallback public response still matches the candidate:

- `/health` returns the complete candidate SHA.
- Every tested footer shows `Build ab685c2`.
- A full-SHA web build emits `index-MN_EBKBT.js`. Its SHA-256 is
  `a542c79e5f8f5e0a4e6dab9264eaae981ed0b2823dccec123283ec7c2de0f5cd`,
  exactly matching the live asset byte-for-byte.

The source identity therefore matches, but the selected production topology
does not.

## Clean-clone quality gates

| Check | Result | Evidence |
| --- | --- | --- |
| Checkout | PASS | Initial `HEAD` and `origin/main` were the exact candidate; worktree was clean. |
| Install | PASS | `npm ci`: 87 packages; 0 reported vulnerabilities. |
| Dependency audit | PASS | `npm audit --omit=dev`: 0 vulnerabilities. |
| Full tests | PASS | `npm test`: 20 Vitest, 34 Rust API, and 40 Playwright tests passed. |
| Type/lint/format | PASS | `npm run check`: Svelte 0 errors/0 warnings; rustfmt and Clippy with warnings denied passed. |
| Exact production build | PASS | Full candidate SHA supplied to build args; `npm run build` emitted `dist/` and `target/release/reminder-proof-api`. |
| Default backend start | PASS | Release binary started with only `PORT=18082`, generated its local data key, served health, and exited cleanly on SIGTERM. |
| Concurrency smoke | PASS | 100 concurrent local `/health` requests returned 100 × 200. |
| Container build | NOT RUN | Docker is not installed in this worker. Dockerfile inspection confirms multi-stage builds, `rust:1-slim`, build args, non-root runtime, and port 8080. |

## End-to-end job and recovery paths

The public demo completed the smallest useful job:

- advancing reminders produced three delivered outcomes and one staff-owned
  exception across four due appointments;
- Jordan L.'s rejected approved WhatsApp template recorded
  `TEMPLATE_REJECTED`, then used consented email and recorded
  `DELIVERED-200`;
- Sofia R.'s SMS opt-out created no messaging-provider attempt and exposed the
  safe next action;
- each timeline showed time, channel, provider result, exact outcome, and
  “Simulated”;
- assigning Sam Rivera, resolving as Called patient, reloading, undoing, and
  resetting all preserved or restored the expected state;
- malformed JSON returned 400, text input 415, a 17 KB body 413, and a
  protected write without auth 401. Each response used JSON recovery copy and
  matching UUID request IDs.

Rust and browser coverage also passed signed and idempotent calendar intake,
tenant ownership, encrypted provider secrets and patient destinations,
approved WhatsApp templates, signed Twilio and Resend callbacks, receipt
replay safety, consented provider fallback, export/delete ownership, recovery
pairing, and Sociobot-hosted subscription return.

## Privacy, security, accessibility, and responsive behavior

- `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run
  test:e2e`: **40/40 passed**.
- Playwright Axe found zero serious or critical findings across `/`, `/demo`,
  `/start`, `/app`, `/privacy`, `/terms`, and the 404 route in light and dark.
- `/opt/fleet/lib/verify-url.sh`: PASS in 770 ms; HTTPS 200, title, `lang=en`,
  one H1, one main landmark, no missing alt text, no unnamed buttons, and no
  console errors.
- At 390 px there was no horizontal overflow, including 200% root text.
  Visible controls met 44 × 44 px. The first Tab exposed the skip link with a
  3 px `#005fcc` ring; Enter focused `<main>`. Settled reduced-motion pages had
  no running animations.
- The cold landing and complete demo flow made 19 requests, all same-origin.
  There were no provider, payment, analytics, or CDN-font calls; no failed
  requests, console errors, or page errors occurred.
- The demo cookie is HttpOnly, Secure, SameSite=Lax, scoped to
  `/api/v1/demo`, and expires after 86,400 seconds.
- HTML and API responses include CSP with header-delivered
  `frame-ancestors 'none'`, HSTS, nosniff, strict-origin referrer policy,
  permissions policy, and COOP. HTML is `no-cache`; hashed JS/CSS use
  `public, max-age=31536000, immutable`.
- All discovered product links resolved. Unknown routes returned a styled
  HTTP 404. Each checked route has its own title, one H1, `lang=en`, and one
  main landmark.

## Authentication, rate limits, and backend boundaries

Sign-in uses only the required Sociobot Microsoft Entra External ID tenant.
The browser loaded authority `sociobotcustomers.ciamlogin.com`, tenant
`35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
`25c704f4-465a-47af-80ab-2c489466b697`, production callback
`/auth/callback`, authorization code flow, and PKCE S256.

Fresh live probes observed:

- Demo creation: **5 requests/client/hour**. Requests 1–5 returned 200;
  request 6 returned 429 with `Retry-After: 3599`.
- General API burst: 100 concurrent auth-config requests returned 44 × 200 and
  56 × 429; metrics returned 42 × 200 and 58 × 429.
- Protected routes: 60 concurrent checkout writes returned 40 × 401 and
  20 × 429; 100 clinic reads returned 46 × 401 and 54 × 429.
- Every sampled 429 had a positive `Retry-After` header. `/health` is
  intentionally exempt.

The small variations above the 40-request burst come from refill while the
concurrent client schedules requests. The persistence logic passes locally,
but QA20-01 means the selected production revision cannot mount or serve that
durable state.

## Performance and bundle budgets

The full-SHA build emitted:

- initial JS: 82.68 KB raw / 28.67 KB gzip;
- lazy authentication JS: 271.99 KB raw / 68.23 KB gzip;
- CSS: 25.92 KB raw / 5.54 KB gzip;
- fonts: 85.97 KB raw total.

Fresh mobile Lighthouse results:

| Measure | Result |
| --- | ---: |
| Performance | 99 |
| Accessibility | 100 |
| Best Practices | 100 |
| SEO | 100 |
| FCP | 1.4 s |
| LCP | 1.4 s |
| TBT | 130 ms |
| CLS | 0.001 |
| Interactive | 1.6 s |
| Total transfer | 89 KiB |

The declared static budgets pass. INP is unavailable from this no-interaction
synthetic run; TBT is within the 200 ms interaction proxy.

## Applicability and limits

This is not a library or CLI, so pack/consumer checks do not apply. It is not
a PWA, registers no service worker, and makes no offline-reload claim. Its
tested offline state keeps an already loaded demo readable and disables
writes. The brief does not need an AI feature.

No real Entra user, clinic record, provider credential, or paid subscription
was available. Those external flows were limited to the correct live CIAM
redirect, protected production boundaries, and repository fixture adapters.
Confirm the production callback registration before inviting clinics.

## Required remediation

Deploy the full 40-character candidate tag through the checked-in
topology-aware rollout. Acceptance requires exactly one healthy, latest,
traffic-bearing revision at 100%, `minReplicas=maxReplicas=1`, and both Azure
Files mounts at `/durable` and `/backups`. Then rerun from a fresh clone:

```sh
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

Both commands must pass before release.
