# apm

A local manager for [Claude Code](https://claude.ai/code) project `CLAUDE.md` files — save, restore, and share them across clones and machines.

## Install

**Homebrew (recommended)**

```bash
brew tap zanets/tap
brew trust --tap zanets/tap
brew install apm
```

**From source**

```bash
cargo install --path .
```

## Concepts

When working on a shared repo you can't commit a personal `CLAUDE.md` — but writing one every time you re-clone is tedious. `apm` stores it locally and restores it automatically, keyed by git remote URL so it follows the repo regardless of where it's cloned.

```
~/.local/share/apm/claudemds/        # $XDG_DATA_HOME/apm
├── <key>/                           # keyed by git remote URL
└── file/
    └── <encoded-path>/              # keyed by absolute file path
```

## Commands

```bash
apm new                    # create a new CLAUDE.md in the current directory, store and symlink immediately
apm save                   # move CLAUDE.md into store, symlink in place
apm save -p                # scan repo and confirm each file interactively
apm save -f <path>         # save a standalone CLAUDE.md by absolute path (no git needed)
apm restore                # recreate all symlinks after a re-clone
apm list                   # show all stored CLAUDE.md files
apm list -u                # show unmanaged CLAUDE.md files in the current git repo
apm remove <key>           # remove from store and clean up symlinks
apm env                    # print apm storage paths
```

Files are symlinked rather than copied — edits to `CLAUDE.md` write directly to the store, no need to re-save.

Claude Code reads `CLAUDE.md` from subdirectories too, so `save`/`restore` handle the full tree:

```
project/
├── CLAUDE.md          → ~/.local/share/apm/claudemds/<key>/CLAUDE.md
└── src/
    └── CLAUDE.md      → ~/.local/share/apm/claudemds/<key>/src/CLAUDE.md
```

The key is derived from the git remote URL (`https://github.com/org/repo` → `github.com_org_repo`), so the same store entry is found regardless of where the repo is cloned.

**Use case: global `~/CLAUDE.md`**

Claude Code reads `~/CLAUDE.md` as global instructions — but the home directory is not a git repo. Use `save -f` to manage it without git:

```bash
# Back it up and symlink it in place
apm save -f ~/CLAUDE.md

# It now appears in the store, keyed by its absolute path
apm list
#   /Users/you/CLAUDE.md              1.2K bytes

# Discover it when not inside a git repo (walks up from cwd to $HOME)
apm list -u

# Remove from store and unlink
apm remove ~/CLAUDE.md
```

**Use case: syncing the store across machines**

The store itself can be tracked as a git repo, so the same CLAUDE.md files follow you to other machines:

```bash
apm store init            # git init the store (idempotent)
apm store sync            # add -A, commit, pull --rebase, push
apm store sync -m "msg"   # use a custom commit message
```

`sync` stages and commits any local changes, then runs `git pull --rebase` before pushing so changes made on other machines are picked up first. If the rebase hits a conflict, it stops and leaves the repo mid-rebase for you to resolve manually (`cd` into the store, fix the file, `git add <file>`, `git rebase --continue`), then re-run `apm store sync`. If no remote is configured yet, `sync` commits locally and skips the pull/push step.

## Design

**XDG Base Directory compliance.** Data lives in `$XDG_DATA_HOME/apm` (`~/.local/share/apm`), respecting the environment variable so it works with non-standard home layouts.

**Symlinks over copies.** The authoritative `CLAUDE.md` lives in the store; the symlink is just a pointer. Edits write directly to the store, no sync step needed.

**CLAUDE.md keyed by remote URL.** Instead of tracking the local path, apm derives a key from `git remote get-url origin` (`https://github.com/org/repo` → `github.com_org_repo`). This means the store entry survives re-clones to a different path and works across machines that share the same remote.

**`git ls-files` for discovery.** Scanning for unmanaged CLAUDE.md files (`apm save -p`, `apm list -u`) delegates to `git ls-files` rather than walking the filesystem. This keeps discovery fast in large repos and respects `.gitignore` for free.

**Standalone path-keyed entries.** `save -f` accepts any CLAUDE.md by absolute path, keying the store entry on that path rather than a git remote. This lets apm manage files that live outside any repo — most usefully `~/CLAUDE.md`, Claude Code's global instruction file.

**Store sync is a thin git wrapper.** `apm store sync` is `git add -A` + commit + `git pull --rebase` + push against the claudemds store — no custom merge logic. Conflicts surface as real git conflicts for you to resolve, not something apm tries to auto-merge.

## Files

| Path | Purpose |
|------|---------|
| `~/.local/share/apm/claudemds/` | Stored CLAUDE.md files — git-keyed under `<key>/`, path-keyed under `file/` |
