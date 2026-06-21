use crate::{
    config::{Agent, Packages},
    lockfile::Lockfile,
    package::{skill::Skill, Package},
};

pub fn run() -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let lock = Lockfile::load()?;

    if packages.skills.is_empty() {
        println!("No skills in packages.toml. Add one with: amp skill add github:user/repo");
        return Ok(());
    }

    println!("{:<20} {:<38} {:<10} {:<10} {}", "NAME", "SOURCE", "REF", "COMMIT", "STATUS");
    println!("{}", "─".repeat(94));

    let skill = Skill::new(Agent::Claude);
    let mut names: Vec<&String> = packages.skills.keys().collect();
    names.sort();

    for name in names {
        let entry = &packages.skills[name];
        let in_store = skill.store_path(name).exists();
        let installed = skill.is_installed(name);

        let commit = lock.skills.get(name).map(|l| l.commit.as_str()).unwrap_or("—");

        let status = match (in_store, installed) {
            (true, true) => "enabled",
            (true, false) => "disabled",
            (false, _) => "not in store",
        };

        println!(
            "{:<20} {:<38} {:<10} {:<10} {}",
            name, entry.source, entry.ref_, commit, status
        );
    }

    Ok(())
}
