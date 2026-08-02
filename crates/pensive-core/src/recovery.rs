use std::{fs, io::Write, path::Path};

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use walkdir::WalkDir;
use zeroize::Zeroizing;

use crate::{
    Vault,
    crypto::{derive_passphrase_key, open, random_bytes, seal, wrap_root_key},
    models::{KeyEnvelope, VaultManifest},
};

const RECOVERY_AAD: &[u8] = b"pensive/recovery-kit/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupReport {
    pub backup_path: String,
    pub file_count: u64,
    pub total_bytes: u64,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryTestReport {
    pub restored_vault_id: String,
    pub source_count: u64,
    pub fragment_count: u64,
    pub audit_valid: bool,
    pub clean_restore: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecoveryFile {
    protocol: String,
    kdf: String,
    salt: String,
    ciphertext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecoveryPayload {
    protocol: String,
    vault_id: String,
    vault_format_version: String,
    vault_root_key: String,
    exported_at: String,
    instructions_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupManifest {
    protocol: String,
    vault_id: String,
    created_at: String,
    files: Vec<BackupFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupFile {
    path: String,
    size: u64,
    blake3: String,
}

impl Vault {
    pub fn export_recovery_kit(
        &mut self,
        output: impl AsRef<Path>,
        recovery_passphrase: &str,
    ) -> Result<()> {
        self.assert_writable()?;
        if recovery_passphrase.chars().count() < 12 {
            bail!("use a recovery passphrase with at least 12 characters")
        }
        let payload = RecoveryPayload {
            protocol: "pensive-recovery-payload/1".into(),
            vault_id: self.vault_id().into(),
            vault_format_version: "0.1.0".into(),
            vault_root_key: B64.encode(self.root_key()),
            exported_at: now(),
            instructions_version: "1.0.0".into(),
        };
        let salt = random_bytes::<16>()?;
        let key = derive_passphrase_key(recovery_passphrase, &salt)?;
        let encrypted = seal(&key, RECOVERY_AAD, &serde_json::to_vec(&payload)?)?;
        let recovery_file = RecoveryFile {
            protocol: "pensive-recovery/1".into(),
            kdf: "argon2id:m=65536,t=3,p=1".into(),
            salt: B64.encode(salt),
            ciphertext: B64.encode(encrypted),
        };
        write_new_file(output.as_ref(), &serde_json::to_vec_pretty(&recovery_file)?)?;
        let mut status = self.recovery_status()?;
        status.recovery_exported = true;
        status.last_exported_at = Some(now());
        self.set_recovery_status(&status)?;
        self.record_event(
            "RECOVERY_EXPORTED",
            None,
            None,
            serde_json::json!({ "format": "pensive-recovery/1" }),
        )?;
        self.audit("RECOVERY_EXPORTED", None, "SUCCEEDED", None)?;
        Ok(())
    }

    pub fn create_backup(&mut self, output: impl AsRef<Path>) -> Result<BackupReport> {
        self.assert_writable()?;
        self.connection()
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        let output = output.as_ref();
        if output.exists() {
            bail!("backup destination already exists")
        }
        fs::create_dir_all(output)?;
        let mut files = Vec::new();
        for entry in WalkDir::new(self.path()).follow_links(false) {
            let entry = entry?;
            let relative = entry.path().strip_prefix(self.path())?;
            if relative.as_os_str().is_empty()
                || relative.starts_with("diagnostics")
                || relative == Path::new("catalog.sqlite-wal")
                || relative == Path::new("catalog.sqlite-shm")
            {
                continue;
            }
            let target = output.join(relative);
            if entry.file_type().is_symlink() {
                bail!("vault backup refuses symbolic links")
            }
            if entry.file_type().is_dir() {
                fs::create_dir_all(&target)?;
                continue;
            }
            let parent = target.parent().context("backup target has no parent")?;
            fs::create_dir_all(parent)?;
            fs::copy(entry.path(), &target)?;
            let bytes = fs::read(&target)?;
            files.push(BackupFile {
                path: portable_relative(relative)?,
                size: bytes.len() as u64,
                blake3: blake3::hash(&bytes).to_hex().to_string(),
            });
        }
        files.sort_by(|left, right| left.path.cmp(&right.path));
        let manifest = BackupManifest {
            protocol: "pensive-local-backup/1".into(),
            vault_id: self.vault_id().into(),
            created_at: now(),
            files,
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        let manifest_digest = blake3::hash(&manifest_bytes).to_hex().to_string();
        write_new_file(&output.join("backup-manifest.json"), &manifest_bytes)?;
        self.record_event(
            "BACKUP_CREATED",
            None,
            None,
            serde_json::json!({ "manifest_digest": manifest_digest }),
        )?;
        self.audit("BACKUP_CREATED", None, "SUCCEEDED", None)?;
        Ok(BackupReport {
            backup_path: output.display().to_string(),
            file_count: manifest.files.len() as u64,
            total_bytes: manifest.files.iter().map(|file| file.size).sum(),
            manifest_digest,
        })
    }

    pub fn test_recovery(
        &mut self,
        backup: impl AsRef<Path>,
        recovery_kit: impl AsRef<Path>,
        recovery_passphrase: &str,
        test_unlock_passphrase: &str,
    ) -> Result<RecoveryTestReport> {
        self.assert_writable()?;
        let temporary = tempfile::tempdir()?;
        let restored = temporary.path().join("restored-vault");
        Self::restore_from_backup(
            backup,
            recovery_kit,
            recovery_passphrase,
            &restored,
            test_unlock_passphrase,
        )?;
        let restored_vault = Self::open(&restored, test_unlock_passphrase)?;
        let status = restored_vault.status()?;
        let audit = restored_vault.verify_audit()?;
        if restored_vault.vault_id() != self.vault_id() {
            bail!("clean restore produced the wrong vault identity")
        }
        let report = RecoveryTestReport {
            restored_vault_id: restored_vault.vault_id().into(),
            source_count: status.source_count,
            fragment_count: status.fragment_count,
            audit_valid: audit.valid,
            clean_restore: audit.valid,
        };
        drop(restored_vault);
        let mut recovery_status = self.recovery_status()?;
        recovery_status.last_tested_at = Some(now());
        self.set_recovery_status(&recovery_status)?;
        self.record_event(
            "RECOVERY_TESTED",
            None,
            None,
            serde_json::json!({ "clean_restore": report.clean_restore }),
        )?;
        self.audit("RECOVERY_TESTED", None, "SUCCEEDED", None)?;
        Ok(report)
    }

    pub fn restore_from_backup(
        backup: impl AsRef<Path>,
        recovery_kit: impl AsRef<Path>,
        recovery_passphrase: &str,
        destination: impl AsRef<Path>,
        new_unlock_passphrase: &str,
    ) -> Result<()> {
        if new_unlock_passphrase.chars().count() < 12 {
            bail!("new unlock passphrase must contain at least 12 characters")
        }
        let backup = backup.as_ref();
        let destination = destination.as_ref();
        let manifest = verify_backup(backup)?;
        if destination.exists() && fs::read_dir(destination)?.next().is_some() {
            bail!("refusing to restore into a non-empty destination")
        }
        let recovery_file: RecoveryFile =
            serde_json::from_slice(&fs::read(recovery_kit.as_ref())?)?;
        if recovery_file.protocol != "pensive-recovery/1" {
            bail!("unsupported recovery protocol")
        }
        let salt = B64.decode(&recovery_file.salt)?;
        let encrypted = B64.decode(&recovery_file.ciphertext)?;
        let recovery_key = derive_passphrase_key(recovery_passphrase, &salt)?;
        let payload_bytes = open(&recovery_key, RECOVERY_AAD, &encrypted)?;
        let payload: RecoveryPayload = serde_json::from_slice(&payload_bytes)?;
        if payload.vault_id != manifest.vault_id {
            bail!("recovery kit and backup belong to different vaults")
        }
        let root_bytes = Zeroizing::new(B64.decode(&payload.vault_root_key)?);
        let root_key: [u8; 32] = root_bytes
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid recovered vault root key"))?;
        fs::create_dir_all(destination)?;
        for file in &manifest.files {
            let source = backup.join(&file.path);
            let target = destination.join(&file.path);
            let parent = target.parent().context("restore target has no parent")?;
            fs::create_dir_all(parent)?;
            fs::copy(source, target)?;
        }
        let new_envelope: KeyEnvelope = wrap_root_key(new_unlock_passphrase, &root_key)?;
        replace_file(
            &destination.join("key-envelope.pmk"),
            &serde_json::to_vec_pretty(&new_envelope)?,
        )?;
        let vault_manifest: VaultManifest =
            serde_json::from_slice(&fs::read(destination.join("vault.json"))?)?;
        if vault_manifest.vault_id != payload.vault_id {
            bail!("restored vault manifest identity mismatch")
        }
        Ok(())
    }
}

fn verify_backup(path: &Path) -> Result<BackupManifest> {
    let manifest_path = path.join("backup-manifest.json");
    let manifest: BackupManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    if manifest.protocol != "pensive-local-backup/1" {
        bail!("unsupported backup protocol")
    }
    for file in &manifest.files {
        let relative = Path::new(&file.path);
        if relative.is_absolute() || file.path.contains("..") || file.path.contains('\\') {
            bail!("unsafe backup path")
        }
        let bytes = fs::read(path.join(relative))
            .with_context(|| format!("missing backup file {}", file.path))?;
        if bytes.len() as u64 != file.size || blake3::hash(&bytes).to_hex().as_str() != file.blake3
        {
            bail!("backup integrity failure for {}", file.path)
        }
    }
    Ok(manifest)
}

fn portable_relative(path: &Path) -> Result<String> {
    let value = path
        .to_str()
        .context("vault path cannot be represented portably")?
        .replace('\\', "/");
    if value.starts_with('/') || value.split('/').any(|part| part == "..") {
        bail!("unsafe relative backup path")
    }
    Ok(value)
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("output has no parent")?;
    fs::create_dir_all(parent)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("refusing to overwrite {}", path.display()))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn replace_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("output has no parent")?;
    let temporary = parent.join(format!(".restore-{}.tmp", Uuid::new_v4()));
    write_new_file(&temporary, bytes)?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_roundtrip_restores_encrypted_catalog() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let vault_path = temporary.path().join("vault");
        let mut vault = Vault::init(&vault_path, "a long unlock passphrase").expect("init");
        let kit = temporary.path().join("recovery.pmr");
        vault
            .export_recovery_kit(&kit, "a separate recovery passphrase")
            .expect("kit");
        let backup = temporary.path().join("backup");
        vault.create_backup(&backup).expect("backup");
        let report = vault
            .test_recovery(
                &backup,
                &kit,
                "a separate recovery passphrase",
                "a temporary restore passphrase",
            )
            .expect("clean restore");
        assert!(report.clean_restore);
    }
}
