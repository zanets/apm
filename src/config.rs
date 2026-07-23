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

pub fn claudemds_dir() -> PathBuf {
    data_dir().join("claudemds")
}
