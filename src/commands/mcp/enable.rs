use crate::{
    config::{Agent, Config, Packages},
    package::mcp::Mcp,
};

pub fn run(name: Option<String>, agent: Option<Agent>) -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let agent = agent.unwrap_or_else(|| Config::load().unwrap_or_default().default_agent);

    let targets: Vec<String> = match name {
        Some(n) => {
            if !packages.mcps.contains_key(&n) {
                anyhow::bail!("'{n}' not found in packages.toml");
            }
            vec![n]
        }
        None => packages.mcps.keys().cloned().collect(),
    };

    let mcp = Mcp::new(agent);
    for name in targets {
        let entry = &packages.mcps[&name];
        let store_path = mcp.store_path(&name);
        let command = entry.command.as_deref()
            .unwrap_or_else(|| store_path.to_str().unwrap_or(""));
        mcp.enable(&name, command, &entry.args)?;
    }

    Ok(())
}
