mod commands;
mod config;
mod git;

use clap::{Parser, Subcommand};
use commands::{instructions, store::StoreCommands};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "apm", about = "Manage local Codex instruction files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and manage an AGENTS.md in the current directory
    New,
    /// Save untracked Codex instruction files and link them back into the repo
    Save {
        /// Confirm each file interactively
        #[arg(short, long, conflicts_with = "file")]
        pick: bool,
        /// Save a standalone file. Global Codex instructions are portable across machines.
        #[arg(short, long, value_name = "PATH")]
        file: Option<PathBuf>,
    },
    /// Restore saved project and standalone instructions
    Restore,
    /// List saved instructions, including legacy CLAUDE.md
    List {
        /// List untracked, unmanaged Codex instructions in the current repo
        #[arg(short, long)]
        unmanaged: bool,
    },
    /// Remove a saved entry by its key from apm list
    Remove { key: String },
    /// Preview or import saved project CLAUDE.md files as AGENTS.md
    Migrate {
        /// Repository-relative CLAUDE.md paths to import
        #[arg(value_name = "PATH", required_if_eq("apply", "true"))]
        paths: Vec<PathBuf>,
        /// Import the listed paths after reviewing the preview
        #[arg(long)]
        apply: bool,
    },
    /// Print apm storage paths
    Env,
    /// Manage the instruction store as a Git repository
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::New => instructions::new::run(),
        Commands::Save { pick, file } => instructions::save::run(pick, file),
        Commands::Restore => instructions::restore::run(),
        Commands::List { unmanaged } => instructions::list::run(unmanaged),
        Commands::Remove { key } => instructions::remove::run(key),
        Commands::Migrate { paths, apply } => instructions::migrate::run(paths, apply),
        Commands::Env => commands::env::run(),
        Commands::Store { command } => commands::store::run(command),
    }
}
