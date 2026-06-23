# apm

A package manager for [Claude Code](https://claude.ai/code) — install and manage skills, MCP servers, and project CLAUDE.md files.

## Install

**Homebrew (recommended)**

```bash
brew tap zanets/apm
brew trust --tap zanets/apm
brew install apm
```

**From source**

```bash
cargo install --path .
```

## Concepts

apm manages three kinds of things:

- **Skills** — slash commands (`.md` files) cloned from GitHub and symlinked into `~/.claude/skills/`
- **MCP servers** — registered via the `claude` CLI and tracked in `packages.toml`
- **CLAUDE.md files** — project-level behavior instructions stored locally and symlinked into repos

```
~/.config/apm/                       # $XDG_CONFIG_HOME/apm
├── packages.toml                    # declared packages & MCP servers
└── packages.lock                    # pinned skill commits

~/.local/share/apm/store/skills/     # $XDG_DATA_HOME/apm
└── <name>/

~/.local/share/apm/claudemds/        # stored CLAUDE.md files
├── <key>/                           # keyed by git remote URL
└── file/
    └── <encoded-path>/              # keyed by absolute file path

~/.claude/skills/
└── <name> -> ~/.local/share/apm/store/skills/<name>
```

## Commands

### Packages

```bash
apm add user/repo              # clone & register
apm add user/repo --ref dev    # pin to a branch or tag
apm add user/repo --name foo   # override the package name
```

`user/repo` and `github:user/repo` are both accepted. A bare name (no slash) is rejected.

```bash
apm enable                     # symlink all packages into ~/.claude/skills/
apm enable <name>              # enable one package
apm disable                    # remove all symlinks, keep store
apm disable <name>             # remove symlink for one package, keep store

apm update                     # git pull all packages
apm update <name>              # update one package

apm remove <name>              # disable + delete from store & packages.toml
apm list                       # show status of all packages
```

### MCP Servers

```bash
apm mcp add <name> <command> [args]  # register & add to Claude
apm mcp remove <name>                # deregister from Claude
apm mcp list                         # list registered servers
```

MCP management delegates to the `claude` CLI (`claude mcp add/remove`).

### CLAUDE.md

When working on a shared repo you can't commit a personal `CLAUDE.md` — but writing one every time you re-clone is tedious. `apm md` stores it locally and restores it automatically, keyed by git remote URL so it follows the repo regardless of where it's cloned.

```bash
apm md new                    # create a new CLAUDE.md, store and symlink immediately
apm md save                   # move CLAUDE.md into store, symlink in place
apm md save -p                # scan repo and confirm each file interactively
apm md save -f <path>         # save a standalone CLAUDE.md by absolute path (no git needed)
apm md restore                # recreate all symlinks after a re-clone
apm md list                   # show all stored CLAUDE.md files
apm md list -u                # show unmanaged CLAUDE.md files in the current git repo
apm md remove <key>           # remove from store and clean up symlinks
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
apm md save -f ~/CLAUDE.md

# It now appears in the store, keyed by its absolute path
apm md list
#   /Users/you/CLAUDE.md              1.2K bytes

# Discover it when not inside a git repo (walks up from cwd to $HOME)
apm md list -u

# Remove from store and unlink
apm md remove ~/CLAUDE.md
```

## Design

**XDG Base Directory compliance.** Config lives in `$XDG_CONFIG_HOME/apm` (`~/.config/apm`) and data in `$XDG_DATA_HOME/apm` (`~/.local/share/apm`). Both paths respect the environment variables, so they work with non-standard home layouts.

**Symlinks over copies.** Both skills and CLAUDE.md files are symlinked rather than copied into place. The authoritative file lives in the store; the symlink is just a pointer. Updates to skills happen at the store (via `git pull`), and edits to CLAUDE.md write directly to the store with no sync step.

**CLAUDE.md keyed by remote URL.** Instead of tracking the local path, apm derives a key from `git remote get-url origin` (`https://github.com/org/repo` → `github.com_org_repo`). This means the store entry survives re-clones to a different path and works across machines that share the same remote.

**`git ls-files` for discovery.** Scanning for unmanaged CLAUDE.md files (`apm md save -p`, `apm md list -u`) delegates to `git ls-files` rather than walking the filesystem. This keeps discovery fast in large repos and respects `.gitignore` for free.

**Standalone path-keyed entries.** `save -f` accepts any CLAUDE.md by absolute path, keying the store entry on that path rather than a git remote. This lets apm manage files that live outside any repo — most usefully `~/CLAUDE.md`, Claude Code's global instruction file.

**MCP delegation.** apm doesn't reimplement MCP registration logic — it delegates to the `claude` CLI and only tracks the config in `packages.toml` for reproducibility.

## Files

| Path | Purpose |
|------|---------|
| `~/.config/apm/packages.toml` | Declared packages & MCP servers — source of truth |
| `~/.config/apm/packages.lock` | Pinned commit hashes and timestamps |
| `~/.local/share/apm/store/` | Git clones |
| `~/.local/share/apm/claudemds/` | Stored CLAUDE.md files — git-keyed under `<key>/`, path-keyed under `file/` |
