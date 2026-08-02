# Architecture

Updated: 2026-08-02

## Core rule

Source records what existed. Memory records a revisable claim about what it may mean. Context Pack selects the minimum needed now. SimWorld tests a candidate without side effects. Spell Kernel alone may change the world after independent authorization. Arcane preserves ciphertext without understanding it.

## v0.1 process boundary

The `pensive-core` Rust crate owns Vault cryptography, SQLCipher storage, import validation, evidence review, lexical retrieval, Context Pack policy, audit, and recovery. Both the `pensive` CLI and the Tauri desktop call this crate directly. There is no localhost public API and no remote service.

The TypeScript UI is a presentation layer. It never opens the database, derives keys, parses a ChatGPT archive, or decides review policy. The Tauri IPC accepts explicit file selections and sends them to the Rust core.

## Write path

1. The owner selects one Source file.
2. The importer rejects symlinks, unsafe ZIP paths, excessive nesting, compression bombs, unsupported size, and malformed conversation records.
3. The original conversation or file bytes are encrypted as a Source object.
4. Searchable Fragment text and locators enter SQLCipher and FTS5.
5. A signed Memory Event and content-free Audit Event are appended.
6. AI does not run and no external URL is fetched.

## Memory authority

Candidate Memory insertion requires a valid Fragment ID. Review is an explicit owner action. Correction creates a new revision and keeps the earlier statement. Contradictions create a Conflict Record and mark claims disputed; they are not resolved with last-write-wins.

## Context disclosure

The v0.1 builder uses local retrieval only. It excludes Secret candidates and any sensitivity outside policy, reports omissions/redactions, fixes a temporal cutoff and 24-hour expiry, hashes the canonical pack, and signs the digest with the active device key. Export is encrypted again with a domain-separated key.

## Storage boundary

- `vault.json`: protocol, format, opaque Vault ID, created time
- `key-envelope.pmk`: Argon2id-wrapped random Vault Root Key
- `catalog.sqlite`: SQLCipher catalog + FTS + current materialized view
- `.pmo`: XChaCha20 encrypted Source object
- `.pmj`: XChaCha20 encrypted signed Memory Event
- `recovery-status.json`: non-secret readiness flags

Every use derives a separate BLAKE3 domain key from the random Vault Root Key.

## Future adapters

The schemas in `schemas/` are public boundaries. v0.1 contains a DRAFT-only Spell Ticket schema but no handoff implementation. Arcane and multi-device transports are roadmap work and must never reuse or directly open another repository's database.
