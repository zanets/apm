mod commands;
mod config;
mod git;
mod lockfile;
mod package;

use clap::{Parser, Subcommand};
use commands::claudemd::ClaudemdCommands;
use commands::mcp::McpCommands;

#[derive(Parser)]
#[command(name = "apm", about = "Agent package manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a package and download it into local store (~/.apm/store/)
    Add {
        /// GitHub source: user/repo or github:user/repo
        source: String,
        /// Branch or tag to track
        #[arg(long, default_value = "main")]
        ref_: String,
        /// Override the package name (default: repo name)
        #[arg(long)]
        name: Option<String>,
    },
    /// Link packages into agent directories (make active)
    Enable {
        /// Enable only this package (omit to enable all)
        name: Option<String>,
        /// Target agent (default: from ~/.apm/config.toml)
        #[arg(long, value_enum)]
        agent: Option<config::Agent>,
    },
    /// Remove symlink from agent, keep store intact (make inactive)
    Disable {
        /// Disable only this package (omit to disable all)
        name: Option<String>,
        /// Target agent (default: from ~/.apm/config.toml)
        #[arg(long, value_enum)]
        agent: Option<config::Agent>,
    },
    /// Remove from store and packages.toml (disables first)
    Remove {
        /// Package name to remove
        name: String,
    },
    /// Update packages to latest commit (symlinks update automatically)
    Update {
        /// Update only this package (omit to update all)
        name: Option<String>,
    },
    /// List packages and their status
    List,
    /// Manage MCP servers
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    /// Manage project CLAUDE.md files (local-only, keyed by git remote)
    #[command(name = "md")]
    Claudemd {
        #[command(subcommand)]
        command: ClaudemdCommands,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Add { source, ref_, name } => commands::skill::add::run(source, ref_, name),
        Commands::Enable { name, agent } => commands::skill::enable::run(name, agent),
        Commands::Disable { name, agent } => commands::skill::disable::run(name, agent),
        Commands::Remove { name } => commands::skill::remove::run(name),
        Commands::Update { name } => commands::skill::update::run(name),
        Commands::List => commands::skill::list::run(),
        Commands::Mcp { command } => commands::mcp::run(command),
        Commands::Claudemd { command } => commands::claudemd::run(command),
    }
}
