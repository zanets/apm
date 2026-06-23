use anyhow::{bail, Context};
use std::path::Path;
use std::process::Command;

pub fn parse_source(source: &str) -> anyhow::Result<String> {
    let path = if let Some(p) = source.strip_prefix("github:") {
        p
    } else {
        bail!("unsupported source '{source}'")
    };
    Ok(format!("https://github.com/{}.git", path.trim_end_matches(".git")))
}

/// Normalizes user input into (canonical_source, name).
/// Accepts: "user/repo", "github:user/repo", either with optional ".git" suffix.
pub fn resolve_source(s: &str) -> anyhow::Result<(String, String)> {
    let s = s.trim_end_matches(".git");

    if let Some(path) = s.strip_prefix("github:") {
        return extract_github(path, s);
    }

    if s.contains('/') {
        return extract_github(s, &format!("github:{s}"));
    }

    anyhow::bail!("ambiguous source '{s}' — use user/{s} to specify the GitHub owner")
}

fn extract_github(path: &str, canonical: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("invalid source '{canonical}' — expected user/repo");
    }
    let name = parts[1].trim_end_matches(".git").to_string();
    Ok((format!("github:{}", path.trim_end_matches(".git")), name))
}

pub fn clone(url: &str, dest: &Path, ref_: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["clone", "--depth=1", "--branch", ref_, url])
        .arg(dest)
        .status()
        .context("git not found")?;

    if !status.success() {
        // ref might be a default branch name that differs — try without --branch
        let status = Command::new("git")
            .args(["clone", "--depth=1", url])
            .arg(dest)
            .status()
            .context("git not found")?;
        if !status.success() {
            bail!("git clone failed for {url}");
        }
    }
    Ok(())
}

pub fn pull(dest: &Path) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(dest)
        .status()
        .context("git not found")?;
    if !status.success() {
        bail!("git pull failed in {}", dest.display());
    }
    Ok(())
}

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

pub fn current_commit(dest: &Path) -> anyhow::Result<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(dest)
        .output()
        .context("git not found")?;
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_source_github_prefix() {
        assert_eq!(
            parse_source("github:user/repo").unwrap(),
            "https://github.com/user/repo.git"
        );
    }

    #[test]
    fn parse_source_strips_dot_git() {
        assert_eq!(
            parse_source("github:user/repo.git").unwrap(),
            "https://github.com/user/repo.git"
        );
    }

    #[test]
    fn parse_source_rejects_no_prefix() {
        let err = parse_source("user/repo").unwrap_err();
        assert!(err.to_string().contains("unsupported source"));
    }

    #[test]
    fn resolve_user_repo_shorthand() {
        let (canonical, name) = resolve_source("user/repo").unwrap();
        assert_eq!(canonical, "github:user/repo");
        assert_eq!(name, "repo");
    }

    #[test]
    fn resolve_github_prefix() {
        let (canonical, name) = resolve_source("github:user/repo").unwrap();
        assert_eq!(canonical, "github:user/repo");
        assert_eq!(name, "repo");
    }

    #[test]
    fn resolve_strips_dot_git() {
        let (canonical, name) = resolve_source("user/repo.git").unwrap();
        assert_eq!(canonical, "github:user/repo");
        assert_eq!(name, "repo");
    }

    #[test]
    fn resolve_bare_name_errors() {
        assert!(resolve_source("mypackage").is_err());
    }

    #[test]
    fn resolve_empty_repo_errors() {
        assert!(resolve_source("user/").is_err());
    }

    #[test]
    fn resolve_empty_user_errors() {
        assert!(resolve_source("github:/repo").is_err());
    }

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
