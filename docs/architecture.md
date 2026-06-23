# Architecture

## Overview

`apm` is a CLI package manager for Claude Code packages (skills, tools, MCP servers). It uses a **store + symlink** architecture: packages are cloned once into a central store, then symlinked into agent directories.

## Directory Layout

```
~/.config/apm/                                        # $XDG_CONFIG_HOME/apm
├── packages.toml                                     # declared packages (source of truth)
├── packages.lock                                     # pinned commits + timestamps
└── config.toml                                       # default agent, etc.

~/.local/share/apm/store/skills/<name>/               # $XDG_DATA_HOME/apm

~/.claude/skills/
└── <name> -> ~/.local/share/apm/store/skills/<name>  # symlink
```

## Module Map

```
src/
├── main.rs             # CLI entry: Commands enum, dispatch
├── config.rs           # Packages / SkillEntry / ToolEntry, path helpers
├── lockfile.rs         # Lockfile / LockEntry, load/save
├── git.rs              # parse_source(), clone(), pull(), current_commit()
├── package/
│   ├── mod.rs          # Package trait
│   ├── skill.rs        # Skill implements Package
│   └── tool.rs         # Tool implements Package
└── commands/
    ├── mod.rs
    ├── skill/          # add, install, uninstall, remove, update, list
    └── tool/           # add, install, uninstall, remove, update, list
```

## Data Flow

### `apm skill add user/repo`

1. `parse_source()` → `github:user/repo` + derived name
2. Conflict check against `packages.toml`
3. `git clone --depth=1` into `~/.apm/store/skills/<name>`
4. Write `packages.toml` + `packages.lock` (only on success)

### `apm skill install`

1. Read `packages.toml`
2. For each skill: `symlink(store_path, link_dir)`
3. `link_dir` is determined by `Agent` variant (currently only `Claude`)

### `apm skill update`

1. `git pull --ff-only` in store dir
2. Update commit hash in `packages.lock`
3. Symlinks point at the directory, so they pick up the new commit automatically

## Package Trait

```rust
pub trait Package {
    fn store_path(&self, name: &str) -> PathBuf;
    fn install(&self, name: &str) -> anyhow::Result<()>;
    fn is_installed(&self, name: &str) -> bool;
    fn uninstall(&self, name: &str) -> anyhow::Result<()>;
}
```

`Skill` and `Tool` both implement this. The command layer is identical between the two; only `store_path()` and `link_dir()` differ.

## Adding a New Package Type (e.g. MCP)

1. Add `McpEntry` to `config.rs`, add `mcps` field to `Packages`
2. Add `mcps` field to `Lockfile`
3. Create `src/package/mcp.rs` implementing `Package`
4. Mirror `src/commands/tool/` → `src/commands/mcp/`
5. Add `Mcp` variant to `Commands` in `main.rs`
