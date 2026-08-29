# Product copy audit

Status: reviewed after polish 1 on 2026-08-29. Word counts treat routes, prices, and hyphenated terms as one word.

## First screen

| Words | Copy | Result |
| ---: | --- | --- |
| 5 | Reminder proof for independent clinics | Pass; audience label |
| 4 | See every reminder outcome. | Pass; job headline |
| 14 | For independent clinics that need delivery proof and a clear next step when reminders fail. | Pass; audience and change |
| 5 | Try it with sample data | Pass; primary action |
| 4 | Opens a sample clinic. | Pass; adjacent result |
| 5 | Nothing touches real clinic data. | Pass; `demo-isolation` |
| 6 | Demo actions use sample data only. | Pass; `demo-isolation` |
| 5 | Reminder contents exclude clinical notes. | Pass; `minimal-reminder-content` |
| 7 | Clinic costs $79 per location each month. | Pass; `public-price` |

Read-aloud check: “See every reminder outcome. For independent clinics that need delivery proof and a clear next step when reminders fail. Try it with sample data.” It names the job, customer, and first action in one breath.

## Landing sections

| Words | Copy | Result |
| ---: | --- | --- |
| 2 | Reminder evidence | Pass; section name |
| 7 | Follow one reminder from schedule to outcome. | Pass; section lead |
| 14 | See the source, consent check, each attempt, provider result, and staff resolution in order. | Pass |
| 6 | How the sample clinic works | Pass; section name |
| 3 | Check consent first | Pass |
| 11 | A blocked channel becomes an exception before any simulated provider attempt. | Pass; `consent-channel-guard` |
| 5 | Use the next allowed channel | Pass |
| 14 | Reminder Proof tries a fallback only when consent and the clinic policy allow it. | Pass; `fallback-order` |
| 5 | Give every failure an owner | Pass |
| 12 | Open sample exceptions stay in the queue until a staff member resolves them. | Pass; `sample-exception-visibility` |
| 3 | Limits and privacy | Pass; section name |
| 8 | This does not replace your calendar or EMR. | Pass |
| 10 | Reminder Proof stores no clinical notes and sends no marketing campaigns. | Pass; `no-marketing-campaigns` |
| 9 | The public demo stays separate from managed clinic data. | Pass; `demo-isolation` |
| 3 | Clinic plan price | Pass; section name |
| 3 | Connect your clinic | Pass; action |

## Demo states and actions

| Words | Copy | Result |
| ---: | --- | --- |
| 9 | Demo — sample data, nothing is saved to your clinic. | Pass |
| 10 | Every provider result is simulated. No real reminder is sent. | Pass |
| 3 | Advance due reminders | Pass; action |
| 7 | No sample reminders need staff action. | Pass; empty state |
| 4 | Recently resolved sample exceptions | Pass; history label |
| 4 | Resolve as Called patient | Pass; action |
| 2 | Undo resolution | Pass; reversible action |
| 2 | Reset demo | Pass; action |
| 3 | Start for real | Pass; action |
| 14 | You’re offline. This ledger was last updated in this browser. Sending and resolving are unavailable. | Pass; offline state |

## README prose

Every prose sentence introduced or cited by review 1 is listed here. Command examples and inventory fragments are excluded.

| Words | Sentence | Result |
| ---: | --- | --- |
| 9 | Reminder Proof records each appointment reminder outcome for independent clinics. | Pass; F-1-3 |
| 10 | It shows source details, consent, provider evidence, and the staff owner. | Pass; F-1-3 |
| 17 | It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record. | Pass |
| 3 | Open `/?demo=1` or `/demo`. | Pass |
| 12 | The server creates a random, 24-hour sample workspace containing five fictional appointments. | Pass |
| 21 | Its compact state stays in an HttpOnly, Secure browser cookie, so a restart or replica change does not lose the sample. | Pass |
| 6 | Every provider event is visibly simulated. | Pass |
| 13 | The demo never calls a messaging provider, checkout, account service, or clinic connector. | Pass |
| 7 | Advance the sample reminders and inspect their evidence. | Pass; F-1-4 |
| 12 | Assign or resolve the sample exception, undo a resolution, and reset the sample clinic. | Pass; F-1-4 |
| 8 | “Start for real” opens the managed clinic workflow. | Pass |
| 21 | A clinic signs in through Sociobot Microsoft Entra, creates its workspace, connects a signed calendar feed, and configures approved delivery providers. | Pass |
| 7 | Reminder Proof checks recorded consent before sending. | Pass; F-1-5 |
| 13 | It records provider receipts and opens a shared exception when delivery proof is missing. | Pass; F-1-5 |
| 9 | A same-origin service protects demo sessions and clinic data. | Pass; F-1-6 |
| 9 | It includes rate limits, health checks, and machine-readable metrics. | Pass; F-1-6 |
| 4 | Delivery-provider fees are separate. | Pass |
| 6 | The public demo is always simulated. | Pass |
| 21 | Live dispatch begins only after a signed-in clinic supplies approved sender credentials, template IDs, consent evidence, and a webhook signing secret. | Pass |
| 12 | The API requires no configuration and uses `PORT` (default `8080`). | Pass |
| 19 | The single-replica SQLite writer runs below `DATA_DIR` (the image defaults to `/data`) instead of on an SMB mount. | Pass |
| 12 | Each saved change writes a matching durable database and key under `DURABLE_DIR`. | Pass; F-1-7 |
| 11 | A daily recovery copy is kept under `BACKUP_DIR` for 30 days. | Pass; F-1-7 |
| 7 | Startup restores the durable pair before serving. | Pass |
| 9 | Entra tenant settings may override the documented Sociobot defaults. | Pass |
| 18 | The production container pins the app to one replica so SQLite and demo-creation limits have one state owner. | Pass |
| 11 | The container mounts separate durable and backup shares at `/durable` and `/backups`. | Pass; F-1-8 |
| 6 | The application runs without root privileges. | Pass; F-1-8 |
| 9 | Recovery steps and the restore regression are documented in `.factory/operations.md`. | Pass |
| 8 | All clinic routes require an Entra bearer token. | Pass |
| 13 | The stable `oid` claim owns the workspace; email is never an identity key. | Pass |
| 7 | Create a signed calendar connector in `/app`. | Pass |
| 12 | Configure Twilio for SMS or approved WhatsApp templates, or Resend for email. | Pass |
| 8 | Credentials and patient destinations are encrypted at rest. | Pass |
| 5 | Receipt event IDs are idempotent. | Pass |
| 13 | A terminal failure tries the next recorded-consent channel; exhaustion opens a shared exception. | Pass |
| 12 | Reminder dispatch accepts one scheduled reminder ID and no client-supplied campaign copy. | Pass |
| 8 | Signed-in clinics can export their own minimized workspace. | Pass |
| 10 | Deletion requires the same clinic's organization ID as explicit confirmation. | Pass |
| 5 | No payment provider is embedded. | Pass |
| 11 | The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. | Pass |
| 9 | The public pages are available at `/privacy` and `/terms`. | Pass |
| 5 | The demo uses fictional aliases. | Pass |
| 2 | See `LICENSE`. | Pass |

No audited sentence exceeds 22 words. No copy contains leverage, seamless, effortless, robust, powerful, intuitive, reimagine, supercharge, unlock, delightful, journey, ecosystem, or AI-powered.

## Terminology

| Concept | One term |
| --- | --- |
| Scheduled patient communication | reminder |
| One provider request | attempt |
| Ordered evidence | timeline |
| Provider-confirmed outcome | delivered |
| Human-owned problem | exception |
| Responsible staff member | owner |
| Calendar or EMR input | source |
| Next allowed channel | fallback |
| Test workspace | demo |

## Catalog description

“Track reminder delivery, fallbacks, and staff-owned exceptions without replacing your clinic calendar.”

Words: 12. Characters: 102. It starts with a verb, stays under 120 characters, and contains no banned term.
