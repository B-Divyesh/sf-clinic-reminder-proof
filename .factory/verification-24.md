# Verify appointment reminder outcomes — verification 24

Date: 2026-09-05 UTC

Work order: `clinic-reminder-proof-verify-24`

Live URL: <https://clinic-reminder-proof.sociobot.in>

Current milestone: M2 — accounts, durable clinic data, and subscriptions

Implementation reviewed: `ffebafb4a2d6c243424f0750f6c18c1b840df079`

Documentation reviewed: `c339ff97e75af447f9c2db206790d4b9221a79c2`

## Verdict

**FAIL — do not accept this release.** There is one high-severity finding and
zero untested declared claims.

The sample reminder job, mobile and desktop use, accessibility, privacy,
backend boundaries, rate limits, local quality gates, immutable image, and
durable topology pass. The mandatory live deployment claim fails because the
active revision changed after the supplied release record and now identifies
itself with the documentation SHA instead of the implementation SHA.

| Severity | Count |
| --- | ---: |
| Critical | 0 |
| High | 1 |
| Medium | 0 |
| Low | 0 |
| Untested claims | 0 |

## Finding

### QA24-01 — High — production does not identify the reviewed implementation

The exact declared command for `single-replica-durable-topology` failed:

```text
Deployment verification failed: live health build identity; expected
ffebafb4a2d6c243424f0750f6c18c1b840df079, got
c339ff97e75af447f9c2db206790d4b9221a79c2
```

Five uncached `/health` requests returned documentation SHA `c339ff9…`. Fresh
inspection limited to this product found a deployment that differs from the
work order's release record:

- Active revision: `sf-clinic-reminder-proof--0000071`
- Active immutable image:
  `sociobotregistry.azurecr.io/sf-clinic-reminder-proof@sha256:80f9e12b2e7d02e95637304cca9128dbef429424601100f32650d0f74bbf2f60`
- Health and footer identity: `c339ff97e75af447f9c2db206790d4b9221a79c2`
- Expected revision and image: `sf-clinic-reminder-proof--0000070` and the
  digest recorded in `.factory/runtime-release.json`

Revision `0000071` is healthy, solely traffic-bearing, and immutable. It uses
Single mode, one ready replica, `clinic-reminder-proof-data` at `/data`, and
`clinic-reminder-proof-backups` at `/backups`. This is an identity regression,
not a storage or availability regression.

The source diff from the implementation SHA to the documentation SHA changes
only `.factory` records. A fresh build with `BUILD_SHA=ffebafb…` produced a
96,535-byte application bundle. Replacing only the embedded `c339ff9…` value
in the live 96,535-byte bundle made both SHA-256 hashes exactly
`bff97caffd33d53fd15091b8c52edb70d58b69f7521c3ff8310317f10d44df14`.
The product behavior is therefore equivalent to the reviewed implementation,
but the required command still fails and the live release no longer matches
the supplied release record.

Required action: restore the recorded immutable implementation image or update
the release-identity contract through an authorized release process, then run
`npm run verify:deployment:current` from a clean checkout.

## First screen and sample workflow

Fresh 1440×900 desktop and 390×844 phone contexts were opened before any
scroll. Both showed:

- Job: `See every reminder outcome.`
- Audience: independent clinics that need delivery proof and a clear next step
  when reminders fail.
- First action: `Try it with sample data`.
- Result: `Opens a sample clinic. Nothing touches real clinic data.`
- Three facts: the demo uses sample data, reminder content excludes clinical
  notes, and Clinic costs $79 per location each month.

On the phone, the last fact ended at 746 px in an 844 px viewport. The action
and all three facts were visible without scrolling.

One click opened five realistic fictional records for Northline Sample Clinic.
The persistent label read `Demo — sample data, nothing is saved to your
clinic.` and kept `Reset demo` and `Start for real` visible. Advancing the
sample reached 4 due, 3 delivered, and 1 exception. The visible outcomes were
three delivered reminders, Sofia R.'s blocked exception, and Noor A.'s source
cancellation.

Desktop and phone checks assigned Sofia R. to Sam Rivera, reloaded, and found
the assignment unchanged. Resolution moved focus to `Undo resolution`. Reset
restored 4 due, 1 delivered, 1 exception, and an empty owner. Reset succeeded
without consuming another new-demo allocation.

Browser storage contained only the namespaced
`demo:clinic-reminder-proof:<workspace>:active` session key. The demo cookie
was HttpOnly, Secure, SameSite=Lax, scoped to `/api/v1/demo`, and limited to 24
hours. Every normal-flow request stayed on this product origin. There were no
console errors, page errors, failed requests, messaging-provider calls,
checkout calls, tracking requests, or real-data keys.

## Claims and clean-checkout gates

A fresh clone of the named repository was checked out at documentation SHA
`c339ff97…`. `npm ci` installed 87 packages with zero reported vulnerabilities.
Every command in `.factory/claims.json` was then run separately.

| Claim | Result |
| --- | --- |
| `demo-isolation` | Pass |
| `sample-outcome-coverage` | Pass |
| `consent-channel-guard` | Pass |
| `fallback-order` | Pass |
| `delivery-timeline` | Pass |
| `exception-ownership` | Pass |
| `sample-exception-visibility` | Pass |
| `demo-reset` | Pass |
| `minimal-reminder-content` | Pass |
| `public-price` | Pass |
| `demo-cookie-lifetime` | Pass |
| `demo-replica-continuity` | Pass |
| `no-tracking` | Pass |
| `explicit-theme-choice` | Pass |
| `request-protection` | Pass |
| `rate-limit-policy` | Pass locally and live |
| `security-headers` | Pass |
| `build-identity` | Pass for the running documentation SHA |
| `managed-auth-storage` | Pass |
| `signed-calendar-intake` | Pass |
| `approved-whatsapp-dispatch` | Pass |
| `twilio-receipt-verification` | Pass |
| `resend-receipt-verification` | Pass |
| `managed-secret-encryption` | Pass |
| `managed-data-minimisation` | Pass |
| `no-marketing-campaigns` | Pass |
| `signed-in-export-delete` | Pass |
| `managed-provider-fallback-receipt` | Pass |
| `managed-billing-return` | Pass with the declared fixture |
| `managed-storage-recovery` | Pass |
| `single-replica-durable-topology` | **Fail — QA24-01** |
| `ciam-sign-in` | Pass |
| `tenant-isolation` | Pass |
| `durable-onboarding` | Pass |
| `subscription-price` | Pass |
| `data-export` | Pass |
| `account-deletion` | Pass |

Result: **36 pass, 1 fail, 0 untested.**

Additional clean-checkout results:

- `npm test`: pass — 22 Vitest, 43 Rust, and 47 Playwright tests.
- `npm run check`: pass — zero Svelte diagnostics; rustfmt and Clippy clean.
- `npm run build`: pass — `dist/` and the release API binary produced.
- Initial JavaScript: 31.78 kB gzip; CSS: 5.79 kB gzip; lazy sign-in
  JavaScript: 68.23 kB gzip.
- A release binary started with only `PORT=18024`, generated its local storage
  defaults, served health and Prometheus metrics, and returned 100 of 100
  concurrent health requests with HTTP 200.
- The test suite covered tenant separation, role boundaries, restart
  persistence, encrypted keys, duplicate intake, signed receipts, migration
  reversal, seven-day deletion recovery, and 30-day backup retention.

## Live accessibility, routes, privacy, and recovery

- `/opt/fleet/lib/verify-url.sh`: pass — 200, correct title, `lang=en`, one H1,
  one main landmark, no missing alt text, no unnamed buttons, and no console
  error.
- Playwright axe found no serious or critical issue on the public, demo,
  account-entry, onboarding, settings, legal, or not-found views.
- All checked routes had one H1, one main landmark, route-specific titles, no
  390 px overflow, and no loss at 200% text. Light and dark treatments pass.
- The first Tab exposed the 188×46 px skip link with a 3 px `#005fcc` focus
  ring. Enter focused `main`. Visible phone controls were at least 44×44 px.
- Reduced-motion pages had no running animation after settling. Offline mode
  kept the loaded ledger readable, showed the read-only notice, and disabled
  reminder changes. The product makes no offline-reload or installable-PWA
  promise, so update-install checks do not apply.
- `/privacy` and `/terms` returned 200. Anonymous export and deletion returned
  structured 401 responses with `WWW-Authenticate: Bearer` and matching UUID
  request IDs. Malformed JSON, wrong content type, and a 17 kB body returned
  structured 400, 415, and 413 responses with matching request IDs.
- The sign-in action reached the shared Sociobot customer tenant with the
  documented tenant, client, production callback, authorization-code flow,
  and PKCE S256. No credential was entered.
- The unknown route returned the designed HTTP 404 with title
  `Page not found — Reminder Proof`, one H1, and a route home. This deliberate
  404 is expected and is not a defect.
- All discovered links resolved. `robots.txt`, `sitemap.xml`, favicon, 180 px
  touch icon, and the 1200×630 social card resolved. Hashed assets have a
  one-year immutable cache policy. CSP, HSTS, nosniff, referrer, permissions,
  and cross-origin headers are present.
- Fresh mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100, FCP 1.35 s, LCP 1.40 s, TBT 29.5 ms, CLS 0.00074, and 93,989
  bytes transferred.

## Backend limits and health

- The fifth new demo in this verifier session succeeded. The next request
  returned 429 with `Retry-After: 3571`, even though caller-controlled
  forwarding prefixes changed.
- A concurrent public-read probe returned 42 HTTP 200 and 28 HTTP 429
  responses. All 429 responses had a positive `Retry-After`.
- A concurrent anonymous billing-write probe returned 4 HTTP 401 and 12 HTTP
  429 responses. All 429 responses had a positive `Retry-After`.
- `/health` is intentionally exempt and returned 100 of 100 HTTP 200 responses.
- Demo reset did not reduce the five-new-demo allowance. Ordinary recovery
  after a deliberate 429 showed the documented wait-and-retry state. The
  browser's generic failed-resource console line for that intentional 429 is
  expected; normal flows were clean.

## Earlier finding disposition

Every earlier review, polish, and verification record was inspected.

| Earlier findings | Current disposition |
| --- | --- |
| Review F-1-1 through F-1-8 | Fixed. Direct headings, exception lifecycle coverage, short sentences, and consistent terms remain. |
| Review F-2-1 through F-2-5 | Fixed. Signed intake, approved WhatsApp, callback validation, encrypted secrets, and data minimisation have passing claims. |
| Review F-3-1 through F-3-8 | Fixed. The unprovable wording is absent; build footer, direct 404 copy, demo terms, and messaging-provider terms remain correct. |
| Initial demo continuity and incomplete-product findings | Fixed for M2. Demo state persists, and the managed clinic path, provider adapters, export/delete, and billing contract are present. |
| QA3–QA7 request protection, request IDs, rate spoofing, and claim structure | Fixed. Structured boundaries and live limiter probes pass; all 37 claims are declared and exercised. |
| Verification 5 text reflow and focus; QA7 theme/meta; QA11 touch targets | Fixed. Live 200% reflow, explicit themes, metadata, focus transfer, and 44 px targets pass. |
| QA3/5/6/11–22 storage, replica, and rollout failures | Fixed. The live app has one ready replica and both required mounts; persistence and recovery regressions pass. |
| QA15 stale deployment verifier and QA18 reused rate identity | Fixed. The current verifier reads the release record and correctly detected QA24-01; live spoof-resistance passes. |
| QA23 mutable image | Fixed. The active image uses an immutable digest. QA24-01 is a different mismatch between that image's build identity and the reviewed implementation. |
| Reset consuming new-demo allowance | Fixed. Repeated reset is covered locally, and live reset did not consume another new workspace. |

No earlier minor finding has reopened.

## Current milestone and external dependency

M2 remains the current building stage. M3 and later capabilities were not
treated as shipped promises. No runtime AI feature is justified for this
deterministic consent and evidence job.

The three pilot checkout URLs for Clinic, Practice, and Network each returned
the documented HTTP 404 `enabled factory product` response. This is the known
operator dependency, not a broken page or unexpected product error, and is not
counted as QA24-01. The operator must register or enable all three recurring
pilot tiers before an authorized checkout, return, cancellation, and revocation
verification can run. The declared local billing-return fixture passed. No
payment, customer credential, real clinic record, or messaging provider was
used during QA.

## Evidence

Fresh screenshots, URL-verifier output, and Lighthouse JSON are under
`/work/.evidence/verification-24/`. The required report copy is
`/work/.evidence/qa-report.md` and the machine verdict is
`/work/.evidence/qa-result.json`.
