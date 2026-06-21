use crate::{
    config::{Agent, Packages},
    package::{skill::Skill, Package},
};

pub fn run(name: Option<String>) -> anyhow::Result<()> {
    let packages = Packages::load()?;

    let targets: Vec<String> = match name {
        Some(n) => {
            if !packages.skills.contains_key(&n) {
                anyhow::bail!("'{n}' not found in packages.toml");
            }
            vec![n]
        }
        None => packages.skills.keys().cloned().collect(),
    };

    let skill = Skill::new(Agent::Claude);
    for name in targets {
        skill.install(&name)?;
    }

    Ok(())
}
