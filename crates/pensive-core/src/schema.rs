pub const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
PRAGMA secure_delete = ON;
PRAGMA temp_store = MEMORY;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS metadata (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS devices (
  device_id TEXT PRIMARY KEY,
  public_key TEXT NOT NULL,
  signing_seed TEXT NOT NULL,
  issued_at TEXT NOT NULL,
  revoked_at TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS sources (
  source_id TEXT PRIMARY KEY,
  source_type TEXT NOT NULL,
  provider TEXT NOT NULL,
  external_id TEXT,
  title TEXT,
  captured_at TEXT NOT NULL,
  occurred_from TEXT,
  occurred_to TEXT,
  original_timezone TEXT,
  content_object_id TEXT NOT NULL,
  plaintext_hash TEXT NOT NULL,
  ciphertext_cid TEXT NOT NULL,
  parser_name TEXT NOT NULL,
  parser_version TEXT NOT NULL,
  provenance_assurance TEXT NOT NULL,
  sensitivity TEXT NOT NULL,
  state TEXT NOT NULL,
  created_by_device TEXT NOT NULL REFERENCES devices(device_id),
  UNIQUE(provider, external_id, plaintext_hash)
) STRICT;

CREATE INDEX IF NOT EXISTS sources_provider_external ON sources(provider, external_id);
CREATE INDEX IF NOT EXISTS sources_captured_at ON sources(captured_at DESC);

CREATE TABLE IF NOT EXISTS source_fragments (
  fragment_id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL REFERENCES sources(source_id),
  external_id TEXT,
  parent_external_id TEXT,
  role TEXT,
  occurred_at TEXT,
  locator_json TEXT NOT NULL,
  normalized_text TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  sensitivity TEXT NOT NULL,
  third_party INTEGER NOT NULL DEFAULT 0 CHECK(third_party IN (0, 1)),
  secret_candidate INTEGER NOT NULL DEFAULT 0 CHECK(secret_candidate IN (0, 1)),
  injection_candidate INTEGER NOT NULL DEFAULT 0 CHECK(injection_candidate IN (0, 1)),
  UNIQUE(source_id, external_id, content_hash)
) STRICT;

CREATE VIRTUAL TABLE IF NOT EXISTS fragments_fts USING fts5(
  fragment_id UNINDEXED,
  normalized_text,
  tokenize='trigram'
);

CREATE TABLE IF NOT EXISTS memory_items (
  memory_id TEXT PRIMARY KEY,
  memory_type TEXT NOT NULL,
  statement TEXT NOT NULL,
  epistemic_status TEXT NOT NULL,
  review_state TEXT NOT NULL,
  evidence_strength TEXT NOT NULL,
  valid_from TEXT,
  valid_to TEXT,
  sensitivity TEXT NOT NULL,
  third_party INTEGER NOT NULL DEFAULT 0 CHECK(third_party IN (0, 1)),
  current_revision INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  reviewed_at TEXT,
  reviewed_by TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS memory_revisions (
  memory_id TEXT NOT NULL REFERENCES memory_items(memory_id),
  revision INTEGER NOT NULL,
  statement TEXT NOT NULL,
  review_state TEXT NOT NULL,
  changed_at TEXT NOT NULL,
  reason TEXT NOT NULL,
  PRIMARY KEY(memory_id, revision)
) STRICT;

CREATE TABLE IF NOT EXISTS memory_evidence (
  memory_id TEXT NOT NULL REFERENCES memory_items(memory_id),
  fragment_id TEXT NOT NULL REFERENCES source_fragments(fragment_id),
  relation TEXT NOT NULL,
  extractor TEXT NOT NULL,
  extractor_version TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY(memory_id, fragment_id, relation)
) STRICT;

CREATE TABLE IF NOT EXISTS memory_links (
  from_memory_id TEXT NOT NULL REFERENCES memory_items(memory_id),
  to_memory_id TEXT NOT NULL REFERENCES memory_items(memory_id),
  relation TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY(from_memory_id, to_memory_id, relation)
) STRICT;

CREATE TABLE IF NOT EXISTS conflicts (
  conflict_id TEXT PRIMARY KEY,
  left_memory_id TEXT NOT NULL REFERENCES memory_items(memory_id),
  right_memory_id TEXT NOT NULL REFERENCES memory_items(memory_id),
  state TEXT NOT NULL,
  reason TEXT NOT NULL,
  created_at TEXT NOT NULL,
  resolved_at TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS context_packs (
  pack_id TEXT PRIMARY KEY,
  purpose TEXT NOT NULL,
  query_text TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  temporal_cutoff TEXT NOT NULL,
  canonical_digest TEXT NOT NULL,
  pack_json TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS memory_events (
  event_id TEXT PRIMARY KEY,
  protocol TEXT NOT NULL,
  vault_id TEXT NOT NULL,
  device_id TEXT NOT NULL REFERENCES devices(device_id),
  hlc TEXT NOT NULL,
  event_type TEXT NOT NULL,
  entity_id TEXT,
  expected_revision INTEGER,
  payload_json TEXT NOT NULL,
  previous_device_event_hash TEXT,
  created_at TEXT NOT NULL,
  event_hash TEXT NOT NULL,
  signature TEXT NOT NULL,
  UNIQUE(device_id, hlc),
  UNIQUE(device_id, event_hash)
) STRICT;

CREATE TABLE IF NOT EXISTS audit_events (
  sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  event_type TEXT NOT NULL,
  opaque_subject TEXT,
  outcome TEXT NOT NULL,
  reason TEXT,
  created_at TEXT NOT NULL,
  previous_hash TEXT,
  event_hash TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE IF NOT EXISTS quarantine_items (
  quarantine_id TEXT PRIMARY KEY,
  source_label_hash TEXT NOT NULL,
  reason TEXT NOT NULL,
  created_at TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS audit_events_no_update
BEFORE UPDATE ON audit_events BEGIN SELECT RAISE(ABORT, 'audit events are immutable'); END;

CREATE TRIGGER IF NOT EXISTS audit_events_no_delete
BEFORE DELETE ON audit_events BEGIN SELECT RAISE(ABORT, 'audit events are immutable'); END;

CREATE TRIGGER IF NOT EXISTS accepted_memory_requires_evidence_insert
BEFORE INSERT ON memory_items
WHEN NEW.review_state = 'ACCEPTED'
BEGIN SELECT RAISE(ABORT, 'accepted memory requires evidence and review transaction'); END;
"#;
