# Cross-device rename error in `apm md save`

**Date**: 2026-06-24
**Affected file**: `src/commands/claudemd/save.rs`
**Platform**: Linux

## Core Concept

Cross-device `rename(2)` — `rename` is atomic only within one filesystem; cross-device moves require copy + unlink.

## Root Cause (ELI5)

Linux 的 `rename()` syscall 只能在同一個 filesystem 內移動檔案。如果 CLAUDE.md 所在的目錄和 apm store（`~/.local/share/apm/claudemds/`）位於不同裝置（NFS、tmpfs、加掛磁碟等），kernel 會直接回傳 `EXDEV`（os error 18）並拒絕執行。

## Fix

Added a `move_file` helper that falls back to `copy` + `remove_file` when `rename` fails with `ErrorKind::CrossesDevices`:

```rust
fn move_file(src: &Path, dst: &Path) -> io::Result<()> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            std::fs::copy(src, dst)?;
            std::fs::remove_file(src)
        }
        Err(e) => Err(e),
    }
}
```

Replaced both `std::fs::rename(...)` calls in `save_standalone` (line 38) and `symlink_into_store` (line 102) with `move_file(...)`.
