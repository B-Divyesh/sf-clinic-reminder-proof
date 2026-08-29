# Reminder Proof production storage and recovery

The container app deliberately runs exactly one replica. This gives the
in-process rate governor one service-wide state owner. Do not raise
`maxReplicas` above one without moving rate-limit state to a shared service.

An Azure Files share named `sf-clinic-reminder-proof-data` is provisioned for
the service. Its SMB mount cannot currently be prepared by the non-root runtime
image, so it is not attached to the running revision. Do not accept real clinic
records until a compatible durable mount or shared database is in place.

## Backup and restore

Once a compatible durable mount is enabled, configure daily share snapshots
with 30 days of retention. Restore is a controlled operation: scale the app to
zero, restore both `clinic-data.sqlite3` and `clinic-data.key` from the same
snapshot into the share, confirm their owner-only modes, then return the app to
one replica. Never restore the database without its matching encryption key.

After each restore, request `/health`, sign in with a non-production fixture
tenant, and verify a previously saved workspace plus its export. The recovery
objective is RPO 24 hours and RTO 4 hours. The deployment configuration is
`deployment/containerapp.json`.
