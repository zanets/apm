use super::{standalone_entries, stored_files, unmanaged};
use crate::{config::store_dir, git};
use std::{env, fs, io::ErrorKind};

pub fn run(unmanaged_only: bool) -> anyhow::Result<()> {
    if unmanaged_only {
        let root = git::repo_root(&env::current_dir()?)?;
        for path in unmanaged(&root)? {
            println!(
                "  {:<50} {} bytes",
                path.display(),
                fs::metadata(root.join(&path))?.len()
            );
        }
        return Ok(());
    }

    let base = store_dir();
    let mut repos = match fs::read_dir(&base) {
        Ok(entries) => entries.collect::<Result<Vec<_>, _>>()?,
        Err(err) if err.kind() == ErrorKind::NotFound => Vec::new(),
        Err(err) => return Err(err.into()),
    };
    repos.sort_by_key(|entry| entry.file_name());
    let mut any = false;
    for repo in repos {
        if repo.file_name() == ".git" || repo.file_name() == "file" {
            continue;
        }
        let path = repo.path();
        let files = stored_files(&path)?;
        if files.is_empty() {
            continue;
        }
        any = true;
        println!("{}", repo.file_name().to_string_lossy());
        for file in files {
            println!(
                "  {:<50} {} bytes",
                file.strip_prefix(&path)?.display(),
                fs::metadata(&file)?.len()
            );
        }
    }

    let standalone = standalone_entries()?;
    if !standalone.is_empty() {
        any = true;
        println!("Standalone files:");
        let mut standalone = standalone;
        standalone.sort_by(|left, right| left.key.cmp(&right.key));
        for entry in standalone {
            let size = fs::metadata(&entry.source)?.len();
            if entry.key.starts_with("codex:") {
                println!(
                    "  {:<50} {size} bytes → {}",
                    entry.key,
                    entry.dest.display()
                );
            } else {
                println!("  {:<50} {size} bytes", entry.key);
            }
        }
    }
    if !any {
        println!("No saved instructions. Save one with: apm save");
    }
    Ok(())
}
