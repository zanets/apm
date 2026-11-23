# apm

apm is a Rust CLI for storing and restoring local Codex instruction files. It preserves access to legacy Claude Code entries in the same store.

- Run `cargo test --locked` to verify changes. Use a temporary Git repository and an isolated `XDG_DATA_HOME` when testing file moves, symlinks, and store commands.
- Start at `src/main.rs` for CLI dispatch, `src/commands/instructions/` for instruction files, and `src/commands/store/` for Git sync.
- Keep repository-relative paths and existing store keys compatible. Preserve Git-tracked instructions and existing files or links during restore and migration.
- See `docs/agents/issue-tracker.md` and `docs/agents/triage-labels.md` for the GitHub issue workflow.
