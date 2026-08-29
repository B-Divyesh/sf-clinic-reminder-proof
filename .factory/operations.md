# Reminder Proof production storage and recovery

The Container App runs exactly one replica. This is the consistency boundary
for SQLite and the in-process per-client rate limiter. Do not raise
`maxReplicas` until both have moved to shared services.

## Durable mounts

`deployment/containerapp.json` attaches two independent ReadWrite Azure Files
shares directly to the non-root application process:

- `clinic-reminder-proof-data` at `/durable` holds a consistent online SQLite
  snapshot and its generated AES-256 key. SQLite itself runs on local `/data`,
  avoiding unsupported SMB file locking.
- `clinic-reminder-proof-backups` at `/backups` holds the latest consistent
  database backup and matching key.

No init container, `chmod`, root process, or mount preparation is required.
Startup restores the durable pair before serving and fails closed if the pair
is incomplete, either mounted location is not writable, or either required
Azure Files mount is absent. The container image sets this mount guard itself,
so deployment drift cannot silently accept clinic records on ephemeral storage.
Every successful workspace mutation uses SQLite's online backup API while the
database mutex is held, then atomically replaces the latest backup pair. The
first mutation each UTC day also creates a dated recovery pair; the service
automatically removes daily pairs older than 30 days.

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

After each Container Apps rollout, run `npm run verify:deployment` from this
repository with Azure access. It fails unless the active revision has exactly
one replica, both named Azure Files mounts, a healthy live endpoint, and a
per-client demo creation boundary of five successful requests followed by 429
with `Retry-After`.
