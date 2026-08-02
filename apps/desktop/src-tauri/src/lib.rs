use std::sync::Mutex;

use pensive_core::{
    BackupReport, ContextPack, ContextPolicy, FragmentSummary, ImportOptions, ImportReport,
    MemorySummary, RecoveryTestReport, SearchHit, SourceSummary, Vault, VaultStatus,
};
use tauri::State;

#[derive(Default)]
struct AppState {
    vault: Mutex<Option<Vault>>,
}

fn with_vault<T>(
    state: &State<'_, AppState>,
    operation: impl FnOnce(&mut Vault) -> Result<T, anyhow::Error>,
) -> Result<T, String> {
    let mut guard = state
        .vault
        .lock()
        .map_err(|_| "Pensive state lock was poisoned".to_owned())?;
    let vault = guard.as_mut().ok_or_else(|| "Vault is locked".to_owned())?;
    operation(vault).map_err(|error| error.to_string())
}

#[tauri::command]
fn init_vault(
    path: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<VaultStatus, String> {
    let vault = Vault::init(path, &passphrase).map_err(|error| error.to_string())?;
    let status = vault.status().map_err(|error| error.to_string())?;
    *state
        .vault
        .lock()
        .map_err(|_| "Pensive state lock was poisoned".to_owned())? = Some(vault);
    Ok(status)
}

#[tauri::command]
fn unlock_vault(
    path: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<VaultStatus, String> {
    let vault = Vault::open(path, &passphrase).map_err(|error| error.to_string())?;
    let status = vault.status().map_err(|error| error.to_string())?;
    *state
        .vault
        .lock()
        .map_err(|_| "Pensive state lock was poisoned".to_owned())? = Some(vault);
    Ok(status)
}

#[tauri::command]
fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    *state
        .vault
        .lock()
        .map_err(|_| "Pensive state lock was poisoned".to_owned())? = None;
    Ok(())
}

#[tauri::command]
fn vault_status(state: State<'_, AppState>) -> Result<Option<VaultStatus>, String> {
    let guard = state
        .vault
        .lock()
        .map_err(|_| "Pensive state lock was poisoned".to_owned())?;
    guard
        .as_ref()
        .map(Vault::status)
        .transpose()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_sources(state: State<'_, AppState>) -> Result<Vec<SourceSummary>, String> {
    with_vault(&state, |vault| vault.list_sources(200))
}

#[tauri::command]
fn list_fragments(
    source_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FragmentSummary>, String> {
    with_vault(&state, |vault| vault.list_fragments(&source_id))
}

#[tauri::command]
fn import_source(
    path: String,
    sensitivity: String,
    state: State<'_, AppState>,
) -> Result<ImportReport, String> {
    with_vault(&state, |vault| {
        vault.import_path(
            path,
            ImportOptions {
                sensitivity,
                ..ImportOptions::default()
            },
        )
    })
}

#[tauri::command]
fn search_vault(query: String, state: State<'_, AppState>) -> Result<Vec<SearchHit>, String> {
    with_vault(&state, |vault| vault.query(&query, 30))
}

#[tauri::command]
fn memory_inbox(state: State<'_, AppState>) -> Result<Vec<MemorySummary>, String> {
    with_vault(&state, |vault| vault.memory_inbox())
}

#[tauri::command]
fn propose_memory(
    statement: String,
    memory_type: String,
    evidence_fragment_id: String,
    sensitivity: String,
    third_party: bool,
    state: State<'_, AppState>,
) -> Result<MemorySummary, String> {
    with_vault(&state, |vault| {
        vault.propose_memory(
            &memory_type,
            &statement,
            &evidence_fragment_id,
            &sensitivity,
            third_party,
        )
    })
}

#[tauri::command]
fn review_memory(
    memory_id: String,
    action: String,
    statement: Option<String>,
    state: State<'_, AppState>,
) -> Result<MemorySummary, String> {
    with_vault(&state, |vault| {
        vault.review_memory(&memory_id, &action, statement.as_deref())
    })
}

#[tauri::command]
fn build_context_pack(
    purpose: String,
    query: String,
    max_tokens: u32,
    state: State<'_, AppState>,
) -> Result<ContextPack, String> {
    with_vault(&state, |vault| {
        vault.build_context_pack(&purpose, &query, max_tokens, ContextPolicy::default())
    })
}

#[tauri::command]
fn export_context_pack(
    pack_id: String,
    output: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    with_vault(&state, |vault| vault.export_context_pack(&pack_id, output))
}

#[tauri::command]
fn export_recovery_kit(
    output: String,
    recovery_passphrase: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    with_vault(&state, |vault| {
        vault.export_recovery_kit(output, &recovery_passphrase)
    })
}

#[tauri::command]
fn create_backup(output: String, state: State<'_, AppState>) -> Result<BackupReport, String> {
    with_vault(&state, |vault| vault.create_backup(output))
}

#[tauri::command]
fn test_recovery(
    backup: String,
    kit: String,
    recovery_passphrase: String,
    test_unlock_passphrase: String,
    state: State<'_, AppState>,
) -> Result<RecoveryTestReport, String> {
    with_vault(&state, |vault| {
        vault.test_recovery(backup, kit, &recovery_passphrase, &test_unlock_passphrase)
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            init_vault,
            unlock_vault,
            lock_vault,
            vault_status,
            list_sources,
            list_fragments,
            import_source,
            search_vault,
            memory_inbox,
            propose_memory,
            review_memory,
            build_context_pack,
            export_context_pack,
            export_recovery_kit,
            create_backup,
            test_recovery,
        ])
        .run(tauri::generate_context!())
        .expect("Pensive desktop runtime failed");
}
