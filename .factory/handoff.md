# Polish 1 handoff — Reminder Proof

Date: 2026-08-29 UTC

Work order: `clinic-reminder-proof-polish-1`

Source review: `ab7ae213b8e77b8b5ab56145d373973f5e7a7cb0`

Repaired runtime commit: `c8e6746c675d0d0d2f9ba42604734bb2f3c754fb`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Result

All findings F-1-1 through F-1-8 are resolved. No finding is deferred. The artifact remains a Svelte/Vite client served by the Rust/axum backend in one container, with the original translucent pulse-ledger design intact.

The landing headings now name their sections. The exception-visibility promise has a manifest claim and a real lifecycle test. Resolved sample exceptions leave the open queue, remain visible as resolved ledger evidence, and can still be undone. All reviewed README sentences are now at most 22 words.

## Verification

- Fresh clone: `/tmp/clinic-reminder-proof-polish-1.dltz6U` at the repaired runtime commit.
- `npm ci`: 87 packages, 0 vulnerabilities.
- Every exact command in `.factory/claims.json`: 25/25 passed independently from the fresh clone.
- `npm test`: 7 Vitest contracts, 27 Rust tests, and 33 Chromium tests passed.
- `npm run check`: Svelte 0 errors/warnings, rustfmt clean, clippy with warnings denied.
- `npm run build`: emitted `dist/` and `target/release/reminder-proof-api`.
- Bundle: public JS 82.38 KB raw / 28.58 KB gzip; CSS 25.75 KB raw / 5.52 KB gzip. The 271.99 KB MSAL chunk remains lazy.
- Axe integration: zero serious or critical findings on seven public routes in light and dark themes.
- Browser suite: titles, one h1/main, metadata, deep links, back navigation, route focus, link crawl, console errors, HTTP 404, keyboard use, reduced motion, offline state, 390 px layout, and 200% text reflow all passed.
- Local `verify-url.sh`: 200; title and `lang=en`; one h1 and main; no missing alt text, unlabeled buttons, or console errors.
- Local mobile Lighthouse: Performance 100, Accessibility 100, LCP 1.25 s, CLS 0, TBT 60 ms.
- Local visual evidence: `.factory/qa-artifacts/polish-1/local-landing-desktop.png`, `local-demo-mobile.png`, and `local-resolved-mobile.png`.

## Deployment and cold production check

- Pushed `c8e6746c675d0d0d2f9ba42604734bb2f3c754fb` to `origin/main`.
- ACR image: `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:c8e6746c675d`.
- Image digest: `sha256:f30c9edcab1e2a8c2f3591efb61f73981427695c332afd1f9a683ca68b6064c7`.
- Active revision: `sf-clinic-reminder-proof--0000028`, 100% traffic.
- Deployment topology: `minReplicas=1`, `maxReplicas=1`; Azure Files mounted at `/durable` and `/backups`.
- `/health`: exact full repaired SHA.
- Cold `verify-url.sh`: 200, correct title/lang/landmarks, no missing alt text or unlabeled button, no console errors.
- Cold production Playwright: 11/11 selected checks passed, covering the repaired claim, demo isolation/reset, no tracking, section copy, metadata, links, axe, mobile/keyboard/offline/focus, and real HTTP 404.
- Live mobile Lighthouse: Performance 100, Accessibility 100, LCP 1.28 s, CLS 0, TBT 52 ms.
- `/privacy` and `/terms`: 200; unknown routes: 404 with the designed recovery page.
- Live visual evidence: `.factory/qa-artifacts/polish-1/live/screenshot-desktop.png`, `screenshot-mobile.png`, and `resolved-mobile.png`.

## Run and verify

```sh
npm ci
npm test
npm run check
npm run build
```

Run any declared claim exactly as listed in `.factory/claims.json`, for example:

```sh
npm run test:e2e -- --grep @claim:sample-exception-visibility
```

## Remaining work

None for this review or polish round. Live provider dispatch, payment submission, and destructive deletion of a real clinic were not performed because the work order supplied no clinic identity, consented recipient, provider credential, or payment authorization. Their deterministic signed-fixture and browser boundary tests pass.
