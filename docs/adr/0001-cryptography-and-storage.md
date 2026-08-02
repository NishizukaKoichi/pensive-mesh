# ADR 0001: Cryptography and storage boundary

- Status: accepted
- Date: 2026-08-02

## Decision

Use maintained Rust implementations of Argon2id, XChaCha20-Poly1305, BLAKE3, and Ed25519. Use SQLCipher through `rusqlite` for the catalog, FTS index, and materialized views. Keep raw Source objects encrypted independently with random nonces and vault-scoped keyed deduplication.

`rusqlite` is pinned to 0.39.0 because 0.40.1's bundled `libsqlite3-sys` 0.38.1 build script references an unavailable `cfg_select!` macro under the project's pinned Rust 1.88 toolchain. This is a reproducible upstream build failure; the previous maintained release retains the same SQLCipher feature and avoids silently falling back to plaintext.

The unlock passphrase derives only a wrapping key. A randomly generated vault root key is wrapped in a separate key envelope. Domain-separated keys are derived for the database, objects, journals, and Context Pack signatures. Plaintext databases and plaintext external indexes are prohibited.

## Why

This matches the product specification, keeps every canonical layer encrypted at rest, and avoids custom cryptographic constructions. Independent object encryption permits content-addressed backup without exposing cross-vault equality.

## Alternatives considered

- Plain SQLite plus encrypted fields: rejected because indexes, metadata, and temporary pages can leak content.
- A custom encrypted database: rejected because it expands cryptographic and recovery risk.

## Risks and rollback

SQLCipher builds are larger and platform packaging needs ongoing CI. The public protocol remains independent of SQLCipher, so a future database implementation can rebuild its materialized view from signed journal events. Roll back by restoring a verified encrypted checkpoint and replaying the previous compatible journal.
