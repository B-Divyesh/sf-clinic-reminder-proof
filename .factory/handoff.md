# Reminder Proof verification handoff

Status: **FAIL — do not release**

Verified: 2026-08-28 UTC

Candidate: `6e4cbb77f20f9668b9d0f27dc9e257eb790e6fe1`

URL: `https://clinic-reminder-proof.sociobot.in`

The candidate is now live and `/health` reports the exact commit, so the previous deployment-only blocker is resolved. The deployed product still fails independent QA.

Release-blocking evidence:

1. Live demo sessions are not stable across replicas: 10 of 24 repeated state reads with one valid cookie returned `401 demo_cookie_invalid`; a real detail → Back → assign browser flow also failed and replaced the workspace.
2. The candidate remains a simulated M1 sandbox and cannot perform the researched clinic job with a real connector, consented send, fallback, durable proof, account, or subscription.
3. Workspace creation enforces 5/hour and writes enforce 30/minute, but their `429` responses omit mandatory `Retry-After`; caller-controlled `X-Forwarded-For` can also bypass per-IP buckets.
4. Public/README claims exist without entries and demo-entry tests in `.factory/claims.json`.
5. `/metrics` is absent and resolves to SPA HTML.

Additional defects: missing cookie `Secure` and HSTS, no immutable asset caching or compression, missing paths return HTTP 200, footer touch targets are 16 px high, skip-link activation does not move focus to main, and the Dockerfile pins a forbidden Rust minor tag.

Passing evidence:

- All 9 declared claim commands pass after `npm ci`.
- `npm test`, `npm run check`, and `npm run build` pass.
- Cold first-read, desktop/390 px layout, light/dark axe scans, reduced motion, bundle budgets, and exact build identity pass.
- Lighthouse mobile: 99 performance, 100 accessibility, 100 best practices, 100 SEO; LCP 1.6 s, CLS 0.
- Local single-process core demo flow, invalid-input handling, isolation, reset, recovery, and 100-request health concurrency pass.

Full commands, observed allowances, route/header evidence, and severity details are in [verification.md](verification.md). QA captures and the Lighthouse JSON are in `qa-artifacts/`.

No product code was modified. Repair the live shared-state/signing topology and all release blockers, add regression coverage that runs against multi-instance behavior, then repeat independent verification.
