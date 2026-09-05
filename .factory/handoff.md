# Repair handoff — Reminder Proof

Date: 2026-09-05 UTC

Work order: `clinic-reminder-proof-repair-17`

Production URL: <https://clinic-reminder-proof.sociobot.in>

## Status

**PASS for the repaired deployment topology and public rate boundary.** The current Azure revision is healthy, has all traffic, uses the immutable product image, runs exactly one replica, and has durable data and backup mounts.

Implementation SHA: `b58c17feaac9b3ffd9e6fc555036641066cd09ae`

The final documentation commit is recorded separately after this handoff is committed. The deployed implementation SHA remains the value above.

## What changed

- Reapplied the complete Container Apps revision template on every rollout: single-revision mode, `minReplicas: 1`, `maxReplicas: 1`, Azure Files data mount at `/data`, and separate Azure Files backup mount at `/backups`.
- Replaced first-hop `X-Forwarded-For` selection with the final valid hop. The app therefore keys general, demo, and billing rate limits from the ingress-appended client address rather than an address a caller can choose.
- Added outcome tests that vary caller-controlled forwarding prefixes while retaining a single client bucket.
- Kept the active SQLite working database on the one permitted container replica and synchronously writes its matching database/key recovery pair to durable `/data` after every saved change. Startup restores that pair before serving; daily recovery copies remain on `/backups` for 30 days. This is the safe SQLite-on-Azure-Files boundary: direct active SQLite access through SMB produced persistent `database is locked` failures during the attempted rollout, while the durable recovery pair is preserved on the required mount.
- Added a regression which holds a transient SQLite handoff lock for six seconds and proves durable startup waits rather than failing.

No Azure Files data, keys, or backups were deleted or overwritten by the repair. Intermediate crash-looping revisions were deactivated before the final rollout.

## Fresh production evidence

- Revision: `sf-clinic-reminder-proof--0000067`
- Image: `sociobotregistry.azurecr.io/sf-clinic-reminder-proof:b58c17feaac9b3ffd9e6fc555036641066cd09ae`
- Azure mode: `Single`; selected and ready revision are both `0000067`.
- Scale: `minReplicas: 1`, `maxReplicas: 1`; one ready running replica, zero restarts.
- Volumes: `clinic-reminder-proof-data` mounted at `/data`; independent `clinic-reminder-proof-backups` mounted at `/backups`.
- `npm run verify:deployment:current`: passed. Public `/health` and the footer report the implementation SHA. Six same-client demo creates with changed caller prefixes returned `200, 200, 200, 200, 200, 429`; the final response included `Retry-After: 3599`.
- Cold HTTPS route checks: `/`, `/privacy`, `/terms`, `/start`, and `/demo/reminders/mina` returned 200; `/does-not-exist` returned the designed HTTP 404.
- Fresh desktop and 390 px phone landing checks showed the job “See every reminder outcome.”, the independent-clinic audience, and “Try it with sample data” before scrolling. The phone load had no console errors.
- The desktop demo request immediately after the six-request public allowance probe correctly showed the explicit 429 recovery message. The full sample and reset path is covered by the final local browser suite; do not infer a product defect from this deliberate public rate-limit recovery state.

## Verification run

- `npm ci`: passed; 87 packages, 0 reported vulnerabilities.
- `npm test`: passed — 21 Vitest, 42 Rust, and 47 Chromium tests.
- `npm run check`: passed — zero Svelte diagnostics, rustfmt, and Clippy with warnings denied.
- Exact-SHA `npm run build`: passed; the final front-end entry is 31.82 kB gzip and CSS is 5.79 kB gzip.
- Default runtime with only `PORT`: final build SHA returned from `/health`; 100 concurrent health requests all returned 200.
- Full local claim coverage passed through the final browser/API suite. The fresh production topology claim was exercised by the successful `npm run verify:deployment:current` command above so its five-request public rate window was not reused.
- `/opt/fleet/lib/verify-url.sh` passed against the cold production landing: 200, title, `lang=en`, one h1, main landmark, alt/control checks, and no console errors. Live Playwright axe found 0 violations, including 0 serious/critical issues.

## Known dependency

Sociobot hosted checkout remains an external platform dependency. Verification 22 recorded HTTP 500 from the offered Clinic, Practice, and Network checkout URLs on both pilot and production hosts. This repair did not touch Sociobot configuration, billing, credentials, or checkout as required by the work order. A real paid checkout, return, cancellation, and revocation must be retested once that platform dependency is restored.

## Operations

Use the immutable implementation SHA above for the deployed image. Future rollouts must use:

```sh
npm run deploy:container -- --image sociobotregistry.azurecr.io/sf-clinic-reminder-proof:<full-commit>
npm run verify:deployment:current
```

The deployment command composes the checked-in durable topology into the revision template and rejects short or unpublished image tags. Do not mount the active SQLite database directly through Azure Files; retain the one-replica working copy plus synchronous `/data` recovery pair described in `.factory/operations.md`.
