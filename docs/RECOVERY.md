# Recovery Runbook

Updated: 2026-08-02

## When to use

- before trusting a new Vault with irreplaceable Source data
- after creating or rotating a Recovery Kit
- after an integrity warning or device loss
- during the monthly restore sample

Do not use recovery to overwrite a non-empty Vault path.

## Prepare

1. Export `pensive-recovery.pmr` with a dedicated passphrase.
2. Create a Backup directory on different storage.
3. Keep the Recovery Kit, Backup, and passphrase in independently protected places.
4. Run `recovery test` before calling the setup safe.

## Clean test

```bash
cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  recovery test \
  --backup /external/pensive-backup-YYYYMMDD \
  --kit /external/pensive-recovery.pmr
```

The test creates a fresh temporary destination, decrypts the root key, rewrites only the local unlock envelope, opens SQLCipher, checks the Vault identity, counts Source/Fragment records, and verifies the content-free audit hash chain. The temporary directory is removed after the test.

## Restore after loss

```bash
cargo run -p pensive -- recovery restore \
  --backup /external/pensive-backup-YYYYMMDD \
  --kit /external/pensive-recovery.pmr \
  --destination /new/location/PensiveVault
```

Choose a new unlock passphrase. The Recovery Kit passphrase is not reused automatically. Open the restored Vault, inspect several Source objects, query known evidence, and run audit verification.

## Failure modes

- Wrong recovery passphrase or modified kit: authenticated decryption fails.
- Missing/modified Backup file: BLAKE3 manifest verification fails.
- Kit and Backup from different Vaults: Vault ID check fails.
- Non-empty destination: restore stops without overwriting.
- Lost kit + passphrase + usable Backup: neither the operator nor project maintainers can recover the Vault.

## Rollback

Restore never modifies the original Vault or Backup. If a new destination fails verification, preserve it for content-free diagnosis and retry only after identifying the exact integrity or credential failure.
