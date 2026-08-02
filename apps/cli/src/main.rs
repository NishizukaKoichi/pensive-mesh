use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use pensive_core::{ContextPolicy, ImportOptions, Vault};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "pensive",
    version,
    about = "Owner-controlled, local-first personal context"
)]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    vault: Option<PathBuf>,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {
        path: PathBuf,
    },
    Status,
    Lock,
    Import(ImportCommand),
    Source(SourceCommand),
    Memory(MemoryCommand),
    Query {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Context(ContextCommand),
    Backup(BackupCommand),
    Recovery(RecoveryCommand),
    Audit(AuditCommand),
    Freeze {
        reason: String,
    },
    Doctor,
}

#[derive(Debug, Args)]
struct ImportCommand {
    #[command(subcommand)]
    command: ImportSubcommand,
}

#[derive(Debug, Subcommand)]
enum ImportSubcommand {
    Chatgpt {
        path: PathBuf,
        #[arg(long, default_value = "SENSITIVE")]
        sensitivity: String,
    },
    File {
        path: PathBuf,
        #[arg(long, default_value = "PERSONAL")]
        sensitivity: String,
    },
}

#[derive(Debug, Args)]
struct SourceCommand {
    #[command(subcommand)]
    command: SourceSubcommand,
}

#[derive(Debug, Subcommand)]
enum SourceSubcommand {
    List {
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Show {
        source_id: String,
    },
}

#[derive(Debug, Args)]
struct MemoryCommand {
    #[command(subcommand)]
    command: MemorySubcommand,
}

#[derive(Debug, Subcommand)]
enum MemorySubcommand {
    Inbox,
    Propose {
        statement: String,
        #[arg(long)]
        evidence: String,
        #[arg(long, default_value = "FACT")]
        memory_type: String,
        #[arg(long, default_value = "PERSONAL")]
        sensitivity: String,
        #[arg(long)]
        third_party: bool,
    },
    Accept {
        memory_id: String,
    },
    Reject {
        memory_id: String,
    },
    Revoke {
        memory_id: String,
    },
    Correct {
        memory_id: String,
        statement: String,
    },
}

#[derive(Debug, Args)]
struct ContextCommand {
    #[command(subcommand)]
    command: ContextSubcommand,
}

#[derive(Debug, Subcommand)]
enum ContextSubcommand {
    Build {
        query: String,
        #[arg(long)]
        purpose: String,
        #[arg(long, default_value_t = 8_000)]
        max_tokens: u32,
    },
    Export {
        pack_id: String,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Args)]
struct BackupCommand {
    #[command(subcommand)]
    command: BackupSubcommand,
}

#[derive(Debug, Subcommand)]
enum BackupSubcommand {
    Run { output: PathBuf },
}

#[derive(Debug, Args)]
struct RecoveryCommand {
    #[command(subcommand)]
    command: RecoverySubcommand,
}

#[derive(Debug, Subcommand)]
enum RecoverySubcommand {
    Export {
        output: PathBuf,
    },
    Test {
        #[arg(long)]
        backup: PathBuf,
        #[arg(long)]
        kit: PathBuf,
    },
    Restore {
        #[arg(long)]
        backup: PathBuf,
        #[arg(long)]
        kit: PathBuf,
        #[arg(long)]
        destination: PathBuf,
    },
}

#[derive(Debug, Args)]
struct AuditCommand {
    #[command(subcommand)]
    command: AuditSubcommand,
}

#[derive(Debug, Subcommand)]
enum AuditSubcommand {
    Verify,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Pensive stopped safely: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { path } => {
            let passphrase = confirmed_passphrase("Create unlock passphrase: ")?;
            let vault = Vault::init(&path, &passphrase)?;
            print_value(cli.json, &vault.status()?)
        }
        Command::Recovery(RecoveryCommand {
            command:
                RecoverySubcommand::Restore {
                    backup,
                    kit,
                    destination,
                },
        }) => {
            let recovery = rpassword::prompt_password("Recovery passphrase: ")?;
            let unlock = confirmed_passphrase("New unlock passphrase: ")?;
            Vault::restore_from_backup(backup, kit, &recovery, destination, &unlock)?;
            print_value(cli.json, &serde_json::json!({ "restored": true }))
        }
        Command::Lock => print_value(
            cli.json,
            &serde_json::json!({ "locked": true, "note": "The CLI never persists an unlocked session." }),
        ),
        command => {
            let vault_path = cli
                .vault
                .context("--vault PATH is required for this command")?;
            let unlock = rpassword::prompt_password("Unlock passphrase: ")?;
            let mut vault = Vault::open(&vault_path, &unlock)?;
            execute_open_command(command, &mut vault, cli.json)
        }
    }
}

fn execute_open_command(command: Command, vault: &mut Vault, json: bool) -> Result<()> {
    match command {
        Command::Status | Command::Doctor => print_value(json, &vault.status()?),
        Command::Import(ImportCommand { command }) => {
            let (path, sensitivity) = match command {
                ImportSubcommand::Chatgpt { path, sensitivity }
                | ImportSubcommand::File { path, sensitivity } => (path, sensitivity),
            };
            let options = ImportOptions {
                sensitivity,
                ..ImportOptions::default()
            };
            print_value(json, &vault.import_path(path, options)?)
        }
        Command::Source(SourceCommand { command }) => match command {
            SourceSubcommand::List { limit } => print_value(json, &vault.list_sources(limit)?),
            SourceSubcommand::Show { source_id } => {
                print_value(json, &vault.list_fragments(&source_id)?)
            }
        },
        Command::Memory(MemoryCommand { command }) => match command {
            MemorySubcommand::Inbox => print_value(json, &vault.memory_inbox()?),
            MemorySubcommand::Propose {
                statement,
                evidence,
                memory_type,
                sensitivity,
                third_party,
            } => print_value(
                json,
                &vault.propose_memory(
                    &memory_type,
                    &statement,
                    &evidence,
                    &sensitivity,
                    third_party,
                )?,
            ),
            MemorySubcommand::Accept { memory_id } => {
                print_value(json, &vault.review_memory(&memory_id, "accept", None)?)
            }
            MemorySubcommand::Reject { memory_id } => {
                print_value(json, &vault.review_memory(&memory_id, "reject", None)?)
            }
            MemorySubcommand::Revoke { memory_id } => {
                print_value(json, &vault.review_memory(&memory_id, "revoke", None)?)
            }
            MemorySubcommand::Correct {
                memory_id,
                statement,
            } => print_value(
                json,
                &vault.review_memory(&memory_id, "correct", Some(&statement))?,
            ),
        },
        Command::Query { query, limit } => print_value(json, &vault.query(&query, limit)?),
        Command::Context(ContextCommand { command }) => match command {
            ContextSubcommand::Build {
                query,
                purpose,
                max_tokens,
            } => print_value(
                json,
                &vault.build_context_pack(
                    &purpose,
                    &query,
                    max_tokens,
                    ContextPolicy::default(),
                )?,
            ),
            ContextSubcommand::Export { pack_id, output } => {
                vault.export_context_pack(&pack_id, output)?;
                print_value(
                    json,
                    &serde_json::json!({ "exported": true, "encrypted": true }),
                )
            }
        },
        Command::Backup(BackupCommand {
            command: BackupSubcommand::Run { output },
        }) => print_value(json, &vault.create_backup(output)?),
        Command::Recovery(RecoveryCommand { command }) => match command {
            RecoverySubcommand::Export { output } => {
                let passphrase = confirmed_passphrase("Create recovery passphrase: ")?;
                vault.export_recovery_kit(output, &passphrase)?;
                print_value(
                    json,
                    &serde_json::json!({ "exported": true, "encrypted": true }),
                )
            }
            RecoverySubcommand::Test { backup, kit } => {
                let recovery = rpassword::prompt_password("Recovery passphrase: ")?;
                let test_unlock = confirmed_passphrase("Temporary clean-restore passphrase: ")?;
                print_value(
                    json,
                    &vault.test_recovery(backup, kit, &recovery, &test_unlock)?,
                )
            }
            RecoverySubcommand::Restore { .. } => unreachable!("handled before vault open"),
        },
        Command::Audit(AuditCommand {
            command: AuditSubcommand::Verify,
        }) => print_value(json, &vault.verify_audit()?),
        Command::Freeze { reason } => {
            vault.freeze(&reason)?;
            print_value(
                json,
                &serde_json::json!({ "frozen": true, "reason": reason }),
            )
        }
        Command::Init { .. } | Command::Lock => bail!("command routing error"),
    }
}

fn confirmed_passphrase(prompt: &str) -> Result<String> {
    let first = rpassword::prompt_password(prompt)?;
    let second = rpassword::prompt_password("Confirm passphrase: ")?;
    if first != second {
        bail!("passphrases did not match")
    }
    Ok(first)
}

fn print_value<T: Serialize>(json_output: bool, value: &T) -> Result<()> {
    if json_output {
        println!("{}", serde_json::to_string(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}
