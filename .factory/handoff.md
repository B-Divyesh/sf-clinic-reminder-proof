# Verification handoff — Reminder Proof

Date: 2026-09-05 UTC

Work order: `clinic-reminder-proof-verify-23`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status

**FAIL.** Independent verification found two release blockers and zero untested
claims. The full report is [`.factory/verification-23.md`](verification-23.md).

Current milestone: M2 — accounts, durable clinic data, and subscriptions.

Implementation reviewed: `b58c17feaac9b3ffd9e6fc555036641066cd09ae`

Documentation reviewed: `533c2cff265f4d3f063d0ac968a94c6669dba3fe`

## Findings

1. The healthy active revision `0000068` uses short image tag `533c2cff265f`.
   It has correct Single mode, one replica, `/data`, and `/backups`, but fails
   the mandatory full-SHA deployment claim.
2. The live service defaults to the Sociobot pilot billing gateway. Clinic,
   Practice, and Network checkout each return 404 there, so M2 purchase and
   subscription-gated dispatch cannot finish. The production gateway now
   returns valid hosted-checkout redirects; the remaining pilot issue is an
   external dependency.

## What passed

- Fresh desktop and phone first-read and full sample/reset flows.
- Persistent demo label, realistic five-record output, isolated storage, and
  no third-party demo requests.
- All 37 declared claim commands were run; 36 passed and the deployment claim
  failed on the short tag. No claim was skipped.
- `npm test`: 21 Vitest, 42 Rust, and 47 Chromium tests passed.
- `npm run check` and the exact-implementation build passed.
- Live light/dark axe: zero violations. Mobile Lighthouse: 100/100/100/100,
  LCP 1.41 s, CLS 0.0007, TBT 31.5 ms.
- Keyboard, focus, 200% text, reduced motion, offline read-only state, legal
  pages, route titles, links, security headers, structured errors, and designed
  HTTP 404 behavior passed.
- Tenant isolation, restart recovery, signed intake/receipts, encryption,
  export/deletion ownership, and backup retention passed fresh fixture tests.
- Live 429 recovery included `Retry-After`; caller-supplied forwarding prefixes
  no longer bypass the shared allowance.

## Candidate comparison

The documentation SHA changes only this handoff relative to the implementation
SHA. The live service reports documentation SHA `533c2cff…`. Its app bundle is
the same length as the exact `b58c17fe…` build and is byte-identical after only
the embedded build value is normalized. Product behavior therefore matches the
implementation candidate, but the short active image tag remains noncompliant.

## Reproduce

```sh
git checkout --detach 533c2cff265f4d3f063d0ac968a94c6669dba3fe
npm ci
npm test
npm run check
BUILD_SHA=b58c17feaac9b3ffd9e6fc555036641066cd09ae \
GIT_SHA=b58c17feaac9b3ffd9e6fc555036641066cd09ae \
SOURCE_COMMIT=b58c17feaac9b3ffd9e6fc555036641066cd09ae npm run build
npm run verify:deployment:current
```

## Required next steps

1. Redeploy the unchanged product with a full 40-character immutable image tag
   and rerun the live deployment verifier after the allowance resets.
2. Enable the three pilot billing tiers or complete the planned move to the
   production catalog. Verify an authorized purchase return, cancellation, and
   revocation before accepting M2.

No product code or live configuration was changed by this verification.
