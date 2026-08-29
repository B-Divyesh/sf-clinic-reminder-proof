# Polish round 2 handoff — Reminder Proof

Date: 2026-08-29 UTC  
Work order: `clinic-reminder-proof-polish-2`  
Status: complete

## What changed

- Registered and implemented six direct claim tests for all five round-2 findings: signed/idempotent calendar intake, approved WhatsApp templates, Twilio signatures, Resend Svix signatures, secret encryption, and managed-data minimisation.
- Made provider callback tests replay-safe and state-observable. Valid callbacks create one receipt; repeats and altered signatures cannot change state.
- Made every managed JSON input reject unknown fields. Signed intake containing clinical-note, diagnosis, or treatment fields fails before persistence.
- Kept credentials and patient destinations encrypted in durable state. Workspace/export responses now redact destinations, while the authorized provider adapter receives the decrypted delivery value.
- Preserved all eight round-1 repairs, the isolated one-click demo, the translucent pulse-ledger visual system, route metadata/focus/404/legal behavior, and mobile layout.
- Updated the catalog line to: “Track reminder outcomes, fallbacks, and staff-owned exceptions beside your clinic calendar.” It is verb-first and 91 characters.

## Verification

| Check | Result |
| --- | --- |
| Every claim command, clean clone | Pass — 31/31 independently from `/tmp/clinic-reminder-proof-polish2-clean.qspaMX` |
| `npm test` | Pass — 8 Vitest, 33 Rust, 39 Chromium |
| `npm run check` | Pass — Svelte 0 errors/0 warnings; rustfmt; Clippy with warnings denied |
| `npm run build` | Pass — `dist/` and `target/release/reminder-proof-api` |
| Browser/accessibility | Pass — axe serious/critical 0; keyboard, focus, reduced motion, 390 px and 200% text |
| Privacy/offline | Pass — same-origin demo flow, no tracking request, isolated cookie/storage, cached offline reads with writes disabled |
| Routing/legal | Pass — route titles/meta/canonical, history and deep links, `/privacy`, `/terms`, and HTTP 404 |
| Bundle | Public JS 28.58 KB gzip; CSS 5.52 KB gzip; fonts 85.96 KB total |

## Evidence

- Finding-by-finding map: `.factory/polish-2.md`
- Claims manifest: `.factory/claims.json`
- Copy audit: `.factory/copy-audit.md`
- Local desktop: `.factory/qa-artifacts/polish-2/local/landing-desktop.png`
- Local 390 px demo: `.factory/qa-artifacts/polish-2/local/demo-mobile.png`

## Deployment and cold live check

The container deployment and final cold live evidence are recorded below after the work-order deploy completes.

## Known gaps and next steps

None for the reviewed release scope.
