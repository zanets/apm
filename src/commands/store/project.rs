use crate::{
    commands::instructions::{
        CODEX, OVERRIDE, exists, is_codex, link, project_destination, repo, stored_files,
    },
    config::store_dir,
    git,
};
use anyhow::{Context, bail};
use clap::Subcommand;
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Subcommand)]
pub enum Action {
    /// Fetch the store, restore missing project links, and print active guidance
    Pull,
    /// Commit and push changes to the current project's managed Codex files
    Push,
}

pub fn run(action: Action) -> anyhow::Result<()> {
    let Ok((root, key)) = repo() else {
        return Ok(());
    };
    let store = store_dir();
    if !store.join(".git").is_dir() {
        return Ok(());
    }
    let Ok(remote) = git::remote_url(&store) else {
        return Ok(());
    };
    if root == store.canonicalize()? || key == git::remote_url_to_key(&remote) {
        return Ok(());
    }
    let lock = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(store.join(".git/apm-project-sync.lock"))?;
    lock.lock()?;

    let entry = store.join(key);
    let tracked: HashSet<_> = git::files(&root, &["--cached"])?.into_iter().collect();
    if matches!(action, Action::Pull) {
        let sync = fetch(&store).and_then(|_| update(&store));
        for source in stored_files(&entry)?
            .into_iter()
            .filter(|path| is_codex(path))
        {
            let relative = source.strip_prefix(&entry)?;
            if !tracked.contains(relative) {
                let dest = project_destination(&root, relative)?;
                if !exists(&dest)? {
                    fs::create_dir_all(dest.parent().unwrap())?;
                    link(&source, &dest)?;
                }
            }
        }
        print_guidance(&root, &entry)?;
        return sync;
    }

    let paths = managed_paths(&root, &store, &entry, &tracked)?;
    if paths.is_empty() {
        return Ok(());
    }
    // Avoid network work at the end of an unchanged turn.
    let upstream = git_command(&store)
        .args(["rev-parse", "--quiet", "--verify", "@{u}"])
        .output()?;
    if upstream.status.success()
        && output(&store, ["rev-parse", "HEAD"])? == upstream.stdout
        && changed_paths(&store, &paths)?.is_empty()
    {
        return Ok(());
    }
    let branch = fetch(&store)?;
    update(&store)?;
    check_pending_commits(&store, &paths)?;
    let changed = changed_paths(&store, &paths)?;
    if !changed.is_empty() {
        let mut args = git_command(&store);
        args.args(["add", "--"]).args(&changed);
        checked(args.output()?)?;
        let mut args = git_command(&store);
        args.args([
            "commit",
            "--only",
            "-m",
            "docs(agents): sync project instructions",
            "--",
        ])
        .args(&changed);
        checked(args.output()?)?;
    }

    if output(&store, ["rev-parse", "HEAD"])? != output(&store, ["rev-parse", "FETCH_HEAD"])? {
        let mut push = git_command(&store);
        push.args(["push", "origin"])
            .arg(format!("HEAD:refs/heads/{branch}"));
        checked(push.output()?)?;
        println!("Synced managed Codex instructions to claudemds.");
    }
    Ok(())
}

fn git_command(store: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(store)
        .env("GIT_LITERAL_PATHSPECS", "1")
        .env("GIT_TERMINAL_PROMPT", "0");
    cmd
}

fn checked(result: Output) -> anyhow::Result<Vec<u8>> {
    if !result.status.success() {
        bail!("{}", String::from_utf8_lossy(&result.stderr).trim());
    }
    Ok(result.stdout)
}

fn output<const N: usize>(store: &Path, args: [&str; N]) -> anyhow::Result<Vec<u8>> {
    checked(git_command(store).args(args).output()?)
}

fn fetch(store: &Path) -> anyhow::Result<String> {
    let branch = String::from_utf8(output(store, ["branch", "--show-current"])?)?;
    let branch = branch.trim();
    if branch.is_empty() {
        bail!("The instruction store is on a detached HEAD.");
    }
    output(store, ["fetch", "origin", &format!("refs/heads/{branch}")])
        .context("Could not fetch claudemds")?;
    Ok(branch.to_owned())
}

fn update(store: &Path) -> anyhow::Result<()> {
    output(store, ["merge", "--ff-only", "FETCH_HEAD"])
        .context("Could not fast-forward claudemds. Local changes are preserved.")?;
    Ok(())
}

fn managed_paths(
    root: &Path,
    store: &Path,
    entry: &Path,
    tracked: &HashSet<PathBuf>,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for source in stored_files(entry)?
        .into_iter()
        .filter(|path| is_codex(path))
    {
        let relative = source.strip_prefix(entry)?;
        if tracked.contains(relative) {
            continue;
        }
        let dest = project_destination(root, relative)?;
        if fs::read_link(dest).is_ok_and(|target| target == source) {
            paths.push(source.strip_prefix(store)?.to_path_buf());
        }
    }
    Ok(paths)
}

fn changed_paths(store: &Path, paths: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    let mut changed = Vec::new();
    for path in paths {
        let tracked = git_command(store)
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(path)
            .output()?;
        let diff = git_command(store)
            .args(["diff", "--quiet", "HEAD", "--"])
            .arg(path)
            .status()?;
        match (tracked.status.code(), diff.code()) {
            (Some(0), Some(0)) => {}
            (Some(0 | 1), Some(0 | 1)) => changed.push(path.clone()),
            _ => bail!("Could not check {}", path.display()),
        }
    }
    Ok(changed)
}

fn check_pending_commits(store: &Path, paths: &[PathBuf]) -> anyhow::Result<()> {
    let commits = String::from_utf8(output(store, ["rev-list", "FETCH_HEAD..HEAD"])?)?;
    for commit in commits.lines() {
        let subject = output(store, ["show", "-s", "--format=%s", commit])?;
        let files = output(
            store,
            [
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                "-z",
                commit,
            ],
        )?;
        if subject != b"docs(agents): sync project instructions\n"
            || files.is_empty()
            || files
                .split(|byte| *byte == 0)
                .filter(|path| !path.is_empty())
                .any(|path| {
                    !paths
                        .iter()
                        .any(|allowed| allowed.as_os_str().as_encoded_bytes() == path)
                })
        {
            bail!("claudemds has unpublished commits outside this project's automatic sync.");
        }
    }
    Ok(())
}

fn print_guidance(root: &Path, entry: &Path) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let dirs: Vec<_> = cwd
        .ancestors()
        .take_while(|dir| dir.starts_with(root))
        .collect();
    for dir in dirs.into_iter().rev() {
        for name in [OVERRIDE, CODEX] {
            let path = dir.join(name);
            if !fs::metadata(&path).is_ok_and(|metadata| metadata.is_file()) {
                continue;
            }
            let Ok(contents) = fs::read_to_string(&path) else {
                continue;
            };
            if contents.trim().is_empty() {
                continue;
            }
            let source = entry.join(path.strip_prefix(root)?);
            if fs::symlink_metadata(&source).is_ok_and(|metadata| metadata.is_file())
                && fs::read_link(&path).is_ok_and(|target| target == source)
            {
                println!(
                    "# Current apm instructions from {}\n\n{contents}",
                    path.display()
                );
            }
            break;
        }
    }
    Ok(())
}
