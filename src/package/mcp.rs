use crate::config::{amp_dir, Agent};
use anyhow::Context;
use std::path::{Path, PathBuf};

pub struct Mcp {
    pub agent: Agent,
}

impl Mcp {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    pub fn store_base() -> PathBuf {
        amp_dir().join("store").join("mcps")
    }

    pub fn store_path(&self, name: &str) -> PathBuf {
        Self::store_base().join(name)
    }

    pub fn enable(&self, name: &str, command: &str, args: &[String]) -> anyhow::Result<()> {
        let path = self.agent.settings_path();
        let mut settings = load_settings(&path)?;

        if settings.get("mcpServers").and_then(|s| s.get(name)).is_some() {
            println!("  {name}: already enabled");
            return Ok(());
        }

        {
            let obj = settings.as_object_mut()
                .context("settings.json root is not a JSON object")?;
            obj.entry("mcpServers")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .context("mcpServers is not an object in settings.json")?
                .insert(name.to_string(), serde_json::json!({
                    "type": "stdio",
                    "command": command,
                    "args": args,
                }));
        }

        save_settings(&path, &settings)?;
        println!("  enabled {name}");
        Ok(())
    }

    pub fn disable(&self, name: &str) -> anyhow::Result<()> {
        let path = self.agent.settings_path();
        let mut settings = load_settings(&path)?;

        let removed = settings
            .get_mut("mcpServers")
            .and_then(|v| v.as_object_mut())
            .map(|servers| servers.remove(name).is_some())
            .unwrap_or(false);

        if removed {
            save_settings(&path, &settings)?;
            println!("  disabled {name}");
        } else {
            println!("  {name}: not enabled, nothing to disable");
        }
        Ok(())
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        let path = self.agent.settings_path();
        load_settings(&path)
            .ok()
            .and_then(|s| s.get("mcpServers")?.get(name).map(|_| true))
            .unwrap_or(false)
    }
}

fn load_settings(path: &Path) -> anyhow::Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::Value::Object(Default::default()));
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(serde_json::Value::Object(Default::default()));
    }
    serde_json::from_str(&content).context("invalid settings.json")
}

fn save_settings(path: &Path, settings: &serde_json::Value) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, serde_json::to_string_pretty(settings)?)
        .with_context(|| format!("cannot write {}", path.display()))
}
