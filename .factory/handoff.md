# Planner handoff

Work order: `venture-clinic-reminder-proof-plan`

Role: planner

Date: 2026-08-28

## What was done

- Wrote `.factory/plan.md` as the venture delivery contract: customer and promise, three jobs, demand evidence, wedge, named monthly tiers, measurable outcomes, detailed architecture/data/privacy/operations, M1–M6 claims/tests/DoD, and risk-retiring experiments.
- Wrote `.factory/design.md` for the “translucent pulse ledger” direction, including light/dark palettes, type, spacing, shape, interaction, motion, accessibility, responsive behavior, 20 component contracts, five screens, route titles, performance budgets, and original-asset provenance requirements.
- Wrote `.factory/claims.json` with nine bounded M1 claims and exact future Playwright tags.
- Wrote `.factory/demo.md` with the isolated 24-hour workspace contract, five fictional sample appointments, storage namespaces, reset behavior, and no-provider verification boundary.
- Wrote `.factory/copy-audit.md` with planned first-screen copy, word counts, banned-word check, and the canonical vocabulary.
- Updated `.factory/brief.json` to the admitted state supplied by the work order.
- Scaffolded Svelte 5 + Vite + strict TypeScript under `apps/web/`, executable light/dark tokens and a machine-readable component inventory under `packages/design-system/`, and a Rust/axum health/static server under `services/api/`.
- Added pinned dependencies, Vitest contracts, a Playwright scaffold smoke test, a Rust route test, root build scripts, a multi-stage non-root Dockerfile, and GitHub Actions for browser setup, tests, checks, and build.
- Rewrote README for the actual planning state. It explicitly says the product, demo, data, messaging, auth, and billing do not exist yet.

No product behavior was built. There are no reminder sends, provider calls, clinic records, account flows, billing calls, analytics, generated imagery, or infrastructure changes.

## How it was verified

- `npm test` — passes: 3 Vitest contract tests, 1 Rust health-route test, and 1 Chromium scaffold smoke test.
- `npm run check` — passes: Svelte diagnostics (0 errors, 0 warnings), `cargo fmt --check`, and clippy with warnings denied.
- `npm run build` — passes and creates `dist/` plus `target/release/reminder-proof-api`.
- Web output: 26.24 KB raw / 10.59 KB gzip JavaScript; 6.56 KB raw / 2.03 KB gzip CSS; total `dist/` on disk 48 KB.
- Runtime smoke: `PORT=18080 DIST_DIR=dist ./target/release/reminder-proof-api`; `/health` returned `{"status":"ok","build_sha":"dev"}` and `/` served the expected title.
- Visual smoke: inspected the full scaffold at 390×844; content reflows to one column, retains one `<h1>`, and keeps readable targets and evidence rows.
- `git diff --check` and JSON parsing for the brief, claims, inventory, package manifest, and lockfile pass.

Docker image execution was not tested because the disposable worker has no Docker CLI. The Dockerfile is exercised structurally by the same npm and Cargo build commands used in its stages.

## Known gaps by design

- M1 has not started. The nine entries in `.factory/claims.json` are acceptance contracts, not passing product claims yet; their tagged browser tests must be implemented with the M1 demo.
- The scaffold has one development page and `/health`. Product routes, metadata set, privacy/terms/404 pages, sandbox API, rate limits, CSP, fonts, and original ledger assets are M1 work.
- PostgreSQL, Entra CIAM, tenant isolation, subscriptions, exports, and deletion are M2 work.
- Live source and messaging behavior begins in M3. No automated test or demo may contact a real provider.
- Lighthouse and axe measurements apply when M1 creates the product routes. The current browser smoke checks title, landmarks, heading, skip link, and console output only.

## What the M1 builder should do next

1. Read `.factory/plan.md`, `.factory/design.md`, `.factory/demo.md`, `.factory/claims.json`, and this handoff.
2. Mark only M1 in progress. Do not begin auth, billing, or live provider work.
3. Implement the isolated sample workflow and the exact nine claim tests, one tag per claim.
4. Replace the scaffold page with the complete landing/demo/privacy/terms/404 route set and original code-drawn ledger assets.
5. Finish M1 quality gates, regenerate the copy audit from rendered text, measure bundles/Lighthouse/axe, update the plan status, and write `.factory/handoff-m1.md`.

## Needs operator action in later milestones

- Before M2 review, confirm or register `https://clinic-reminder-proof.sociobot.in/auth/callback` on Entra client `25c704f4-465a-47af-80ab-2c489466b697`.
- In M2, register the Clinic, Practice, and Network recurring tiers in the Sociobot pilot billing catalog and record the returned recurring checkout identifiers. Do not guess them or call Dodo directly.
- No operator action is required for this planning commit.
