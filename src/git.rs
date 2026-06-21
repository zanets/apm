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
}
