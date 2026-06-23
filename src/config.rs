use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Packages {
    #[serde(default)]
    pub skills: HashMap<String, SkillEntry>,
    #[serde(default)]
    pub mcps: HashMap<String, McpEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SkillEntry {
    pub source: String,
    #[serde(rename = "ref", default = "default_ref")]
    pub ref_: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpEntry {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Cursor,
    Windsurf,
}

impl Agent {
    pub fn all() -> &'static [Agent] {
        &[Agent::Claude, Agent::Cursor, Agent::Windsurf]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Cursor => "cursor",
            Agent::Windsurf => "windsurf",
        }
    }

    pub fn settings_path(&self) -> PathBuf {
        let home = dirs::home_dir().expect("no home dir");
        match self {
            Agent::Claude => home.join(".claude").join("settings.json"),
            Agent::Cursor => home.join(".cursor").join("settings.json"),
            Agent::Windsurf => home.join(".windsurf").join("settings.json"),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_agent")]
    pub default_agent: Agent,
}

fn default_agent() -> Agent {
    Agent::Claude
}

impl Default for Config {
    fn default() -> Self {
        Self { default_agent: Agent::Claude }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_dir().join("config.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        toml::from_str(&content).context("invalid config.toml")
    }
}

/// $XDG_CONFIG_HOME/apm — packages.toml, packages.lock, config.toml
pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().expect("no home dir").join(".config"))
        .join("apm")
}

/// $XDG_DATA_HOME/apm — git clones (store/)
pub fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().expect("no home dir").join(".local/share"))
        .join("apm")
}

pub fn claudemds_dir() -> PathBuf {
    data_dir().join("claudemds")
}

pub fn packages_path() -> PathBuf {
    config_dir().join("packages.toml")
}

pub fn lockfile_path() -> PathBuf {
    config_dir().join("packages.lock")
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
"#;
        let p: Packages = toml::from_str(toml).unwrap();
        assert_eq!(p.skills["my-skill"].source, "github:user/repo");
        assert_eq!(p.skills["my-skill"].ref_, "main");
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
    }
}
