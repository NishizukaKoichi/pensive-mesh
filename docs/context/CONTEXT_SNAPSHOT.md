# Context Snapshot

## Header

- Date: 2026-08-02
- Scope: Pensive Mesh v0.1 Local Memory Core, public source repository
- Audience: owner, contributors, security reviewers
- Canonical sources:
  - Repo: `/Volumes/Pensive/pensive-mesh`
  - Product spec: `docs/PRODUCT_SPEC.md` (Desktop source SHA-256 `570b5bba2b59822fd01a6ac37b9837426cb718db8d004f31497e5ba187b393e8`)
  - Related boundaries: `NishizukaKoichi/spell-runtime`, `NishizukaKoichi/arcane-commons-mesh`
  - Chat: owner instruction dated 2026-08-02

## Goal and Success Criteria

- Goal: ship the smallest coherent, locally usable implementation of the Pensive Mesh specification and publish it for public inspection and use.
- Success criteria: encrypted local vault; idempotent ChatGPT import with branches; Source/Memory separation; evidence review; local search; signed Context Pack; recovery export and clean restore; functional macOS desktop and CLI; green local/CI gates.
- Out of scope for v0.1: external model inference, voice/STT, real Spell execution, real Arcane remote storage, multi-device sync, platform signing/notarization, external security review, geographic restore, and third-party portable-reader certification.

## Current State

- The canonical repository now contains a working v0.1 Local Memory Core at the path required by the specification.
- `pensieve-local` exists as a separate prototype and is not migrated or modified.
- Spell Runtime and Arcane Commons Mesh remain separate repositories and databases.
- Local format, lint, typecheck, test, integration, build, encrypted-at-rest, recovery, browser, desktop bundle, launch, and dependency-audit gates have run.
- The arm64 `.app` and valid DMG exist locally; the application is not Developer ID signed or notarized.
- GitHub authentication is available; `NishizukaKoichi/pensive-mesh` does not yet exist.

## Constraints

- Canonical development occurs only under the writable, resolved `/Volumes/Pensive` mount.
- No network/model access is required for the first usable journey.
- The code license is an owner decision; publication is gated until that ADR is resolved.
- Security boundaries take precedence over feature breadth.

## Decisions

- 2026-08-02: Implement v0.1 before claiming v1.0. v1.0 contains physical-device, geographic, third-party, and independent-review evidence that cannot be manufactured by code.
- 2026-08-02: Use a Rust core shared by the CLI and Tauri desktop; keep the UI thin and the protocol schemas public.
- 2026-08-02: Use SQLCipher, XChaCha20-Poly1305, BLAKE3, Ed25519, and Argon2id through maintained libraries; no custom cryptography.
- 2026-08-02: Do not copy or mutate the separate `pensieve-local` prototype.

## Open Questions

- Blocking before public-use publication: owner code-license choice.
- Non-blocking for v0.1: production signing identity and remote Arcane deployment.

## Next Actions

- Codex: commit the verified v0.1 implementation, resolve the owner license gate, create and publish the GitHub repository, and confirm CI.

## Risks

- Risk: broad v1 specification creates false-completion pressure. Mitigation: report each version only against its explicit acceptance evidence.
- Risk: local malware while unlocked. Mitigation: document as residual threat; minimize plaintext lifetime and lock explicitly.
- Risk: recovery material loss. Mitigation: onboarding warning remains until an encrypted kit is exported and clean-tested.
