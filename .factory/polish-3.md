# Polish round 3 evidence

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-polish-3`

Review commit: `75a231532b880030ba935085385044e9de26051d`

Repair implementation and live build: `785e4bc956d0ddaefc0f2babff0efd5d6a54b189`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Every cumulative finding

| Finding | Change made | Test evidence | Screenshot | Live check |
| --- | --- | --- | --- | --- |
| F-1-1 | Kept the direct landing headings `Reminder evidence`, `Limits and privacy`, and `Clinic plan price`. | Playwright `landing sections use descriptive headings and the first screen names the job, audience, and next step` | `qa-artifacts/polish-3/live/landing-desktop-cold.png` | `/` passed the live browser suite. |
| F-1-2 | Kept the declared open/resolved/undo exception lifecycle. | `npm run test:e2e -- --grep @claim:sample-exception-visibility` | `qa-artifacts/polish-3/live/query-demo-mobile.png` | `/?demo=1` kept the exception visible until resolution and preserved its ledger evidence. |
| F-1-3 | Kept the README opening as short, concrete sentences. | Vitest `README cumulative plain-words repairs keep each reviewed sentence short and concrete` | `qa-artifacts/polish-3/live/landing-desktop-cold.png` | `/` retained the matching direct product description. |
| F-1-4 | Kept the README demo actions in two short sentences. | Same Vitest copy contract; `.factory/copy-audit.md` | `qa-artifacts/polish-3/live/query-demo-mobile.png` | `/?demo=1` exposed the named advance, assign, resolve, undo, and reset actions. |
| F-1-5 | Kept consent checks and receipt/exception handling in separate README sentences. | Same Vitest copy contract; `@claim:managed-provider-fallback-receipt` | `qa-artifacts/polish-3/live/demo-mobile.png` | `/demo` displayed consent and messaging-provider evidence separately. |
| F-1-6 | Kept the implementation summary as plain sentences about separation, limits, health, and metrics. | Same Vitest copy contract; `@claim:build-identity` and `@claim:rate-limit-policy` | `qa-artifacts/polish-3/live/screenshot-mobile.png` | `/health`, `/metrics`, and `/` passed live checks. |
| F-1-7 | Kept durable writes, daily recovery, and startup restoration as separate sentences. | Same Vitest copy contract; `@claim:managed-storage-recovery` | `qa-artifacts/polish-3/live/landing-desktop-cold.png` | Live revision `0000036` is healthy with both recovery mounts. |
| F-1-8 | Kept storage mounts and non-root execution as separate sentences. | Same Vitest copy contract; `@claim:single-replica-durable-topology` | `qa-artifacts/polish-3/live/landing-desktop-cold.png` | Azure reports one replica and `/durable` plus `/backups` mounts. |
| F-2-1 | Retained the declared signed calendar intake and duplicate-source protection. | `npm run test:e2e -- --grep @claim:signed-calendar-intake`; Rust `managed_claim_signed_calendar_intake_is_authenticated_and_idempotent` | `qa-artifacts/polish-3/live/screenshot-mobile.png` | `/start` exposes the managed entry; protected intake remained inaccessible anonymously. |
| F-2-2 | Retained the approved-template WhatsApp dispatch claim and fixture proof. | `npm run test:e2e -- --grep @claim:approved-whatsapp-dispatch`; Rust `managed_claim_approved_whatsapp_uses_template_and_records_receipt` | `qa-artifacts/polish-3/live/demo-mobile.png` | Live public copy distinguishes simulated demo events from managed messaging providers. |
| F-2-3 | Retained separate signature and replay tests for Twilio and Resend receipts. | `@claim:twilio-receipt-verification`; `@claim:resend-receipt-verification` | `qa-artifacts/polish-3/live/demo-mobile.png` | Live receipt routes remained protected and returned structured responses. |
| F-2-4 | Retained encrypted credential/destination storage, export redaction, and adapter-only decryption. | `npm run test:e2e -- --grep @claim:managed-secret-encryption`; Rust `managed_claim_secrets_and_destinations_are_encrypted_and_adapter_scoped` | `qa-artifacts/polish-3/live/screenshot-mobile.png` | `/privacy` states the bounded managed-data handling and passed the live crawl. |
| F-2-5 | Retained strict rejection of clinical fields on every managed JSON input and minimized export proof. | `npm run test:e2e -- --grep @claim:managed-data-minimisation`; Rust `managed_claim_clinical_fields_are_rejected_at_every_json_write` | `qa-artifacts/polish-3/live/screenshot-mobile.png` | `/privacy` and the landing facts use the same minimized-content boundary. |
| F-3-1 | Removed the unprovable “Original hand-authored” wording. The remaining self-hosted-font statement is now covered by `no-tracking`, which proves both declared families resolve from this origin. The stronger test also exposed and fixed the `Instrument Sans Variable` family-name mismatch. | `npm run test:e2e -- --grep @claim:no-tracking`; Vitest `review-three public copy uses direct 404 and messaging-provider terms` | `qa-artifacts/polish-3/live/landing-desktop-cold.png` | The live flow loaded no third-party request and fetched both font families from this site. |
| F-3-2 | Replaced the milestone label with `Build <short SHA>` on every route. The Docker build argument now reaches both the API and Vite. | `npm run test:e2e -- --grep @claim:build-identity` | `qa-artifacts/polish-3/live/screenshot-mobile.png` | Footer `Build 785e4bc` matches `/health` SHA `785e4bc956d0ddaefc0f2babff0efd5d6a54b189`. |
| F-3-3 | Changed the styled 404 H1 to `Page not found`; the pulse-ledger illustration remains visual context only. | Playwright `unknown browser routes return an HTTP 404 with the styled recovery page`; Vitest review-three copy contract | `qa-artifacts/polish-3/live/404-mobile.png` | `/not-a-real-route` returned HTTP 404 with the direct heading and home action. |
| F-3-4 | Standardized the visitor term as `demo`; README now says `Try the demo`. | Vitest review-three copy contract | `qa-artifacts/polish-3/live/query-demo-mobile.png` | `/?demo=1` entered the populated demo directly. |
| F-3-5 | Replaced cookie implementation language in the demo introduction with a plain 24-hour availability sentence. Technical cookie details remain in the testable privacy contract. | Vitest README copy contract; `@claim:demo-cookie-lifetime` | `qa-artifacts/polish-3/live/query-demo-mobile.png` | The live banner says sample data is not saved to the clinic and offers Reset demo and Start for real. |
| F-3-6 | Replaced the visitor-facing `same-origin` phrase with `A service on this site`. | Vitest review-three copy contract; `@claim:no-tracking` | `qa-artifacts/polish-3/live/landing-desktop-cold.png` | All recorded live requests stayed on the product origin. |
| F-3-7 | Replaced `idempotent appointment upserts` with `stores each appointment once, even when it receives the same update twice`. | Vitest review-three copy contract; `@claim:signed-calendar-intake` | `qa-artifacts/polish-3/live/screenshot-mobile.png` | `/start` and all public routes passed the live copy/crawl suite. |
| F-3-8 | Standardized visitor and workflow copy on `messaging provider` or `messaging-provider` where it modifies a noun. | Vitest review-three copy contract; all messaging-provider claim tests | `qa-artifacts/polish-3/live/demo-mobile.png` | `/`, `/demo`, `/start`, `/privacy`, `/terms`, and `/404` passed the live crawl with consistent terminology. |

## Clean-clone and live acceptance

- Fresh clone: `/tmp/clinic-reminder-proof-polish3-clean.M2u9lu` from pushed `main`.
- `npm ci`: 87 packages, zero reported vulnerabilities.
- All 31 exact commands in `.factory/claims.json`: 31/31 passed independently.
- `npm test`: 9 Vitest, 33 Rust, and 39 Chromium tests passed.
- `npm run check`: Svelte 0 errors/0 warnings; rustfmt clean; Clippy passed with warnings denied.
- `npm run build`: `dist/` and the release API binary produced. Public entry JS is 28.62 KB gzip, CSS is 5.53 KB gzip, and the lazy sign-in chunk is 68.23 KB gzip.
- Live browser suite: 39/39 passed against the production origin. It covered claims, both-theme axe checks, one-click `?demo=1`, reset/isolation, request logging, offline reads, keyboard and focus behavior, 390 px and 200% reflow, metadata, links, titles, legal routes, security headers, rate limits, and the real 404 response.
- Factory URL verifier: 200, no console errors, title present, `lang=en`, one H1, one main, no missing image alt, and no unlabeled button.
- axe CLI 4.11.4: 0 violations. The live multi-route Playwright axe test also found no serious or critical issue in either theme.
- Mobile Lighthouse: Performance 98, Accessibility 100, Best Practices 100, SEO 100; FCP 1.35 s, LCP 1.45 s, TBT 152 ms, CLS 0.0007.
- Operational smoke: 100 concurrent `/health` requests returned 100 × 200. The browser suite proved live 429 responses include `Retry-After`.
- Deployment: image `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:785e4bc956d0`, healthy revision `sf-clinic-reminder-proof--0000036`, 100% traffic, one replica, and separate Azure Files mounts at `/durable` and `/backups`.

No finding from review 1, 2, or 3 remains open.
