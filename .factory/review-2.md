# Adversarial first-read review 2 — FAIL

Date: 2026-08-29 UTC
Live URL: <https://clinic-reminder-proof.sociobot.in>
Reviewed commit: `ee61fd1af77a8c855447911e30d75a640d3b5e06`

## Verdict

**FAIL.** The cold first-read gate, one-click demo, stated claim commands, demo isolation checks, routing, accessibility smoke checks, and earlier-finding recheck all pass. Five README promises are not listed as claims in `.factory/claims.json`. The claims contract requires each visitor-reliant promise to have its own declared, observable test; related fixture coverage is not a substitute for registering the promise. There are zero other findings.

## Cold first read

Fresh Chromium contexts opened the production origin before scrolling at 390 × 844 and 1440 × 900.

| Question | What the first screen says |
| --- | --- |
| What does it do? | “See every reminder outcome.” |
| For whom? | “For independent clinics that need delivery proof and a clear next step when reminders fail.” |
| What should I click? | “Try it with sample data” — “Opens a sample clinic. Nothing touches real clinic data.” |

This gate passes on both sizes. The mobile action is visible and has a 44 px-or-larger target. The desktop and phone had no load-time console errors or third-party requests.

## Findings

### F-2-1 — Medium — source connector and idempotent intake are unlisted README claims

**Location / quote:** README, “What is included”: “A signed calendar/EMR webhook connector with idempotent appointment upserts.”

**Why this fails:** A clinic can rely on this as the advertised way to connect its calendar. No `claims.json` entry names that connector contract or idempotency. `managed-auth-storage` happens to post a fixture batch while proving tenancy, but its declared claim is about authenticated durable storage, not connector authenticity or duplicate handling.

**Concrete fix:** add a `signed-calendar-intake` claim and `@claim:signed-calendar-intake` test. From a fresh signed-in fixture, send one correctly signed normalized batch twice, verify one source/reminder set exists, then send an invalid signature and verify rejection. Alternatively remove the connector promise from the README.

### F-2-2 — Medium — approved WhatsApp delivery is an unlisted capability claim

**Location / quote:** README, “What is included”: “Twilio SMS and approved WhatsApp dispatch, Resend email fallback, and signed receipt reconciliation.”

**Why this fails:** The declared `managed-provider-fallback-receipt` claim proves fixture SMS rejection followed by fixture email acceptance. It does not declare or test the separately advertised approved-WhatsApp path.

**Concrete fix:** add `approved-whatsapp-dispatch` with an `@claim:approved-whatsapp-dispatch` fixture test. Configure an approved WhatsApp template, dispatch one consented reminder, and assert the exact template-only request and stored signed receipt. Or change the sentence to name only the channels that are declared and tested.

### F-2-3 — Medium — provider-callback verification promises have no declared proof

**Location / quote:** README, “Clinic integration contract”: “Twilio receives its status callback URL during dispatch and is verified with `X-Twilio-Signature`. Resend receipt callbacks use its Svix headers (`svix-id`, `svix-timestamp`, and `svix-signature`) and the stored `whsec_…` webhook secret.”

**Why this fails:** These are security promises a clinic can rely on when exposing provider callbacks. No claims entry or named test declares signature validation, replay/idempotency, or rejection of an invalid Twilio/Svix signature.

**Concrete fix:** add separate `twilio-receipt-verification` and `resend-receipt-verification` entries. Their tests should submit valid signed callbacks, repeat a callback ID, and submit altered signatures; assert one accepted receipt and no state change from invalid/replayed events. If unsupported, delete the verification wording.

### F-2-4 — Medium — “credentials are encrypted at rest” is an unlisted privacy claim

**Location / quote:** README, “Clinic integration contract”: “Credentials and patient destinations are encrypted at rest.”

**Why this fails:** This is a direct privacy/security assurance. `managed-auth-storage` says a workspace is encrypted in its claim text, but it does not list provider credentials or patient destinations, nor does its declared sandbox assert that database values are unreadable ciphertext and are only decrypted for the owning clinic’s dispatch path.

**Concrete fix:** add `managed-secret-encryption` and a deterministic store-level test that saves fixture provider credentials and a destination, asserts plaintext is absent from the durable backing store/export, then proves an authorized dispatch adapter receives the decrypted value. Otherwise remove “encrypted at rest” from public README copy.

### F-2-5 — Medium — managed-data minimisation is an unlisted privacy claim

**Location / quote:** README, “Privacy and terms”: “Managed records contain only reminder operations data; clinics must not send clinical notes, diagnoses, or treatment details.”

**Why this fails:** `minimal-reminder-content` is explicitly limited to **sample** reminder contents. The live privacy statement makes the stronger managed-workspace promise without a claim that rejects forbidden fields at all managed write boundaries or proves they are absent from export/storage.

**Concrete fix:** add `managed-data-minimisation` with a fixture signed-in clinic test. Attempt to submit clinical-note, diagnosis, and treatment fields through intake and all available writes; assert validation rejects them and export/storage contain none. Or qualify the wording as a clinic policy rather than a product-enforced guarantee.

## Copy audit

Counting method: whitespace-delimited words; prices, routes, and hyphenated terms count as one word. Headings and controls are included because they are visitor-facing copy. No landing or README sentence exceeds 22 words. No landing heading is a mood/metaphor heading, no landing control uses a non-result verb, and no banned marketing term appears.

### Landing page

| Words | Copy | Result |
| ---: | --- | --- |
| 5 | Reminder proof for independent clinics | audience label |
| 4 | See every reminder outcome. | pass |
| 15 | For independent clinics that need delivery proof and a clear next step when reminders fail. | pass |
| 5 | Try it with sample data | result-naming action |
| 4 | Opens a sample clinic. | pass; demo entry |
| 5 | Nothing touches real clinic data. | `demo-isolation` |
| 6 | Demo actions use sample data only. | `demo-isolation` |
| 5 | Reminder contents exclude clinical notes. | `minimal-reminder-content` |
| 7 | Clinic costs $79 per location each month. | `public-price` |
| 2 | Reminder evidence | descriptive heading |
| 7 | Follow one reminder from schedule to outcome. | pass |
| 14 | See the source, consent check, each attempt, provider result, and staff resolution in order. | `delivery-timeline` |
| 6 | How the sample clinic works | descriptive heading |
| 3 | Check consent first | descriptive step heading |
| 11 | A blocked channel becomes an exception before any simulated provider attempt. | `consent-channel-guard` |
| 5 | Use the next allowed channel | descriptive step heading |
| 14 | Reminder Proof tries a fallback only when consent and the clinic policy allow it. | `fallback-order` |
| 5 | Give every failure an owner | descriptive step heading |
| 12 | Open sample exceptions stay in the queue until a staff member resolves them. | `sample-exception-visibility` |
| 3 | Limits and privacy | descriptive heading |
| 8 | This does not replace your calendar or EMR. | scope boundary |
| 10 | Reminder Proof stores no clinical notes and sends no marketing campaigns. | `minimal-reminder-content`; `no-marketing-campaigns` |
| 9 | The public demo stays separate from managed clinic data. | `demo-isolation` |
| 3 | Clinic plan price | descriptive heading |
| 3 | Connect your clinic | result-naming action |

### README

The table includes every prose sentence and visitor-facing inventory item; shell commands and code examples are excluded.

| Words | Copy | Result |
| ---: | --- | --- |
| 9 | Reminder Proof records each appointment reminder outcome for independent clinics. | product definition |
| 10 | It shows source details, consent, provider evidence, and the staff owner. | product definition |
| 17 | It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record. | scope boundary |
| 3 | Open `/?demo=1` or `/demo`. | instruction |
| 12 | The server creates a random, 24-hour sample workspace containing five fictional appointments. | demo contract |
| 21 | Its compact state stays in an HttpOnly, Secure browser cookie, so a restart or replica change does not lose the sample. | `demo-cookie-lifetime`; `demo-replica-continuity` |
| 6 | Every provider event is visibly simulated. | demo contract |
| 13 | The demo never calls a messaging provider, checkout, account service, or clinic connector. | `demo-isolation` |
| 7 | Advance the sample reminders and inspect their evidence. | instruction |
| 12 | Assign or resolve the sample exception, undo a resolution, and reset the sample clinic. | instruction |
| 8 | “Start for real” opens the managed clinic workflow. | instruction |
| 21 | A clinic signs in through Sociobot Microsoft Entra, creates its workspace, connects a signed calendar feed, and configures approved delivery providers. | `managed-auth-storage` |
| 7 | Reminder Proof checks recorded consent before sending. | consent guard |
| 13 | It records provider receipts and opens a shared exception when delivery proof is missing. | `managed-provider-fallback-receipt` |
| 10 | Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes. | inventory item |
| 9 | A same-origin service protects demo sessions and clinic data. | architecture label |
| 9 | It includes rate limits, health checks, and machine-readable metrics. | `rate-limit-policy`; `build-identity` |
| 10 | A signed calendar/EMR webhook connector with idempotent appointment upserts. | **F-2-1** |
| 11 | Twilio SMS and approved WhatsApp dispatch, Resend email fallback, and signed receipt reconciliation. | **F-2-2** |
| 18 | Shared exception assignment and resolution, clinic export/delete, and Sociobot-hosted subscription checkout at $79 per location each month. | declared feature coverage |
| 4 | Delivery-provider fees are separate. | price qualification |
| 16 | Original hand-authored pulse-ledger art, favicon, touch icon, social card, and self-hosted Instrument Sans / Fragment Mono assets. | asset provenance |
| 11 | Playwright claim tests that begin with a fresh demo browser context. | test inventory |
| 6 | The public demo is always simulated. | demo contract |
| 21 | Live dispatch begins only after a signed-in clinic supplies approved sender credentials, template IDs, consent evidence, and a webhook signing secret. | managed workflow boundary |
| 12 | The API requires no configuration and uses `PORT` (default `8080`). | developer instruction |
| 19 | The single-replica SQLite writer runs below `DATA_DIR` (the image defaults to `/data`) instead of on an SMB mount. | `single-replica-durable-topology` |
| 12 | Each saved change writes a matching durable database and key under `DURABLE_DIR`. | `managed-storage-recovery` |
| 11 | A daily recovery copy is kept under `BACKUP_DIR` for 30 days. | `managed-storage-recovery` |
| 7 | Startup restores the durable pair before serving. | `managed-storage-recovery` |
| 9 | Entra tenant settings may override the documented Sociobot defaults. | developer instruction |
| 18 | The production container pins the app to one replica so SQLite and demo-creation limits have one state owner. | `single-replica-durable-topology` |
| 11 | The container mounts separate durable and backup shares at `/durable` and `/backups`. | `single-replica-durable-topology` |
| 6 | The application runs without root privileges. | deployment implementation |
| 9 | Recovery steps and the restore regression are documented in `.factory/operations.md`. | instruction |
| 8 | All clinic routes require an Entra bearer token. | `managed-auth-storage` |
| 13 | The stable `oid` claim owns the workspace; email is never an identity key. | `managed-auth-storage` |
| 7 | Create a signed calendar connector in `/app`. | instruction |
| 12 | Configure Twilio for SMS or approved WhatsApp templates, or Resend for email. | **F-2-2** |
| 8 | Credentials and patient destinations are encrypted at rest. | **F-2-4** |
| 13 | Twilio receives its status callback URL during dispatch and is verified with `X-Twilio-Signature`. | **F-2-3** |
| 17 | Resend receipt callbacks use its Svix headers and the stored `whsec_…` webhook secret. | **F-2-3** |
| 5 | Receipt event IDs are idempotent. | **F-2-3** |
| 13 | A terminal failure tries the next recorded-consent channel; exhaustion opens a shared exception. | `managed-provider-fallback-receipt` |
| 12 | Reminder dispatch accepts one scheduled reminder ID and no client-supplied campaign copy. | `no-marketing-campaigns` |
| 20 | JSON API writes require `application/json`, accept at most 16 KB, and return structured errors with a correlatable request ID. | `request-protection` |
| 8 | Signed-in clinics can export their own minimized workspace. | `signed-in-export-delete` |
| 10 | Deletion requires the same clinic's organization ID as explicit confirmation. | `signed-in-export-delete` |
| 17 | The signed-in workspace requests checkout through the same-origin billing route, which returns only the Sociobot checkout URL. | `managed-billing-return` |
| 5 | No payment provider is embedded. | billing architecture boundary |
| 11 | Every `@claim:<id>` Playwright test is runnable on its own, for example. | instruction |
| 17 | The multi-stage Dockerfile builds the web output and API without Git metadata, runs as a non-root user, and listens on `PORT`. | deployment implementation |
| 11 | The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. | deployment fact |
| 14 | Do not put provider keys, clinic data, payments, or Entra configuration in this repository. | instruction |
| 9 | The public pages are available at `/privacy` and `/terms`. | navigation instruction |
| 5 | The demo uses fictional aliases. | demo contract |
| 17 | Managed records contain only reminder operations data; clinics must not send clinical notes, diagnoses, or treatment details. | **F-2-5** |
| 2 | See `LICENSE`. | instruction |

## Demo, claims, sandbox, and quality gates

- One click from the landing action opened `/demo` with Northline Sample Clinic, five fictional appointments, delivery states, an exception queue, the persistent “Demo — sample data, nothing is saved to your clinic.” banner, **Reset demo**, and **Start for real** already present. The first product screen is populated; it is not an empty setup screen.
- The fresh demo cookie was `HttpOnly`, `Secure`, `SameSite=Lax`, scoped to `/api/v1/demo`, and had a 24-hour expiry. Page storage used only the `demo:clinic-reminder-proof:<workspace-id>:` session namespace. Landing-to-demo request logging showed only the product origin.
- All 25 commands declared in `.factory/claims.json` were run independently from fresh clone `/tmp/clinic-reminder-proof-review2-clean.BhfgPE`: **25/25 PASS**. This includes all managed fixture commands and each `@claim:` browser command.
- `npm test` passed (7 Vitest, 27 Rust, 33 Playwright tests). `npm run check` passed with Svelte 0 errors/0 warnings, clean rustfmt, and Clippy warnings denied. `npm run build` passed and created `dist/` plus the release API binary.
- The production page made no offline claim. Demo read behavior was checked under the repository’s offline browser tests; send/resolve actions are unavailable offline.

## Earlier findings and history

Read every existing `.factory/review-*.md`, `.factory/polish-*.md`, and handoff. All eight review-1 findings are genuinely fixed in current live copy/code, not merely marked fixed:

| Earlier finding | Recheck |
| --- | --- |
| F-1-1 descriptive landing labels | Live headings are `Reminder evidence`, `Limits and privacy`, and `Clinic plan price`. |
| F-1-2 exception lifecycle coverage | `sample-exception-visibility` is declared and its independently run command passed. |
| F-1-3 through F-1-8 README sentence repairs | Exact current sentences are in the audit above; each is at most 21 words and the jargon-heavy API bullet is gone. |

The older M1/verification issues (isolated demo, pricing, one-replica durability, headers, 404 status, metadata, focus, theme, and normal-flow console cleanliness) also remain fixed under the live and local checks above.

## Structure and interaction

- `/`, `/demo`, `/demo/reminders/mina`, `/start`, `/privacy`, `/terms`, `/404`, and an unknown path were checked live. Each valid route had one `<h1>`, one `<main>`, one description meta, canonical, social card, favicon, consistent header/footer, and route-specific title.
- The unknown path returned the designed ledger-style 404 with HTTP 404 and a home action. The browser’s expected failed-resource 404 console line was the only console entry on that deliberately unknown URL; normal routes were clean.
- Local and discovered internal live links resolved; `https://sociobot.in/` returned 200. Privacy and Terms are present in the footer and header navigation.
- A live axe scan reported no serious or critical issue for the public, demo, legal, start, evidence, or 404 views. The cold 390 px screen is readable and the documented pulse-ledger treatment is visibly product-specific rather than a generic SaaS card grid.
- Live responses sent CSP with response-header `frame-ancestors 'none'`, HSTS, `nosniff`, and strict-origin referrer policy. The social SVG, favicon, touch icon, robots, and sitemap loaded successfully.

## Missed leverage

No finding. The brief’s immediately expected operational extensions are present: a signed calendar/EMR intake path, provider fallback, exception ownership, an export/delete path for signed-in clinics, and a one-click demo. An AI feature is not implied by the job and would be decorative here.

## What would make this perfect

Add the five missing claims and their observable tests (or delete/qualify the five promises), rerun every manifest command in a clean clone, and repeat this full cold review. At that point the product would have no remaining finding.
