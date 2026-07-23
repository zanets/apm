use crate::{config::claudemds_dir, git};
use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn run(key: String) -> anyhow::Result<()> {
    // Standalone: key is an absolute path (starts with /)
    if key.starts_with('/') {
        let file_path = Path::new(&key);
        let encoded = super::save::path_to_store_key(file_path);
        let store_dir = claudemds_dir().join("file").join(&encoded);
        if !store_dir.exists() {
            anyhow::bail!("'{key}' not found — run `apm list` to see saved entries");
        }
        if file_path.is_symlink() {
            let _ = std::fs::remove_file(file_path);
        }
        std::fs::remove_dir_all(&store_dir)?;
        println!("  removed {key}");
        return Ok(());
    }

    // Git-backed entry
    let store_dir = claudemds_dir().join(&key);
    if !store_dir.exists() {
        anyhow::bail!("'{key}' not found — run `apm list` to see saved entries");
    }

    let cwd = env::current_dir()?;
    if let Ok(repo_root) = git::repo_root(&cwd) {
        if let Ok(url) = git::remote_url(&cwd) {
            if git::remote_url_to_key(&url) == key {
                clean_symlinks(&store_dir, &repo_root);
            }
        }
    }

    std::fs::remove_dir_all(&store_dir)?;
    println!("  removed {key}");
    Ok(())
}

/// Walk store dir, remove any symlink in the repo that points into it.
fn clean_symlinks(store_dir: &Path, repo_root: &Path) {
    let mut store_files = Vec::new();
    collect_claude_mds(store_dir, &mut store_files);

    for store_file in store_files {
        let rel = store_file
            .parent()
            .unwrap()
            .strip_prefix(store_dir)
            .unwrap_or(Path::new(""));
        let symlink_path = repo_root.join(rel).join("CLAUDE.md");

        if symlink_path.is_symlink() {
            if let Ok(target) = std::fs::read_link(&symlink_path) {
                if target == store_file {
                    if let Err(e) = std::fs::remove_file(&symlink_path) {
                        eprintln!("  warning: could not remove {}: {e}", symlink_path.display());
                    } else {
                        println!("  unlinked {}", symlink_path.display());
                    }
                }
            }
        }
    }
}

fn collect_claude_mds(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_claude_mds(&path, out);
        } else if path.file_name() == Some(OsStr::new("CLAUDE.md")) {
            out.push(path);
        }
    }
}
