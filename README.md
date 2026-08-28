# Reminder Proof

Reminder Proof gives independent clinic teams a clear proof trail for each appointment reminder: what the source said, what consent allowed, what a provider returned, and which person owns a problem. It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record.

## Try the public sandbox

Open `/?demo=1` or `/demo`. The server creates a random, 24-hour sample workspace containing five fictional appointments. Its compact state stays in an HttpOnly, Secure browser cookie, so a restart or replica change does not lose the sample. Every provider event is visibly simulated. The demo never calls a messaging provider, checkout, account service, or clinic connector.

You can advance sample reminders, inspect provider evidence, see a consent block, follow a simulated WhatsApp-to-email fallback, assign and resolve the sample exception, undo the safe resolution, and reset the whole sample clinic.

“Start for real” opens a local CSV evidence tool. It imports calendar/provider exports, applies consent precedence, records fallback outcomes, creates owner fields for exceptions, survives reload, and exports a proof CSV. Imported rows stay in that browser and can be deleted there. It does not dispatch patient messages or replace the provider’s signed webhook record.

## What is in M1

- Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes.
- A Rust/axum same-origin API with isolated HttpOnly Secure demo cookies, 24-hour expiry, JSON and 16 KB body guards, security headers, `/health`, and `/metrics`.
- Original hand-authored pulse-ledger art, favicon, touch icon, social card, and self-hosted Instrument Sans / Fragment Mono assets.
- Playwright claim tests that begin with a fresh demo browser context. See [`.factory/claims.json`](.factory/claims.json).

CIAM sign-in, managed clinic storage, live provider dispatch, and Sociobot/Dodo subscriptions still require operator credentials and regulated-data readiness. They are not represented as available.

## Run locally

Requirements: Node 22.12+, Rust stable with `rustfmt` and `clippy`, and Chromium for Playwright.

```sh
npm ci
npm run dev                 # Vite web development server on http://127.0.0.1:5173
npm run build:web
npm run dev:api             # Same-origin app server on http://127.0.0.1:8080
```

The API requires no configuration and uses `PORT` (default `8080`). Demo state is compact, fictional, and carried only by its scoped cookie; no process-local workspace must survive a restart.

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
docker run --rm -p 8080:8080 reminder-proof
curl http://127.0.0.1:8080/health
```

The factory deploys the container to `https://clinic-reminder-proof.sociobot.in`. Do not put provider keys, clinic data, payments, or Entra configuration in this repository.

## Privacy and terms

The public pages are available at `/privacy` and `/terms`. The demo uses fictional aliases and contains no clinical notes, diagnosis, date of birth, address, insurance details, real phone number, or real email address.

## License

MIT. See [LICENSE](LICENSE).
