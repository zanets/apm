use crate::{
    config::{Agent, Packages, SkillEntry},
    git,
    lockfile::{LockEntry, Lockfile},
    package::{skill::Skill, Package},
};
use chrono::Utc;

pub fn run(source: String, ref_: String, name_override: Option<String>) -> anyhow::Result<()> {
    let (source_canonical, derived_name) = parse_source(&source)?;
    let name = name_override.unwrap_or(derived_name);

    let mut packages = Packages::load()?;

    if let Some(existing) = packages.skills.get(&name) {
        if existing.source != source_canonical {
            anyhow::bail!(
                "'{name}' already exists pointing to {}\n  \
                 use --name <alias> to install under a different name",
                existing.source
            );
        }
        anyhow::bail!("'{name}' already exists in packages.toml");
    }

    // clone first — only persist if successful
    let skill = Skill::new(Agent::Claude);
    std::fs::create_dir_all(Skill::store_base())?;

    let repo = skill.store_path(&name);
    let url = git::parse_source(&source_canonical)?;
    print!("  getting {name} ...");
    git::clone(&url, &repo, &ref_)?;

    let commit = git::current_commit(&repo)?;

    packages.skills.insert(name.clone(), SkillEntry { source: source_canonical.clone(), ref_: ref_.clone() });
    packages.save()?;

    let mut lock = Lockfile::load()?;
    lock.skills.insert(name.clone(), LockEntry { commit: commit.clone(), updated_at: Utc::now().to_rfc3339() });
    lock.save()?;

    println!(" done ({commit})");
    println!("Run `amp skill enable` to activate it");
    Ok(())
}

/// 接受以下格式：
///   user/repo          → github:user/repo, name = repo
///   github:user/repo   → github:user/repo, name = repo
///   name               → error（無法確定 user）
fn parse_source(s: &str) -> anyhow::Result<(String, String)> {
    let s = s.trim_end_matches(".git");

    if let Some(path) = s.strip_prefix("github:") {
        return extract_github(path, s);
    }

    if s.contains('/') {
        return extract_github(s, &format!("github:{s}"));
    }

    anyhow::bail!(
        "ambiguous source '{s}' — use user/{s} to specify the GitHub owner"
    )
}

fn extract_github(path: &str, canonical: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("invalid source '{canonical}' — expected user/repo");
    }
    let name = parts[1].trim_end_matches(".git").to_string();
    Ok((format!("github:{}", path.trim_end_matches(".git")), name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_user_repo_shorthand() {
        let (canonical, name) = parse_source("user/repo").unwrap();
        assert_eq!(canonical, "github:user/repo");
        assert_eq!(name, "repo");
    }

    #[test]
    fn parse_github_prefix() {
        let (canonical, name) = parse_source("github:user/repo").unwrap();
        assert_eq!(canonical, "github:user/repo");
        assert_eq!(name, "repo");
    }

    #[test]
    fn parse_strips_dot_git() {
        let (canonical, name) = parse_source("user/repo.git").unwrap();
        assert_eq!(canonical, "github:user/repo");
        assert_eq!(name, "repo");
    }

    #[test]
    fn parse_bare_name_errors() {
        assert!(parse_source("myskill").is_err());
    }

    #[test]
    fn parse_empty_repo_errors() {
        assert!(parse_source("user/").is_err());
    }

    #[test]
    fn parse_empty_user_errors() {
        assert!(parse_source("github:/repo").is_err());
    }
}
