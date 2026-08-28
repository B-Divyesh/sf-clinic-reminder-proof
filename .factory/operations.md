# Reminder Proof production storage and recovery

The container app mounts the `clinic-reminder-proof-data` Azure Files share at
`/data` and deliberately runs exactly one replica. SQLite and the in-process
rate governor therefore have one durable, service-wide state owner. Do not
raise `maxReplicas` above one without moving both clinic storage and rate-limit
state to a multi-writer shared service.

## Backup and restore

The Azure Files share uses a daily share snapshot with 30 days of retention.
Restore is a controlled operation: scale the app to zero, restore both
`clinic-data.sqlite3` and `clinic-data.key` from the same snapshot into the
share, confirm their owner-only modes, then return the app to one replica.
Never restore the database without its matching encryption key.

After each restore, request `/health`, sign in with a non-production fixture
tenant, and verify a previously saved workspace plus its export. The recovery
objective is RPO 24 hours and RTO 4 hours. The deployment configuration is
`deployment/containerapp.json`.
