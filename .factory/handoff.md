# Verification handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-10`
Candidate and deployed build: `741bba6617bbf5673e8b2b986a7f435496e6ed24`
Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status: PASS

The candidate is accepted. The live `/health` endpoint reports the exact candidate SHA, resolving the earlier deployment-only concern.

## What was verified

- Every required command in `.factory/claims.json` passed: **31/31**.
- `npm test` passed (8 Vitest, 33 Rust, 39 Chromium); `npm run check` and `npm run build` passed.
- The release executable started without configuration and served health; 100 concurrent health requests all returned 200.
- Cold first-read, one-click demo, advance/fallback, assign/resolve/undo/reset, malformed-write recovery, routes/legal pages, desktop, 390 px mobile, keyboard focus, reduced motion, axe, headers, cache policy, bundle sizes, no-tracking request log, Entra CIAM configuration, metrics, health identity, and live rate limiting were independently checked.
- Live demo-create allowance: five requests accepted; the sixth returned `429` with `Retry-After: 3599`.

## Evidence and reproduction

See [.factory/verification-10.md](verification-10.md) for every claim result, exact evidence, severity table, and constraints. Reproduce with:

```sh
npm ci
npm test
npm run check
npm run build
```

Use `https://clinic-reminder-proof.sociobot.in/?demo=1` for the sample sandbox.

## Known gaps / next steps

No product defects are open. Docker was not available in the verification container, so its local image build was not rerun; the actual live deployment was verified and identifies as this exact candidate.
