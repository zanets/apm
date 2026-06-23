# amp

A package manager for [Claude Code](https://claude.ai/code) — install and manage skills, tools, and MCP servers from GitHub.

## Install

```bash
cargo install --path .
```

## Concepts

| Type | What it is | Where it lives |
|------|-----------|----------------|
| **Skill** | A slash command (e.g. `/review`) | `~/.claude/skills/` |
| **Tool** | A Claude tool definition | `~/.claude/tools/` |
| **MCP** | An MCP server process | registered via `claude` CLI |

amp uses a **store + symlink** model: packages are cloned once into `~/.amp/store/` and symlinked into agent directories. `update` is a single `git pull`; the symlink picks it up automatically.

```
~/.amp/
├── packages.toml       # declared packages
├── packages.lock       # pinned commits
└── store/
    ├── skills/<name>/
    └── tools/<name>/

~/.claude/
├── skills/<name> -> ~/.amp/store/skills/<name>
└── tools/<name>  -> ~/.amp/store/tools/<name>
```

## Commands

### Skills

```bash
amp skill add user/repo              # clone & register
amp skill add user/repo --ref dev    # pin to a branch or tag
amp skill add user/repo --name foo   # override the skill name

amp skill enable                     # symlink all skills into ~/.claude/skills/
amp skill enable <name>              # enable one skill
amp skill disable [name]             # remove symlink, keep store

amp skill update                     # git pull all skills
amp skill update <name>              # update one skill

amp skill remove <name>              # disable + delete from store & packages.toml
amp skill list                       # show status of all skills
```

### Tools

```bash
amp tool add user/repo               # same flags as skill add
amp tool enable [name]
amp tool disable [name]
amp tool update [name]
amp tool remove <name>
amp tool list
```

### MCP Servers

```bash
amp mcp add <name> <command> [args]  # register & add to Claude
amp mcp remove <name>               # deregister from Claude
amp mcp list                         # list registered servers
```

MCP management delegates to the `claude` CLI (`claude mcp add/remove`).

## Source format

`user/repo` and `github:user/repo` are both accepted. A bare name (no slash) is rejected.

## Files

| Path | Purpose |
|------|---------|
| `~/.amp/packages.toml` | Declared packages — source of truth |
| `~/.amp/packages.lock` | Pinned commit hashes and timestamps |
| `~/.amp/store/` | Git clones |
