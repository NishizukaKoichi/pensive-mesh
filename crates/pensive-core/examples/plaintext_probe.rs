use std::fs;

use anyhow::{Result, bail};
use pensive_core::{ImportOptions, Vault};
use walkdir::WalkDir;

const MARKER: &str = "PENSIVE_PLAINTEXT_PROBE_7df4b2c91a";

fn main() {
    if let Err(error) = run() {
        eprintln!("plaintext probe failed: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let vault_path = temporary.path().join("vault");
    let source_path = temporary.path().join("probe.md");
    fs::write(&source_path, MARKER)?;
    let mut vault = Vault::init(&vault_path, "plaintext probe unlock passphrase")?;
    vault.import_path(
        &source_path,
        ImportOptions {
            sensitivity: "SECRET".into(),
            ..ImportOptions::default()
        },
    )?;
    drop(vault);

    let mut scanned = 0_u64;
    for entry in WalkDir::new(&vault_path).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        scanned += 1;
        let bytes = fs::read(entry.path())?;
        if bytes
            .windows(MARKER.len())
            .any(|window| window == MARKER.as_bytes())
        {
            bail!(
                "plaintext marker found at rest in {}",
                entry.path().display()
            )
        }
    }
    println!(
        "{{\"plaintext_marker_found\":false,\"files_scanned\":{scanned},\"sqlcipher_required\":true}}"
    );
    Ok(())
}
