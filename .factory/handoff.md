# Verification handoff — Reminder Proof

Date: 2026-08-29 UTC
Work order: `clinic-reminder-proof-verify-16`
Candidate: `4a23898a51958f530909bfe9b3e86678403ad86d`
Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status: FAIL — do not release

The candidate deployment revision is unhealthy. It has 100% declared traffic but lacks `/durable` and `/backups` Azure Files mounts and has `maxReplicas: 3`. The API safely panics at startup rather than serving with unsafe storage. Public requests fall back to the previous healthy `659284a…` revision, so the live site is not the candidate.

## Verification summary

- `npm ci`, `npm test` (15 Vitest, 34 Rust, 40 Chromium), `npm run check`, and `npm run build` passed locally.
- The 31-claim manifest exists. Its required final deployment command failed: `npm run verify:deployment:current` reports that production does not have both replica limits set to one.
- Fresh public checks of the older fallback build passed: first-read/demo, 390 px mobile, keyboard focus, reduced motion, Playwright Axe serious/critical, zero console errors, same-origin request log, headers, immutable asset cache, and the 96.86 KB gzip initial-JS budget.
- Demo creation rate limit was observed at five successful requests per first `X-Forwarded-For` client per hour; request six returned 429 and `Retry-After: 3599`.

See `.factory/verification-16.md` for exact commands, control-plane output, screenshots, and defects.

## Required next step

Redeploy the full candidate SHA with the checked-in one-replica topology and both durable Azure Files mounts. Verify it is healthy and the sole traffic revision, then rerun the manifest and production verification. No product code was modified by this verifier.
