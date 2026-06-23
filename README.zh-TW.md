# apm

[Claude Code](https://claude.ai/code) 的套件管理器 — 從 GitHub 安裝與管理 skills、tools 及 MCP servers。

## 安裝

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

| 類型 | 說明 | 存放位置 |
|------|------|---------|
| **Skill** | 斜線指令（如 `/review`） | `~/.claude/skills/` |
| **Tool** | Claude 工具定義 | `~/.claude/tools/` |
| **MCP** | MCP server 程序 | 透過 `claude` CLI 註冊 |

apm 採用 **store + symlink** 架構：套件統一 clone 到 `~/.apm/store/`，再以 symlink 連結到 agent 目錄。`update` 只需執行一次 `git pull`，symlink 自動生效。

```
~/.apm/
├── packages.toml       # 宣告的套件清單
├── packages.lock       # 鎖定的 commit 版本
└── store/
    ├── skills/<name>/
    └── tools/<name>/

~/.claude/
├── skills/<name> -> ~/.apm/store/skills/<name>
└── tools/<name>  -> ~/.apm/store/tools/<name>
```

## 指令

### Skills

```bash
apm skill add user/repo              # clone 並註冊
apm skill add user/repo --ref dev    # 鎖定特定 branch 或 tag
apm skill add user/repo --name foo   # 自訂 skill 名稱

apm skill enable                     # 將所有 skills symlink 到 ~/.claude/skills/
apm skill enable <name>              # 啟用單一 skill
apm skill disable [name]             # 移除 symlink，保留 store

apm skill update                     # git pull 所有 skills
apm skill update <name>              # 更新單一 skill

apm skill remove <name>              # 停用並從 store 及 packages.toml 刪除
apm skill list                       # 顯示所有 skills 的狀態
```

### Tools

```bash
apm tool add user/repo               # 參數與 skill add 相同
apm tool enable [name]
apm tool disable [name]
apm tool update [name]
apm tool remove <name>
apm tool list
```

### MCP Servers

```bash
apm mcp add <name> <command> [args]  # 註冊並加入 Claude
apm mcp remove <name>               # 從 Claude 移除
apm mcp list                         # 列出已註冊的 servers
```

MCP 管理委派給 `claude` CLI（`claude mcp add/remove`）。

## 來源格式

`user/repo` 與 `github:user/repo` 皆可接受。不含斜線的純名稱會被拒絕並提示錯誤。

## 檔案說明

| 路徑 | 用途 |
|------|------|
| `~/.apm/packages.toml` | 宣告的套件清單，唯一事實來源 |
| `~/.apm/packages.lock` | 鎖定的 commit hash 與時間戳記 |
| `~/.apm/store/` | Git clone 存放目錄 |
