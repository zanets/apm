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
    let (source_canonical, derived_name) = parse_source(&source)?;
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

fn parse_source(s: &str) -> anyhow::Result<(String, String)> {
    let s = s.trim_end_matches(".git");

    if let Some(path) = s.strip_prefix("github:") {
        return extract_github(path, s);
    }

    if s.contains('/') {
        return extract_github(s, &format!("github:{s}"));
    }

    anyhow::bail!("ambiguous source '{s}' — use user/{s} to specify the GitHub owner")
}

fn extract_github(path: &str, canonical: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("invalid source '{canonical}' — expected user/repo");
    }
    let name = parts[1].trim_end_matches(".git").to_string();
    Ok((format!("github:{}", path.trim_end_matches(".git")), name))
}
