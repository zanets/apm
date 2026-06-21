pub mod mcp;
pub mod skill;
pub mod tool;

use std::path::PathBuf;

pub trait Package {
    /// 這個 package 在 store 裡的路徑
    fn store_path(&self, name: &str) -> PathBuf;

    /// 把 store 裡的 package 裝進 agent（symlink、寫 config、等）
    fn install(&self, name: &str) -> anyhow::Result<()>;

    /// 是否已裝進 agent
    fn is_installed(&self, name: &str) -> bool;

    /// 從 agent 移除（拔 symlink），store 保留
    fn uninstall(&self, name: &str) -> anyhow::Result<()>;
}
