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
- Corrected the evidence-row collision found during the live 200% text inspection. Narrow screens now stack the appointment, outcome, and evidence link, with browser geometry coverage.
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
| Bundle | Public JS 28.58 KB gzip; CSS 5.53 KB gzip; fonts 85.96 KB total |
| Lighthouse mobile | Performance 100; Accessibility 100; Best Practices 100; SEO 100; LCP 1.281 s; CLS 0; TBT 25.5 ms |
| Live rate limit | Five clinic-creation requests returned 200; request six returned 429 with `Retry-After: 3599` |
| Live load smoke | 100 concurrent `/health` requests returned 100 responses with status 200 |

## Evidence

- Finding-by-finding map: `.factory/polish-2.md`
- Claims manifest: `.factory/claims.json`
- Copy audit: `.factory/copy-audit.md`
- Local desktop: `.factory/qa-artifacts/polish-2/local/landing-desktop.png`
- Local 390 px demo: `.factory/qa-artifacts/polish-2/local/demo-mobile.png`
- Live route and demo-isolation report: `.factory/qa-artifacts/polish-2/live/cold-browser.json`
- Live 200% geometry report: `.factory/qa-artifacts/polish-2/live/high-zoom.json`
- Live 200% screenshot: `.factory/qa-artifacts/polish-2/live/demo-mobile-200.png`
- Live Lighthouse report: `.factory/qa-artifacts/polish-2/live/lighthouse.json`
- Live baseline verification: `.factory/qa-artifacts/polish-2/live/verify.json`

## Deployment and cold live check

- Live URL: <https://clinic-reminder-proof.sociobot.in>
- Deployed source: `7055965c76d2146de574368ae2882a79624de526`
- Image: `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:7055965c76d2`
- Active revision: `sf-clinic-reminder-proof--0000033`, serving 100% of traffic
- Runtime: one replica, `clinic-reminder-proof-data` mounted at `/durable`, and `clinic-reminder-proof-backups` mounted at `/backups`
- `/health`: `{"status":"ok","build_sha":"7055965c76d2146de574368ae2882a79624de526"}`
- Cold routes: `/`, `/demo`, `/demo/reminders/mina`, `/start`, `/privacy`, and `/terms` returned 200; an unknown route returned HTTP 404.
- Every cold route had the correct title, metadata, canonical URL, one H1, one main landmark, legal links, zero console errors, and zero serious/critical axe findings.
- The cold demo opened with five sample reminders, moved focus to its H1, changed its isolated workspace on reset, made no cross-origin requests, and used an HttpOnly, Secure, SameSite=Lax cookie scoped to `/api/v1/demo`.

## Known gaps and next steps

None for the reviewed release scope.
