# Database migrations

The service applies numbered SQLite migrations at startup. Production runs one
writer with a durable database/key snapshot because the factory supplies only
`PORT` and no managed PostgreSQL connection.

- `0001_managed_clinic.sql` creates the existing encrypted workflow store.
- `0002_accounts_subscriptions.up.sql` adds normalized M2 account, tenant,
  location, membership, subscription, audit, preference, and export tables.
- `0002_accounts_subscriptions.down.sql` reverses only M2. The migration test
  applies up, rolls down, checks removal, and applies up again.

Foreign keys and tenant-qualified queries are mandatory. The application keeps
the encrypted workflow document for the later reminder engine while normalized
M2 records remain the authority for identity, roles, billing, and deletion.
