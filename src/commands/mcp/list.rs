use crate::{
    config::{Agent, Packages},
    package::mcp::Mcp,
};

pub fn run() -> anyhow::Result<()> {
    let packages = Packages::load()?;

    if packages.mcps.is_empty() {
        println!("No MCP servers in packages.toml. Add one with: amp mcp add <name> <command> [args...]");
        return Ok(());
    }

    println!("{:<20} {:<35} {}", "NAME", "COMMAND", "CLAUDE");
    println!("{}", "─".repeat(65));

    let mut names: Vec<&String> = packages.mcps.keys().collect();
    names.sort();

    for name in names {
        let entry = &packages.mcps[name];
        let display_cmd = if entry.args.is_empty() {
            entry.command.clone()
        } else {
            format!("{} {}", entry.command, entry.args.join(" "))
        };

        let status = if Mcp::new(Agent::Claude).is_enabled(name) { "enabled" } else { "not registered" };

        println!("{:<20} {:<35} {}", name, display_cmd, status);
    }

    Ok(())
}
