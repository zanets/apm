mod list;
mod remove;
mod restore;
mod save;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum ClaudemdCommands {
    /// Save the current project's CLAUDE.md to apm store (keyed by git remote)
    Save {
        /// Scan repo and confirm each CLAUDE.md interactively before saving
        #[arg(short = 'p', long)]
        pick: bool,
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
        /// Key shown in `apm claudemd list`
        key: String,
    },
}

pub fn run(cmd: ClaudemdCommands) -> anyhow::Result<()> {
    match cmd {
        ClaudemdCommands::Save { pick } => save::run(pick),
        ClaudemdCommands::Restore => restore::run(),
        ClaudemdCommands::List { unmanaged } => list::run(unmanaged),
        ClaudemdCommands::Remove { key } => remove::run(key),
    }
}
