# Privacy

Pensive Mesh v0.1 is local-first and offline by construction.

- No analytics, telemetry, crash upload, or account identifier.
- No external model adapter and no automatic provider fallback.
- No URL fetch from imported content.
- No API key, OAuth token, passkey, recovery passphrase, or root key in logs.
- No plaintext Source name/path in diagnostics.
- Context Pack preview precedes encrypted export.
- Secret candidates and third-party Memory are excluded by default.

The development browser preview uses only hard-coded synthetic data. It never scans the filesystem or localStorage. Actual Vault access occurs only inside the Tauri Rust process after explicit path selection and unlock.
