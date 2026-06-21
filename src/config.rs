use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Packages {
    #[serde(default)]
    pub skills: HashMap<String, SkillEntry>,
    #[serde(default)]
    pub tools: HashMap<String, ToolEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SkillEntry {
    pub source: String,
    #[serde(rename = "ref", default = "default_ref")]
    pub ref_: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolEntry {
    pub source: String,
    #[serde(rename = "ref", default = "default_ref")]
    pub ref_: String,
}

fn default_ref() -> String {
    "main".to_string()
}

impl Packages {
    pub fn load() -> anyhow::Result<Self> {
        let path = packages_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        toml::from_str(&content).context("invalid packages.toml")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = packages_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, toml::to_string_pretty(self)?)
            .with_context(|| format!("cannot write {}", path.display()))
    }
}

/// AI agent 種類，決定 package 安裝到哪裡
#[derive(Debug, Clone, Copy)]
pub enum Agent {
    Claude,
}

/// ~/.amp/ — amp 的根目錄
pub fn amp_dir() -> PathBuf {
    dirs::home_dir().expect("no home dir").join(".amp")
}

pub fn packages_path() -> PathBuf {
    amp_dir().join("packages.toml")
}

pub fn lockfile_path() -> PathBuf {
    amp_dir().join("packages.lock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packages_round_trip() {
        let toml = r#"
[skills.my-skill]
source = "github:user/repo"
ref = "main"

[tools.my-tool]
source = "github:user/tool"
ref = "v1.0"
"#;
        let p: Packages = toml::from_str(toml).unwrap();
        assert_eq!(p.skills["my-skill"].source, "github:user/repo");
        assert_eq!(p.skills["my-skill"].ref_, "main");
        assert_eq!(p.tools["my-tool"].ref_, "v1.0");
    }

    #[test]
    fn packages_default_ref_is_main() {
        let toml = r#"
[skills.my-skill]
source = "github:user/repo"
"#;
        let p: Packages = toml::from_str(toml).unwrap();
        assert_eq!(p.skills["my-skill"].ref_, "main");
    }

    #[test]
    fn empty_packages_toml_is_valid() {
        let p: Packages = toml::from_str("").unwrap();
        assert!(p.skills.is_empty());
        assert!(p.tools.is_empty());
    }
}
