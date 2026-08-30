PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
  oid TEXT PRIMARY KEY,
  display_name TEXT NOT NULL DEFAULT '',
  last_sign_in INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS organizations (
  id TEXT PRIMARY KEY,
  owner_oid TEXT NOT NULL REFERENCES users(oid),
  display_name TEXT NOT NULL,
  jurisdiction TEXT NOT NULL,
  retention_days INTEGER NOT NULL CHECK(retention_days IN (30, 90, 365)),
  deletion_scheduled_at INTEGER,
  deletion_cancel_until INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS locations (
  id TEXT PRIMARY KEY,
  organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  timezone TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS locations_organization_idx ON locations(organization_id);

CREATE TABLE IF NOT EXISTS memberships (
  id TEXT PRIMARY KEY,
  organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  user_oid TEXT REFERENCES users(oid),
  display_name TEXT NOT NULL,
  email TEXT NOT NULL DEFAULT '',
  role TEXT NOT NULL CHECK(role IN ('owner', 'manager', 'staff', 'viewer')),
  state TEXT NOT NULL CHECK(state IN ('active', 'pending')),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS memberships_org_user_idx
  ON memberships(organization_id, user_oid) WHERE user_oid IS NOT NULL;
CREATE INDEX IF NOT EXISTS memberships_user_idx ON memberships(user_oid);

CREATE TABLE IF NOT EXISTS subscriptions (
  id TEXT PRIMARY KEY,
  organization_id TEXT NOT NULL UNIQUE REFERENCES organizations(id) ON DELETE CASCADE,
  entitlement_hash TEXT,
  tier TEXT CHECK(tier IN ('clinic', 'practice', 'network')),
  status TEXT NOT NULL CHECK(status IN ('none', 'active', 'grace', 'past_due', 'cancelled', 'revoked', 'unavailable')),
  checked_at INTEGER,
  expires_at INTEGER,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_events (
  id TEXT PRIMARY KEY,
  organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  actor_oid TEXT NOT NULL,
  action TEXT NOT NULL,
  target TEXT NOT NULL,
  occurred_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS audit_events_organization_idx
  ON audit_events(organization_id, occurred_at);

CREATE TABLE IF NOT EXISTS notification_preferences (
  membership_id TEXT PRIMARY KEY REFERENCES memberships(id) ON DELETE CASCADE,
  digest_enabled INTEGER NOT NULL DEFAULT 0 CHECK(digest_enabled IN (0, 1)),
  exception_email INTEGER NOT NULL DEFAULT 0 CHECK(exception_email IN (0, 1)),
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS export_jobs (
  id TEXT PRIMARY KEY,
  organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  requester_oid TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('ready', 'expired')),
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS export_jobs_organization_idx
  ON export_jobs(organization_id, created_at);

INSERT OR IGNORE INTO schema_migrations(version, name, applied_at)
VALUES (2, 'accounts_subscriptions', unixepoch());
