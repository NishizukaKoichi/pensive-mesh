# ADR 0002: Code license decision required

- Status: owner decision required
- Date: 2026-08-02

## Context

The owner requested a public GitHub repository that anyone can use. Public visibility alone does not grant permission to copy, modify, or distribute code. The product specification explicitly reserves the code-license choice for the owner.

## Options

- MIT: shortest and most permissive; consistent with Spell Runtime; permits proprietary reuse.
- Apache-2.0: permissive with an explicit patent grant and notice obligations; longer text.
- AGPL-3.0: requires network-service modifications to remain shareable; strongest reciprocity and the highest adoption/compliance cost.

## Recommendation

Apache-2.0 provides broad reuse with a clear patent grant. No `LICENSE` file will be generated until the owner chooses.

## Rollback

Before accepting outside contributions, the owner can change the initial license. Relicensing after contributions may require every contributor's consent.
