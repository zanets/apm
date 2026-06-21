mod add;
mod disable;
mod enable;
mod list;
mod remove;
mod update;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum SkillCommands {
    /// Add a skill and download it into local store (~/.amp/store/skills/)
    Add {
        /// GitHub source: user/repo or github:user/repo
        source: String,
        /// Branch or tag to track
        #[arg(long, default_value = "main")]
        ref_: String,
        /// Override the skill name (default: repo name)
        #[arg(long)]
        name: Option<String>,
    },
    /// Link skills into agent directories (make active)
    Enable {
        /// Enable only this skill (omit to enable all)
        name: Option<String>,
    },
    /// Remove symlink from agent, keep store intact (make inactive)
    Disable {
        /// Disable only this skill (omit to disable all)
        name: Option<String>,
    },
    /// Remove from store and packages.toml (disables first)
    Remove {
        /// Skill name to remove
        name: String,
    },
    /// Update skills to latest commit (symlinks update automatically)
    Update {
        /// Update only this skill (omit to update all)
        name: Option<String>,
    },
    /// List skills and their status
    List,
}

pub fn run(cmd: SkillCommands) -> anyhow::Result<()> {
    match cmd {
        SkillCommands::Add { source, ref_, name } => add::run(source, ref_, name),
        SkillCommands::Enable { name } => enable::run(name),
        SkillCommands::Disable { name } => disable::run(name),
        SkillCommands::Remove { name } => remove::run(name),
        SkillCommands::Update { name } => update::run(name),
        SkillCommands::List => list::run(),
    }
}
