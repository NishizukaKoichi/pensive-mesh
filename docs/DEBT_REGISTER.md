# Technical Debt Register

Updated: 2026-08-02

| Rank | Item                                                                                                  | Impact                                        | Risk   | Paydown gate                 |
| ---- | ----------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------ | ---------------------------- |
| 1    | OS-protected daily key storage is not yet implemented; v0.1 asks for an unlock passphrase per session | usability and shoulder-surfing exposure       | medium | before signed public package |
| 2    | Tauri's Linux-only GTK3 lockfile graph has 17 RustSec warnings, including `glib` unsoundness          | Linux desktop support and audit noise         | medium | before Linux desktop claim   |
| 3    | The macOS bundle is not Developer ID signed or notarized                                              | Gatekeeper blocks frictionless binary install | medium | before public binary release |
| 4    | Backup is a verified directory, not a resumable Arcane transport                                      | continuity across device loss                 | medium | v0.4                         |
| 5    | SQL materialized view rebuild from encrypted journals is documented but not exposed as a command      | repair depth                                  | medium | before schema migration v2   |
| 6    | Desktop UI is Japanese-first without runtime language switch                                          | contributor adoption                          | low    | v0.2                         |
| 7    | Large individual attachments above 64 MiB are reported but not streamed into encrypted objects        | archive completeness for exceptional exports  | low    | v0.2 streaming object writer |

No item above is hidden inside a v0.1 completion claim. The v0.1 core tests protect current behavior before each paydown.

The GTK3 warnings come through Tauri's conditional Linux desktop graph and are not linked into the verified macOS arm64 package. The project does not claim Linux desktop support until that graph is upgraded or replaced and re-audited.
