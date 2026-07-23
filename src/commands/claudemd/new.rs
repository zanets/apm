use crate::{config::claudemds_dir, git};
use std::{env, path::Path};

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;

    let claude_md = cwd.join("CLAUDE.md");
    if claude_md.exists() {
        anyhow::bail!(
            "CLAUDE.md already exists — use `apm save` to store an existing file"
        );
    }

    let repo_root = git::repo_root(&cwd)?;
    let url = git::remote_url(&cwd)?;
    let key = git::remote_url_to_key(&url);

    let rel = cwd.strip_prefix(&repo_root).unwrap_or(Path::new(""));
    let store_file = claudemds_dir().join(&key).join(rel).join("CLAUDE.md");

    std::fs::create_dir_all(store_file.parent().unwrap())?;
    std::fs::write(&store_file, "")?;

    #[cfg(unix)]
    std::os::unix::fs::symlink(&store_file, &claude_md)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&store_file, &claude_md)?;

    let display = super::save::rel_display(rel);
    println!("  created {display}/CLAUDE.md → {key}");
    Ok(())
}
