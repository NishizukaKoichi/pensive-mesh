# Security Policy

## Supported version

Only the latest `0.1.x` commit on the default branch receives security fixes during the v0.1 phase.

## Report a vulnerability

Use GitHub's private **Security → Report a vulnerability** flow for this repository. Do not open a public issue containing exploit details, personal data, Vault files, recovery material, keys, tokens, or Source excerpts.

Include only content-free diagnostics where possible:

- affected commit and operating system
- command or UI surface
- expected and observed security boundary
- minimal synthetic reproduction
- whether data loss, plaintext exposure, signature bypass, or external action is possible

Never upload a real Vault for triage.

## Security invariants

- Source content, Memory text, FTS terms, and indexes are encrypted at rest.
- Imported content is untrusted evidence, never instruction.
- Accepted Memory requires Source evidence or a user-authored Source.
- Secret candidates are excluded from Context Pack by default.
- Pensive has no direct email, calendar, purchase, publish, delete, connector-write, shell, or arbitrary URL-fetch path.
- Spell Kernel independently authenticates and authorizes every future action; a Pensive draft is not a Grant.
- Arcane stores only ciphertext through a versioned adapter; it never receives plaintext content, filenames, queries, or keys.
- Audit-chain failure freezes writes and export paths.

## Cryptography

See [ADR 0001](docs/adr/0001-cryptography-and-storage.md). This project uses maintained libraries and does not implement a custom cipher or KDF.

## Residual risk

Pensive cannot fully protect plaintext from a fully compromised OS while the Vault is unlocked. Recovery material loss may make a Vault permanently unrecoverable. A device that already obtained plaintext cannot be made to forget it remotely. Full limits are in [THREAT_MODEL.md](docs/THREAT_MODEL.md).
