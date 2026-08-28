# Independent product verification — FAIL

Date: 2026-08-28 UTC
Verifier work order: `clinic-reminder-proof-verify-2`
Candidate commit: `b7b2b9d615836b3aa9d708f058b24f9f30f390a2`
Live URL: `https://clinic-reminder-proof.sociobot.in`

## Verdict

**FAIL — do not release as the researched Reminder Proof product.**

The prior deployment-only failure is no longer present. Fresh production evidence from `GET /health` was:

```json
{"status":"ok","build_sha":"b7b2b9d615836b3aa9d708f058b24f9f30f390a2"}
```

The live web assets match the candidate build names and sizes. The candidate is technically deployable, but it does not deliver the brief's real job-to-be-done.

## Release-blocking finding

### Critical — simulated sandbox and local CSV audit are not the required product

The researched brief requires a vendor-neutral **EMR/calendar connector, consent-aware template sender, approved SMS/email/WhatsApp fallback policy, delivery/response timeline, and staff exception queue** so a clinic can reliably act on real appointments without replacing its EMR.

Fresh evidence shows that this candidate deliberately stops before that job:

- Live `/privacy` says: “Accounts, managed clinic storage, live provider sending, and subscriptions are not available.”
- Live `/start` calls itself “Audit real reminder results” and says it “does not dispatch patient messages, connect to an EMR, or verify provider signatures.” Its only real-data path is a browser-local CSV import, classification, owner text field, and CSV export.
- The public API has only demo workspace/state/advance/exception routes, plus `/health` and `/metrics`; there are no tenant, authentication, connector, policy/template, provider-dispatch, webhook/receipt, billing, or durable-clinic endpoints.
- The demo explicitly labels every provider result “Simulated” and sends no message.

The product is candid about these limits, but a real independent clinic cannot connect its existing calendar, safely send an approved reminder, observe a real provider outcome, or use a durable shared staff exception queue. This violates the repository Definition of Done and the brief's smallest useful product. It is not an acceptable release merely because the sandbox works.

## Technical verification evidence

### Mandatory claim gate — PASS (17/17)

Started at the requested clean candidate checkout. After `npm ci` (85 packages; audit: 0 vulnerabilities), I executed every exact test command declared in `.factory/claims.json` against the product demo entry point. All passed; the final Playwright result was `{"status":"passed","failedTests":[]}`.

| Claim IDs passed |
| --- |
| `demo-isolation`, `sample-outcome-coverage`, `consent-channel-guard`, `fallback-order`, `delivery-timeline`, `exception-ownership`, `demo-reset`, `minimal-reminder-content`, `public-price`, `demo-cookie-lifetime`, `demo-replica-continuity`, `no-tracking`, `request-protection`, `rate-limit-policy`, `security-headers`, `build-identity`, `real-csv-proof` |

The full repository browser suite subsequently passed all 21 tests, including these claims.

### First-read gate — PASS

Cold production load gave, in plain words:

- What: “See every reminder outcome.”
- For whom: “independent clinics that need delivery proof and a clear next step when reminders fail.”
- First action: **Try it with sample data**, with adjacent explanation that it opens a sample clinic and does not touch real data.

The action is a visible first-screen link and opens `/demo` in one click.

### Build and repository gates — PASS

| Command | Evidence |
| --- | --- |
| `npm test` | PASS: 6 Vitest, 10 Rust unit/integration, 21 Chromium tests |
| `npm run check` | PASS: Svelte 0 errors/warnings; rustfmt; clippy with `-D warnings` |
| `npm run build` | PASS: Vite production `dist/` and release Rust binary |
| Production JS budget | `index-Ccgh6mq4.js`: 70,563 bytes raw / 25,261 bytes gzip |
| Production CSS budget | `index-Csu9adpE.css`: 22,127 bytes raw / 5,033 bytes gzip |

Docker is unavailable in this verifier container, so I could not repeat `docker build`. Static inspection confirms a compliant multi-stage shape, `rust:1-slim`, non-root runtime user, `ARG BUILD_SHA=dev`, and port 8080.

### Independent live functional checks — PASS for the delivered sandbox

- A cold demo was created, due reminders advanced, Mina P.'s timeline opened, Sofia R.'s exception assigned to Sam Rivera, resolved as “Called patient,” and undone. The timeline contained source time, consent, SMS channel, simulated provider result `DELIVERED-200`, and outcome in order.
- Normal-flow browser requests were all same-origin: document, self-hosted JS/CSS/font, and same-origin `/api/v1/demo/*`. There were no normal-flow console or page errors.
- Live malformed JSON returned `400` JSON `json_invalid`; a 17 KB body returned `413` JSON `body_too_large`.
- Live demo writes 1–30 returned `200`; write 31 returned `429` JSON with `Retry-After: 55`. Thus the observed per-workspace demo-write allowance is **30/minute**. The product's general API limiter is documented/configured at 20 req/s with burst 40. `/health` is appropriately exempt.
- Response headers included CSP with `frame-ancestors 'none'`, HSTS, `nosniff`, strict-origin referrer policy, permissions policy, and COOP. The hashed JS response used `Cache-Control: public, max-age=31536000, immutable`.
- Desktop and 390 px mobile had no horizontal overflow (`390/390`). On the clean landing page, first Tab focused the visible skip link (3 px `#005fcc` outline); Enter moved focus to `#main`. Reduced-motion media was active and the CSS supplies a reduced-motion override.
- Axe via `@axe-core/playwright` found **zero serious or critical violations** on `/`, `/demo`, `/start`, `/privacy`, `/terms`, and `/404`.

## Non-blocking documentation defect

### Medium — demo contract contradicts the delivered Start for real action

`.factory/demo.md` says “Start for real” takes the visitor to sign-in after M2 and, in M1, explains accounts will arrive later. The delivered action actually routes to `/start`, a browser-local CSV evidence workflow. Update the contract and handoff together when this is corrected.

## Required path to a PASS

Build the planned authenticated, durable multi-tenant service before presenting it as the researched product: Sociobot Entra CIAM, organization/location ownership, calendar/EMR connector, consent and approved-template policy, server-side provider dispatch/receipt reconciliation with idempotency, real fallback execution, shared exception operations, export/delete/retention controls, and the Sociobot subscription flow. Maintain the current isolated demo and its claim coverage while doing so.
