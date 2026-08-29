# Polish round 2 evidence

Date: 2026-08-29 UTC  
Work order: `clinic-reminder-proof-polish-2`  
Review commit: `9c0a87a715d90c8572aa35ef425f3fe38da71893`  
Repair implementation: `2abe63f045b2c6fe7641822be881a977049d9bba`  
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Finding closure

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 | Kept the round-1 descriptive headings `Reminder evidence`, `Limits and privacy`, and `Clinic plan price`. | Playwright `landing sections use descriptive headings and the first screen names the job, audience, and next step`; `.factory/qa-artifacts/polish-2/local/landing-desktop.png`; live `/` cold check. |
| F-1-2 | Kept the declared `sample-exception-visibility` lifecycle and its open/resolved/undo presentation. | `npm run test:e2e -- --grep @claim:sample-exception-visibility`; `.factory/qa-artifacts/polish-2/local/demo-mobile.png`; live `/?demo=1`. |
| F-1-3 | Kept the split README introduction at 9 and 10 words. | Vitest `README plain-words repairs keep each reviewed sentence short and concrete`; `.factory/copy-audit.md`. |
| F-1-4 | Kept the README demo actions as two sentences of 7 and 12 words. | Same Vitest copy contract and `.factory/copy-audit.md`. |
| F-1-5 | Kept consent and receipt/exception behavior as separate README sentences. | Same Vitest copy contract and `.factory/copy-audit.md`. |
| F-1-6 | Kept the plain same-origin service wording and separate developer detail. | Same Vitest copy contract and `.factory/copy-audit.md`. |
| F-1-7 | Kept durable writes and daily recovery as separate short sentences. | Same Vitest copy contract; `@claim:managed-storage-recovery`. |
| F-1-8 | Kept storage mounts and non-root execution as separate short sentences. | Same Vitest copy contract; `@claim:single-replica-durable-topology`. |
| F-2-1 | Added `signed-calendar-intake`. The integration test posts one signed appointment twice, proves one upserted source record, then rejects an altered payload and preserves state. | `npm run test:e2e -- --grep @claim:signed-calendar-intake`; Rust `managed_claim_signed_calendar_intake_is_authenticated_and_idempotent`; live `/health` and `/app` route check. |
| F-2-2 | Added `approved-whatsapp-dispatch`. The fixture captures the exact approved-template request, proves no free-form body, and records a valid signed delivery receipt. | `npm run test:e2e -- --grep @claim:approved-whatsapp-dispatch`; Rust `managed_claim_approved_whatsapp_uses_template_and_records_receipt`; live `/start` capability check. |
| F-2-3 | Added separate Twilio and Resend claims. Each submits a valid callback twice plus an altered signature and proves exactly one receipt event. | `@claim:twilio-receipt-verification`; `@claim:resend-receipt-verification`; Rust replay-safe integration tests; live response/header check. |
| F-2-4 | Added `managed-secret-encryption`; ciphertext checks cover the local and durable databases. Workspace/export responses now redact contact destinations. Only the authorized provider adapter receives decrypted credentials and destinations. | `npm run test:e2e -- --grep @claim:managed-secret-encryption`; Rust `managed_claim_secrets_and_destinations_are_encrypted_and_adapter_scoped`; live Privacy route check. |
| F-2-5 | Added strict unknown-field rejection to every managed JSON input, including nested intake channels and billing returns. The claim posts a signed appointment with a clinical note and proves 422, no stored reminder, and no exported value. | `npm run test:e2e -- --grep @claim:managed-data-minimisation`; Rust `managed_claim_clinical_fields_are_rejected_at_every_json_write`; live Privacy route check. |

## Cumulative acceptance evidence

- Every one of the 31 exact `.factory/claims.json` commands passed independently in clean clone `/tmp/clinic-reminder-proof-polish2-clean.qspaMX`.
- `npm test`: 8 Vitest contracts, 33 Rust tests, and 39 Chromium tests passed.
- `npm run check`: Svelte 0 errors/0 warnings, rustfmt clean, Clippy warnings denied.
- `npm run build`: `dist/` and the release API binary produced. Public JS is 28.58 KB gzip; CSS is 5.52 KB gzip.
- The browser suite covers one-click `?demo=1`, isolated reset, same-origin request logging, offline reads, keyboard paths, 390 px and 200% reflow, reduced motion, route focus/history, per-route titles/metadata, legal links, axe serious/critical scans, and a styled HTTP 404.
- Local visual evidence: `.factory/qa-artifacts/polish-2/local/landing-desktop.png` and `.factory/qa-artifacts/polish-2/local/demo-mobile.png`.

No finding from review 1 or review 2 remains open.
