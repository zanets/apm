use anyhow::{bail, Context};
use std::path::Path;
use std::process::Command;

/// Find unmanaged CLAUDE.md files under `repo_root` via `git ls-files`.
/// Returns relative paths (e.g. `PathBuf::from("src/CLAUDE.md")`).
/// Excludes symlinks (already managed by apm).
pub fn find_claude_mds(repo_root: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut paths = std::collections::HashSet::new();

    for args in [
        vec!["ls-files"],
        vec!["ls-files", "--others", "--exclude-standard"],
        vec!["ls-files", "--others", "--ignored", "--exclude-standard"],
    ] {
        let out = Command::new("git")
            .args(&args)
            .current_dir(repo_root)
            .output()
            .context("git not found")?;
        for line in String::from_utf8(out.stdout)?.lines() {
            if line == "CLAUDE.md" || line.ends_with("/CLAUDE.md") {
                paths.insert(line.to_string());
            }
        }
    }

    let mut result: Vec<std::path::PathBuf> = paths
        .into_iter()
        .map(std::path::PathBuf::from)
        .filter(|p| !repo_root.join(p).is_symlink())
        .collect();
    result.sort();
    Ok(result)
}

pub fn repo_root(dir: &Path) -> anyhow::Result<std::path::PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .context("git not found")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(std::path::PathBuf::from(String::from_utf8(out.stdout)?.trim()))
}

pub fn remote_url(dir: &Path) -> anyhow::Result<String> {
    let out = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(dir)
        .output()
        .context("git not found")?;
    if !out.status.success() {
        anyhow::bail!("no git remote 'origin' — is this a git repo with a remote?");
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

/// Convert a git remote URL to a filesystem-safe key.
/// `https://github.com/org/repo.git` → `github.com_org_repo`
/// `git@github.com:org/repo.git`     → `github.com_org_repo`
pub fn remote_url_to_key(url: &str) -> String {
    let url = url.trim_end_matches(".git");
    let normalized = if let Some(rest) = url.strip_prefix("git@") {
        rest.replacen(':', "/", 1)
    } else if let Some(rest) = url.strip_prefix("https://") {
        rest.to_string()
    } else if let Some(rest) = url.strip_prefix("http://") {
        rest.to_string()
    } else {
        url.to_string()
    };
    normalized.replace('/', "_")
}

fn require_repo(dir: &Path) -> anyhow::Result<()> {
    if !dir.join(".git").exists() {
        bail!("{} is not a git repository — run `apm store init` first", dir.display());
    }
    Ok(())
}

/// `git init` the given directory (idempotent — no-op if already a repo).
pub fn init_repo(dir: &Path) -> anyhow::Result<()> {
    if dir.join(".git").exists() {
        println!("  already a git repository: {}", dir.display());
        return Ok(());
    }
    std::fs::create_dir_all(dir)?;
    let status = Command::new("git")
        .arg("init")
        .current_dir(dir)
        .status()
        .context("git not found")?;
    if !status.success() {
        bail!("git init failed in {}", dir.display());
    }
    println!("  initialized git repository in {}", dir.display());
    Ok(())
}

/// `git add -A` in the given directory.
pub fn add_all(dir: &Path) -> anyhow::Result<()> {
    require_repo(dir)?;
    let status = Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir)
        .status()
        .context("git not found")?;
    if !status.success() {
        bail!("git add failed in {}", dir.display());
    }
    Ok(())
}

/// Whether there are staged changes ready to commit in the given directory.
pub fn has_staged_changes(dir: &Path) -> anyhow::Result<bool> {
    require_repo(dir)?;
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(dir)
        .status()
        .context("git not found")?;
    Ok(!status.success())
}

/// `git commit -m <message>` in the given directory.
pub fn commit(dir: &Path, message: &str) -> anyhow::Result<()> {
    require_repo(dir)?;
    let status = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(dir)
        .status()
        .context("git not found")?;
    if !status.success() {
        bail!("git commit failed in {}", dir.display());
    }
    Ok(())
}

/// `git pull --rebase` in the given directory. On conflict, aborts and leaves the
/// repo mid-rebase for manual resolution rather than attempting to merge.
pub fn pull(dir: &Path) -> anyhow::Result<()> {
    require_repo(dir)?;
    let status = Command::new("git")
        .args(["pull", "--rebase"])
        .current_dir(dir)
        .status()
        .context("git not found")?;
    if !status.success() {
        bail!(
            "git pull --rebase failed in {} — resolve the conflict (cd in, fix the file, `git add <file>`, `git rebase --continue`), then re-run `apm store sync`",
            dir.display()
        );
    }
    Ok(())
}

/// `git push` in the given directory. Sets upstream (`-u origin <branch>`) on the first push.
pub fn push(dir: &Path) -> anyhow::Result<()> {
    require_repo(dir)?;

    let has_upstream = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .current_dir(dir)
        .output()
        .context("git not found")?
        .status
        .success();

    let status = if has_upstream {
        Command::new("git").arg("push").current_dir(dir).status()
    } else {
        let out = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()
            .context("git not found")?;
        let branch = String::from_utf8(out.stdout)?.trim().to_string();
        Command::new("git")
            .args(["push", "-u", "origin", &branch])
            .current_dir(dir)
            .status()
    }
    .context("git not found")?;

    if !status.success() {
        bail!("git push failed in {}", dir.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_url_to_key_https() {
        assert_eq!(
            remote_url_to_key("https://github.com/org/repo.git"),
            "github.com_org_repo"
        );
    }

    #[test]
    fn remote_url_to_key_ssh() {
        assert_eq!(
            remote_url_to_key("git@github.com:org/repo.git"),
            "github.com_org_repo"
        );
    }

    #[test]
    fn remote_url_to_key_no_dot_git() {
        assert_eq!(
            remote_url_to_key("https://github.com/org/repo"),
            "github.com_org_repo"
        );
    }
}
