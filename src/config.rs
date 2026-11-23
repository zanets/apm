use std::path::PathBuf;

/// $XDG_CONFIG_HOME/apm
pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().expect("no home dir").join(".config"))
        .join("apm")
}

/// $XDG_DATA_HOME/apm — claudemds/
pub fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().expect("no home dir").join(".local/share"))
        .join("apm")
}

pub fn store_dir() -> PathBuf {
    data_dir().join("claudemds")
}

/// Resolve the active Codex profile for the current machine.
pub fn codex_home() -> std::io::Result<PathBuf> {
    let home = std::env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().expect("no home dir").join(".codex"));
    match home.canonicalize() {
        Ok(home) => Ok(home),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => std::path::absolute(home),
        Err(err) => Err(err),
    }
}
