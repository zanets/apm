use anyhow::Context;
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
