# amp

[Claude Code](https://claude.ai/code) 的套件管理器 — 從 GitHub 安裝與管理 skills、tools 及 MCP servers。

## 安裝

```bash
cargo install --path .
```

## 概念

| 類型 | 說明 | 存放位置 |
|------|------|---------|
| **Skill** | 斜線指令（如 `/review`） | `~/.claude/skills/` |
| **Tool** | Claude 工具定義 | `~/.claude/tools/` |
| **MCP** | MCP server 程序 | 透過 `claude` CLI 註冊 |

amp 採用 **store + symlink** 架構：套件統一 clone 到 `~/.amp/store/`，再以 symlink 連結到 agent 目錄。`update` 只需執行一次 `git pull`，symlink 自動生效。

```
~/.amp/
├── packages.toml       # 宣告的套件清單
├── packages.lock       # 鎖定的 commit 版本
└── store/
    ├── skills/<name>/
    └── tools/<name>/

~/.claude/
├── skills/<name> -> ~/.amp/store/skills/<name>
└── tools/<name>  -> ~/.amp/store/tools/<name>
```

## 指令

### Skills

```bash
amp skill add user/repo              # clone 並註冊
amp skill add user/repo --ref dev    # 鎖定特定 branch 或 tag
amp skill add user/repo --name foo   # 自訂 skill 名稱

amp skill enable                     # 將所有 skills symlink 到 ~/.claude/skills/
amp skill enable <name>              # 啟用單一 skill
amp skill disable [name]             # 移除 symlink，保留 store

amp skill update                     # git pull 所有 skills
amp skill update <name>              # 更新單一 skill

amp skill remove <name>              # 停用並從 store 及 packages.toml 刪除
amp skill list                       # 顯示所有 skills 的狀態
```

### Tools

```bash
amp tool add user/repo               # 參數與 skill add 相同
amp tool enable [name]
amp tool disable [name]
amp tool update [name]
amp tool remove <name>
amp tool list
```

### MCP Servers

```bash
amp mcp add <name> <command> [args]  # 註冊並加入 Claude
amp mcp remove <name>               # 從 Claude 移除
amp mcp list                         # 列出已註冊的 servers
```

MCP 管理委派給 `claude` CLI（`claude mcp add/remove`）。

## 來源格式

`user/repo` 與 `github:user/repo` 皆可接受。不含斜線的純名稱會被拒絕並提示錯誤。

## 檔案說明

| 路徑 | 用途 |
|------|------|
| `~/.amp/packages.toml` | 宣告的套件清單，唯一事實來源 |
| `~/.amp/packages.lock` | 鎖定的 commit hash 與時間戳記 |
| `~/.amp/store/` | Git clone 存放目錄 |
