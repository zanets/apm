use crate::{
    config::{Agent, Config, Packages},
    git,
    lockfile::{now_unix_secs, LockEntry, Lockfile},
    package::{mcp::Mcp, skill::Skill, Package},
};

pub fn run() -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let agent = Config::load().unwrap_or_default().default_agent;

    let skill = Skill::new(agent);
    let mcp = Mcp::new(Agent::Claude);
    let mut lock = Lockfile::load()?;

    std::fs::create_dir_all(Skill::store_base())?;

    for (name, entry) in &packages.skills {
        let repo = skill.store_path(name);
        if !repo.exists() {
            let url = git::parse_source(&entry.source)?;
            print!("  getting {name} ...");
            git::clone(&url, &repo, &entry.ref_)?;
            let commit = git::current_commit(&repo)?;
            lock.skills.insert(name.clone(), LockEntry {
                commit: commit.clone(),
                updated_at: now_unix_secs(),
            });
            println!(" done ({commit})");
        }
        skill.install(name)?;
    }

    for (name, entry) in &packages.mcps {
        if !mcp.is_enabled(name) {
            mcp.enable(name, &entry.command, &entry.args)?;
        }
    }

    lock.save()?;
    Ok(())
}
