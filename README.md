# apm

A package manager for [Claude Code](https://claude.ai/code) — install and manage skills and MCP servers from GitHub.

## Install

**Homebrew (recommended)**

```bash
brew tap zanets/apm
brew install apm
```

**From source**

```bash
cargo install --path .
```

## Concepts

Packages are slash commands (`.md` files) installed into `~/.claude/skills/`. apm clones them from GitHub into a local store and symlinks them into agent directories.

```
~/.config/apm/                     # $XDG_CONFIG_HOME/apm
├── packages.toml                  # declared packages
└── packages.lock                  # pinned commits

~/.local/share/apm/store/skills/   # $XDG_DATA_HOME/apm
└── <name>/

~/.claude/skills/
└── <name> -> ~/.local/share/apm/store/skills/<name>
```

## Commands

### Packages

```bash
apm add user/repo              # clone & register
apm add user/repo --ref dev    # pin to a branch or tag
apm add user/repo --name foo   # override the package name

apm enable                     # symlink all packages into ~/.claude/skills/
apm enable <name>              # enable one package
apm disable [name]             # remove symlink, keep store

apm update                     # git pull all packages
apm update <name>              # update one package

apm remove <name>              # disable + delete from store & packages.toml
apm list                       # show status of all packages
```

### MCP Servers

```bash
apm mcp add <name> <command> [args]  # register & add to Claude
apm mcp remove <name>               # deregister from Claude
apm mcp list                         # list registered servers
```

MCP management delegates to the `claude` CLI (`claude mcp add/remove`).

## Source format

`user/repo` and `github:user/repo` are both accepted. A bare name (no slash) is rejected.

## Files

| Path | Purpose |
|------|---------|
| `~/.config/apm/packages.toml` | Declared packages — source of truth |
| `~/.config/apm/packages.lock` | Pinned commit hashes and timestamps |
| `~/.local/share/apm/store/` | Git clones |
