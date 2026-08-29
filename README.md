# Reminder Proof

Reminder Proof gives independent clinic teams a clear proof trail for each appointment reminder: what the source said, what consent allowed, what a provider returned, and which person owns a problem. It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record.

## Try the public sandbox

Open `/?demo=1` or `/demo`. The server creates a random, 24-hour sample workspace containing five fictional appointments. Its compact state stays in an HttpOnly, Secure browser cookie, so a restart or replica change does not lose the sample. Every provider event is visibly simulated. The demo never calls a messaging provider, checkout, account service, or clinic connector.

You can advance sample reminders, inspect provider evidence, see a consent block, follow a simulated WhatsApp-to-email fallback, assign and resolve the sample exception, undo the safe resolution, and reset the whole sample clinic.

“Start for real” opens the managed clinic workflow. A clinic signs in through Sociobot Microsoft Entra, creates its workspace, connects a signed calendar feed, and configures approved delivery providers. Reminder Proof checks recorded consent in policy order, sends only an approved template, ingests signed provider receipts, and opens a shared staff exception when proof is missing.

## What is included

- Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes.
- A Rust/axum same-origin API with isolated demo cookies, Entra JWT validation, encrypted durable clinic data, rate limits, security headers, `/health`, and `/metrics`.
- A signed calendar/EMR webhook connector with idempotent appointment upserts.
- Twilio SMS and approved WhatsApp dispatch, Resend email fallback, and signed receipt reconciliation.
- Shared exception assignment and resolution, clinic export/delete, and Sociobot-hosted subscription checkout at $79 per location each month, plus published messaging charges.
- Original hand-authored pulse-ledger art, favicon, touch icon, social card, and self-hosted Instrument Sans / Fragment Mono assets.
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

The API requires no configuration and uses `PORT` (default `8080`). Clinic data and the generated AES-256 data key persist below `DATA_DIR` (the image defaults to `/data`). Each successful workspace write also creates a consistent SQLite online backup and matching key below `BACKUP_DIR` (default `/backups`), plus a daily recovery pair retained for 30 days. Entra tenant settings may override the documented Sociobot defaults. Provider credentials are entered by a signed-in clinic and encrypted at rest.

The production container pins the app to one replica so SQLite and demo-creation limits have one state owner. Separate durable Azure Files shares mount directly at `/data` and `/backups`; the non-root process creates and updates files without a privileged init container. Recovery steps and the restore regression are documented in [`.factory/operations.md`](.factory/operations.md). Register `https://clinic-reminder-proof.sociobot.in/auth/callback` on the shared Sociobot Entra SPA before sign-in is opened to clinics.

## Clinic integration contract

All clinic routes require an Entra bearer token. The stable `oid` claim owns the workspace; email is never an identity key.

- Create a signed calendar connector in `/app`. Post normalized appointment batches to `/api/v1/connectors/intake` with `X-Reminder-Timestamp` and `X-Reminder-Signature`.
- Sign the UTF-8 string `<timestamp>:<connector-id>:<appointment-count>` with HMAC-SHA256. Encode the result as URL-safe base64 without padding.
- Configure Twilio for SMS or approved WhatsApp templates, or Resend for email. Credentials and patient destinations are encrypted at rest.
- Twilio receives its status callback URL during dispatch and is verified with `X-Twilio-Signature`. Resend receipt callbacks use its Svix headers (`svix-id`, `svix-timestamp`, and `svix-signature`) and the stored `whsec_…` webhook secret.
- Receipt event IDs are idempotent. A terminal failure tries the next recorded-consent channel; exhaustion opens a shared exception.
- The signed-in workspace requests checkout through the same-origin billing route, which returns only the Sociobot checkout URL. No payment provider is embedded.

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
docker run --rm -p 8080:8080 -v reminder-data:/data -v reminder-backups:/backups reminder-proof
curl http://127.0.0.1:8080/health
```

The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. Do not put provider keys, clinic data, payments, or Entra configuration in this repository.

## Privacy and terms

The public pages are available at `/privacy` and `/terms`. The demo uses fictional aliases. Managed records contain only reminder operations data; clinics must not send clinical notes, diagnoses, or treatment details.

## License

MIT. See [LICENSE](LICENSE).
