use super::Package;
use crate::config::{amp_dir, Agent};
use std::path::{Path, PathBuf};

pub struct Tool {
    pub agent: Agent,
}

impl Tool {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    pub fn store_base() -> PathBuf {
        amp_dir().join("store").join("tools")
    }

    fn link_dir(&self) -> PathBuf {
        let home = dirs::home_dir().expect("no home dir");
        match self.agent {
            Agent::Claude => home.join(".claude").join("tools"),
            Agent::Cursor => home.join(".cursor").join("tools"),
            Agent::Windsurf => home.join(".windsurf").join("tools"),
        }
    }
}

impl Package for Tool {
    fn store_path(&self, name: &str) -> PathBuf {
        Self::store_base().join(name)
    }

    fn install(&self, name: &str) -> anyhow::Result<()> {
        let repo = self.store_path(name);
        if !repo.exists() {
            anyhow::bail!("'{name}' not in store — run `amp tool add` first");
        }
        symlink(name, &repo, &self.link_dir())
    }

    fn is_installed(&self, name: &str) -> bool {
        self.link_dir().join(name).is_symlink()
    }

    fn uninstall(&self, name: &str) -> anyhow::Result<()> {
        let link_path = self.link_dir().join(name);
        if link_path.is_symlink() {
            std::fs::remove_file(&link_path)?;
            println!("  unlinked {name}");
        } else {
            println!("  {name}: not linked, nothing to uninstall");
        }
        Ok(())
    }
}

fn symlink(name: &str, repo: &Path, link_dir: &Path) -> anyhow::Result<()> {
    let link_path = link_dir.join(name);
    std::fs::create_dir_all(link_dir)?;

    if link_path.is_symlink() {
        if std::fs::read_link(&link_path)? == repo {
            println!("  {name}: already linked");
            return Ok(());
        }
        std::fs::remove_file(&link_path)?;
    } else if link_path.exists() {
        anyhow::bail!(
            "{} exists and is not a symlink — remove it manually first",
            link_path.display()
        );
    }

    std::os::unix::fs::symlink(repo, &link_path)?;
    println!("  linked {name} → {}", link_path.display());
    Ok(())
}
