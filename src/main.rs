mod commands;
mod config;
mod git;
mod lockfile;
mod package;

use clap::{Parser, Subcommand};
use commands::skill::SkillCommands;
use commands::tool::ToolCommands;

#[derive(Parser)]
#[command(name = "amp", about = "Agent package manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage skills
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
    /// Manage tools
    Tool {
        #[command(subcommand)]
        command: ToolCommands,
    },
    // Mcp { #[command(subcommand)] command: McpCommands } // 未來加
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Skill { command } => commands::skill::run(command),
        Commands::Tool { command } => commands::tool::run(command),
    }
}
