use super::{
    StandaloneEntry, link_new, project_destination, repo, standalone_entries, stored_files,
};
use crate::{config::store_dir, git};
use std::collections::HashSet;

pub fn run() -> anyhow::Result<()> {
    if let Ok((root, key)) = repo() {
        let dir = store_dir().join(key);
        let tracked: HashSet<_> = git::files(&root, &["--cached"])?.into_iter().collect();
        for source in stored_files(&dir)? {
            let relative = source.strip_prefix(&dir)?;
            if tracked.contains(relative) {
                eprintln!("  skipping {}: tracked by Git", relative.display());
                continue;
            }
            link_new(&source, &project_destination(&root, relative)?)?;
        }
    }
    for StandaloneEntry { dest, source, .. } in standalone_entries()? {
        if let Ok(root) = git::repo_root(dest.parent().unwrap())
            && git::files(&root, &["--cached"])?
                .iter()
                .any(|path| root.join(path) == dest)
        {
            eprintln!("  skipping {}: tracked by Git", dest.display());
            continue;
        }
        link_new(&source, &dest)?;
    }
    Ok(())
}
