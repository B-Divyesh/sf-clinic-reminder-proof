# Reminder Proof

Reminder Proof is the planned reliability and evidence layer for independent dental and outpatient clinics. It will connect to an existing calendar or EMR, apply consent-aware reminder policies, record channel outcomes, use safe fallbacks, and give unresolved reminders a staff owner.

This repository is currently a **planning and tooling scaffold**. It does not send reminders, store clinic data, offer a working demo, or take payment yet. M1 is defined in [`.factory/plan.md`](.factory/plan.md).

## Who it is for

Clinic owners, practice managers, and front desk teams who need to know what happened to each appointment reminder without replacing their scheduling system.

## Repository map

- `apps/web/` — Svelte 5 and Vite entry point
- `services/api/` — Rust/axum API and static-file server
- `packages/design-system/` — executable tokens and component inventory
- `.factory/brief.json` — researched opportunity
- `.factory/plan.md` — PRD, architecture, milestones, acceptance, and risks
- `.factory/design.md` — translucent pulse ledger visual contract
- `.factory/claims.json` — M1 claim contract; implementation arrives in M1
- `.factory/demo.md` — M1 sandbox dataset and isolation contract

## Requirements

- Node.js 22.12 or newer
- Rust stable with `rustfmt` and `clippy`

## Develop

```sh
npm ci
npm run dev          # web scaffold on http://127.0.0.1:5173
npm run dev:api      # API/static server on http://127.0.0.1:8080
```

Build the web output before running the API if you want axum to serve the page:

```sh
npm run build:web
npm run dev:api
```

## Test and build

```sh
npm test             # Vitest, Rust, and Chromium scaffold tests
npm run check        # Svelte types, rustfmt, and clippy
npm run build        # writes web files to dist/ and builds the release API
```

Playwright 1.58.2 is pinned for M1 claim tests. The planning scaffold has no browser claim implementation yet; M1 will add one tagged test for every entry in `.factory/claims.json`.

## Runtime and deployment

The container starts with only `PORT` (default `8080`), serves `/health` with the build SHA, and serves the Vite build from `dist/`. The factory will deploy the finished product to `https://clinic-reminder-proof.sociobot.in`; this planning work does not change infrastructure, DNS, billing, or provider accounts.

```sh
docker build --build-arg BUILD_SHA=local -t reminder-proof .
docker run --rm -p 8080:8080 reminder-proof
```

Future production configuration, Entra CIAM, Sociobot billing, database migrations, privacy pages, demo behavior, and provider adapters are milestone work described in the plan. Never commit secrets or contact a real messaging provider from tests.

## License

MIT. See [`LICENSE`](LICENSE).
