# Verification Report

Updated: 2026-08-02
Target: Pensive Mesh v0.1 Local Memory Core

## Current evidence

- Product spec: 2,625 lines, SHA-256 `570b5bba2b59822fd01a6ac37b9837426cb718db8d004f31497e5ba187b393e8`
- Rust core unit tests: encryption/tamper, wrong passphrase, domain separation, SQLCipher at-rest check, streaming JSON, injection label, Recovery roundtrip
- Integration tests: repeated ChatGPT import, partial update, branch/timestamp, attachment, evidence review, contradiction, retrieval trace, Context policy, encrypted export, audit, clean restore, wrong-key/object tamper
- Browser: meaningful content, no Vite overlay, no external resources, no console errors, navigation/search/Context interactions, no horizontal overflow at 500 px
- Lighthouse snapshot: Accessibility 100, Best Practices 100, SEO 100
- Desktop release: arm64 `.app` and DMG built; DMG checksum valid; packaged application launched and remained running during the smoke test
- CLI release path: help, interactive Vault initialization, unlock, and offline doctor status completed against a disposable encrypted Vault
- Product spec checksum: verified after formatting and before commit

## Final gate table

The local implementation gates below were run in the canonical repository. The owner selected Apache-2.0; public CI runs after the GitHub repository is created.

| Gate                   | Command / evidence                     | Status                                                            |
| ---------------------- | -------------------------------------- | ----------------------------------------------------------------- |
| Product spec integrity | `shasum -a 256 -c PRODUCT_SPEC.sha256` | passed                                                            |
| Format                 | `pnpm format:check`                    | passed                                                            |
| Lint                   | `pnpm lint`                            | passed; safety scan plus Clippy                                   |
| Typecheck              | `pnpm typecheck`                       | passed                                                            |
| Unit + integration     | `pnpm test`                            | passed; 9 tests                                                   |
| Dedicated integration  | `pnpm test:integration`                | passed; import/review/context/recovery and tamper paths           |
| Build                  | `pnpm build`                           | passed; frontend and Rust workspace                               |
| CLI smoke              | command against disposable Vault       | passed; interactive init, unlock, and offline doctor status       |
| Desktop bundle         | `pnpm desktop:build`                   | passed; `.app` and arm64 DMG                                      |
| Desktop smoke          | packaged app launch                    | passed; process remained healthy and was then stopped             |
| DMG integrity          | `hdiutil verify`                       | passed                                                            |
| MVP acceptance         | `pnpm verify:mvp`                      | passed; external models disabled and direct actions absent        |
| Plaintext scan         | `pnpm scan:plaintext`                  | passed; marker absent from all 7 generated Vault files            |
| JS dependency audit    | `pnpm audit --audit-level high`        | passed; no known vulnerabilities                                  |
| Rust dependency audit  | `cargo audit`                          | no known vulnerabilities; 17 allowed transitive warnings recorded |
| Browser smoke          | Chrome DevTools automation             | passed                                                            |
| Browser accessibility  | Lighthouse snapshot                    | 100                                                               |
| License                | `LICENSE` and package metadata         | passed; Apache-2.0 selected by owner                              |
| CI                     | GitHub Actions                         | pending publication                                               |

## Recorded release limitations

- The Tauri lockfile includes Linux-only GTK3 transitive dependencies that RustSec marks unmaintained; `glib 0.18.5` also has an unsound advisory. They are not linked into the tested macOS arm64 artifact. The upgrade is tracked in the debt register rather than hidden.
- The app bundle has only an ad-hoc linker signature. Gatekeeper distribution requires an owner-controlled Apple Developer identity, signing, notarization, and a separate release approval.
- Source publication is authorized under Apache-2.0. Signed binary publication remains a separate release gate.

## v1.0 external blockers

Not claimed: two physical devices, geographic Arcane restore, external security review, two-OS practical use, signed/notarized public binary, and independent portable reader.
