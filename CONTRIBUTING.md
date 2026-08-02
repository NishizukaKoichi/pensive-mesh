# Contributing to Pensive Mesh

Pensive deals with intimate personal context. Small, auditable changes are preferred over broad feature additions.

## Before changing code

1. Read `AGENTS.md`, `docs/PRODUCT_SPEC.md`, and `docs/THREAT_MODEL.md`.
2. State which trust boundary the change touches.
3. Add a failing test for security- or data-semantics changes.
4. Keep Source, Memory, Context, Simulation, Action, and Backup responsibilities separate.

## Local checks

```bash
pnpm install --frozen-lockfile
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm test:integration
pnpm build
pnpm verify:mvp
pnpm scan:plaintext
cargo audit
```

## Pull requests

Describe:

- what changed and why
- evidence or test that proves it
- privacy/security boundary affected
- alternatives considered
- rollback or recovery path

Do not include real owner data, production keys, exported conversations, Vault files, or screenshots containing personal context. Use synthetic fixtures.

## Dependency changes

Dependency additions/removals are medium-impact decisions. Record the purpose, alternatives, risks, and rollback in `docs/adr/` and keep the lockfiles updated.

## License note

Pensive Mesh is licensed under Apache-2.0. Unless explicitly stated otherwise, intentionally submitted contributions are accepted under the same license, as described by Section 5 of `LICENSE`.
