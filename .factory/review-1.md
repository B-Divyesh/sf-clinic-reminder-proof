# Adversarial first-read review 1 — FAIL

Date: 2026-08-29 UTC
Live URL: <https://clinic-reminder-proof.sociobot.in>
Reviewed candidate: live `b8ef87c632ac95f5ef2d41dec162cba7639eddd5`; local review base `68acd295e3a6bc49da27e2efd8a27fa53ae9653d`.

## Verdict

**FAIL.** The first-read, sample-demo, route, privacy-request, accessibility, claims-command, build, and historic-regression checks pass. The remaining findings are copy and claims-contract defects. A PASS requires zero findings, including minor ones.

## Cold first read

Fresh Chromium contexts were opened at 390 × 844 and 1440 × 900 before scrolling. Both first screens stated:

- **What it does:** “See every reminder outcome.”
- **For whom:** “For independent clinics that need delivery proof and a clear next step when reminders fail.”
- **First action:** **Try it with sample data**; adjacent text says, “Opens a sample clinic. Nothing touches real clinic data.”

This gate passes. The first click went directly to `/demo`; its first rendered screen already showed Northline Sample Clinic, five realistic fictional appointments, delivery states, the exception queue, and the persistent “Demo — sample data, nothing is saved to your clinic.” banner. Reset restored the seed. The isolated `rp_demo` cookie was HttpOnly, Secure, SameSite=Lax, scoped to `/api/v1/demo`, and had a 24-hour expiry. The normal landing-to-demo request log contained only the product origin.

## Findings

### F-1-1 — Medium — landing section labels do not name their sections

**Location / quote:** landing eyebrow and headings: “A proof ledger, not another calendar”; “Follow one reminder from schedule to outcome.”; “Plain boundaries”; “This does not replace your calendar or EMR.”; “Monthly plan”; “One clear clinic price.”

**Why this fails:** Several headings are slogans or assertions rather than names a screen-reader user can use to identify a section. “Plain boundaries” gives no useful topic; “One clear clinic price” uses an unsupported marketing adjective and is not a section name. This conflicts with the plain-words and site-structure requirement that headings name their section out of context.

**Concrete fix:** use `Reminder evidence` as the evidence section heading and retain “Follow one reminder from schedule to outcome.” as its lead; use `Limits and privacy` for the boundary section; use `Clinic plan price` for pricing. Remove the decorative eyebrow copies, or make them concise, informative labels.

### F-1-2 — Medium — unresolved-exception visibility is an unlisted landing claim

**Location / quote:** landing “How the sample clinic works” step 3: “Blocked and exhausted reminders stay visible until a staff member resolves them.”

**Why this fails:** This is a visitor-reliant behavioural promise. No `.factory/claims.json` entry claims or tests that a blocked *and exhausted* reminder remains visible until resolution. `exception-ownership` proves assignment, resolution, reload, and undo for the seeded blocked exception; it does not prove the stated visibility lifecycle or an exhausted state.

**Concrete fix:** add a `sample-exception-visibility` claim with an `@claim:sample-exception-visibility` test. From a fresh demo, create/advance both relevant sample paths, assert their ledger/exception rows remain present before resolution, then resolve one and assert the exact intended post-resolution presentation. Alternatively remove this promise.

### F-1-3 — Minor — README opening sentence exceeds the 22-word cap

**Location / quote:** README opening, 31 words: “Reminder Proof gives independent clinic teams a clear proof trail for each appointment reminder: what the source said, what consent allowed, what a provider returned, and which person owns a problem.”

**Why this fails:** It carries four separate ideas and is over the hard plain-words limit.

**Concrete fix:** “Reminder Proof records each appointment reminder outcome for independent clinics. It shows source details, consent, provider evidence, and the staff owner.”

### F-1-4 — Minor — README demo-action sentence exceeds the 22-word cap

**Location / quote:** README “Try the public sandbox”, 33 words: “You can advance sample reminders, inspect provider evidence, see a consent block, follow a simulated WhatsApp-to-email fallback, assign and resolve the sample exception, undo the safe resolution, and reset the whole sample clinic.”

**Why this fails:** The long action list is difficult to scan and combines seven actions.

**Concrete fix:** “Advance the sample reminders and inspect their evidence. Assign or resolve the sample exception, undo a resolution, and reset the sample clinic.”

### F-1-5 — Minor — README managed-workflow sentence exceeds the 22-word cap

**Location / quote:** README “Try the public sandbox”, 27 words: “Reminder Proof checks recorded consent in policy order, sends only an approved template, ingests signed provider receipts, and opens a shared staff exception when proof is missing.”

**Why this fails:** It combines policy, dispatch, receipt ingestion, and exception handling in one sentence.

**Concrete fix:** “Reminder Proof checks recorded consent before sending. It records provider receipts and opens a shared exception when delivery proof is missing.”

### F-1-6 — Minor — README implementation bullet exceeds the 22-word cap and uses unexplained jargon

**Location / quote:** README “What is included”, 23 words: “A Rust/axum same-origin API with isolated demo cookies, Entra JWT validation, encrypted durable clinic data, rate limits, security headers, /health, and /metrics.”

**Why this fails:** This is a sentence fragment with stack jargon (`Rust/axum`, `JWT`) and an unscannable feature list. README is in scope for plain words.

**Concrete fix:** “A same-origin service protects demo sessions and clinic data. It includes rate limits, health checks, and machine-readable metrics.” Put implementation terms in a separate developer architecture section.

### F-1-7 — Minor — README recovery sentence exceeds the 22-word cap

**Location / quote:** README “Run locally”, 31 words: “Each acknowledged workspace mutation synchronously checkpoints a consistent database and matching key below DURABLE_DIR (default /durable), then writes a daily recovery pair below BACKUP_DIR (default /backups) with 30-day retention.”

**Why this fails:** The reader must parse persistence, key matching, paths, backup cadence, and retention at once.

**Concrete fix:** “Each saved change writes a matching durable database and key under `DURABLE_DIR`. A daily recovery copy is kept under `BACKUP_DIR` for 30 days.”

### F-1-8 — Minor — README storage-topology sentence exceeds the 22-word cap

**Location / quote:** README “Run locally”, 28 words: “Separate durable Azure Files shares mount directly at /durable and /backups; the non-root process creates and updates snapshots without a privileged init container or running SQLite over SMB.”

**Why this fails:** It combines deployment topology, permissions, snapshot behaviour, and an implementation constraint.

**Concrete fix:** “The container mounts separate durable and backup shares at `/durable` and `/backups`. The application runs without root privileges.” Move the SQLite/SMB rationale to the operations guide.

## Copy audit

Word counts treat a number, route, and price as one token. Buttons, labels, and headings are included because they are landing copy; code blocks and command examples are excluded from the README sentence list. No landing sentence exceeds 22 words. The flagged items above are marked by finding ID.

### Landing page

| Words | Copy | Result |
| ---: | --- | --- |
| 5 | Reminder proof for independent clinics | label; useful audience context |
| 4 | See every reminder outcome. | pass |
| 15 | For independent clinics that need delivery proof and a clear next step when reminders fail. | pass |
| 4 | Opens a sample clinic. | pass |
| 5 | Nothing touches real clinic data. | pass; `demo-isolation` |
| 6 | Demo actions use sample data only. | pass; `demo-isolation` |
| 5 | Reminder contents exclude clinical notes. | pass; `minimal-reminder-content` |
| 7 | Clinic costs $79 per location each month. | pass; `public-price` |
| 6 | A proof ledger, not another calendar | F-1-1 heading/eyebrow |
| 7 | Follow one reminder from schedule to outcome. | F-1-1 heading |
| 14 | See the source, consent check, each attempt, provider result, and staff resolution in order. | pass; delivery timeline coverage |
| 5 | How the sample clinic works | pass |
| 3 | Check consent first | pass |
| 11 | A blocked channel becomes an exception before any simulated provider attempt. | pass; `consent-channel-guard` |
| 5 | Use the next allowed channel | pass |
| 14 | Reminder Proof tries a fallback only when consent and the clinic policy allow it. | pass; `fallback-order` |
| 5 | Give every failure an owner | pass |
| 12 | Blocked and exhausted reminders stay visible until a staff member resolves them. | F-1-2 |
| 2 | Plain boundaries | F-1-1 heading/eyebrow |
| 8 | This does not replace your calendar or EMR. | F-1-1 heading |
| 11 | Reminder Proof stores no clinical notes and sends no marketing campaigns. | pass; boundary plus `no-marketing-campaigns` |
| 9 | The public demo stays separate from managed clinic data. | pass; `demo-isolation` |
| 2 | Monthly plan | F-1-1 heading/eyebrow |
| 4 | One clear clinic price. | F-1-1 heading/marketing adjective |
| 3 | Connect your clinic | pass; result-naming verb |

### README prose

| Words | Sentence | Result |
| ---: | --- | --- |
| 31 | Reminder Proof gives independent clinic teams a clear proof trail for each appointment reminder: what the source said, what consent allowed, what a provider returned, and which person owns a problem. | F-1-3 |
| 17 | It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record. | pass |
| 3 | Open `/?demo=1` or `/demo`. | pass |
| 12 | The server creates a random, 24-hour sample workspace containing five fictional appointments. | pass; demo contract |
| 21 | Its compact state stays in an HttpOnly, Secure browser cookie, so a restart or replica change does not lose the sample. | pass; `demo-cookie-lifetime`, `demo-replica-continuity` |
| 6 | Every provider event is visibly simulated. | pass |
| 13 | The demo never calls a messaging provider, checkout, account service, or clinic connector. | pass; `demo-isolation` |
| 33 | You can advance sample reminders, inspect provider evidence, see a consent block, follow a simulated WhatsApp-to-email fallback, assign and resolve the sample exception, undo the safe resolution, and reset the whole sample clinic. | F-1-4 |
| 8 | “Start for real” opens the managed clinic workflow. | pass |
| 21 | A clinic signs in through Sociobot Microsoft Entra, creates its workspace, connects a signed calendar feed, and configures approved delivery providers. | pass; managed-auth contract |
| 27 | Reminder Proof checks recorded consent in policy order, sends only an approved template, ingests signed provider receipts, and opens a shared staff exception when proof is missing. | F-1-5 |
| 12 | Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes. | fragment; pass as inventory item |
| 23 | A Rust/axum same-origin API with isolated demo cookies, Entra JWT validation, encrypted durable clinic data, rate limits, security headers, `/health`, and `/metrics`. | F-1-6 |
| 10 | A signed calendar/EMR webhook connector with idempotent appointment upserts. | fragment; pass as inventory item |
| 13 | Twilio SMS and approved WhatsApp dispatch, Resend email fallback, and signed receipt reconciliation. | fragment; pass as inventory item |
| 18 | Shared exception assignment and resolution, clinic export/delete, and Sociobot-hosted subscription checkout at $79 per location each month. | fragment; pass as inventory item |
| 4 | Delivery-provider fees are separate. | pass |
| 16 | Original hand-authored pulse-ledger art, favicon, touch icon, social card, and self-hosted Instrument Sans / Fragment Mono assets. | fragment; pass as inventory item |
| 11 | Playwright claim tests that begin with a fresh demo browser context. | fragment; pass as inventory item |
| 6 | The public demo is always simulated. | pass |
| 21 | Live dispatch begins only after a signed-in clinic supplies approved sender credentials, template IDs, consent evidence, and a webhook signing secret. | pass; managed boundary |
| 12 | The API requires no configuration and uses `PORT` (default `8080`). | pass |
| 19 | The single-replica SQLite writer runs below `DATA_DIR` (the image defaults to `/data`) instead of on an SMB mount. | pass; topology claim |
| 31 | Each acknowledged workspace mutation synchronously checkpoints a consistent database and matching key below `DURABLE_DIR` (default `/durable`), then writes a daily recovery pair below `BACKUP_DIR` (default `/backups`) with 30-day retention. | F-1-7 |
| 7 | Startup restores the durable pair before serving. | pass; storage-recovery claim |
| 9 | Entra tenant settings may override the documented Sociobot defaults. | pass |
| 18 | The production container pins the app to one replica so SQLite and demo-creation limits have one state owner. | pass; topology claim |
| 28 | Separate durable Azure Files shares mount directly at `/durable` and `/backups`; the non-root process creates and updates snapshots without a privileged init container or running SQLite over SMB. | F-1-8 |
| 9 | Recovery steps and the restore regression are documented in `.factory/operations.md`. | pass |
| 15 | Register `https://clinic-reminder-proof.sociobot.in/auth/callback` on the shared Sociobot Entra SPA before sign-in is opened to clinics. | pass |
| 8 | All clinic routes require an Entra bearer token. | pass |
| 13 | The stable `oid` claim owns the workspace; email is never an identity key. | pass |
| 7 | Create a signed calendar connector in `/app`. | pass |
| 13 | Post normalized appointment batches to `/api/v1/connectors/intake` with `X-Reminder-Timestamp` and `X-Reminder-Signature`. | pass |
| 9 | Sign the UTF-8 string `<timestamp>:<connector-id>:<appointment-count>` with HMAC-SHA256. | pass |
| 8 | Encode the result as URL-safe base64 without padding. | pass |
| 12 | Configure Twilio for SMS or approved WhatsApp templates, or Resend for email. | pass |
| 8 | Credentials and patient destinations are encrypted at rest. | pass |
| 13 | Twilio receives its status callback URL during dispatch and is verified with `X-Twilio-Signature`. | pass |
| 17 | Resend receipt callbacks use its Svix headers and the stored webhook secret. | pass |
| 5 | Receipt event IDs are idempotent. | pass |
| 13 | A terminal failure tries the next recorded-consent channel; exhaustion opens a shared exception. | pass; managed fallback claim |
| 12 | Reminder dispatch accepts one scheduled reminder ID and no client-supplied campaign copy. | pass; `no-marketing-campaigns` |
| 20 | JSON API writes require `application/json`, accept at most 16 KB, and return structured errors with a correlatable request ID. | pass; `request-protection` |
| 8 | Signed-in clinics can export their own minimized workspace. | pass; `signed-in-export-delete` |
| 10 | Deletion requires the same clinic's organization ID as explicit confirmation. | pass; `signed-in-export-delete` |
| 17 | The signed-in workspace requests checkout through the same-origin billing route, which returns only the Sociobot checkout URL. | pass; managed billing claim |
| 5 | No payment provider is embedded. | pass |
| 11 | Every `@claim:<id>` Playwright test is runnable on its own, for example. | pass |
| 17 | The multi-stage Dockerfile builds the web output and API without Git metadata, runs as a non-root user, and listens on `PORT`. | pass |
| 11 | The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. | pass |
| 14 | Do not put provider keys, clinic data, payments, or Entra configuration in this repository. | pass |
| 9 | The public pages are available at `/privacy` and `/terms`. | pass |
| 5 | The demo uses fictional aliases. | pass |
| 17 | Managed records contain only reminder operations data; clinics must not send clinical notes, diagnoses, or treatment details. | pass |
| 2 | See `LICENSE`. | pass |

## Claims, sandbox, and quality gates

- Read all 24 entries in `.factory/claims.json`. The exact declared commands were run from this clean installed checkout; all completed successfully. The complete `npm test` repeat passed 6 Vitest tests, 27 Rust tests, and 31 Playwright tests, including every declared `@claim:` tag.
- `npm run check` passed: Svelte 0 errors / 0 warnings, rustfmt clean, clippy warnings denied.
- `npm run build` passed and produced `dist/` and the release API binary. Public entry JS is 28.34 KB gzip; the lazy authentication chunk is 68.23 KB gzip.
- Normal browser request logging from landing through demo showed only same-origin assets and demo API requests. There are no third-party runtime requests or analytics requests. The product makes no offline claim, so no unsupported offline promise is present.
- Demo entry, reset, advance, assign, resolve, undo, and direct `?demo=1` use the sample scope. The publicly visible cookie and all observed request URLs were demo-scoped; claim coverage additionally verifies isolation and reset behaviour.

## Structure and interaction checks

- `/`, `/demo`, `/start`, `/privacy`, `/terms`, `/404`, and each sample reminder deep link loaded with route-specific titles, one `<h1>`, one `<main>`, one description meta element, canonical and Open Graph image metadata, favicon assets, header/footer, and no normal-flow console errors.
- All local links discovered from the demo and public pages returned 200. `https://sociobot.in/` returned 200. An unknown live path returned the designed page with HTTP 404 and a way home.
- Back/deep-link/focus, keyboard, 390 px layout, 200% text, reduced motion, axe serious/critical scan, and console/link tests pass in the repository suite. The visual treatment is recognisably the documented translucent pulse ledger rather than a generic SaaS card layout.

## Earlier verification and handoff findings

All earlier `.factory/verification*.md` and handoff records were read. Their historical blockers were rechecked against live behaviour and code/test coverage:

| Earlier concern | Recheck result |
| --- | --- |
| simulated-only M1, no managed path | fixed: `/start`, managed API surface, fixture-backed Entra/storage/dispatch/billing claims present |
| subscription and checkout proof | fixed in fixture-backed `managed-billing-return`; live start correctly requires sign-in before checkout |
| replica-local rate limits / missing Retry-After | fixed by one-replica topology and passing rate-limit-policy/topology tests |
| durability, recovery, file permissions | fixed by durable/backup topology plus storage-recovery tests |
| missing headers, cache policy, metrics, 404 status | fixed; security/build identity claims and live unknown-route probe pass |
| metadata, touch targets, skip-link/focus, race concerns | fixed by current browser accessibility/route suite |
| missing explicit theme choice and duplicate description | fixed by `explicit-theme-choice` test and one live description meta per route |
| previous handoff’s low-severity deliberate 429 console entry | not a normal-flow regression; fresh ordinary routes are console-clean |

## What would make this perfect

Resolve the listed findings: use descriptive landing section headings, add a direct claim/test for the remaining landing behavioural promise, and split the flagged README sentences into plain, single-purpose wording. Then rerun the full review from a new browser context and clean checkout.
