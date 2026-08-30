# Independent product verification 21 — PASS

Date: 2026-08-30 UTC

Work order: `clinic-reminder-proof-verify-21`

Candidate: `e40583f5c49e4754a850274ed5467327e2a156ea`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Verdict

**PASS — candidate accepted.** Fresh evidence does not reproduce the earlier
deployment-only failure. The active image, public health identity, browser
footer, and candidate web asset all identify the requested candidate. Azure
has one healthy traffic-bearing replica with both required durable shares.

No release-blocking or lower-severity product defect was found.

## Mandatory first gates

### First-read and one-click demo: PASS

A cold 1440 × 900 visit answered the three required questions before a
scroll:

- **What it does:** “See every reminder outcome.”
- **For whom:** independent clinics that need delivery proof and a clear next
  step when a reminder fails.
- **What to click first:** “Try it with sample data.” Adjacent copy says,
  “Opens a sample clinic. Nothing touches real clinic data.”

The one visible sample action opened `/demo` in one click. The first demo
screen was already populated with five fictional appointments, a delivery
ledger, and an exception queue. Its persistent banner says “Demo — sample
data, nothing is saved to your clinic” and offers **Reset demo** and **Start
for real**.

### Claims gate: 31/31 PASS

`.factory/claims.json` exists with 31 unique IDs. Every ID occurs exactly once
as an `@claim:<id>` test tag. After `npm ci`, I ran every manifest `test`
command separately, in manifest order, through the shipped demo entry point.
All passed. Per-claim output and timestamps were retained during verification
under `/tmp/clinic-reminder-proof-claim-logs/`.

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
| `managed-auth-storage` | PASS |
| `signed-calendar-intake` | PASS |
| `approved-whatsapp-dispatch` | PASS |
| `twilio-receipt-verification` | PASS |
| `resend-receipt-verification` | PASS |
| `managed-secret-encryption` | PASS |
| `managed-data-minimisation` | PASS |
| `no-marketing-campaigns` | PASS |
| `signed-in-export-delete` | PASS |
| `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS |
| `managed-storage-recovery` | PASS |
| `single-replica-durable-topology` | PASS |

The landing page and README were cross-checked against the claim manifest and
the executable contract suite. No unsupported public product promise was
found.

## Clean-clone local quality gates

| Check | Result |
| --- | --- |
| Checkout | PASS — initial worktree clean; `HEAD` and `origin/main` both exactly matched the candidate. |
| `npm ci` | PASS — 87 locked packages; npm reported 0 vulnerabilities. |
| `npm audit --omit=dev` | PASS — 0 vulnerabilities. |
| `npm test` | PASS — 21 Vitest tests, 34 Rust tests, and 40 Chromium tests. |
| `npm run check` | PASS — Svelte 0 errors/0 warnings, rustfmt clean, Clippy warnings denied. |
| Exact production build | PASS — `BUILD_SHA`, `GIT_SHA`, and `SOURCE_COMMIT` set to the full candidate; `npm run build` emitted `dist/` and the release API binary. |
| Default runtime | PASS — the release service started with only `PORT=4811`, generated local key state without exposing it, and served health and metrics. |
| Local concurrency | PASS — 100 requests at concurrency 20 to `/health` returned 100 × 200. |
| Container build | Not run — neither Docker nor Podman exists in this verifier. The exact deployed candidate image was verified independently below. |

The exact web build emitted initial JS of 82.68 KB raw / 28.67 KB gzip and
CSS of 25.92 KB raw / 5.54 KB gzip. All emitted font files total 85.97 KB.
The 271.99 KB authentication module is a lazy chunk and is not loaded on the
public first paint.

## Live deployment and candidate identity

`npm run verify:deployment:current` passed against fresh Azure and public
evidence:

```json
{
  "revision": "sf-clinic-reminder-proof--0000060",
  "image": "sociobotregistry.azurecr.io/sf-clinic-reminder-proof:e40583f5c49e4754a850274ed5467327e2a156ea",
  "replicas": 1,
  "buildSha": "e40583f5c49e4754a850274ed5467327e2a156ea",
  "rateStatuses": [200, 200, 200, 200, 200, 429],
  "retryAfter": "3599"
}
```

The verifier also required one healthy latest revision at 100% traffic,
`minReplicas=maxReplicas=1`, the `clinic-data` Azure Files share mounted at
`/durable`, and the `clinic-backups` share mounted at `/backups`.

Live `/health` returns the full candidate SHA. Every checked footer displays
`Build e40583f`. Building the web application with the full SHA emitted
`/assets/index-BEL-X4S7.js`; its local and live SHA-256 are identical:

```text
b69af0a1c6bb432fd5d60bf626524437ff9a4dec108b920cc9be3c33e036d67d
```

The complete browser suite was also run against production with
`PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in`; **40/40
passed**.

## End-to-end product behavior

Fresh independent desktop and 390 px mobile flows completed the smallest
useful job:

- Advancing four due sample reminders produced delivery evidence or a
  staff-owned exception for every due row.
- Jordan L.'s approved WhatsApp attempt recorded `TEMPLATE_REJECTED`, then the
  next consented email channel recorded simulated acceptance.
- Sofia R.'s SMS opt-out produced no provider attempt and kept a visible
  exception with a safe next action.
- Every inspected timeline showed source, consent, channel, time, provider
  result, exact outcome, and the simulated label.
- Assigning Sofia R. to Sam Rivera and resolving as “Called patient” persisted
  through reload. Keyboard-only undo restored the open exception. Reset
  restored the original sample clinic.
- Deep links, browser Back, direct reload, styled HTTP 404 recovery, and the
  already-loaded offline read-only state worked.

Boundary and invalid-input probes returned actionable structured errors:

| Case | Live result |
| --- | --- |
| Malformed JSON | 400 `json_invalid` |
| Wrong content type | 415 `content_type_invalid` |
| Exactly 16,384-byte valid JSON body | 200 |
| 16,385-byte body | 413 `body_too_large` |
| 17 KB protected write | 413 `body_too_large` before authentication |
| Anonymous protected checkout | 401 `bearer_required`, `WWW-Authenticate: Bearer` |

Every error carried a unique UUID request ID that exactly matched its
`X-Request-Id` header.

## Privacy, security, authentication, and API limits

- A complete live landing-to-demo interaction made 19 requests. Every request
  was same-origin. No tracker, CDN font, messaging provider, checkout, or
  account endpoint was contacted by the demo.
- No console error, page error, or failed request remained in the settled
  independent flows or the full live suite.
- The demo cookie is HttpOnly, Secure, SameSite=Lax, scoped to
  `/api/v1/demo`, and expires after 86,400 seconds.
- HTML is `no-cache`. Hashed JS, CSS, and font responses use
  `public, max-age=31536000, immutable`.
- Responses include CSP with header-delivered `frame-ancestors 'none'`, HSTS,
  `nosniff`, strict-origin referrer policy, permissions policy, and COOP.
- No tracked secret-like key or private-key material was found.
- Sign-in navigated only through the required Sociobot Microsoft Entra
  External ID authority, tenant `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, and
  client `25c704f4-465a-47af-80ab-2c489466b697`. The authorize request used
  the production `/auth/callback`, authorization code flow, PKCE S256, state,
  and nonce.

Observed rate limits:

- Demo creation: **five requests per client per hour**. Requests 1–5 returned
  200; request 6 returned 429 with `Retry-After: 3599`.
- General public API burst: 60 concurrent auth-config reads returned 44 × 200
  and 16 × 429. The small refill above the 40-request burst is expected; every
  429 had `Retry-After: 1`.
- Protected write burst: 60 concurrent checkout writes returned 40 × 401 and
  20 × 429. Every 429 had `Retry-After: 1`.
- `/health` is intentionally exempt. A live 100-request, 20-way smoke returned
  100 × 200.

Fixture-backed Rust and browser tests additionally passed tenant isolation,
encrypted durable storage, signed/idempotent calendar intake, approved
WhatsApp templates, Twilio and Resend signature/replay checks, consented
fallback, minimized export/delete, Sociobot billing return, and 30-day
recovery pairing.

## Accessibility, responsive behavior, and performance

- Axe found **0 serious and 0 critical** violations on all public routes in
  light and dark themes, including landing, demo, clinic entry, privacy,
  terms, and 404.
- At 390 px, landing and demo had no horizontal overflow. The 200% text checks
  reflowed without losing content or controls.
- Every independently measured visible demo control was at least 44 px high;
  the smallest measured control was 91 × 44 px.
- Keyboard traversal reached every landing interactive in source order. Each
  interactive received the designed 3 px focus outline. The skip link moved
  focus to main; native owner selection, resolution, focus restoration, and
  undo all worked by keyboard.
- Reduced-motion mode had no running animation after state changes settled.
- Factory `verify-url.sh` passed: HTTPS 200, 756 ms load, correct title/lang,
  one H1, one main landmark, no missing image alternatives, no unnamed
  buttons, and no console errors.
- Fresh mobile Lighthouse: performance 95, accessibility 100, best practices
  100, SEO 100; FCP 1.4 s, LCP 1.5 s, CLS 0.001, TTI 1.8 s, and 89 KiB total
  transfer. Synthetic INP is unavailable without a recorded interaction.

Every checked public route has its own title, one H1, one description meta,
canonical URL, social image, header, main, and footer. `/robots.txt` and
`/sitemap.xml` return 200; an unknown browser route returns a styled HTTP 404.

## Defects by severity

| Severity | Findings |
| --- | --- |
| Critical | None |
| High | None |
| Medium | None |
| Low | None |

## Applicability and limits

This is not a library or CLI, so package/consumer checks do not apply. It is
not a PWA, registers no service worker, and makes no offline-reload claim. The
brief does not benefit from runtime AI; deterministic consent, templates,
fallback, and receipts are the safer product behavior.

No real clinic identity, patient record, provider credential, or paid
subscription was used. Live verification therefore covered the CIAM redirect
and protected production boundaries; signed fixture adapters covered the
credentialed connector, messaging, billing, export, deletion, and recovery
paths without sending a real reminder or making a charge.

## Reproduce

```sh
git checkout --detach e40583f5c49e4754a850274ed5467327e2a156ea
npm ci
npm test
npm run check
candidate_sha=e40583f5c49e4754a850274ed5467327e2a156ea
BUILD_SHA="$candidate_sha" GIT_SHA="$candidate_sha" SOURCE_COMMIT="$candidate_sha" npm run build
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
npm run verify:deployment:current
```

Use <https://clinic-reminder-proof.sociobot.in/?demo=1> for the isolated
sample workflow.
