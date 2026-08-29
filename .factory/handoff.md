# Repair handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-repair-15`

Verifier base: `7d3175f4a60bc02d05248227b312308583f8f441`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: repaired, deployed, and verified

The release-blocking Azure topology defect from
[verification-19.md](verification-19.md) is repaired. The product’s passed
demo, clinic workflow, privacy, accessibility, and delivery-proof behavior is
unchanged.

## Reproduction and repair

- **QA19-01 reproduced first:** `npm run verify:deployment:current` failed
  exactly with `deployment topology must set minReplicas and maxReplicas to 1`.
  Fresh Azure inspection showed selected revision `0000056` at 100% traffic,
  unhealthy, with a short `9791736e1428` image tag, `maxReplicas: 3`, and no
  `clinic-data` or `clinic-backups` volume mounts. Healthy revision `0000055`
  had the full candidate tag and both shares, but zero declared traffic.
- **Root cause:** an image-only update replaced the Container App revision
  template. It discarded the required Azure Files volumes and relaxed the
  SQLite single-writer boundary. The process correctly refused to start with
  missing `/durable` and `/backups` mounts.
- **Repair:** the topology-aware rollout still composes the full checked-in
  template, and now also requires the latest revision itself to be the same
  healthy, full-SHA, sole-100%-traffic revision. The production verifier now
  explicitly checks selected revision identity and health.
- **Regression coverage:** `@regression:qa19-01` recreates the exact reported
  `0000055`/`0000056` state for candidate
  `9791736e1428961621a50ef8e9e1785c365e76b4`, rejects the unsafe selected
  revision, proves latest-revision convergence is false, and asserts that the
  repaired patch restores one-replica scale, both Azure Files shares, and both
  mount paths.

## Verification

- `npm ci`: PASS — 87 packages; `npm audit --omit=dev`: 0 vulnerabilities.
- `npm test`: PASS — 20 Vitest contracts, 34 Rust API tests, and 40 Chromium
  workflows. The browser suite includes desktop, 390 px mobile, keyboard and
  skip-link use, 200% text, reduced motion, dark/light Axe checks, demo
  isolation/reset, offline read-only behavior, privacy, response policy,
  CIAM redirect, route/404/link, and console checks.
- `npm run check`: PASS — Svelte 0 errors/0 warnings, rustfmt, and Clippy with
  warnings denied.
- `npm run build`: PASS — emits `dist/` and the release API. Public JS is
  82.64 KB raw / 28.63 KB gzip; CSS is 25.92 KB raw / 5.54 KB gzip.
- Default runtime start: PASS with only `PORT`; it generated its data key and
  served `/health`. A local 100-request `/health` smoke returned 100 × 200.
- `/opt/fleet/lib/verify-url.sh`: PASS — HTTPS 200, title, `lang=en`, one
  `<h1>`, main landmark, image alts, named buttons, and no page or console
  errors. The live checker loaded in 678 ms.
- Standalone Axe WCAG 2 A/AA scan: PASS — zero violations. A matching
  disposable ChromeDriver was used because the bundled driver did not match
  Playwright Chromium.

## Deployment evidence

- ACR run `ch1a0` built the repair image with the exact full tag
  `201d7a026870354a194c5784e6ec24ccfb458e9e` and digest
  `sha256:d3d0e81036a7ebb4ff0ba88e932503b04c68b4221e0df314ff820111d33b9e61`.
- The topology-aware rollout created
  `sf-clinic-reminder-proof--0000057`, healthy and `RunningAtMaxScale`, at
  declared 100% traffic. Its template has `minReplicas: 1`, `maxReplicas: 1`,
  `clinic-data` → `/durable`, and `clinic-backups` → `/backups`.
- `npm run verify:deployment:current`: PASS — selected revision `0000057`,
  one replica, the full image tag and public `/health` build SHA, rate statuses
  `200,200,200,200,200,429`, and `Retry-After: 3599`.
- `PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run
  test:e2e`: PASS — 40/40 live Chromium tests (`test-results/.last-run.json`:
  `passed`).

## Release operation

Build and deploy only through the checked-in command; do not use an image-only
Container Apps update:

```sh
git push origin main
az acr build --registry sociobotregistry --image sf-clinic-reminder-proof:<full-HEAD-SHA> \
  --file Dockerfile --platform linux/amd64 \
  --build-arg BUILD_SHA=<full-HEAD-SHA> \
  --build-arg GIT_SHA=<full-HEAD-SHA> \
  --build-arg SOURCE_COMMIT=<full-HEAD-SHA> .
npm run deploy:container -- --image sociobotregistry.azurecr.io/sf-clinic-reminder-proof:<full-HEAD-SHA>
npm run verify:deployment:current
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
```

## Known limits and operator check

- This is a web service, not a package or CLI, so package/consumer validation
  does not apply. It is not a PWA and makes no offline-reload claim; tested
  offline behavior keeps an already loaded ledger readable and disables writes.
- The verifier had no real Entra identity, messaging-provider credential, or
  paid Sociobot subscription. Fixture adapters and protected live boundaries
  passed. Confirm that
  `https://clinic-reminder-proof.sociobot.in/auth/callback` remains registered
  before inviting clinics.
