use crate::{config::claudemds_dir, git};
use std::process::Command;

pub fn run(message: Option<String>) -> anyhow::Result<()> {
    let dir = claudemds_dir();
    git::add_all(&dir)?;

    if git::has_staged_changes(&dir)? {
        let message = message.unwrap_or_else(default_message);
        git::commit(&dir, &message)?;
    } else {
        println!("  no local changes to commit");
    }

    if git::remote_url(&dir).is_err() {
        println!(
            "  no remote configured — skipping pull/push (run `git remote add origin <url>` in {})",
            dir.display()
        );
        return Ok(());
    }

    git::pull(&dir)?;
    git::push(&dir)?;
    println!("  synced {}", dir.display());
    Ok(())
}

fn default_message() -> String {
    let out = Command::new("date").arg("+apm: sync %Y-%m-%d %H:%M").output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "apm: sync".to_string(),
    }
}
