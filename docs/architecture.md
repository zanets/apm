# Architecture

apm keeps local Codex instructions in a central store and links them back to their original locations. Changes made through a link edit the stored file directly.

- `src/main.rs` defines the CLI.
- `src/commands/instructions/` handles discovery, storage, linking, and migration.
- `src/config.rs` resolves XDG paths.
- `src/git.rs` wraps Git for discovery, remote keys, and sync.
- `src/commands/store/` initializes and syncs the store's Git repository.

## Store layout

The directory remains `${XDG_DATA_HOME:-$HOME/.local/share}/apm/claudemds/` so upgrades retain the store's Git history and existing links. Repository entries use the normalized `origin` key and keep the file's repository-relative path. Standalone entries normally use `file/<encoded-path>/` with a `.path` containing the original absolute path. Global Codex instructions use `file/.codex-home/<filename>/`. These entries resolve their destination against the active Codex home at restore time.

The project scan uses Git's untracked-file list, including ignored files, to find regular `AGENTS.md` and `AGENTS.override.md` files. Saving copies a file into the store without overwriting an existing entry. It stages the symlink on the working tree filesystem and renames it into place. This also works when the store is on another filesystem.

Restore recreates links for Codex and legacy files at their saved locations. It skips tracked paths and existing destinations. Project migration copies selected `CLAUDE.md` files to sibling `AGENTS.md` paths after checking for conflicts in the working tree and store.

## Design history

[Earlier planning notes](decisions.md) describe the original package-manager design.
