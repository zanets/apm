use super::{CODEX, LEGACY, create_stored, exists, link, project_destination, repo, stored_files};
use crate::{config::store_dir, git};
use anyhow::bail;
use std::{collections::HashSet, ffi::OsStr, fs, path::PathBuf};

pub fn run(paths: Vec<PathBuf>, apply: bool) -> anyhow::Result<()> {
    if apply && paths.is_empty() {
        bail!("specify the repository-relative CLAUDE.md paths to import");
    }
    let (root, key) = repo()?;
    let dir = store_dir().join(key);
    let legacy: Vec<_> = stored_files(&dir)?
        .into_iter()
        .filter(|path| path.file_name() == Some(OsStr::new(LEGACY)))
        .collect();
    let available: Vec<_> = legacy
        .iter()
        .map(|path| path.strip_prefix(&dir))
        .collect::<Result<_, _>>()?;
    for path in &paths {
        if !available.contains(&path.as_path()) {
            bail!("no saved CLAUDE.md at {}", path.display());
        }
    }
    let tracked: HashSet<_> = git::files(&root, &["--cached"])?.into_iter().collect();
    for (source, relative) in legacy.iter().zip(available) {
        if !paths.is_empty() && !paths.iter().any(|path| path == relative) {
            continue;
        }
        let target = relative.with_file_name(CODEX);
        let dest = project_destination(&root, &target)?;
        let stored = dir.join(&target);
        if tracked.contains(&target) || exists(&dest)? || exists(&stored)? {
            println!(
                "  skipping {} -> {}: destination tracked or already exists",
                relative.display(),
                target.display()
            );
            continue;
        }
        if apply {
            fs::create_dir_all(dest.parent().unwrap())?;
            create_stored(&stored, Some(source))?;
            if let Err(err) = link(&stored, &dest) {
                fs::remove_file(&stored)?;
                return Err(err.into());
            }
        }
        println!("  {} -> {}", relative.display(), target.display());
    }
    if legacy.is_empty() {
        println!("No saved project CLAUDE.md files to migrate.");
    }
    Ok(())
}
