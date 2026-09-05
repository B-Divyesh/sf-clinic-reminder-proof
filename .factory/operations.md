# Reminder Proof production storage and recovery

The Container App runs exactly one replica. This is the consistency boundary
for SQLite and the in-process per-client rate limiter. Do not raise
`maxReplicas` until both have moved to shared services.

## Durable mounts

`deployment/containerapp.json` attaches two ReadWrite Azure Files shares
directly to the non-root application process:

- `clinic-reminder-proof-data` mounts at `/data` for the generated AES-256 key
  and matching online recovery snapshot. The active SQLite working file stays
  on the one permitted container replica. Every saved change synchronously
  updates the matching pair on `/data`, and startup restores it before serving.
  This avoids SQLite byte-lock failures on SMB while retaining durable state.
- `clinic-reminder-proof-backups` at `/backups` holds the latest consistent
  database backup and matching key.

No init container, `chmod`, root process, or mount preparation is required.
Startup restores the durable pair before serving and fails closed if the pair
is incomplete, a required mounted location is not writable, or a required
Azure Files mount is absent. The container image sets this mount guard itself,
so deployment drift cannot silently accept clinic records on ephemeral storage.
Every successful workspace mutation uses SQLite's online backup API while the
database mutex is held, then atomically replaces the latest backup pair. The
first mutation each UTC day also creates a dated recovery pair; the service
automatically removes daily pairs older than 30 days.

M2 account records use normalized tables in the same database. Migration
`0002_accounts_subscriptions.up.sql` adds users, organizations, locations,
memberships, subscriptions, audit events, notification preferences, and export
jobs. Its matching down migration is exercised before every release. The
encrypted workflow document remains separate so later reminder processing can
change without weakening the account and role boundary.

## Backup and restore

The latest application backup has an RPO of one successful workspace mutation,
and dated recovery points have 30-day retention.
Enable daily Azure Files share snapshots with 30-day retention on both shares
for a second recovery layer. The data and backup shares are separate so a bad
database write does not overwrite the only recoverable copy.

To restore, scale the app to zero, copy
`clinic-data.latest.sqlite3` and `clinic-data.latest.key` from the backup share
to `clinic-data.sqlite3` and `clinic-data.key` on the data share, then return
the app to one replica. Never restore a database without its matching key.

After restore, request `/health`, sign in with a non-production account, and
verify the restored workspace and export. The automated regression
`managed_backup_pair_restores_after_database_loss` performs the same backup
pair restore into a fresh data directory and reads the original clinic.

## Release topology check

Commit and push the implementation before a rollout. Use
`/opt/fleet/lib/deploy-container.sh clinic-reminder-proof "$PWD" Dockerfile
8080`. The wrapper builds the committed source, resolves its ACR tag to an
immutable manifest digest, and retains the existing volumes, probes,
environment, and one-replica boundary. Do not use an image-only Container Apps
update for this service.

After each rollout, run `npm run verify:deployment:current` from this
repository with Azure access. It verifies the immutable digest, one active
healthy replica, both named Azure Files mounts, matching public health/footer
identity, and a per-client demo creation boundary of five successful requests
followed by 429 with `Retry-After`. When an evidence-only commit follows an
implementation commit, `.factory/runtime-release.json` names the implementation
SHA that the verifier must expect.
