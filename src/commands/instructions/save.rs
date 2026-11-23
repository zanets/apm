use super::{
    CODEX_ENTRY_DIR, create_stored, is_codex, is_instruction, replace_with_link, repo, unmanaged,
};
use crate::{
    config::{codex_home, store_dir},
    git,
};
use anyhow::{Context, bail};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub fn run(pick: bool, file: Option<PathBuf>) -> anyhow::Result<()> {
    if let Some(path) = file {
        return save_standalone(&path);
    }
    let (root, key) = repo()?;
    let files = unmanaged(&root)?;
    if files.is_empty() {
        println!("No unmanaged Codex instructions found.");
    }
    for path in files {
        if pick {
            print!("  save {}? [y/N] ", path.display());
            io::stdout().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            if !answer.trim().eq_ignore_ascii_case("y") {
                continue;
            }
        }
        save_file(&root.join(&path), &store_dir().join(&key).join(&path))?;
    }
    Ok(())
}

fn save_standalone(path: &Path) -> anyhow::Result<()> {
    if !is_instruction(path) || !fs::symlink_metadata(path)?.is_file() {
        bail!("expected a regular instruction file at {}", path.display());
    }
    let path = path.canonicalize()?;
    if let Ok(root) = git::repo_root(path.parent().unwrap()) {
        let relative = path.strip_prefix(&root)?;
        if git::files(&root, &["--cached"])?.contains(&relative.to_path_buf()) {
            bail!("{} is tracked by Git", path.display());
        }
    }

    let name = path.file_name().unwrap();
    let root = store_dir().join("file");
    let portable = is_codex(&path) && path.parent() == Some(codex_home()?.as_path());
    let dir = if portable {
        root.join(CODEX_ENTRY_DIR).join(name)
    } else {
        root.join(path_to_store_key(&path))
    };
    fs::create_dir_all(dir.parent().unwrap())?;
    fs::create_dir(&dir).with_context(|| {
        format!(
            "standalone entry already exists or cannot be created: {}",
            path.display()
        )
    })?;
    let result = (|| -> anyhow::Result<()> {
        if !portable {
            let path_text = path
                .to_str()
                .context("standalone path must be valid UTF-8")?;
            fs::write(dir.join(".path"), path_text)?;
        }
        save_file(&path, &dir.join(name))
    })();
    if result.is_err() {
        fs::remove_dir_all(&dir)?;
    }
    result
}

fn path_to_store_key(path: &Path) -> String {
    let key = path
        .to_string_lossy()
        .trim_start_matches('/')
        .replace('/', "_");
    #[cfg(windows)]
    let key = key.replace(['\\', ':'], "_");
    key
}

fn save_file(source: &Path, stored: &Path) -> anyhow::Result<()> {
    if !fs::symlink_metadata(source)?.is_file() {
        bail!("expected a regular file at {}", source.display());
    }
    create_stored(stored, Some(source))?;
    if let Err(err) = replace_with_link(stored, source) {
        fs::remove_file(stored)?;
        return Err(err);
    }
    println!("  saved {}", source.display());
    Ok(())
}
