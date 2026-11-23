mod init;
mod project;
mod sync;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum StoreCommands {
    /// Initialize the instruction store as a git repository (idempotent)
    Init,
    /// Pull or push only the current project’s managed Codex instructions
    Project {
        #[command(subcommand)]
        action: project::Action,
    },
    /// Stage, commit, pull (rebase), and push all changes in the instruction store
    Sync {
        /// Commit message (default: "apm: sync <timestamp>")
        #[arg(short = 'm', long)]
        message: Option<String>,
    },
}

pub fn run(cmd: StoreCommands) -> anyhow::Result<()> {
    match cmd {
        StoreCommands::Init => init::run(),
        StoreCommands::Project { action } => project::run(action),
        StoreCommands::Sync { message } => sync::run(message),
    }
}
