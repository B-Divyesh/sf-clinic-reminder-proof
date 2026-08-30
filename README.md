# Reminder Proof

Reminder Proof records each appointment reminder outcome for independent clinics. It shows source details, consent, messaging-provider evidence, and the staff owner. It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record.

## Try the demo

Open `/?demo=1` or `/demo`. The server creates a random, 24-hour sample workspace containing five fictional appointments. A protected browser cookie keeps the sample available for 24 hours, including after a server restart. Every messaging-provider event is visibly simulated. The demo never calls a messaging provider, checkout, account service, or clinic connector.

Advance the sample reminders and inspect their evidence. Assign or resolve the sample exception, undo a resolution, and reset the sample clinic.

“Start for real” opens the managed clinic workflow. A clinic signs in through the shared Sociobot Microsoft Entra tenant. Setup records its jurisdiction, retention choice, first location, and staff roles. Reminder Proof checks recorded consent before sending. It records messaging-provider receipts and opens a shared exception when delivery proof is missing.

## What is included

- Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes.
- A service on this site keeps demo sessions separate from clinic data. It includes rate limits, health checks, and machine-readable metrics.
- A signed calendar/EMR connection stores each appointment once, even when it receives the same update twice.
- Twilio SMS and approved WhatsApp dispatch, Resend email fallback, and signed receipt reconciliation.
- Owner-only clinic export and a cancelable seven-day deletion window.
- Sociobot-hosted monthly plan checkout starts at $79 per location. Messaging-provider fees are separate.
- The site includes pulse-ledger art, a favicon, a touch icon, a social card, and self-hosted Instrument Sans and Fragment Mono fonts.
- Playwright claim tests that begin with a fresh demo browser context. See [`.factory/claims.json`](.factory/claims.json).

The public demo is always simulated. Live dispatch begins only after a signed-in clinic supplies approved sender credentials, template IDs, consent evidence, and a webhook signing secret.

## Run locally

Requirements: Node 22.12+, Rust stable with `rustfmt` and `clippy`, and Chromium for Playwright.

```sh
npm ci
npm run dev                 # Vite web development server on http://127.0.0.1:5173
npm run build:web
npm run dev:api             # Same-origin app server on http://127.0.0.1:8080
```

The API requires no configuration and uses `PORT` (default `8080`). The single-replica SQLite writer runs below `DATA_DIR` instead of on an SMB mount. M2 adds reversible migrations for accounts, clinics, locations, roles, subscriptions, audit events, preferences, and exports. Each saved change writes a matching durable database and key under `DURABLE_DIR`. A daily recovery copy is kept under `BACKUP_DIR` for 30 days. Startup restores the durable pair before serving. Entra tenant settings may override the documented Sociobot defaults.

Billing defaults to the live Sociobot pilot gateway and Dodo test mode. Set `SOCIOBOT_BILLING_BASE_URL` only to change the gateway. The Clinic, Practice, and Network choices are allowlisted on the server. The pilot product must be enabled by a factory operator before checkout can finish.

The production container pins the app to one replica so SQLite and demo-creation limits have one state owner. The container mounts separate durable and backup shares at `/durable` and `/backups`. The application runs without root privileges. Recovery steps and the restore regression are documented in [`.factory/operations.md`](.factory/operations.md). Register `https://clinic-reminder-proof.sociobot.in/auth/callback` on the shared Sociobot Entra SPA before sign-in is opened to clinics.

The production image refuses to start when either required share is missing. Commit and push the final handoff before running `npm run deploy:container -- --image <registry/image:full-commit>`. The command rejects dirty, unpublished, short-tagged, or mismatched candidates. It reapplies the checked-in mounts and one-replica boundary. It waits until that exact healthy revision has all traffic and serves its health and footer build identity. After deployment, run `npm run verify:deployment:current` with Azure access. It checks the active revision, mounts, replica count, public identity, and six-request rate limit.

## Clinic integration contract

All clinic routes require an Entra bearer token. The stable `oid` claim links the user; email is never an identity key. Owners control billing, exports, deletion, and staff roles. Active staff can read only their clinic.

- Create a signed calendar connector in `/app`. Post normalized appointment batches to `/api/v1/connectors/intake` with `X-Reminder-Timestamp` and `X-Reminder-Signature`.
- Sign the UTF-8 string `<timestamp>:<connector-id>:<appointment-count>` with HMAC-SHA256. Encode the result as URL-safe base64 without padding.
- Configure Twilio for SMS or approved WhatsApp templates, or Resend for email. Messaging-provider credentials and patient destinations are encrypted at rest.
- Twilio receives its status callback URL during dispatch and is verified with `X-Twilio-Signature`. Resend receipt callbacks use its Svix headers (`svix-id`, `svix-timestamp`, and `svix-signature`) and the stored `whsec_…` webhook secret.
- Repeated receipt event IDs are ignored. A terminal failure tries the next recorded-consent channel; exhaustion opens a shared exception.
- Reminder dispatch accepts one scheduled reminder ID and no client-supplied campaign copy. JSON API writes require `application/json`, accept at most 16 KB, and return structured errors with a correlatable request ID.
- Owners can export their clinic’s minimized workspace. Deletion waits seven days and can be cancelled during that time.
- The clinic requests checkout through this site, which returns only the Sociobot checkout URL. No payment provider is embedded.

## Verify

```sh
npm test        # Vitest contracts, Rust API tests, and all Playwright claim tests
npm run check   # Svelte diagnostics, rustfmt, and clippy with warnings denied
npm run build   # emits dist/ and target/release/reminder-proof-api
```

Every `@claim:<id>` Playwright test is runnable on its own, for example:

```sh
npm run test:e2e -- --grep @claim:demo-reset
```

## Container deployment

The multi-stage `Dockerfile` builds the web output and API without Git metadata, runs as a non-root user, and listens on `PORT`.

```sh
docker build --build-arg BUILD_SHA=local -t reminder-proof .
docker run --rm -p 8080:8080 -v reminder-durable:/durable -v reminder-backups:/backups reminder-proof
curl http://127.0.0.1:8080/health
```

The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. Do not put messaging-provider keys, clinic data, payments, or Entra configuration in this repository.

After pushing an image to the factory registry, deploy it with the checked-in
topology rather than an image-only update:

```sh
git push origin main
npm run deploy:container -- --image sociobotregistry.azurecr.io/sf-clinic-reminder-proof:<full-HEAD-commit>
npm run verify:deployment:current
```
## Privacy and terms

The public pages are available at `/privacy` and `/terms`. The demo uses fictional aliases. Managed records contain only reminder operations data; clinics must not send clinical notes, diagnoses, or treatment details.

## License

MIT. See [LICENSE](LICENSE).
