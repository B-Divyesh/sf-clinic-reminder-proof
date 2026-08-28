# Reminder Proof

Reminder Proof gives independent clinic teams a clear proof trail for each appointment reminder: what the source said, what consent allowed, what a provider returned, and which person owns a problem. It sits beside an existing calendar or EMR; it is not a replacement scheduler or medical record.

## Try the public sandbox

Open `/?demo=1` or `/demo`. The server creates a random, signed, 24-hour sample workspace containing five fictional appointments. Every provider event is visibly simulated. The demo never calls a messaging provider, checkout, account service, or clinic connector.

You can advance sample reminders, inspect provider evidence, see a consent block, follow a simulated WhatsApp-to-email fallback, assign and resolve the sample exception, undo the safe resolution, and reset the whole sample clinic.

## What is in M1

- Public landing, demo ledger, reminder evidence, Privacy, Terms, and styled 404 routes.
- A Rust/axum same-origin API with signed HttpOnly demo cookies, random isolated workspaces, 24-hour expiry, request/body limits, security headers, and health endpoint.
- Original hand-authored pulse-ledger art, favicon, touch icon, social card, and self-hosted Instrument Sans / Fragment Mono assets.
- Nine Playwright claim tests that begin with a fresh demo browser context. See [`.factory/claims.json`](.factory/claims.json).

CIAM sign-in, durable PostgreSQL clinic data and migrations, and Sociobot/Dodo monthly subscriptions are deliberately M2 work. They are not reachable in this demo.

## Run locally

Requirements: Node 22.12+, Rust stable with `rustfmt` and `clippy`, and Chromium for Playwright.

```sh
npm ci
npm run dev                 # Vite web development server on http://127.0.0.1:5173
npm run build:web
npm run dev:api             # Same-origin app server on http://127.0.0.1:8080
```

The API requires no configuration. It uses `PORT` (default `8080`) and generates a random demo-cookie signing secret the first time it starts. The secret is persisted beneath `DATA_DIR` (default `data/` locally, `/data` in the container). `DEMO_COOKIE_SECRET` may supply a 32-byte-or-longer hexadecimal value when an operator needs a managed secret.

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

The multi-stage `Dockerfile` builds the web output and API without Git metadata, runs as a non-root user, persists generated runtime state in `/data`, and listens on `PORT`.

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
