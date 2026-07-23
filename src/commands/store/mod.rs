mod init;
mod sync;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum StoreCommands {
    /// Initialize the claudemds store as a git repository (idempotent)
    Init,
    /// Stage, commit, pull (rebase), and push all changes in the claudemds store
    Sync {
        /// Commit message (default: "apm: sync <timestamp>")
        #[arg(short = 'm', long)]
        message: Option<String>,
    },
}

pub fn run(cmd: StoreCommands) -> anyhow::Result<()> {
    match cmd {
        StoreCommands::Init => init::run(),
        StoreCommands::Sync { message } => sync::run(message),
    }
}
