use crate::{
    config::{Agent, Packages},
    lockfile::Lockfile,
    package::{tool::Tool, Package},
};

pub fn run() -> anyhow::Result<()> {
    let packages = Packages::load()?;
    let lock = Lockfile::load()?;

    if packages.tools.is_empty() {
        println!("No tools in packages.toml. Add one with: amp tool add user/repo");
        return Ok(());
    }

    println!("{:<20} {:<38} {:<10} {:<10} {}", "NAME", "SOURCE", "REF", "COMMIT", "STATUS");
    println!("{}", "─".repeat(94));

    let tool = Tool::new(Agent::Claude);
    let mut names: Vec<&String> = packages.tools.keys().collect();
    names.sort();

    for name in names {
        let entry = &packages.tools[name];
        let in_store = tool.store_path(name).exists();
        let installed = tool.is_installed(name);

        let commit = lock.tools.get(name).map(|l| l.commit.as_str()).unwrap_or("—");

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
