use crate::{
    config::{Agent, Config, Packages},
    package::{skill::Skill, Package},
};

pub fn run(name: Option<String>, agent: Option<Agent>) -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let agent = agent.unwrap_or_else(|| Config::load().unwrap_or_default().default_agent);

    let targets: Vec<String> = match name {
        Some(n) => {
            if !packages.skills.contains_key(&n) {
                anyhow::bail!("'{n}' not found in packages.toml");
            }
            vec![n]
        }
        None => packages.skills.keys().cloned().collect(),
    };

    let skill = Skill::new(agent);
    for name in targets {
        skill.install(&name)?;
    }

    Ok(())
}
