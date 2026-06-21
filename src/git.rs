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
}
