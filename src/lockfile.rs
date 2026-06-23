use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Lockfile {
    #[serde(default)]
    pub skills: HashMap<String, LockEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LockEntry {
    pub commit: String,
    pub updated_at: String,
}

impl Lockfile {
    pub fn load() -> anyhow::Result<Self> {
        let path = crate::config::lockfile_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        toml::from_str(&content).context("invalid packages.lock")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = crate::config::lockfile_path();
        std::fs::write(&path, toml::to_string_pretty(self)?)
            .with_context(|| format!("cannot write {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfile_round_trip() {
        let toml = r#"
[skills.my-skill]
commit = "abc1234"
updated_at = "2026-06-21T00:00:00+00:00"
"#;
        let lock: Lockfile = toml::from_str(toml).unwrap();
        assert_eq!(lock.skills["my-skill"].commit, "abc1234");
    }

    #[test]
    fn empty_lockfile_is_valid() {
        let lock: Lockfile = toml::from_str("").unwrap();
        assert!(lock.skills.is_empty());
    }
}
