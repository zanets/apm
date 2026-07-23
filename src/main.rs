mod commands;
mod config;
mod git;

use clap::{Parser, Subcommand};
use commands::store::StoreCommands;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "apm", about = "Manage project CLAUDE.md files (local-only, keyed by git remote)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new CLAUDE.md in the current directory and store it immediately
    New,
    /// Save the current project's CLAUDE.md to apm store (keyed by git remote)
    Save {
        /// Scan repo and confirm each CLAUDE.md interactively before saving
        #[arg(short = 'p', long)]
        pick: bool,
        /// Save a standalone CLAUDE.md by absolute path (key = absolute path, no git needed)
        #[arg(short = 'f', long, value_name = "PATH")]
        file: Option<PathBuf>,
    },
    /// Restore CLAUDE.md for the current project from apm store
    Restore,
    /// List all saved project CLAUDE.md files
    List {
        /// List unmanaged CLAUDE.md files in the current repo instead
        #[arg(short = 'u', long)]
        unmanaged: bool,
    },
    /// Remove a saved CLAUDE.md from the store
    Remove {
        /// Key shown in `apm list`
        key: String,
    },
    /// Print apm storage paths
    Env,
    /// Manage the claudemds store as a git repository
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New => commands::claudemd::new::run(),
        Commands::Save { pick, file } => commands::claudemd::save::run(pick, file),
        Commands::Restore => commands::claudemd::restore::run(),
        Commands::List { unmanaged } => commands::claudemd::list::run(unmanaged),
        Commands::Remove { key } => commands::claudemd::remove::run(key),
        Commands::Env => commands::env::run(),
        Commands::Store { command } => commands::store::run(command),
    }
}
