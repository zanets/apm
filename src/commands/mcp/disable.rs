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
        mcp.disable(&name)?;
    }

    Ok(())
}
