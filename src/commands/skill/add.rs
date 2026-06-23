use crate::{
    config::{Agent, Packages, SkillEntry},
    git,
    lockfile::{LockEntry, Lockfile},
    package::{skill::Skill, Package},
};
use chrono::Utc;

pub fn run(source: String, ref_: String, name_override: Option<String>) -> anyhow::Result<()> {
    let (source_canonical, derived_name) = git::resolve_source(&source)?;
    let name = name_override.unwrap_or(derived_name);

    let mut packages = Packages::load()?;

    if let Some(existing) = packages.skills.get(&name) {
        if existing.source != source_canonical {
            anyhow::bail!(
                "'{name}' already exists pointing to {}\n  \
                 use --name <alias> to install under a different name",
                existing.source
            );
        }
        anyhow::bail!("'{name}' already exists in packages.toml");
    }

    let skill = Skill::new(Agent::Claude);
    std::fs::create_dir_all(Skill::store_base())?;

    let repo = skill.store_path(&name);
    let url = git::parse_source(&source_canonical)?;
    print!("  getting {name} ...");
    git::clone(&url, &repo, &ref_)?;

    let commit = git::current_commit(&repo)?;

    packages.skills.insert(name.clone(), SkillEntry { source: source_canonical, ref_: ref_.clone() });
    packages.save()?;

    let mut lock = Lockfile::load()?;
    lock.skills.insert(name.clone(), LockEntry { commit: commit.clone(), updated_at: Utc::now().to_rfc3339() });
    lock.save()?;

    println!(" done ({commit})");
    println!("Run `apm enable` to activate it");
    Ok(())
}
