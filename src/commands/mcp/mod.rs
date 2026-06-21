mod add;
mod disable;
mod enable;
mod list;
mod remove;
mod update;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum McpCommands {
    /// Add an MCP server and download it into local store (~/.amp/store/mcps/)
    Add {
        /// GitHub source: user/repo or github:user/repo
        source: String,
        /// Branch or tag to track
        #[arg(long, default_value = "main")]
        ref_: String,
        /// Override the server name (default: repo name)
        #[arg(long)]
        name: Option<String>,
        /// Command to run the server (default: store path)
        #[arg(long)]
        command: Option<String>,
        /// Arguments to pass to the command
        #[arg(long, num_args = 0..)]
        args: Vec<String>,
    },
    /// Register MCP server in agent's settings.json
    Enable {
        /// Enable only this server (omit to enable all)
        name: Option<String>,
        /// Target agent (default: from ~/.amp/config.toml)
        #[arg(long, value_enum)]
        agent: Option<crate::config::Agent>,
    },
    /// Remove MCP server from agent's settings.json
    Disable {
        /// Disable only this server (omit to disable all)
        name: Option<String>,
        /// Target agent (default: from ~/.amp/config.toml)
        #[arg(long, value_enum)]
        agent: Option<crate::config::Agent>,
    },
    /// Remove from store and packages.toml (disables from all agents first)
    Remove {
        /// Server name to remove
        name: String,
    },
    /// Update MCP servers to latest commit
    Update {
        /// Update only this server (omit to update all)
        name: Option<String>,
    },
    /// List MCP servers and their status per agent
    List,
}

pub fn run(cmd: McpCommands) -> anyhow::Result<()> {
    match cmd {
        McpCommands::Add { source, ref_, name, command, args } => {
            add::run(source, ref_, name, command, args)
        }
        McpCommands::Enable { name, agent } => enable::run(name, agent),
        McpCommands::Disable { name, agent } => disable::run(name, agent),
        McpCommands::Remove { name } => remove::run(name),
        McpCommands::Update { name } => update::run(name),
        McpCommands::List => list::run(),
    }
}
