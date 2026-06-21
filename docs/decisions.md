# Design Decisions

## Store + Symlink (Homebrew pattern)

**Decision:** Clone packages into `~/.amp/store/` and symlink into agent directories, rather than cloning directly into `~/.claude/skills/`.

**Why:** Separates amp's managed state from Claude's runtime directories. Consequences:
- `update` only needs `git pull` in one place; symlinks pick it up automatically
- Multiple agents can share the same store copy
- Easy to inspect what amp manages without touching agent config

---

## `add` = register + clone (merged)

**Decision:** `amp skill add` both writes `packages.toml` and clones to store in one step. There is no separate `get` command.

**Why:** The two-step workflow (`add` then `get`) added friction with no benefit for the common case. If the clone fails, `packages.toml` is not written (clone-first, write-on-success), so there is no orphaned entry.

---

## Nested subcommands (`amp skill <cmd>`, `amp tool <cmd>`)

**Decision:** Use `amp skill add` / `amp tool add` instead of flat `amp add --type skill`.

**Why:** Anticipates `amp mcp <cmd>` without a breaking CLI change. Each package type can have type-specific flags without polluting a shared namespace.

---

## Shell-out to git, not libgit2

**Decision:** Use `std::process::Command` to call `git` for clone/pull/rev-parse.

**Why:** libgit2 (via `git2` crate) adds significant compile weight and complexity. The operations needed (shallow clone, ff-only pull, rev-parse HEAD) are simple and well-handled by the system git. Users who have amp installed will have git installed.

---

## `parse_source` accepts `user/repo` shorthand

**Decision:** `github:user/repo` is canonical, but `user/repo` is accepted and inferred. A bare `name` (no slash) is rejected with a helpful error.

**Why:** `user/repo` is the format users naturally copy from GitHub. Requiring `github:` prefix every time is needless friction. A bare name is ambiguous (which GitHub user?), so it errors rather than guessing.

---

## Clone-first, write-on-success

**Decision:** In `add`, clone to store before writing `packages.toml` or `packages.lock`.

**Why:** If the clone fails (bad URL, no network, private repo), the package must not appear in `packages.toml`. An orphaned entry with `not in store` status is confusing and hard to clean up without a separate command.

---

## `Agent` enum for future multi-agent support

**Decision:** `Skill` and `Tool` hold an `Agent` enum that controls `link_dir()`.

**Why:** Enables installing the same package into different agents (Cursor, Windsurf, etc.) by adding variants to `Agent` and updating `link_dir()`, without changing the command layer. Install/uninstall commands will take a `--agent` flag (defaulting to a config value) when a second agent is supported.
