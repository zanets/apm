use super::{CODEX, create_stored, exists, link, repo};
use crate::{config::store_dir, git};
use anyhow::bail;
use std::{env, fs};

pub fn run() -> anyhow::Result<()> {
    let (root, key) = repo()?;
    let dest = env::current_dir()?.join(CODEX);
    if exists(&dest)? {
        bail!("{} already exists", dest.display());
    }

    let relative = dest.strip_prefix(&root)?;
    if git::files(&root, &["--cached"])?
        .iter()
        .any(|path| path == relative)
    {
        bail!("{} is tracked by Git", dest.display());
    }
    let stored = store_dir().join(key).join(relative);
    create_stored(&stored, None)?;
    if let Err(err) = link(&stored, &dest) {
        fs::remove_file(&stored)?;
        return Err(err.into());
    }
    println!("  created {}", dest.display());
    Ok(())
}
