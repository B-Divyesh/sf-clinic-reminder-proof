# M2 deployment repair handoff — Reminder Proof

Date: 2026-09-05 UTC

Work order: `clinic-reminder-proof-m2-build-1`

Live URL: <https://clinic-reminder-proof.sociobot.in>

## Status

M2 remains the current milestone. The mutable-image finding is fixed. The
production app now runs the implementation through an immutable ACR manifest
digest. Pilot billing registration remains an external dependency, so M2 is
not accepted as a purchasable live-dispatch release.

## Release identity

- Implementation commit: `1e543b6d6d25997267a1af48586f17609d3b0eeb`
- Documentation baseline reviewed before this repair: `91f56aa6591358192569074303fda1e93679cc88`
- Live revision: `sf-clinic-reminder-proof--0000069`
- Live image: `sociobotregistry.azurecr.io/sf-clinic-reminder-proof@sha256:cb3684c090e566fc46864c3617b66e203043f5a00f5f029f174580fc682a5e3b`

The implementation and documentation records are intentionally separate. The
footer and `/health` report the implementation SHA. Documentation-only commits
do not require a new product image; `.factory/runtime-release.json` keeps the
current deployment verifier bound to the implementation SHA.

## What changed

- Updated the deployment verifier to require the product's immutable ACR
  digest instead of a mutable image tag. The regression rejects short tags and
  other repositories.
- Built and deployed the clean implementation with
  `/opt/fleet/lib/deploy-container.sh`. The wrapper resolved the ACR build to
  the digest above without changing the committed Dockerfile.
- The active revision is in Single mode, has one running replica, and preserves
  `clinic-data` at `/data` plus `clinic-backups` at `/backups`.
- Added a runtime-release record so the topology claim can verify the deployed
  implementation when later commits contain evidence only.

## Verification

- `npm ci` passed with zero reported vulnerabilities.
- `npm test` passed: 22 Vitest contracts, 42 Rust tests, and 47 Playwright
  browser tests.
- `npm run check` passed with zero Svelte diagnostics, clean rustfmt, and
  Clippy warnings denied.
- Full-SHA `npm run build` produced `dist/` and the release API binary. Initial
  JavaScript was 31.82 KB gzip and CSS was 5.79 KB gzip.
- All 37 exact commands in `.factory/claims.json` passed. The live topology
  command observed `200, 200, 200, 200, 200, 429` and `Retry-After: 3599`.
- `npm run verify:deployment:current` passed against revision `0000069`. It
  confirmed the digest, health SHA, footer build identity, one replica, both
  Azure Files mounts, and the live rate-limit boundary.
- `/health` returned the implementation SHA. Fresh desktop and 390 px phone
  sessions showed the job, audience, action, and three facts before scrolling.
  They had no console errors.
- The live mount binding remained `clinic-data` → `/data` and
  `clinic-backups` → `/backups`. The durable share's existing key file was
  present after redeploy. No clinic workspace exists on the live share, so no
  customer record was read or changed for this check.

## Pilot billing dependency

The Clinic plan remains paid at **$79 per location each month**. Practice and
Network remain paid monthly choices in M2. No paid deliverable was removed or
made free.

The live service defaults to the pilot Sociobot billing catalog. Its checkout
for this product remains unavailable, so a new clinic cannot complete the paid
activation required before real dispatch. This is an external catalog
registration dependency, not a mock checkout. Public offer metadata is at
`/work/.evidence/billing-offer.json`.

The operator must enable the three recurring pilot tiers, then complete an
authorized test checkout, return, cancellation, and revocation check. Until
then the app keeps free demo, read, export, and safety paths available and
shows the billing-unavailable recovery state.

## Known limits and next steps

1. Enable the pilot billing catalog for Clinic, Practice, and Network. This is
   required before M2 can claim a purchasable live-dispatch path.
2. Run the authorized paid return, cancellation, and revocation journey after
   registration. Do not use real patient data in the public demo.
3. Before adding a second API or worker replica, move SQLite state and leases
   to PostgreSQL as specified in the venture plan.
