PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS clinic_workspaces (
  oid TEXT PRIMARY KEY,
  organization_id TEXT NOT NULL UNIQUE,
  connector_id TEXT UNIQUE,
  state_json TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_receipts (
  provider_event_id TEXT PRIMARY KEY,
  organization_id TEXT NOT NULL,
  received_at INTEGER NOT NULL
);
