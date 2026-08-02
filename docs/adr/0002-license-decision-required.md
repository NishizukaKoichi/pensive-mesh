# ADR 0002: Use Apache License 2.0

- Status: accepted
- Date: 2026-08-02

## Context

The owner requested a public GitHub repository that anyone can use. Public visibility alone does not grant permission to copy, modify, or distribute code. The product specification explicitly reserves the code-license choice for the owner.

## Options

- MIT: shortest and most permissive; consistent with Spell Runtime; permits proprietary reuse.
- Apache-2.0: permissive with an explicit patent grant and notice obligations; longer text.
- AGPL-3.0: requires network-service modifications to remain shareable; strongest reciprocity and the highest adoption/compliance cost.

## Decision

The owner selected the recommended Apache License 2.0 on 2026-08-02. It provides broad reuse with a clear patent grant while preserving attribution and modification-notice requirements. The SPDX identifier is recorded in Rust and Node package metadata, and the canonical license text is stored at repository root.

## Consequences

- Anyone may use, modify, and redistribute the work subject to Apache-2.0.
- Distributed modifications must preserve the license and relevant notices and identify changed files.
- The license does not grant rights to Pensive Mesh trade names or marks beyond customary descriptive use.
- Contributions intentionally submitted for inclusion use Apache-2.0 unless explicitly stated otherwise.

## Rollback

This decision can be reverted before outside contributions are accepted. Relicensing after contributions may require every contributor's consent.
