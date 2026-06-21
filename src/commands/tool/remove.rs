use crate::{
    config::{Agent, Packages},
    lockfile::Lockfile,
    package::{tool::Tool, Package},
};

pub fn run(name: String) -> anyhow::Result<()> {
    let mut packages = Packages::load()?;

    if !packages.tools.contains_key(&name) {
        anyhow::bail!("'{name}' not found in packages.toml");
    }

    let tool = Tool::new(Agent::Claude);

    tool.uninstall(&name)?;

    let repo = tool.store_path(&name);
    if repo.exists() {
        std::fs::remove_dir_all(&repo)?;
        println!("  removed store/{name}");
    }

    packages.tools.remove(&name);
    packages.save()?;

    let mut lock = Lockfile::load()?;
    lock.tools.remove(&name);
    lock.save()?;

    println!("  removed '{name}' from packages.toml");

    Ok(())
}
