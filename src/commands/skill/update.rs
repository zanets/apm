use crate::{
    config::{Agent, Packages},
    git,
    lockfile::{LockEntry, Lockfile},
    package::{skill::Skill, Package},
};
use chrono::Utc;

pub fn run(name: Option<String>) -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let mut lock = Lockfile::load()?;

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
        let repo = skill.store_path(&name);
        if !repo.exists() {
            println!("  {name}: not in store — run `apm add` first");
            continue;
        }

        let before = git::current_commit(&repo)?;
        print!("  updating {name} ...");
        git::pull(&repo)?;
        let after = git::current_commit(&repo)?;

        if before == after {
            println!(" already up to date ({after})");
        } else {
            println!(" ✓ {before} → {after}");
        }

        lock.skills.insert(
            name.clone(),
            LockEntry { commit: after, updated_at: Utc::now().to_rfc3339() },
        );
    }

    lock.save()?;
    Ok(())
}
