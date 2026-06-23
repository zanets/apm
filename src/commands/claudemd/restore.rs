use crate::{config::claudemds_dir, git};
use std::{env, ffi::OsStr, path::{Path, PathBuf}};

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = git::repo_root(&cwd)?;
    let url = git::remote_url(&cwd)?;
    let key = git::remote_url_to_key(&url);

    let store_dir = claudemds_dir().join(&key);
    if !store_dir.exists() {
        anyhow::bail!("no saved CLAUDE.md files for '{key}'\n  run `apm claudemd save` first");
    }

    let store_files = find_claude_mds(&store_dir)?;
    if store_files.is_empty() {
        anyhow::bail!("store entry '{key}' exists but contains no CLAUDE.md files");
    }

    for store_file in store_files {
        let rel = store_file.parent().unwrap().strip_prefix(&store_dir).unwrap_or(Path::new(""));
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
