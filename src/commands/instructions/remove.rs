use super::{
    project_destination, repo, standalone_entries, stored_files, unlink_owned, validate_key,
};
use crate::config::store_dir;
use anyhow::bail;
use std::{fs, path::Path};

pub fn run(key: String) -> anyhow::Result<()> {
    let dir = if Path::new(&key).is_absolute()
        || matches!(key.as_str(), "codex:AGENTS.md" | "codex:AGENTS.override.md")
    {
        let Some(entry) = standalone_entries()?
            .into_iter()
            .find(|entry| entry.key == key)
        else {
            bail!("'{key}' not found — run apm list to see saved entries");
        };
        unlink_owned(&entry.source, &entry.dest)?;
        entry.source.parent().unwrap().to_path_buf()
    } else {
        validate_key(&key)?;
        let dir = store_dir().join(&key);
        let files = stored_files(&dir)?;
        if files.is_empty() {
            bail!("'{key}' not found — run apm list to see saved entries");
        }
        if let Ok((root, current_key)) = repo()
            && current_key == key
        {
            for source in files {
                unlink_owned(
                    &source,
                    &project_destination(&root, source.strip_prefix(&dir)?)?,
                )?;
            }
        }
        dir
    };
    fs::remove_dir_all(dir)?;
    println!("  removed {key}");
    Ok(())
}
