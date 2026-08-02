# Context Pack Protocol

Protocol: `pensive-context-pack/1`

A Context Pack is a purpose-bound, time-bound selection. It is not a copy of the Vault and does not grant action authority.

## v0.1 guarantees

- fixed `pack_id`, creation time, expiry, and temporal cutoff
- explicit provider/model target (`local` / `none` in v0.1)
- explicit sensitivity and third-party policy
- Source Fragment references and revisable Memory items
- contradictions, omissions, and redactions listed separately
- BLAKE3 canonical digest and Ed25519 device signature
- encrypted `.pmx` export
- no external provider call

## Defaults

- expiry: 24 hours
- sensitivity: `PERSONAL`, `SENSITIVE`
- Secret: excluded
- third-party Memory: excluded
- Candidate/Disputed: included only with their state visible
- action authority: none

The normative JSON shape is [context-pack-v1.json](../schemas/context-pack-v1.json).
