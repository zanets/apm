# apm

A package manager for [Claude Code](https://claude.ai/code) — install and manage skills, tools, and MCP servers from GitHub.

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

| Type | What it is | Where it lives |
|------|-----------|----------------|
| **Skill** | A slash command (e.g. `/review`) | `~/.claude/skills/` |
| **Tool** | A Claude tool definition | `~/.claude/tools/` |
| **MCP** | An MCP server process | registered via `claude` CLI |

apm uses a **store + symlink** model: packages are cloned once into `~/.apm/store/` and symlinked into agent directories. `update` is a single `git pull`; the symlink picks it up automatically.

```
~/.apm/
├── packages.toml       # declared packages
├── packages.lock       # pinned commits
└── store/
    ├── skills/<name>/
    └── tools/<name>/

~/.claude/
├── skills/<name> -> ~/.apm/store/skills/<name>
└── tools/<name>  -> ~/.apm/store/tools/<name>
```

## Commands

### Skills

```bash
apm skill add user/repo              # clone & register
apm skill add user/repo --ref dev    # pin to a branch or tag
apm skill add user/repo --name foo   # override the skill name

apm skill enable                     # symlink all skills into ~/.claude/skills/
apm skill enable <name>              # enable one skill
apm skill disable [name]             # remove symlink, keep store

apm skill update                     # git pull all skills
apm skill update <name>              # update one skill

apm skill remove <name>              # disable + delete from store & packages.toml
apm skill list                       # show status of all skills
```

### Tools

```bash
apm tool add user/repo               # same flags as skill add
apm tool enable [name]
apm tool disable [name]
apm tool update [name]
apm tool remove <name>
apm tool list
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
| `~/.apm/packages.toml` | Declared packages — source of truth |
| `~/.apm/packages.lock` | Pinned commit hashes and timestapms |
| `~/.apm/store/` | Git clones |
