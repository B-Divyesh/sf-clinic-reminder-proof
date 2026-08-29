# Polish round 3 handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-polish-3`

Base review: `75a231532b880030ba935085385044e9de26051d`

Repair implementation and live build: `785e4bc956d0ddaefc0f2babff0efd5d6a54b189`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status: PASS

All 21 findings from review rounds 1–3 are fixed and reverified. The detailed finding-to-change-to-evidence map is in [polish-3.md](polish-3.md).

## What changed

- Rewrote every review-three jargon or terminology issue in the README and product copy.
- Removed the untestable originality claim and expanded `no-tracking` to prove that both bundled font families load from this origin.
- Corrected the interface font-family name exposed by that stronger claim test.
- Added a visible footer build SHA on every route and made the Vite build use the same Docker `BUILD_SHA` as `/health`.
- Replaced the metaphorical 404 H1 with `Page not found` while preserving the product-specific pulse-ledger art.
- Strengthened claim and regression tests for font provenance, copy terminology, the 404, and build identity.
- Updated the verb-first, 76-character catalog description.
- Restored the production deployment contract after detecting drift: one replica and separate durable/backup Azure Files mounts.

## Exact verification evidence

Fresh clone `/tmp/clinic-reminder-proof-polish3-clean.M2u9lu` was created from pushed `main` at `785e4bc956d0ddaefc0f2babff0efd5d6a54b189`.

- `npm ci`: 87 packages; zero vulnerabilities reported.
- Every exact `.factory/claims.json` command: 31/31 passed independently.
- `npm test`: 9 Vitest, 33 Rust, 39 Chromium; all passed.
- `npm run check`: Svelte 0 errors/0 warnings, rustfmt clean, Clippy clean with warnings denied.
- `npm run build`: passed and produced `dist/` plus `target/release/reminder-proof-api`.
- Build budgets: entry JS 28.62 KB gzip, CSS 5.53 KB gzip, lazy sign-in JS 68.23 KB gzip.
- Production browser rerun: 39/39 passed against the live origin.
- `/opt/fleet/lib/verify-url.sh`: HTTP 200, zero console errors, title/lang/one H1/main/alt/control-label checks passed.
- `npx @axe-core/cli` 4.11.4: 0 violations on the live landing page. The browser suite also ran multi-route axe checks in light and dark themes.
- Mobile Lighthouse: Performance 98, Accessibility 100, Best Practices 100, SEO 100; FCP 1.35 s, LCP 1.45 s, TBT 152 ms, CLS 0.0007.
- A 20-way, 100-request live `/health` smoke returned 100 × 200.
- `/health` reports `785e4bc956d0ddaefc0f2babff0efd5d6a54b189`; every route footer displays `Build 785e4bc`.
- Azure reports healthy revision `sf-clinic-reminder-proof--0000036`, image tag `785e4bc956d0`, 100% traffic, `minReplicas=1`, `maxReplicas=1`, and Azure Files mounted at `/durable` and `/backups`.

## Live evidence files

- Cold desktop landing: `qa-artifacts/polish-3/live/landing-desktop-cold.png`
- Cold 390 px landing: `qa-artifacts/polish-3/live/screenshot-mobile.png`
- Direct `/?demo=1` sample path: `qa-artifacts/polish-3/live/query-demo-mobile.png`
- Direct `/demo`: `qa-artifacts/polish-3/live/demo-mobile.png`
- Real 404: `qa-artifacts/polish-3/live/404-mobile.png`
- URL verifier: `qa-artifacts/polish-3/live/verify.json`
- axe CLI: `qa-artifacts/polish-3/live/axe.json`
- Lighthouse: `qa-artifacts/polish-3/live/lighthouse.json`

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

## Remaining work

No product, review, accessibility, privacy, performance, routing, demo, claim, or deployment finding remains.

Before inviting a real clinic to sign in, the operator must confirm that `https://clinic-reminder-proof.sociobot.in/auth/callback` is registered on the shared Sociobot Entra SPA. No real patient data, clinic credentials, messaging-provider dispatch, payment, or destructive customer deletion was used during verification; the deterministic fixture suites cover those safety boundaries.
