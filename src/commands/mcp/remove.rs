use crate::{
    config::Packages,
    package::mcp::Mcp,
};

pub fn run(name: String) -> anyhow::Result<()> {
    let mut packages = Packages::load()?;

    if !packages.mcps.contains_key(&name) {
        anyhow::bail!("'{name}' not found in packages.toml");
    }

    for &agent in crate::config::Agent::all() {
        Mcp::new(agent).disable(&name)?;
    }

    packages.mcps.remove(&name);
    packages.save()?;

    println!("  removed '{name}' from packages.toml");
    Ok(())
}
