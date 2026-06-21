use super::Package;
use crate::config::{amp_dir, Agent};
use std::path::{Path, PathBuf};

pub struct Skill {
    pub agent: Agent,
}

impl Skill {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    pub fn store_base() -> PathBuf {
        amp_dir().join("store").join("skills")
    }

    fn link_dir(&self) -> PathBuf {
        let home = dirs::home_dir().expect("no home dir");
        match self.agent {
            Agent::Claude => home.join(".claude").join("skills"),
            Agent::Cursor => home.join(".cursor").join("skills"),
            Agent::Windsurf => home.join(".windsurf").join("skills"),
        }
    }
}

impl Package for Skill {
    fn store_path(&self, name: &str) -> PathBuf {
        Self::store_base().join(name)
    }

    fn install(&self, name: &str) -> anyhow::Result<()> {
        let repo = self.store_path(name);
        if !repo.exists() {
            anyhow::bail!("'{name}' not in store — run `amp skill add` first");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(label: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("amp_skill_test_{label}"));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn symlink_creates_link() {
        let store = tmpdir("creates_store");
        let link_dir = tmpdir("creates_link");

        symlink("myskill", &store, &link_dir).unwrap();

        let link_path = link_dir.join("myskill");
        assert!(link_path.is_symlink());
        assert_eq!(std::fs::read_link(&link_path).unwrap(), store);

        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&link_dir);
    }

    #[test]
    fn symlink_same_target_is_noop() {
        let store = tmpdir("noop_store");
        let link_dir = tmpdir("noop_link");

        symlink("myskill", &store, &link_dir).unwrap();
        symlink("myskill", &store, &link_dir).unwrap(); // second call must not error

        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&link_dir);
    }

    #[test]
    fn symlink_relinks_different_target() {
        let store1 = tmpdir("relink_store1");
        let store2 = tmpdir("relink_store2");
        let link_dir = tmpdir("relink_link");

        symlink("myskill", &store1, &link_dir).unwrap();
        symlink("myskill", &store2, &link_dir).unwrap();

        let link_path = link_dir.join("myskill");
        assert_eq!(std::fs::read_link(&link_path).unwrap(), store2);

        let _ = std::fs::remove_dir_all(&store1);
        let _ = std::fs::remove_dir_all(&store2);
        let _ = std::fs::remove_dir_all(&link_dir);
    }

    #[test]
    fn symlink_errors_on_non_symlink_file() {
        let store = tmpdir("conflict_store");
        let link_dir = tmpdir("conflict_link");
        std::fs::write(link_dir.join("myskill"), b"").unwrap();

        let result = symlink("myskill", &store, &link_dir);
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&link_dir);
    }
}
