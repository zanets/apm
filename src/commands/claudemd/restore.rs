use crate::{config::claudemds_dir, git};
use std::{env, ffi::OsStr, path::{Path, PathBuf}};

pub fn run() -> anyhow::Result<()> {
    let mut any = false;

    let cwd = env::current_dir()?;
    match git::repo_root(&cwd).and_then(|root| git::remote_url(&cwd).map(|url| (root, url))) {
        Ok((repo_root, url)) => {
            let key = git::remote_url_to_key(&url);
            let store_dir = claudemds_dir().join(&key);
            if store_dir.exists() {
                restore_repo(&store_dir, &repo_root)?;
                any = true;
            } else {
                eprintln!("  no saved CLAUDE.md for '{key}' (run `apm md save` first)");
            }
        }
        Err(_) => {}
    }

    let standalone_restored = restore_standalone()?;
    if standalone_restored > 0 {
        any = true;
    }

    if !any {
        eprintln!("nothing to restore");
    }

    Ok(())
}

fn restore_repo(store_dir: &Path, repo_root: &Path) -> anyhow::Result<()> {
    let store_files = find_claude_mds(store_dir)?;
    if store_files.is_empty() {
        return Ok(());
    }

    for store_file in store_files {
        let rel = store_file.parent().unwrap().strip_prefix(store_dir).unwrap_or(Path::new(""));
        let dest = repo_root.join(rel).join("CLAUDE.md");

        if dest.is_symlink() {
            std::fs::remove_file(&dest)?;
        } else if dest.exists() {
            eprintln!("  skipping {}: exists and is not a symlink (remove manually to restore)", dest.display());
            continue;
        }

        std::fs::create_dir_all(dest.parent().unwrap())?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(&store_file, &dest)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&store_file, &dest)?;

        let display = super::save::rel_display(rel);
        println!("  restored {display}/CLAUDE.md");
    }

    Ok(())
}

fn restore_standalone() -> anyhow::Result<usize> {
    let file_dir = claudemds_dir().join("file");
    if !file_dir.is_dir() {
        return Ok(0);
    }

    let mut count = 0;
    let mut entries: Vec<_> = std::fs::read_dir(&file_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path_file = entry.path().join(".path");
        if !path_file.exists() {
            continue;
        }
        let dest = PathBuf::from(std::fs::read_to_string(&path_file)?.trim());
        let store_file = entry.path().join("CLAUDE.md");
        if !store_file.exists() {
            continue;
        }

        if dest.is_symlink() {
            std::fs::remove_file(&dest)?;
        } else if dest.exists() {
            eprintln!("  skipping {}: exists and is not a symlink (remove manually to restore)", dest.display());
            continue;
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&store_file, &dest)?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&store_file, &dest)?;

        println!("  restored {}", dest.display());
        count += 1;
    }

    Ok(count)
}

fn find_claude_mds(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    collect(dir, &mut results)?;
    results.sort();
    Ok(results)
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else if path.file_name() == Some(OsStr::new("CLAUDE.md")) {
            out.push(path);
        }
    }
    Ok(())
}
