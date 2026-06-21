use crate::{
    config::{Agent, Packages},
    lockfile::Lockfile,
    package::mcp::Mcp,
};

pub fn run() -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let lock = Lockfile::load()?;

    if packages.mcps.is_empty() {
        println!("No MCP servers in packages.toml. Add one with: amp mcp add github:user/repo");
        return Ok(());
    }

    let agents = Agent::all();
    let agent_col_width = 10usize;
    let agent_header: String = agents
        .iter()
        .map(|a| format!("{:<width$}", a.as_str().to_uppercase(), width = agent_col_width))
        .collect::<Vec<_>>()
        .join(" ");

    println!("{:<20} {:<38} {:<10} {:<10} {}", "NAME", "SOURCE", "REF", "COMMIT", agent_header);
    println!("{}", "─".repeat(82 + agents.len() * (agent_col_width + 1)));

    let mut names: Vec<&String> = packages.mcps.keys().collect();
    names.sort();

    for name in names {
        let entry = &packages.mcps[name];
        let store_ref = Mcp::new(agents[0]);
        let in_store = store_ref.store_path(name).exists();
        let commit = lock.mcps.get(name).map(|l| l.commit.as_str()).unwrap_or("—");

        let agent_cols: String = agents
            .iter()
            .map(|&a| {
                let status = if !in_store {
                    "not in store"
                } else if Mcp::new(a).is_enabled(name) {
                    "enabled"
                } else {
                    "disabled"
                };
                format!("{:<width$}", status, width = agent_col_width)
            })
            .collect::<Vec<_>>()
            .join(" ");

        println!(
            "{:<20} {:<38} {:<10} {:<10} {}",
            name, entry.source, entry.ref_, commit, agent_cols
        );
    }

    Ok(())
}
