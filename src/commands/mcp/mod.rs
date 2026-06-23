mod add;
mod list;
mod remove;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum McpCommands {
    /// Add an MCP server and register it with Claude
    Add {
        /// Server name
        name: String,
        /// Command and arguments (e.g. uvx blender-mcp)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command_and_args: Vec<String>,
    },
    /// Remove an MCP server from Claude and packages.toml
    Remove {
        /// Server name to remove
        name: String,
    },
    /// List registered MCP servers
    List,
}

pub fn run(cmd: McpCommands) -> anyhow::Result<()> {
    match cmd {
        McpCommands::Add { name, command_and_args } => add::run(name, command_and_args),
        McpCommands::Remove { name } => remove::run(name),
        McpCommands::List => list::run(),
    }
}
