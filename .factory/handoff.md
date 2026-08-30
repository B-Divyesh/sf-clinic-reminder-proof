# Independent verification handoff — Reminder Proof

Date: 2026-08-30 UTC

Work order: `clinic-reminder-proof-verify-21`

Tested candidate: `e40583f5c49e4754a850274ed5467327e2a156ea`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: PASS

Candidate `e40583f5c49e4754a850274ed5467327e2a156ea` passes independent
product QA. The earlier deployment-only failure is not present in fresh
evidence. Active revision `sf-clinic-reminder-proof--0000060` uses the full
candidate image, is healthy, owns 100% traffic, runs one replica, and mounts
the required `/durable` and `/backups` Azure Files shares.

No product code was modified. The complete evidence is in
`.factory/verification-21.md`.

## Verification summary

- Mandatory claims: 31/31 exact manifest commands passed separately.
- Cold first read: passed; job, audience, and first action are all visible.
- One-click sample: passed; `/demo` is populated, isolated, resettable, and
  clearly labeled.
- Local suite: 21 Vitest, 34 Rust, and 40 Chromium tests passed.
- Live browser suite: 40/40 passed against production.
- Type/lint/format: Svelte check, rustfmt, and Clippy with warnings denied all
  passed.
- Exact candidate build: passed; `dist/` and the release API binary emitted.
- Candidate identity: live JS matched the exact candidate build byte-for-byte;
  live health and footers matched the full/short SHA.
- Accessibility: zero serious/critical Axe findings; desktop, 390 px, 200%
  text, keyboard, focus, touch targets, dark theme, and reduced motion passed.
- Privacy/security: same-origin demo requests, secure scoped 24-hour cookie,
  security headers, immutable asset caching, and no console/page errors passed.
- Backend: structured boundary errors, one-replica durable topology, live
  limits, metrics, health, and 100-request concurrency smoke passed.
- Performance: Lighthouse 95/100/100/100; LCP 1.5 s, CLS 0.001, 89 KiB
  transfer; bundle budgets passed.

## Rate-limit evidence

- Demo workspace creation allowance observed: five per client per hour.
  Request six returned 429 with `Retry-After: 3599`.
- General API burst: 44 × 200 and 16 × 429 from 60 concurrent reads; every
  429 had `Retry-After: 1`.
- Protected write burst: 40 × 401 and 20 × 429 from 60 concurrent anonymous
  writes; every 429 had `Retry-After: 1`.
- `/health` is exempt and returned 100 × 200 at concurrency 20.

## How to verify

```sh
git checkout --detach e40583f5c49e4754a850274ed5467327e2a156ea
npm ci
npm test
npm run check
candidate_sha=e40583f5c49e4754a850274ed5467327e2a156ea
BUILD_SHA="$candidate_sha" GIT_SHA="$candidate_sha" SOURCE_COMMIT="$candidate_sha" npm run build
PLAYWRIGHT_BASE_URL=https://clinic-reminder-proof.sociobot.in npm run test:e2e
npm run verify:deployment:current
```

The direct demo entry is
<https://clinic-reminder-proof.sociobot.in/?demo=1>.

## Known limits and next steps

- Docker and Podman were unavailable in the verifier. The exact live candidate
  image and topology were inspected through Azure and public build identity.
- No real clinic identity, patient data, messaging-provider secret, or paid
  subscription was used. Fixture-backed tests cover those destructive or
  credentialed paths; live checks confirmed the CIAM redirect and protected
  boundaries.
- Before onboarding clinics, retain the existing operator check that the
  production Entra callback registration and provider credentials remain
  current.

## Defects

Critical: none. High: none. Medium: none. Low: none.
