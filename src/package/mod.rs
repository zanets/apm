pub mod mcp;
pub mod skill;
pub mod tool;

use std::path::{Path, PathBuf};

pub trait Package {
    /// 這個 package 在 store 裡的路徑
    fn store_path(&self, name: &str) -> PathBuf;

    /// 把 store 裡的 package 裝進 agent（symlink、寫 config、等）
    fn install(&self, name: &str) -> anyhow::Result<()>;

    /// 是否已裝進 agent
    fn is_installed(&self, name: &str) -> bool;

    /// 從 agent 移除（拔 symlink），store 保留
    fn uninstall(&self, name: &str) -> anyhow::Result<()>;
}

pub(crate) fn symlink(name: &str, repo: &Path, link_dir: &Path) -> anyhow::Result<()> {
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
        let d = std::env::temp_dir().join(format!("amp_symlink_test_{label}"));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn symlink_creates_link() {
        let store = tmpdir("creates_store");
        let link_dir = tmpdir("creates_link");

        symlink("pkg", &store, &link_dir).unwrap();

        let link_path = link_dir.join("pkg");
        assert!(link_path.is_symlink());
        assert_eq!(std::fs::read_link(&link_path).unwrap(), store);

        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&link_dir);
    }

    #[test]
    fn symlink_same_target_is_noop() {
        let store = tmpdir("noop_store");
        let link_dir = tmpdir("noop_link");

        symlink("pkg", &store, &link_dir).unwrap();
        symlink("pkg", &store, &link_dir).unwrap();

        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&link_dir);
    }

    #[test]
    fn symlink_relinks_different_target() {
        let store1 = tmpdir("relink_store1");
        let store2 = tmpdir("relink_store2");
        let link_dir = tmpdir("relink_link");

        symlink("pkg", &store1, &link_dir).unwrap();
        symlink("pkg", &store2, &link_dir).unwrap();

        let link_path = link_dir.join("pkg");
        assert_eq!(std::fs::read_link(&link_path).unwrap(), store2);

        let _ = std::fs::remove_dir_all(&store1);
        let _ = std::fs::remove_dir_all(&store2);
        let _ = std::fs::remove_dir_all(&link_dir);
    }

    #[test]
    fn symlink_errors_on_non_symlink_file() {
        let store = tmpdir("conflict_store");
        let link_dir = tmpdir("conflict_link");
        std::fs::write(link_dir.join("pkg"), b"").unwrap();

        let result = symlink("pkg", &store, &link_dir);
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&store);
        let _ = std::fs::remove_dir_all(&link_dir);
    }
}
