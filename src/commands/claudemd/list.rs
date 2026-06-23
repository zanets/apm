use crate::{config::claudemds_dir, git};
use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn run(unmanaged: bool) -> anyhow::Result<()> {
    if unmanaged {
        run_unmanaged()
    } else {
        run_stored()
    }
}

fn run_stored() -> anyhow::Result<()> {
    let base = claudemds_dir();
    if !base.exists() {
        println!("No saved CLAUDE.md files. Save one with: apm md save");
        return Ok(());
    }

    let mut repos: Vec<_> = std::fs::read_dir(&base)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    if repos.is_empty() {
        println!("No saved CLAUDE.md files. Save one with: apm md save");
        return Ok(());
    }

    repos.sort_by_key(|e| e.file_name());

    for repo in repos {
        let dir_name = repo.file_name().to_string_lossy().to_string();

        if dir_name == "file" {
            list_standalone(&repo.path())?;
            continue;
        }

        println!("{dir_name}");
        let mut files = Vec::new();
        collect_store(&repo.path(), &repo.path(), &mut files)?;
        files.sort();
        for (rel, size) in files {
            let display = super::save::rel_display(&rel);
            println!("  {display:<50} {size} bytes");
        }
    }

    Ok(())
}

fn run_unmanaged() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = git::repo_root(&cwd)?;
    let files = git::find_claude_mds(&repo_root)?;

    if files.is_empty() {
        println!("No unmanaged CLAUDE.md files found.");
        return Ok(());
    }

    for rel_path in files {
        let rel_dir = rel_path.parent().unwrap_or(Path::new(""));
        let display = super::save::rel_display(rel_dir);
        let size = std::fs::metadata(repo_root.join(&rel_path))
            .map(|m| m.len())
            .unwrap_or(0);
        println!("  {display:<50} {size} bytes");
    }

    Ok(())
}

fn list_standalone(file_dir: &Path) -> anyhow::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(file_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path_file = entry.path().join(".path");
        let key = if path_file.exists() {
            std::fs::read_to_string(&path_file).unwrap_or_default()
        } else {
            entry.file_name().to_string_lossy().to_string()
        };
        let claude_md = entry.path().join("CLAUDE.md");
        let size = std::fs::metadata(&claude_md).map(|m| m.len()).unwrap_or(0);
        println!("  {key:<50} {size} bytes");
    }
    Ok(())
}

fn collect_store(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, u64)>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_store(root, &path, out)?;
        } else if path.file_name() == Some(OsStr::new("CLAUDE.md")) {
            let rel = path.parent().unwrap().strip_prefix(root).unwrap_or(Path::new("")).to_path_buf();
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            out.push((rel, size));
        }
    }
    Ok(())
}
