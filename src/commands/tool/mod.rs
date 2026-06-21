mod add;
mod disable;
mod enable;
mod list;
mod remove;
mod update;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum ToolCommands {
    /// Add a tool and download it into local store (~/.amp/store/tools/)
    Add {
        /// GitHub source: user/repo or github:user/repo
        source: String,
        /// Branch or tag to track
        #[arg(long, default_value = "main")]
        ref_: String,
        /// Override the tool name (default: repo name)
        #[arg(long)]
        name: Option<String>,
    },
    /// Link tools into agent directories (make active)
    Enable {
        /// Enable only this tool (omit to enable all)
        name: Option<String>,
        /// Target agent (default: from ~/.amp/config.toml)
        #[arg(long, value_enum)]
        agent: Option<crate::config::Agent>,
    },
    /// Remove symlink from agent, keep store intact (make inactive)
    Disable {
        /// Disable only this tool (omit to disable all)
        name: Option<String>,
        /// Target agent (default: from ~/.amp/config.toml)
        #[arg(long, value_enum)]
        agent: Option<crate::config::Agent>,
    },
    /// Remove from store and packages.toml (disables first)
    Remove {
        /// Tool name to remove
        name: String,
    },
    /// Update tools to latest commit (symlinks update automatically)
    Update {
        /// Update only this tool (omit to update all)
        name: Option<String>,
    },
    /// List tools and their status
    List,
}

pub fn run(cmd: ToolCommands) -> anyhow::Result<()> {
    match cmd {
        ToolCommands::Add { source, ref_, name } => add::run(source, ref_, name),
        ToolCommands::Enable { name, agent } => enable::run(name, agent),
        ToolCommands::Disable { name, agent } => disable::run(name, agent),
        ToolCommands::Remove { name } => remove::run(name),
        ToolCommands::Update { name } => update::run(name),
        ToolCommands::List => list::run(),
    }
}
