# Adversarial first-read review 3 — FAIL

Date: 2026-08-29 UTC
Live URL: <https://clinic-reminder-proof.sociobot.in>
Reviewed repository commit: `7f88ac0ff7cdfc83018779cb9fa5c0e42cbd46f1`
Deployed application build: `741bba6617bbf5673e8b2b986a7f435496e6ed24`

## Verdict

**FAIL.** There are eight findings: one unlisted claim, one missing footer requirement, one metaphorical 404 heading, and five plain-language or terminology defects. There are no blocking findings. The cold first screen, one-click demo, sandbox isolation, reset, all 31 claim commands, complete test suite, build, routing, accessibility, and every earlier finding pass.

## Cold first read

Fresh Chromium contexts opened the production origin at 390 × 844 and 1440 × 900 before scrolling.

| Question | Answer visible on the first screen |
| --- | --- |
| What does it do? | “See every reminder outcome.” |
| For whom? | “For independent clinics that need delivery proof and a clear next step when reminders fail.” |
| What should I click first? | “Try it with sample data,” next to “Opens a sample clinic. Nothing touches real clinic data.” |

The first-read gate passes at both sizes. The primary action and all three facts are visible without scrolling at 390 px. The action has a 44 px target. There were no console errors or third-party requests on either cold load.

## Findings

### F-3-1 — Medium — the README makes an unlisted originality claim

**Location / quote:** README, “What is included”: “Original hand-authored pulse-ledger art, favicon, touch icon, social card, and self-hosted Instrument Sans / Fragment Mono assets.”

**Why this fails:** “Original hand-authored” is a provenance claim a reader could rely on, but `.factory/claims.json` has no entry for it. `no-tracking` proves same-origin runtime requests; it does not prove authorship or originality. The claims contract does not allow an untestable public assurance to remain unlisted.

**Concrete fix:** remove the untestable phrase and write: “The site includes pulse-ledger art, a favicon, a touch icon, a social card, and self-hosted Instrument Sans and Fragment Mono fonts.” Map the self-hosting statement to `no-tracking`, or add an `asset-provenance` claim that checks the asset manifest, font licences, and same-origin requests.

### F-3-2 — Medium — the footer has no version or build ID

**Location / quote:** footer on every route: `managed-clinic-workflow`.

**Why this fails:** This is a static internal label, not a version or build identifier. It cannot identify the deployed revision, although the required site skeleton calls for a footer version/build ID. The actual build SHA is available from `/health` but is not shown in the page.

**Concrete fix:** inject a short immutable build SHA or release version into the web build and render it in the footer, for example `Build 741bba6`. Add a browser assertion that the footer value matches `/health.build_sha`.

### F-3-3 — Minor — the 404 heading is a ledger metaphor

**Location / quote:** live unknown route and `/404` H1: “This page has no ledger entry”.

**Why this fails:** A person following a broken link must translate product imagery before learning what happened. The plain-words rule prohibits metaphor headings and requires a heading that names the page out of context.

**Concrete fix:** use `Page not found` as the H1. Keep any ledger-themed explanation or art below it.

### F-3-4 — Minor — the README renames the demo a “sandbox”

**Location / quote:** README heading: “Try the public sandbox”. The landing page, navigation, banner, and terminology table call it a `demo`.

**Why this fails:** “Sandbox” is developer jargon and introduces a second term for the same one-click sample experience.

**Concrete fix:** rename the heading to `Try the demo` or `Try it with sample data`.

### F-3-5 — Minor — the public-demo introduction uses cookie implementation jargon

**Location / quote:** README, “Try the public sandbox”: “Its compact state stays in an HttpOnly, Secure browser cookie, so a restart or replica change does not lose the sample.”

**Why this fails:** `HttpOnly`, `Secure`, and `replica change` describe implementation rather than the result a visitor can use. A first-time clinic reader should not need browser-security or deployment vocabulary to understand demo persistence.

**Concrete fix:** “A protected browser cookie keeps the sample available for 24 hours, including after a server restart.” Put the exact cookie attributes in `.factory/demo.md` or the operations section.

### F-3-6 — Minor — “same-origin” is unexplained README jargon

**Location / quote:** README, “What is included”: “A same-origin service protects demo sessions and clinic data.”

**Why this fails:** “Same-origin” is a browser security term, not a user outcome. It does not explain the protection to a clinic reader.

**Concrete fix:** “A service on this site keeps demo sessions separate from clinic data.” Keep origin-policy details in the technical section.

### F-3-7 — Minor — “idempotent appointment upserts” is database jargon

**Location / quote:** README, “What is included”: “A signed calendar/EMR webhook connector with idempotent appointment upserts.”

**Why this fails:** A reader must understand both `idempotent` and `upsert` to learn that duplicate updates do not duplicate appointments.

**Concrete fix:** “A signed calendar/EMR connection stores each appointment once, even when it receives the same update twice.”

### F-3-8 — Minor — “provider” is ambiguous and inconsistently qualified

**Location / quotes:** landing: “provider result”; README: “provider evidence”, “provider event”, “messaging provider”, “approved delivery providers”, “provider receipts”, and “Delivery-provider fees”.

**Why this fails:** In a clinic, `provider` normally means a clinician. The copy also alternates among bare `provider`, `messaging provider`, and `delivery provider` for the messaging service. A first-time reader can misread “provider result” as a clinical result.

**Concrete fix:** use `messaging provider` consistently: “messaging-provider result”, “messaging-provider evidence”, “approved messaging providers”, “messaging-provider receipts”, and “Messaging-provider fees are separate.”

## Copy audit

Counting method: whitespace-delimited words; routes, prices, and hyphenated terms count as one word; punctuation-only symbols do not count. Repeated navigation/footer labels are counted once. Commands and code blocks are excluded, while headings and action labels are included because they are part of the reader’s copy.

### Landing page

| Words | Copy | Result |
| ---: | --- | --- |
| 2 | Reminder Proof | brand label |
| 1 | Demo | navigation label |
| 2 | For clinics | navigation label |
| 3 | How it works | navigation label |
| 1 | Privacy | navigation label |
| 1 | Theme | control label |
| 1 | System | option label |
| 2 | Clinic daylight | option label |
| 2 | After hours | option label |
| 5 | Reminder proof for independent clinics | audience label |
| 4 | See every reminder outcome. | pass |
| 15 | For independent clinics that need delivery proof and a clear next step when reminders fail. | pass |
| 5 | Try it with sample data | result-naming action |
| 4 | Opens a sample clinic. | pass |
| 5 | Nothing touches real clinic data. | `demo-isolation` |
| 6 | Demo actions use sample data only. | `demo-isolation` |
| 5 | Reminder contents exclude clinical notes. | `minimal-reminder-content` |
| 7 | Clinic costs $79 per location each month. | `public-price` |
| 1 | Delivered | preview label |
| 2 | SMS · Simulated | preview label |
| 1 | Fallback | preview label |
| 2 | WhatsApp → Email | preview label |
| 2 | Needs owner | preview label |
| 2 | Consent blocked | preview label |
| 2 | Reminder evidence | descriptive heading |
| 7 | Follow one reminder from schedule to outcome. | pass |
| 14 | See the source, consent check, each attempt, provider result, and staff resolution in order. | **F-3-8** |
| 5 | How the sample clinic works | descriptive heading |
| 3 | Check consent first | descriptive heading |
| 11 | A blocked channel becomes an exception before any simulated provider attempt. | `consent-channel-guard`; **F-3-8** |
| 5 | Use the next allowed channel | descriptive heading |
| 14 | Reminder Proof tries a fallback only when consent and the clinic policy allow it. | `fallback-order` |
| 5 | Give every failure an owner | descriptive heading |
| 13 | Open sample exceptions stay in the queue until a staff member resolves them. | `sample-exception-visibility` |
| 3 | Limits and privacy | descriptive heading |
| 8 | This does not replace your calendar or EMR. | scope boundary |
| 11 | Reminder Proof stores no clinical notes and sends no marketing campaigns. | `managed-data-minimisation`; `no-marketing-campaigns` |
| 9 | The public demo stays separate from managed clinic data. | `demo-isolation` |
| 3 | Clinic plan price | descriptive heading |
| 1 | $79 | price |
| 4 | per location each month | price unit |
| 3 | Connect your clinic | result-naming action |
| 13 | Reminder Proof records delivery evidence and staff-owned exceptions around an existing clinic calendar. | product footer |
| 1 | Terms | footer link |
| 4 | Built by Param Factory | external link |
| 1 | managed-clinic-workflow | **F-3-2** |

No landing sentence exceeds 22 words. No banned marketing adjective appears. The two action labels name their result.

### README

| Words | Copy | Result |
| ---: | --- | --- |
| 2 | Reminder Proof | title |
| 10 | Reminder Proof records each appointment reminder outcome for independent clinics. | product definition |
| 11 | It shows source details, consent, provider evidence, and the staff owner. | **F-3-8** |
| 17 | It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record. | scope boundary |
| 4 | Try the public sandbox | **F-3-4** |
| 4 | Open `/?demo=1` or `/demo`. | instruction |
| 12 | The server creates a random, 24-hour sample workspace containing five fictional appointments. | demo contract |
| 21 | Its compact state stays in an HttpOnly, Secure browser cookie, so a restart or replica change does not lose the sample. | **F-3-5** |
| 6 | Every provider event is visibly simulated. | **F-3-8** |
| 13 | The demo never calls a messaging provider, checkout, account service, or clinic connector. | `demo-isolation`; **F-3-8** |
| 8 | Advance the sample reminders and inspect their evidence. | instruction |
| 14 | Assign or resolve the sample exception, undo a resolution, and reset the sample clinic. | instruction |
| 8 | “Start for real” opens the managed clinic workflow. | instruction |
| 21 | A clinic signs in through Sociobot Microsoft Entra, creates its workspace, connects a signed calendar feed, and configures approved delivery providers. | `managed-auth-storage`; **F-3-8** |
| 7 | Reminder Proof checks recorded consent before sending. | consent guard |
| 14 | It records provider receipts and opens a shared exception when delivery proof is missing. | `managed-provider-fallback-receipt`; **F-3-8** |
| 3 | What is included | descriptive heading |
| 12 | Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes. | inventory item |
| 9 | A same-origin service protects demo sessions and clinic data. | **F-3-6** |
| 9 | It includes rate limits, health checks, and machine-readable metrics. | `rate-limit-policy`; `build-identity` |
| 9 | A signed calendar/EMR webhook connector with idempotent appointment upserts. | `signed-calendar-intake`; **F-3-7** |
| 13 | Twilio SMS and approved WhatsApp dispatch, Resend email fallback, and signed receipt reconciliation. | provider claims; **F-3-8** |
| 17 | Shared exception assignment and resolution, clinic export/delete, and Sociobot-hosted subscription checkout at $79 per location each month. | declared feature coverage |
| 4 | Delivery-provider fees are separate. | **F-3-8** |
| 17 | Original hand-authored pulse-ledger art, favicon, touch icon, social card, and self-hosted Instrument Sans / Fragment Mono assets. | **F-3-1** |
| 11 | Playwright claim tests that begin with a fresh demo browser context. | test inventory |
| 2 | See `.factory/claims.json`. | instruction |
| 6 | The public demo is always simulated. | demo contract |
| 21 | Live dispatch begins only after a signed-in clinic supplies approved sender credentials, template IDs, consent evidence, and a webhook signing secret. | managed workflow boundary |
| 2 | Run locally | descriptive heading |
| 13 | Requirements: Node 22.12+, Rust stable with `rustfmt` and `clippy`, and Chromium for Playwright. | developer requirement |
| 10 | The API requires no configuration and uses `PORT` (default `8080`). | developer instruction |
| 18 | The single-replica SQLite writer runs below `DATA_DIR` (the image defaults to `/data`) instead of on an SMB mount. | operator detail |
| 12 | Each saved change writes a matching durable database and key under `DURABLE_DIR`. | `managed-storage-recovery` |
| 11 | A daily recovery copy is kept under `BACKUP_DIR` for 30 days. | `managed-storage-recovery` |
| 7 | Startup restores the durable pair before serving. | `managed-storage-recovery` |
| 9 | Entra tenant settings may override the documented Sociobot defaults. | developer instruction |
| 18 | The production container pins the app to one replica so SQLite and demo-creation limits have one state owner. | `single-replica-durable-topology` |
| 12 | The container mounts separate durable and backup shares at `/durable` and `/backups`. | `single-replica-durable-topology` |
| 6 | The application runs without root privileges. | deployment implementation |
| 10 | Recovery steps and the restore regression are documented in `.factory/operations.md`. | instruction |
| 14 | Register `https://clinic-reminder-proof.sociobot.in/auth/callback` on the shared Sociobot Entra SPA before sign-in is opened to clinics. | operator instruction |
| 3 | Clinic integration contract | descriptive heading |
| 8 | All clinic routes require an Entra bearer token. | `managed-auth-storage` |
| 13 | The stable `oid` claim owns the workspace; email is never an identity key. | `managed-auth-storage` |
| 7 | Create a signed calendar connector in `/app`. | instruction |
| 10 | Post normalized appointment batches to `/api/v1/connectors/intake` with `X-Reminder-Timestamp` and `X-Reminder-Signature`. | integration instruction |
| 7 | Sign the UTF-8 string `<timestamp>:<connector-id>:<appointment-count>` with HMAC-SHA256. | integration instruction |
| 8 | Encode the result as URL-safe base64 without padding. | integration instruction |
| 12 | Configure Twilio for SMS or approved WhatsApp templates, or Resend for email. | `approved-whatsapp-dispatch` |
| 8 | Credentials and patient destinations are encrypted at rest. | `managed-secret-encryption` |
| 13 | Twilio receives its status callback URL during dispatch and is verified with `X-Twilio-Signature`. | `twilio-receipt-verification` |
| 17 | Resend receipt callbacks use its Svix headers (`svix-id`, `svix-timestamp`, and `svix-signature`) and the stored `whsec_…` webhook secret. | `resend-receipt-verification` |
| 5 | Receipt event IDs are idempotent. | receipt-verification claims |
| 13 | A terminal failure tries the next recorded-consent channel; exhaustion opens a shared exception. | `managed-provider-fallback-receipt` |
| 12 | Reminder dispatch accepts one scheduled reminder ID and no client-supplied campaign copy. | `no-marketing-campaigns` |
| 19 | JSON API writes require `application/json`, accept at most 16 KB, and return structured errors with a correlatable request ID. | `request-protection` |
| 8 | Signed-in clinics can export their own minimized workspace. | `signed-in-export-delete` |
| 10 | Deletion requires the same clinic's organization ID as explicit confirmation. | `signed-in-export-delete` |
| 17 | The signed-in workspace requests checkout through the same-origin billing route, which returns only the Sociobot checkout URL. | `managed-billing-return` |
| 5 | No payment provider is embedded. | `managed-billing-return` |
| 1 | Verify | descriptive heading |
| 11 | Every `@claim:<id>` Playwright test is runnable on its own, for example: | instruction |
| 2 | Container deployment | descriptive heading |
| 21 | The multi-stage Dockerfile builds the web output and API without Git metadata, runs as a non-root user, and listens on `PORT`. | deployment implementation |
| 7 | The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. | deployment fact |
| 14 | Do not put provider keys, clinic data, payments, or Entra configuration in this repository. | security instruction |
| 3 | Privacy and terms | descriptive heading |
| 9 | The public pages are available at `/privacy` and `/terms`. | navigation instruction |
| 5 | The demo uses fictional aliases. | demo contract |
| 17 | Managed records contain only reminder operations data; clinics must not send clinical notes, diagnoses, or treatment details. | `managed-data-minimisation` |
| 1 | License | descriptive heading |
| 1 | MIT. | licence statement |
| 2 | See `LICENSE`. | instruction |

No README sentence exceeds 22 words and no banned marketing adjective appears. The integration and operator sections retain necessary protocol identifiers; the findings above cover jargon used in visitor-facing summaries.

### Terminology check

| Concept | Current words | Required single term |
| --- | --- | --- |
| One-click sample experience | demo, public sandbox | demo |
| External service that sends a reminder | provider, messaging provider, delivery provider | messaging provider |
| Scheduled patient communication | reminder | reminder |
| One sending request | attempt | attempt |
| Human-owned problem | exception | exception |
| Responsible staff member | owner | owner |
| Calendar or EMR input | source | source |
| Next allowed channel | fallback | fallback |

## Demo and sandbox behaviour

- One click on `Try it with sample data` opened `/demo` and immediately showed Northline Sample Clinic, five fictional appointments, current outcomes, and an exception queue.
- The persistent banner said: “Demo — sample data, nothing is saved to your clinic.” It included `Reset demo` and `Start for real`.
- Advancing due reminders changed the live ledger. Reset sent `DELETE /api/v1/demo/workspaces`; a subsequent authenticated read returned the original seed: Mina and Jordan scheduled, Sofia’s exception open and unassigned, Eli delivered, and Noor cancelled.
- The live demo used an HttpOnly, Secure, SameSite=Lax cookie scoped to `/api/v1/demo`, with a 24-hour maximum age. Its browser state used only `demo:clinic-reminder-proof:<workspace-id>:active` in session storage.
- Sentinel non-demo keys placed in local and session storage remained unchanged after entering, advancing, and resetting the demo.
- The landing-to-demo request log contained only `clinic-reminder-proof.sociobot.in`. No messaging, checkout, analytics, font CDN, or other third-party request occurred.
- The product makes no public offline-capability claim. The complete local browser suite nevertheless confirmed cached demo reads while write actions remain unavailable offline.

The demo gate passes.

## Claims

All 31 exact `test` commands from `.factory/claims.json` were run independently in fresh clone `/tmp/clinic-reminder-proof-review3-clean.w8l3rZ`. All passed.

| Claim | Result | Claim | Result |
| --- | --- | --- | --- |
| `demo-isolation` | PASS | `sample-outcome-coverage` | PASS |
| `consent-channel-guard` | PASS | `fallback-order` | PASS |
| `delivery-timeline` | PASS | `exception-ownership` | PASS |
| `sample-exception-visibility` | PASS | `demo-reset` | PASS |
| `minimal-reminder-content` | PASS | `public-price` | PASS |
| `demo-cookie-lifetime` | PASS | `demo-replica-continuity` | PASS |
| `no-tracking` | PASS | `explicit-theme-choice` | PASS |
| `request-protection` | PASS | `rate-limit-policy` | PASS |
| `security-headers` | PASS | `build-identity` | PASS |
| `managed-auth-storage` | PASS | `signed-calendar-intake` | PASS |
| `approved-whatsapp-dispatch` | PASS | `twilio-receipt-verification` | PASS |
| `resend-receipt-verification` | PASS | `managed-secret-encryption` | PASS |
| `managed-data-minimisation` | PASS | `no-marketing-campaigns` | PASS |
| `signed-in-export-delete` | PASS | `managed-provider-fallback-receipt` | PASS |
| `managed-billing-return` | PASS | `managed-storage-recovery` | PASS |
| `single-replica-durable-topology` | PASS |  |  |

The landing and README cross-check found one unlisted claim: **F-3-1**. No declared claim was left untested.

## History recheck

Every earlier `.factory/review-*.md`, `.factory/polish-*.md`, `.factory/handoff.md`, and `.factory/handoff-m1.md` was read. Each earlier finding was checked in current live output and current code, not only in its closure note.

| Earlier finding | Current verification |
| --- | --- |
| F-1-1 — slogan-like section labels | Fixed. Live headings are `Reminder evidence`, `Limits and privacy`, and `Clinic plan price`; current Svelte source matches. |
| F-1-2 — exception visibility unlisted | Fixed. `sample-exception-visibility` is declared; its exact command and the full lifecycle suite passed. |
| F-1-3 — long README opening | Fixed. It is split into 10- and 11-word sentences. |
| F-1-4 — long demo-action list | Fixed. It is split into 8- and 14-word sentences. |
| F-1-5 — long managed-workflow sentence | Fixed. Consent and receipt/exception behavior are separate sentences. |
| F-1-6 — long jargon-heavy API inventory | The length defect is fixed. The remaining `same-origin` jargon is newly recorded as F-3-6. |
| F-1-7 — long recovery sentence | Fixed. Durable save and 30-day recovery are separate sentences; the recovery claim passed. |
| F-1-8 — long storage-topology sentence | Fixed. Mounts and non-root execution are separate sentences; the topology claim passed. |
| F-2-1 — signed intake/idempotency unlisted | Fixed in code and manifest. `signed-calendar-intake` passed duplicate-upsert and altered-signature checks. |
| F-2-2 — approved WhatsApp unlisted | Fixed in code and manifest. `approved-whatsapp-dispatch` passed the exact-template and signed-receipt checks. |
| F-2-3 — callback verification unlisted | Fixed in code and manifest. Both Twilio and Resend signature/replay claims passed. |
| F-2-4 — secret encryption unlisted | Fixed in code and manifest. `managed-secret-encryption` passed local/durable store, export, and adapter-scope checks. |
| F-2-5 — managed minimisation unlisted | Fixed in code and manifest. `managed-data-minimisation` passed unknown-field rejection, no-write, and export checks. |

The previous handoff’s deployment concern remains closed: live `/health` reports build `741bba6…`, and repository changes after that build are documentation only. No earlier finding regressed.

## Structure, accessibility, and visual identity

- `/`, `/demo`, `/demo/reminders/mina`, `/start`, `/privacy`, `/terms`, and `/404` have route-specific titles and canonical URLs, one H1, one main landmark, descriptions, Open Graph/Twitter metadata, favicon, touch icon, and consistent navigation/footer. An unknown path returns the designed page with HTTP 404.
- Keyboard entry to Mina’s evidence route moved focus to its H1. Browser Back returned to `/demo` and restored focus to `Today’s sample reminders`.
- The landing crawl found no dead link: the home, demo, start, how-it-works anchor, Privacy, Terms, and Sociobot links all returned 200.
- `robots.txt`, `sitemap.xml`, favicon, touch icon, and the 1200 × 630 social card returned 200. The sitemap includes the public and managed routes.
- `/opt/fleet/lib/verify-url.sh` passed on the live origin with one H1, `lang=en`, a main landmark, no missing alt text, no unlabeled button, and no console error.
- `@axe-core/cli` 4.11.4 reported zero violations on the live landing page. The repository’s full browser suite reported no serious/critical axe issue across public, demo, legal, start, evidence, and 404 views.
- Focus rings, 44 px controls, light/dark contrast contracts, reduced-motion CSS, 390 px reflow, and 200% text reflow passed the local suite.
- The asymmetric mineral ledger, clipped evidence planes, status shapes, mono evidence type, and cyan/amber pulse treatment match `.factory/design.md`. It is visually distinct from a generic centered-hero/three-card SaaS template.
- Response headers include CSP with header-delivered `frame-ancestors 'none'`, HSTS, `nosniff`, and strict-origin referrer policy.

The structural exceptions are **F-3-2** and **F-3-3**.

## Quality gates

From the same fresh clone:

- `npm test`: PASS — 8 Vitest, 33 Rust, and 39 Chromium tests.
- `npm run check`: PASS — Svelte 0 errors/0 warnings, rustfmt clean, Clippy warnings denied.
- `npm run build`: PASS — `dist/` and the release API binary produced.
- Public application JavaScript: 28.58 KB gzip; lazy authentication chunk: 68.23 KB gzip.

## Missed leverage

No finding. The brief’s obvious extensions are already present: signed calendar/EMR intake, consent-aware SMS/email/approved-WhatsApp fallback, receipt reconciliation, staff exception ownership, clinic export/delete, and managed billing. An AI step is not implied by this reliability workflow and would be decorative.

## What would make this perfect

Resolve F-3-1 through F-3-8: remove or register the originality claim, display a real build identifier, replace the 404 metaphor, and make demo/messaging-provider language consistent and plain. Then rerun all 31 claim commands and the full cold live review. Zero findings are required for PASS.
