use crate::{
    config::{Agent, McpEntry, Packages},
    package::mcp::Mcp,
};

pub fn run(name: String, command_and_args: Vec<String>) -> anyhow::Result<()> {
    let (command, args) = command_and_args.split_first().ok_or_else(|| {
        anyhow::anyhow!("command is required: amp mcp add <name> <command> [args...]")
    })?;

    let mut packages = Packages::load()?;
    if packages.mcps.contains_key(&name) {
        anyhow::bail!("'{name}' already exists in packages.toml");
    }

    Mcp::new(Agent::Claude).enable(&name, command, args)?;

    packages.mcps.insert(name.clone(), McpEntry {
        command: command.to_string(),
        args: args.to_vec(),
    });
    packages.save()?;

    println!("  added '{name}' to packages.toml");
    Ok(())
}
