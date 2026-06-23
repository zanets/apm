use crate::config::Agent;
use anyhow::Context;
use std::path::Path;

pub struct Mcp {
    pub agent: Agent,
}

impl Mcp {
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    pub fn enable(&self, name: &str, command: &str, args: &[String]) -> anyhow::Result<()> {
        match self.agent {
            Agent::Claude => enable_via_claude(name, command, args),
            a => anyhow::bail!("MCP is not supported for agent '{}'", a.as_str()),
        }
    }

    pub fn disable(&self, name: &str) -> anyhow::Result<()> {
        match self.agent {
            Agent::Claude => disable_via_claude(name),
            a => anyhow::bail!("MCP is not supported for agent '{}'", a.as_str()),
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        let path = self.agent.settings_path();
        load_settings(&path)
            .ok()
            .and_then(|s| s.get("mcpServers")?.get(name).map(|_| true))
            .unwrap_or(false)
    }
}

fn enable_via_claude(name: &str, command: &str, args: &[String]) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("claude");
    cmd.args(["mcp", "add", name, "--", command]);
    cmd.args(args);
    let status = cmd.status().context("claude not found in PATH")?;
    if !status.success() {
        anyhow::bail!("claude mcp add failed for {name}");
    }
    println!("  enabled {name} (claude)");
    Ok(())
}

fn disable_via_claude(name: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("claude")
        .args(["mcp", "remove", name])
        .status()
        .context("claude not found in PATH")?;
    if !status.success() {
        anyhow::bail!("claude mcp remove failed for {name}");
    }
    println!("  disabled {name} (claude)");
    Ok(())
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
