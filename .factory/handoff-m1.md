# M1 builder handoff — public proof sandbox

Work order: `venture-clinic-reminder-proof-m1`  
Date: 2026-08-28  
Status: implemented locally; awaiting the required independent review and polish loop.

## What shipped

- Public landing page, `/demo` sandbox, per-reminder evidence routes, `/privacy`, `/terms`, and a styled `/404` route. Each route has its specified title, one h1, focus handoff, canonical/social metadata, header, footer, skip link, and responsive layout.
- The hand-authored “translucent pulse ledger” art direction: original SVG evidence field, social card, favicon, 180 px touch icon, self-hosted Instrument Sans and Fragment Mono assets, dark-token support, and reduced-motion fallback. Asset provenance is recorded in [design.md](design.md) and `apps/web/public/fonts/README.md`.
- Same-origin Rust/axum demo API: CSPRNG workspace IDs, signed HttpOnly cookie, 24-hour TTL, workspace isolation, reset with a fresh seed, body cap, CSP/security headers, health identity, and `tower_governor` rate limiting keyed from the trusted forwarded IP. The demo secret is generated and persisted at first boot unless supplied.
- The exact five fictional sample appointments specified in [demo.md](demo.md): first-channel delivery, rejected WhatsApp then allowed email fallback, opted-out SMS block with an exception, delivered email plus reply, and source cancellation.
- Demo actions for advancing deterministic attempts, inspecting evidence, assigning Sofia R. to Sam Rivera, resolving as Called patient, safe undo, and resetting the complete sample clinic.
- Original pricing and M1-honest copy. No provider, payment, Entra, account, connector, or clinic data path is reachable in demo mode. CIAM, PostgreSQL data/migrations, and recurring Sociobot billing remain M2 scope as the delivery plan requires.

## Verification evidence

Run from a clean install with `npm ci`:

| Check | Result |
| --- | --- |
| `npm test` | Pass — 6 Vitest contracts, 6 Rust tests, 12 Chromium tests |
| M1 claims | Pass — all 9 `@claim:*` Playwright tests from `?demo=1` |
| Browser quality | Pass — axe zero serious/critical findings for `/`, `/demo`, `/privacy`, `/terms`, `/missing`; local link crawl and console-error check pass |
| `npm run check` | Pass — Svelte 0 errors / 0 warnings, rustfmt, clippy warnings denied |
| `npm run build` | Pass — `dist/` and `target/release/reminder-proof-api` produced |
| Bundle | JS 23.10 KB gzip; CSS 4.69 KB gzip; font assets 85.96 KB total, all below M1 budgets |
| Lighthouse, local mobile | Performance 99; Accessibility 100; LCP 1.61 s; CLS 0; TBT 83 ms |
| Rate limit smoke | 41 same-IP API reads return `429` with `Retry-After`; Rust test verifies it too |
| Load smoke | 100 concurrent `/health` requests: 100 × 200 |
| Visual checks | Inspected landing at desktop and demo at 390 × 844; the ledger preview and the demo stack cleanly |

Playwright traces/screenshots are configured to stay under ignored `test-results/` on failure. The successful claims run is reproducible with `npm run test:e2e` or an individual `--grep @claim:<id>`.

## Deployment and live check

The repository was prepared for the container deployment contract: multi-stage image, non-root runtime, `PORT`, `/data`, build SHA args, and no `.git` dependency. Docker is not installed in this worker, so a local image run was not possible.

The injected work order specifies only `deploy: container`, without an Azure Container App, registry, resource group, or factory deployment script for this slug. The visible Azure account has no `clinic-reminder-proof` resource; the only matching resource is an unrelated `sf-reminder-mailroom`. Following the repository contract, this builder did not create or modify infrastructure or deploy to an unrelated app. Consequently the production URL could not be cold-verified from this work order. This is an operator/factory deployment configuration gap, not an application code gap.

## What M2 needs

1. Complete the independent M1 review and polish loop before advancing the plan status to complete.
2. Register/confirm `https://clinic-reminder-proof.sociobot.in/auth/callback` on the shared Entra CIAM SPA, then add actual PKCE UI and RS256/JWKS validation.
3. Provision the planned PostgreSQL service and implement reversible M2 migrations/RLS for users, organizations, memberships, locations, subscriptions, audit events, notification preferences, and exports.
4. Register the recurring Clinic, Practice, and Network pilot tiers through the Sociobot billing catalog; record returned identifiers rather than guessing them.
5. Keep the demo API and the nine M1 claims working unchanged while adding account data.
