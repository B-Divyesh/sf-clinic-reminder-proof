# Product copy audit

Status: reviewed after polish 3 on 2026-08-29. Counts are whitespace-delimited; routes, prices, and hyphenated terms count as one word.

## First screen

| Words | Copy | Result |
| ---: | --- | --- |
| 5 | Reminder proof for independent clinics | Pass; audience label |
| 4 | See every reminder outcome. | Pass; job headline |
| 15 | For independent clinics that need delivery proof and a clear next step when reminders fail. | Pass; audience and change |
| 5 | Try it with sample data | Pass; primary action |
| 4 | Opens a sample clinic. | Pass; adjacent result |
| 5 | Nothing touches real clinic data. | Pass; `demo-isolation` |
| 6 | Demo actions use sample data only. | Pass; `demo-isolation` |
| 5 | Reminder contents exclude clinical notes. | Pass; `minimal-reminder-content` |
| 7 | Clinic costs $79 per location each month. | Pass; `public-price` |

Read-aloud check: “See every reminder outcome. For independent clinics that need delivery proof and a clear next step when reminders fail. Try it with sample data.” It names the job, customer, and first action in one breath.

## Landing page after the first screen

| Words | Copy | Result |
| ---: | --- | --- |
| 2 | Reminder evidence | Pass; section name |
| 7 | Follow one reminder from schedule to outcome. | Pass; section lead |
| 14 | See the source, consent check, each attempt, messaging-provider result, and staff resolution in order. | Pass; `delivery-timeline`, F-3-8 |
| 5 | How the sample clinic works | Pass; section name |
| 3 | Check consent first | Pass |
| 11 | A blocked channel becomes an exception before any simulated messaging-provider attempt. | Pass; `consent-channel-guard`, F-3-8 |
| 5 | Use the next allowed channel | Pass |
| 14 | Reminder Proof tries a fallback only when consent and the clinic policy allow it. | Pass; `fallback-order` |
| 5 | Give every failure an owner | Pass |
| 13 | Open sample exceptions stay in the queue until a staff member resolves them. | Pass; `sample-exception-visibility` |
| 3 | Limits and privacy | Pass; section name |
| 8 | This does not replace your calendar or EMR. | Pass |
| 11 | Reminder Proof stores no clinical notes and sends no marketing campaigns. | Pass; `managed-data-minimisation`, `no-marketing-campaigns` |
| 9 | The public demo stays separate from managed clinic data. | Pass; `demo-isolation` |
| 3 | Clinic plan price | Pass; section name |
| 3 | Connect your clinic | Pass; action |
| 13 | Reminder Proof records delivery evidence and staff-owned exceptions around an existing clinic calendar. | Pass; footer description |
| 2 | Build {short SHA} | Pass; `build-identity` |

## Demo and recovery routes

| Words | Copy | Result |
| ---: | --- | --- |
| 9 | Demo — sample data, nothing is saved to your clinic. | Pass; persistent banner |
| 10 | Every messaging-provider result is simulated. No real reminder is sent. | Pass; F-3-8 |
| 3 | Advance due reminders | Pass; action |
| 7 | No sample reminders need staff action. | Pass; empty state |
| 4 | Recently resolved sample exceptions | Pass; history label |
| 4 | Resolve as Called patient | Pass; action |
| 2 | Undo resolution | Pass; reversible action |
| 2 | Reset demo | Pass; action |
| 3 | Start for real | Pass; action |
| 14 | You’re offline. This ledger was last updated in this browser. Sending and resolving are unavailable. | Pass; offline state |
| 3 | Sample reminder not found | Pass; direct missing-record heading |
| 3 | Page not found | Pass; direct 404 heading, F-3-3 |
| 14 | This address does not match a page. Return to the Reminder Proof home page. | Pass; recovery direction |

## README

Headings, inventory statements, and every prose sentence are included. Commands and code blocks are excluded.

| Words | Copy | Result |
| ---: | --- | --- |
| 2 | Reminder Proof | Pass; title |
| 10 | Reminder Proof records each appointment reminder outcome for independent clinics. | Pass; F-1-3 |
| 10 | It shows source details, consent, messaging-provider evidence, and the staff owner. | Pass; F-3-8 |
| 17 | It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record. | Pass |
| 3 | Try the demo | Pass; F-3-4 |
| 4 | Open `/?demo=1` or `/demo`. | Pass |
| 12 | The server creates a random, 24-hour sample workspace containing five fictional appointments. | Pass |
| 16 | A protected browser cookie keeps the sample available for 24 hours, including after a server restart. | Pass; F-3-5 |
| 6 | Every messaging-provider event is visibly simulated. | Pass; F-3-8 |
| 13 | The demo never calls a messaging provider, checkout, account service, or clinic connector. | Pass; `demo-isolation` |
| 8 | Advance the sample reminders and inspect their evidence. | Pass; F-1-4 |
| 14 | Assign or resolve the sample exception, undo a resolution, and reset the sample clinic. | Pass; F-1-4 |
| 8 | “Start for real” opens the managed clinic workflow. | Pass |
| 21 | A clinic signs in through Sociobot Microsoft Entra, creates its workspace, connects a signed calendar feed, and configures approved messaging providers. | Pass; F-3-8 |
| 7 | Reminder Proof checks recorded consent before sending. | Pass; F-1-5 |
| 14 | It records messaging-provider receipts and opens a shared exception when delivery proof is missing. | Pass; F-3-8 |
| 3 | What is included | Pass; heading |
| 12 | Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes. | Pass; inventory |
| 12 | A service on this site keeps demo sessions separate from clinic data. | Pass; F-3-6 |
| 9 | It includes rate limits, health checks, and machine-readable metrics. | Pass; `rate-limit-policy`, `build-identity` |
| 16 | A signed calendar/EMR connection stores each appointment once, even when it receives the same update twice. | Pass; `signed-calendar-intake`, F-3-7 |
| 13 | Twilio SMS and approved WhatsApp dispatch, Resend email fallback, and signed receipt reconciliation. | Pass; registered messaging claims |
| 17 | Shared exception assignment and resolution, clinic export/delete, and Sociobot-hosted subscription checkout at $79 per location each month. | Pass; registered managed claims |
| 4 | Messaging-provider fees are separate. | Pass; F-3-8 |
| 21 | The site includes pulse-ledger art, a favicon, a touch icon, a social card, and self-hosted Instrument Sans and Fragment Mono fonts. | Pass; `no-tracking`, F-3-1 |
| 11 | Playwright claim tests that begin with a fresh demo browser context. | Pass |
| 2 | See `.factory/claims.json`. | Pass |
| 6 | The public demo is always simulated. | Pass |
| 21 | Live dispatch begins only after a signed-in clinic supplies approved sender credentials, template IDs, consent evidence, and a webhook signing secret. | Pass |
| 2 | Run locally | Pass; heading |
| 13 | Requirements: Node 22.12+, Rust stable with `rustfmt` and `clippy`, and Chromium for Playwright. | Pass |
| 10 | The API requires no configuration and uses `PORT` (default `8080`). | Pass |
| 18 | The single-replica SQLite writer runs below `DATA_DIR` (the image defaults to `/data`) instead of on an SMB mount. | Pass |
| 12 | Each saved change writes a matching durable database and key under `DURABLE_DIR`. | Pass; F-1-7 |
| 11 | A daily recovery copy is kept under `BACKUP_DIR` for 30 days. | Pass; F-1-7 |
| 7 | Startup restores the durable pair before serving. | Pass |
| 9 | Entra tenant settings may override the documented Sociobot defaults. | Pass |
| 18 | The production container pins the app to one replica so SQLite and demo-creation limits have one state owner. | Pass |
| 12 | The container mounts separate durable and backup shares at `/durable` and `/backups`. | Pass; F-1-8 |
| 6 | The application runs without root privileges. | Pass; F-1-8 |
| 10 | Recovery steps and the restore regression are documented in `.factory/operations.md`. | Pass |
| 14 | Register `https://clinic-reminder-proof.sociobot.in/auth/callback` on the shared Sociobot Entra SPA before sign-in is opened to clinics. | Pass |
| 12 | The production image refuses to start when either required share is missing. | Pass; `single-replica-durable-topology` |
| 18 | Use `npm run deploy:container -- --image <registry/image:tag>` for every Container Apps image rollout; it reapplies the checked-in mounts and one-replica boundary. | Pass; `single-replica-durable-topology` |
| 11 | The command waits until that exact healthy revision has all traffic. | Pass; `single-replica-durable-topology` |
| 21 | After deployment, run `npm run verify:deployment` with Azure access to check the active revision, mounts, replica count, and six-request rate limit. | Pass; `rate-limit-policy`, `single-replica-durable-topology` |
| 3 | Clinic integration contract | Pass; heading |
| 8 | All clinic routes require an Entra bearer token. | Pass |
| 13 | The stable `oid` claim owns the workspace; email is never an identity key. | Pass |
| 7 | Create a signed calendar connector in `/app`. | Pass |
| 10 | Post normalized appointment batches to `/api/v1/connectors/intake` with `X-Reminder-Timestamp` and `X-Reminder-Signature`. | Pass |
| 7 | Sign the UTF-8 string `<timestamp>:<connector-id>:<appointment-count>` with HMAC-SHA256. | Pass |
| 8 | Encode the result as URL-safe base64 without padding. | Pass |
| 12 | Configure Twilio for SMS or approved WhatsApp templates, or Resend for email. | Pass |
| 9 | Messaging-provider credentials and patient destinations are encrypted at rest. | Pass; `managed-secret-encryption` |
| 13 | Twilio receives its status callback URL during dispatch and is verified with `X-Twilio-Signature`. | Pass; `twilio-receipt-verification` |
| 17 | Resend receipt callbacks use its Svix headers (`svix-id`, `svix-timestamp`, and `svix-signature`) and the stored `whsec_…` webhook secret. | Pass; `resend-receipt-verification` |
| 6 | Repeated receipt event IDs are ignored. | Pass; receipt verification |
| 13 | A terminal failure tries the next recorded-consent channel; exhaustion opens a shared exception. | Pass; `managed-provider-fallback-receipt` |
| 12 | Reminder dispatch accepts one scheduled reminder ID and no client-supplied campaign copy. | Pass; `no-marketing-campaigns` |
| 19 | JSON API writes require `application/json`, accept at most 16 KB, and return structured errors with a correlatable request ID. | Pass; `request-protection` |
| 8 | Signed-in clinics can export their own minimized workspace. | Pass; `signed-in-export-delete` |
| 10 | Deletion requires the same clinic's organization ID as explicit confirmation. | Pass; `signed-in-export-delete` |
| 15 | The signed-in workspace requests checkout through this site, which returns only the Sociobot checkout URL. | Pass; `managed-billing-return` |
| 5 | No payment provider is embedded. | Pass; qualified term |
| 1 | Verify | Pass; heading |
| 11 | Every `@claim:<id>` Playwright test is runnable on its own, for example. | Pass |
| 2 | Container deployment | Pass; heading |
| 21 | The multi-stage Dockerfile builds the web output and API without Git metadata, runs as a non-root user, and listens on `PORT`. | Pass |
| 7 | The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. | Pass |
| 14 | Do not put messaging-provider keys, clinic data, payments, or Entra configuration in this repository. | Pass |
| 19 | After pushing an image to the factory registry, deploy it with the checked-in topology rather than an image-only update. | Pass; `single-replica-durable-topology` |
| 3 | Privacy and terms | Pass; heading |
| 9 | The public pages are available at `/privacy` and `/terms`. | Pass |
| 5 | The demo uses fictional aliases. | Pass |
| 17 | Managed records contain only reminder operations data; clinics must not send clinical notes, diagnoses, or treatment details. | Pass; `managed-data-minimisation` |
| 1 | License | Pass; heading |
| 1 | MIT. | Pass |
| 2 | See `LICENSE`. | Pass |

No sentence exceeds 22 words. No banned word appears. Protocol names remain only in the developer integration section.

## Terminology

| Concept | One term |
| --- | --- |
| One-click sample experience | demo |
| External service that sends a reminder | messaging provider |
| Scheduled patient communication | reminder |
| One messaging-provider request | attempt |
| Ordered evidence | timeline |
| Messaging-provider-confirmed outcome | delivered |
| Human-owned problem | exception |
| Responsible staff member | owner |
| Calendar or EMR input | source |
| Next allowed channel | fallback |

## Catalog description

“Track every reminder outcome and give failed deliveries a clear staff owner.”

Words: 12. Characters: 76. It starts with a verb, stays under 120 characters, and contains no banned word.

## Claim mapping added or retained through review 3

| Public promise | Claim test |
| --- | --- |
| Demo isolation and no messaging-provider call | `@claim:demo-isolation` |
| Open exceptions remain until resolved | `@claim:sample-exception-visibility` |
| Fonts load from this site and no third party tracks the visit | `@claim:no-tracking` |
| Footer build matches health | `@claim:build-identity` |
| Signed connector stores duplicate updates once | `@claim:signed-calendar-intake` |
| Approved WhatsApp template dispatch | `@claim:approved-whatsapp-dispatch` |
| Twilio callback verification and replay handling | `@claim:twilio-receipt-verification` |
| Resend Svix verification and replay handling | `@claim:resend-receipt-verification` |
| Messaging-provider credentials and destinations encrypted at rest | `@claim:managed-secret-encryption` |
| Managed records reject clinical fields | `@claim:managed-data-minimisation` |
