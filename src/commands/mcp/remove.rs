use crate::{
    config::{Agent, Packages},
    lockfile::Lockfile,
    package::mcp::Mcp,
};

pub fn run(name: String) -> anyhow::Result<()> {
    let mut packages = Packages::load()?;

    if !packages.mcps.contains_key(&name) {
        anyhow::bail!("'{name}' not found in packages.toml");
    }

    // disable from all agents first
    for &agent in crate::config::Agent::all() {
        Mcp::new(agent).disable(&name)?;
    }

    let mcp = Mcp::new(Agent::Claude);
    let repo = mcp.store_path(&name);
    if repo.exists() {
        std::fs::remove_dir_all(&repo)?;
        println!("  removed store/{name}");
    }

    packages.mcps.remove(&name);
    packages.save()?;

    let mut lock = Lockfile::load()?;
    lock.mcps.remove(&name);
    lock.save()?;

    println!("  removed '{name}' from packages.toml");
    Ok(())
}
