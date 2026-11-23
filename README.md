# apm

Manage local Codex instruction files across clones and machines. apm stores each file centrally and links it back into place, so editing the linked file updates the store.

[繁體中文](README.zh-TW.md)

## Install

```sh
brew tap zanets/tap
brew trust --tap zanets/tap
brew install apm
```

From source, run `cargo install --path .`.

## Project instructions

```sh
apm new                 # Create and manage an AGENTS.md in the current directory
apm save                # Save untracked AGENTS.md and AGENTS.override.md files
apm save -p             # Confirm each file before saving
apm list -u             # Show unmanaged Codex instructions in the current repo
apm restore             # Recreate links in the current clone and for standalone entries
apm list                # Show saved entries and their keys
apm remove <key>        # Delete the entire saved entry and unlink matching local links
apm env                 # Show storage paths
```

Project commands that access the store use the Git `origin` URL as the repository key. The scan covers the repository, including ignored files, and skips Git-tracked files and symlinks. Use a local ignore rule such as `.git/info/exclude` for personal links you want to keep out of commits. Restore skips Git-tracked paths and any existing file or symlink.

## Global Codex instructions

Use `save -f` to store an existing `AGENTS.md` or `AGENTS.override.md` from the root of your Codex home. apm uses the configured `CODEX_HOME` or defaults to `$HOME/.codex`. The entry follows that location across machines.

```sh
apm save -f "${CODEX_HOME:-$HOME/.codex}/AGENTS.md"
```

On another machine with the same store, run `apm restore` to link the file into its active Codex home. `apm list` shows the destination and a stable key such as `codex:AGENTS.md` for `apm remove`. Other standalone files and existing absolute-path entries still restore to their original paths.

## Migrate from Claude Code

Existing `CLAUDE.md` entries still appear in `apm list` and can be restored under their original names. From the repository, preview saved project files and select the ones to import.

```sh
apm migrate
apm migrate --apply src/CLAUDE.md
```

Review the selected files for Claude-specific commands and settings. Migration copies the selected content into `AGENTS.md` at the same directory level and skips any tracked or existing destination. The original remains in the store. After import, edit the linked `AGENTS.md` to adapt its instructions for Codex.

For global instructions, review the old file and put the adapted content in `${CODEX_HOME:-$HOME/.codex}/AGENTS.md`, then use `apm save -f` as above.

## Sync the store

The store remains at `${XDG_DATA_HOME:-$HOME/.local/share}/apm/claudemds/`. Existing Git history and remotes continue to work.

```sh
apm store init
apm store sync           # Stage, commit, pull --rebase, and push
apm store sync -m "Update project instructions"
```

Without an `origin` remote, sync commits locally. If a rebase conflicts, resolve it in the path shown by `apm env`, finish the rebase, and rerun `apm store sync`.
