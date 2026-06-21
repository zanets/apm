use crate::{
    config::{Agent, McpEntry, Packages},
    git,
    lockfile::{LockEntry, Lockfile},
    package::mcp::Mcp,
};
use chrono::Utc;

pub fn run(
    source: String,
    ref_: String,
    name_override: Option<String>,
    command: Option<String>,
    args: Vec<String>,
) -> anyhow::Result<()> {
    let (source_canonical, derived_name) = git::resolve_source(&source)?;
    let name = name_override.unwrap_or(derived_name);

    let mut packages = Packages::load()?;

    if packages.mcps.contains_key(&name) {
        anyhow::bail!("'{name}' already exists in packages.toml");
    }

    let mcp = Mcp::new(Agent::Claude);
    std::fs::create_dir_all(Mcp::store_base())?;

    let repo = mcp.store_path(&name);
    let url = git::parse_source(&source_canonical)?;
    print!("  getting {name} ...");
    git::clone(&url, &repo, &ref_)?;

    let commit = git::current_commit(&repo)?;

    packages.mcps.insert(name.clone(), McpEntry {
        source: source_canonical,
        ref_: ref_.clone(),
        command,
        args,
    });
    packages.save()?;

    let mut lock = Lockfile::load()?;
    lock.mcps.insert(name.clone(), LockEntry {
        commit: commit.clone(),
        updated_at: Utc::now().to_rfc3339(),
    });
    lock.save()?;

    println!(" done ({commit})");
    println!("Run `amp mcp enable` to activate it");
    Ok(())
}
