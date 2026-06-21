use crate::{
    config::{Agent, Config, Packages},
    package::{tool::Tool, Package},
};

pub fn run(name: Option<String>, agent: Option<Agent>) -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let agent = agent.unwrap_or_else(|| Config::load().unwrap_or_default().default_agent);

    let targets: Vec<String> = match name {
        Some(n) => {
            if !packages.tools.contains_key(&n) {
                anyhow::bail!("'{n}' not found in packages.toml");
            }
            vec![n]
        }
        None => packages.tools.keys().cloned().collect(),
    };

    let tool = Tool::new(agent);
    for name in targets {
        tool.install(&name)?;
    }

    Ok(())
}
