use crate::{
    config::{Agent, Packages},
    lockfile::Lockfile,
    package::{skill::Skill, Package},
};

pub fn run(name: String) -> anyhow::Result<()> {
    let mut packages = Packages::load()?;

    if !packages.skills.contains_key(&name) {
        anyhow::bail!("'{name}' not found in packages.toml");
    }

    let skill = Skill::new(Agent::Claude);

    // 先 uninstall（拔 symlink），再刪 store
    skill.uninstall(&name)?;

    let repo = skill.store_path(&name);
    if repo.exists() {
        std::fs::remove_dir_all(&repo)?;
        println!("  removed store/{name}");
    }

    packages.skills.remove(&name);
    packages.save()?;

    let mut lock = Lockfile::load()?;
    lock.skills.remove(&name);
    lock.save()?;

    println!("  removed '{name}' from packages.toml");

    Ok(())
}
