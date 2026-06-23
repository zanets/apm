# apm

[Claude Code](https://claude.ai/code) 的套件管理器 — 從 GitHub 安裝與管理 skills 及 MCP servers。

## 安��

**Homebrew（建議）**

```bash
brew tap zanets/apm
brew install apm
```

**從原始碼編譯**

```bash
cargo install --path .
```

## 概念

套件是安裝到 `~/.claude/skills/` 的斜線指令（`.md` 檔）。apm 從 GitHub clone 到本地 store，再以 symlink 連結到 agent 目錄。

```
~/.config/apm/                     # $XDG_CONFIG_HOME/apm
├── packages.toml                  # 宣告的套件清單
└── packages.lock                  # 鎖定的 commit 版本

~/.local/share/apm/store/skills/   # $XDG_DATA_HOME/apm
└── <name>/

~/.claude/skills/
└── <name> -> ~/.local/share/apm/store/skills/<name>
```

## 指令

### 套件

```bash
apm add user/repo              # clone 並註冊
apm add user/repo --ref dev    # 鎖定特定 branch 或 tag
apm add user/repo --name foo   # 自訂套件名稱

apm enable                     # 將所有套件 symlink 到 ~/.claude/skills/
apm enable <name>              # 啟用單一套件
apm disable [name]             # 移除 symlink，保留 store

apm update                     # git pull 所有套件
apm update <name>              # 更新單一套件

apm remove <name>              # 停用並從 store 及 packages.toml 刪除
apm list                       # 顯示所有套件的狀態
```

### MCP Servers

```bash
apm mcp add <name> <command> [args]  # 註冊並加入 Claude
apm mcp remove <name>               # 從 Claude 移除
apm mcp list                         # 列出已註冊的 servers
```

MCP 管理委派給 `claude` CLI���`claude mcp add/remove`）。

## 來源格式

`user/repo` 與 `github:user/repo` 皆可接受。不含斜線的純名稱會被拒絕並提示錯誤。

## 檔案說明

| 路徑 | 用途 |
|------|------|
| `~/.config/apm/packages.toml` | 宣告的套件清單，唯一事實來源 |
| `~/.config/apm/packages.lock` | 鎖定的 commit hash 與時間戳記 |
| `~/.local/share/apm/store/` | Git clone 存放目錄 |
