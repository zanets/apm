pub mod list;
pub mod migrate;
pub mod new;
pub mod remove;
pub mod restore;
pub mod save;

use crate::config::{codex_home, store_dir};
use anyhow::{Context, bail};
use std::{
    ffi::OsStr,
    fs, io,
    path::{Component, Path, PathBuf},
};

const CODEX: &str = "AGENTS.md";
const OVERRIDE: &str = "AGENTS.override.md";
const LEGACY: &str = "CLAUDE.md";

fn is_codex(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(OsStr::to_str),
        Some(CODEX | OVERRIDE)
    )
}

fn is_instruction(path: &Path) -> bool {
    is_codex(path) || path.file_name() == Some(OsStr::new(LEGACY))
}

fn repo() -> anyhow::Result<(PathBuf, String)> {
    let cwd = std::env::current_dir()?;
    let root = crate::git::repo_root(&cwd)?;
    let key = crate::git::remote_url_to_key(&crate::git::remote_url(&root)?);
    validate_key(&key)?;
    Ok((root, key))
}

fn validate_key(key: &str) -> anyhow::Result<()> {
    let mut parts = Path::new(key).components();
    if !matches!(parts.next(), Some(Component::Normal(part)) if part == OsStr::new(key))
        || parts.next().is_some()
        || matches!(key, ".git" | "file")
    {
        bail!("invalid repository key: {key}");
    }
    Ok(())
}

fn exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn is_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file())
}

fn project_destination(root: &Path, relative: &Path) -> anyhow::Result<PathBuf> {
    let mut parent = root.to_path_buf();
    for component in relative.parent().unwrap().components() {
        parent.push(component);
        if exists(&parent)? && !fs::symlink_metadata(&parent)?.is_dir() {
            bail!("{} is not a regular directory", parent.display());
        }
    }
    Ok(root.join(relative))
}

fn unmanaged(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    // Git omits index entries from --others, even in the ignored pass.
    let mut paths = crate::git::files(root, &["--others", "--exclude-standard"])?;
    paths.extend(crate::git::files(
        root,
        &["--others", "--ignored", "--exclude-standard"],
    )?);
    paths.retain(|path| is_codex(path) && is_file(&root.join(path)));
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn stored_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_dir() && entry.file_name() != ".git" {
                collect(&path, files)?;
            } else if kind.is_file() && is_instruction(&path) {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    if fs::symlink_metadata(dir).is_ok_and(|meta| meta.is_dir()) {
        collect(dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

const CODEX_ENTRY_DIR: &str = ".codex-home";

struct StandaloneEntry {
    key: String,
    dest: PathBuf,
    source: PathBuf,
}

fn standalone_entries() -> anyhow::Result<Vec<StandaloneEntry>> {
    let root = store_dir().join("file");
    if !exists(&root)? {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for name in [CODEX, OVERRIDE] {
        let dir = root.join(CODEX_ENTRY_DIR).join(name);
        let source = dir.join(name);
        if fs::symlink_metadata(&dir).is_ok_and(|meta| meta.is_dir()) && is_file(&source) {
            entries.push(StandaloneEntry {
                key: format!("codex:{name}"),
                dest: codex_home()?.join(name),
                source,
            });
        }
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() || entry.file_name() == CODEX_ENTRY_DIR {
            continue;
        }
        let path_file = entry.path().join(".path");
        if !is_file(&path_file) {
            continue;
        }
        let key = fs::read_to_string(path_file)?;
        let dest = PathBuf::from(&key);
        if dest.is_absolute() && is_instruction(&dest) {
            let source = entry.path().join(dest.file_name().unwrap());
            if is_file(&source) {
                entries.push(StandaloneEntry { key, dest, source });
            }
        }
    }
    Ok(entries)
}

fn link(source: &Path, dest: &Path) -> io::Result<()> {
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    #[cfg(windows)]
    use std::os::windows::fs::symlink_file as symlink;
    symlink(source, dest)
}

fn link_new(source: &Path, dest: &Path) -> anyhow::Result<bool> {
    if fs::read_link(dest).is_ok_and(|target| target == source) {
        return Ok(false);
    }
    if exists(dest)? {
        eprintln!("  skipping {}: destination already exists", dest.display());
        return Ok(false);
    }
    fs::create_dir_all(dest.parent().unwrap())?;
    link(source, dest).with_context(|| format!("could not link {}", dest.display()))?;
    println!("  linked {}", dest.display());
    Ok(true)
}

fn unlink_owned(source: &Path, dest: &Path) -> io::Result<()> {
    if fs::read_link(dest).is_ok_and(|target| target == source) {
        fs::remove_file(dest)?;
    }
    Ok(())
}

fn create_stored(dest: &Path, source: Option<&Path>) -> anyhow::Result<()> {
    fs::create_dir_all(dest.parent().unwrap())?;
    let mut input = source.map(fs::File::open).transpose()?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
        .with_context(|| format!("could not create {}", dest.display()))?;
    if let Some(input) = input.as_mut() {
        let result = (|| -> io::Result<()> {
            io::copy(input, &mut output)?;
            output.set_permissions(input.metadata()?.permissions())
        })();
        if let Err(err) = result {
            drop(output);
            fs::remove_file(dest)?;
            return Err(err.into());
        }
    }
    Ok(())
}

// Stage the link on the working tree filesystem, then replace the original in one rename.
fn replace_with_link(source: &Path, dest: &Path) -> anyhow::Result<()> {
    let parent = dest.parent().unwrap();
    let mut index = 0;
    let stage = loop {
        let path = parent.join(format!(".apm-link-{}-{index}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => break path,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => index += 1,
            Err(err) => return Err(err.into()),
        }
    };
    let temp = stage.join("link");
    let result = link(source, &temp).and_then(|()| fs::rename(&temp, dest));
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    let _ = fs::remove_dir(stage);
    result.with_context(|| format!("could not replace {} with a symlink", dest.display()))
}
