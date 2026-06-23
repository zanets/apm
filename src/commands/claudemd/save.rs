use crate::{config::claudemds_dir, git};
use std::{
    env,
    io::{self, Write},
    path::Path,
};

pub fn run(pick: bool) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = git::repo_root(&cwd)?;
    let url = git::remote_url(&cwd)?;
    let key = git::remote_url_to_key(&url);

    if pick {
        run_pick(&repo_root, &key)
    } else {
        save_one(&cwd, &repo_root, &key)
    }
}

fn run_pick(repo_root: &Path, key: &str) -> anyhow::Result<()> {
    let candidates = git::find_claude_mds(repo_root)?;

    if candidates.is_empty() {
        println!("No unmanaged CLAUDE.md files found in repo.");
        return Ok(());
    }

    let mut saved = 0;
    for rel_path in candidates {
        let rel_dir = rel_path.parent().unwrap_or(Path::new(""));
        let display = rel_display(rel_dir);
        print!("  save {display}/CLAUDE.md? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y") {
            symlink_into_store(&repo_root.join(&rel_path), rel_dir, key)?;
            saved += 1;
        }
    }

    if saved == 0 {
        println!("Nothing saved.");
    }
    Ok(())
}

fn save_one(cwd: &Path, repo_root: &Path, key: &str) -> anyhow::Result<()> {
    let claude_md = cwd.join("CLAUDE.md");
    if !claude_md.exists() {
        anyhow::bail!("no CLAUDE.md found in {}", cwd.display());
    }
    if claude_md.is_symlink() {
        anyhow::bail!("CLAUDE.md is already a symlink — already managed by apm");
    }

    let rel = cwd.strip_prefix(repo_root).unwrap_or(Path::new(""));
    symlink_into_store(&claude_md, rel, key)?;
    Ok(())
}

fn symlink_into_store(claude_md: &Path, rel: &Path, key: &str) -> anyhow::Result<()> {
    let store_file = claudemds_dir().join(key).join(rel).join("CLAUDE.md");
    std::fs::create_dir_all(store_file.parent().unwrap())?;
    std::fs::rename(claude_md, &store_file)?;

    #[cfg(unix)]
    std::os::unix::fs::symlink(&store_file, claude_md)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&store_file, claude_md)?;

    let display = rel_display(rel);
    println!("  saved {display}/CLAUDE.md → {key}");
    println!("  (symlinked in place — edits are live)");
    Ok(())
}

pub(super) fn rel_display(rel: &Path) -> String {
    if rel == Path::new("") {
        ".".to_string()
    } else {
        rel.display().to_string()
    }
}

