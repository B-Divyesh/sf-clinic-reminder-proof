# Repair 7 handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-repair-7`
Base verifier report: [verification-11.md](verification-11.md)
Repair commit: `aae2ef63533d5daf18c175feabb03aa482152d9f`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status: PASS

All QA11 release blockers are repaired, committed, pushed, deployed, and verified on the active production revision. The app is now safe to continue independent verification; no real clinic data, patient data, provider credential, or payment was used during this repair.

## Repairs

1. **QA11-01 — durable single-owner deployment**
   - Built and pushed `aae2ef6` to `origin/main`.
   - Added a production mount guard. The container image sets `REQUIRE_DURABLE_MOUNTS=1`; it refuses startup unless `/durable` and `/backups` are real mounts, preventing a future topology drift from silently accepting clinic state on ephemeral storage.
   - Added `npm run verify:deployment`. It inspects the active Container Apps revision, validates the exact Azure Files bindings and one-replica scale, checks live build identity, and proves the sixth same-client demo creation returns `429` with `Retry-After`.
   - Updated the declared topology claim so it runs both the checked-in-template assertion and the live deployment verifier.
   - ACR build `ch10u` succeeded from the `.git`-excluded source archive. Deployed revision `sf-clinic-reminder-proof--0000038` has image `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:aae2ef63533d`, `minReplicas=1`, `maxReplicas=1`, 100% latest traffic, and both mounts.
   - Live `npm run verify:deployment` returned five `200`s, then `429`, `Retry-After: 3599`; `/health` reports the full repair SHA.
   - Azure in-container mount evidence showed CIFS shares at `/durable` and `/backups`. A uniquely named zero-data probe was created on both shares, the active revision was restarted, and Azure Files reported both probes still present. Both probe files were then deleted and verified absent. The existing Rust fixture regression continues to verify signed managed-state backup/restore.

2. **QA11-02 — 390 px evidence-link targets**
   - Added the `evidence-link` control treatment: inline flex, at least 44×44 CSS px, with the existing designed focus ring preserved.
   - Added `@regression:qa11-02`, which opens the real demo at 390×844 and measures every visible demo link, button, and select. All ten controls meet both 44 px dimensions.

## Verification evidence

- Clean install: `npm ci` installed 87 locked packages; audit reported zero vulnerabilities.
- Local complete suite: `npm test` passed 9 Vitest tests, 34 Rust tests, and 40 Chromium tests.
- Exact claim sweep: all 31 `.factory/claims.json` commands passed independently in manifest order, including the production topology claim.
- `npm run check`: Svelte 0 errors/0 warnings, rustfmt clean, Clippy warnings denied.
- `npm run build`: produced `dist/` and `target/release/reminder-proof-api`; entry JS is 28.63 KB gzip and CSS 5.54 KB gzip.
- Release binary test: with only `PORT=4811`, `/health` returned `{"status":"ok","build_sha":"dev"}` and startup generated a local key without printing it.
- Container build: ACR build `ch10u` passed using the multi-stage Dockerfile and no Git metadata.
- Live browser: `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e` passed all 40 desktop/mobile, keyboard, privacy, offline-read, response-policy, routing, and reduced-motion checks.
- Accessibility: the production Playwright axe audit passed on seven public routes in light and dark themes with zero serious/critical findings. The direct axe CLI cannot launch the preinstalled Playwright Chromium through its Selenium driver here; the repository's Playwright axe integration is the equivalent supported check.
- URL verification: `/opt/fleet/lib/verify-url.sh` passed live with 200, title, `lang=en`, one H1, main landmark, image alt coverage, labeled controls, and zero console errors.
- Live identity smoke: the Microsoft sign-in button reached `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/oauth2/v2.0/authorize` with no callback-registration error.
- Live Lighthouse: Performance 99, Accessibility 100, Best Practices 100, SEO 100; FCP 1.38 s, LCP 1.445 s, TBT 79 ms, CLS 0.000742.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
EXPECTED_BUILD_SHA=aae2ef63533d5daf18c175feabb03aa482152d9f npm run verify:deployment
```

## Known limits and next steps

- The repair used fictional demo data and fixture identities only. Before a clinic onboards, complete its normal consent, provider credential, data-retention, and operational-backup review.
- Keep `maxReplicas` at one until the managed SQLite and in-process rate limiter are replaced by shared services. The mount guard and `verify:deployment` command now make any accidental drift visible and fail-safe.
