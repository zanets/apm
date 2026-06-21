use crate::{
    config::{Agent, Packages},
    package::{tool::Tool, Package},
};

pub fn run(name: Option<String>) -> anyhow::Result<()> {
    let packages = Packages::load()?;

    let targets: Vec<String> = match name {
        Some(n) => {
            if !packages.tools.contains_key(&n) {
                anyhow::bail!("'{n}' not found in packages.toml");
            }
            vec![n]
        }
        None => packages.tools.keys().cloned().collect(),
    };

    let tool = Tool::new(Agent::Claude);
    for name in targets {
        tool.uninstall(&name)?;
    }

    Ok(())
}
