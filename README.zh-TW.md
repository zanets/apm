# apm

[Claude Code](https://claude.ai/code) 的套件管理器 — 安裝與管理 skills、MCP servers 及專案 CLAUDE.md 檔案。

## 安裝

**Homebrew（建議）**

```bash
brew tap zanets/apm
brew trust --tap zanets/apm
brew install apm
```

**從原始碼編譯**

```bash
cargo install --path .
```

## 概念

apm 管理三種東西：

- **Skills** — 從 GitHub clone 的斜線指令（`.md` 檔），symlink 到 `~/.claude/skills/`
- **MCP servers** — 透過 `claude` CLI 註冊，追蹤於 `packages.toml`
- **CLAUDE.md 檔案** — 專案層級的行為規則，本機儲存並 symlink 到各個 repo

```
~/.config/apm/                       # $XDG_CONFIG_HOME/apm
├── packages.toml                    # 宣告的套件與 MCP servers
└── packages.lock                    # 鎖定的 skill commit 版本

~/.local/share/apm/store/skills/     # $XDG_DATA_HOME/apm
└── <name>/

~/.local/share/apm/claudemds/        # 儲存的 CLAUDE.md 檔案
├── <key>/                           # 以 git remote URL 為 key
└── file/
    └── <encoded-path>/              # 以絕對路徑為 key

~/.claude/skills/
└── <name> -> ~/.local/share/apm/store/skills/<name>
```

## 指令

### 套件

```bash
apm add user/repo              # clone 並註冊
apm add user/repo --ref dev    # 鎖定特定 branch 或 tag
apm add user/repo --name foo   # 自訂套件名稱
```

`user/repo` 與 `github:user/repo` 皆可接受。不含斜線的純名稱會被拒絕並提示錯誤。

```bash
apm enable                     # 將所有套件 symlink 到 ~/.claude/skills/
apm enable <name>              # 啟用單一套件
apm disable                    # 移除所有 symlink，保留 store
apm disable <name>             # 移除單一套件的 symlink，保留 store

apm update                     # git pull 所有套件
apm update <name>              # 更新單一套件

apm remove <name>              # 停用並從 store 及 packages.toml 刪除
apm list                       # 顯示所有套件的狀態
```

### MCP Servers

```bash
apm mcp add <name> <command> [args]  # 註冊並加入 Claude
apm mcp remove <name>                # 從 Claude 移除
apm mcp list                         # 列出已註冊的 servers
```

MCP 管理委派給 `claude` CLI（`claude mcp add/remove`）。

### CLAUDE.md

在共用 repo 上工作時，個人的 `CLAUDE.md` 無法 commit — 但每次 re-clone 都要重寫也很麻煩。`apm md` 將它存在本機，並以 git remote URL 為 key，讓它跟著 repo 走，不管 clone 到哪裡都能還原。

```bash
apm md new                    # 在當前目錄建立新的 CLAUDE.md，自動存入 store 並建立 symlink
apm md save                   # 將 CLAUDE.md 移入 store，原位置建立 symlink
apm md save -p                # 掃描 repo，逐一確認每個檔案是否存入
apm md save -f <path>         # 以絕對路徑儲存獨立的 CLAUDE.md（不需要 git）
apm md restore                # re-clone 後重建所有 symlink
apm md list                   # 顯示所有已儲存的 CLAUDE.md
apm md list -u                # 顯示目前 git repo 中未納管的 CLAUDE.md
apm md remove <key>           # 從 store 刪除並清除對應 symlink
```

檔案以 symlink 而非複製的方式存放 — 直接編輯 `CLAUDE.md` 即寫入 store，不需要重新 save。

Claude Code 也會讀取子目錄的 `CLAUDE.md`，因此 `save`/`restore` 會處理整棵樹：

```
project/
├── CLAUDE.md          → ~/.local/share/apm/claudemds/<key>/CLAUDE.md
└── src/
    └── CLAUDE.md      → ~/.local/share/apm/claudemds/<key>/src/CLAUDE.md
```

Key 由 git remote URL 推導（`https://github.com/org/repo` → `github.com_org_repo`），因此不管 clone 到哪個路徑，都能找到同一份 store 記錄。

**使用情境：全域 `~/CLAUDE.md`**

Claude Code 會讀取 `~/CLAUDE.md` 作為全域指令 — 但家目錄不是 git repo。用 `save -f` 不需要 git 即可納管：

```bash
# 存入 store 並原位建立 symlink
apm md save -f ~/CLAUDE.md

# 現在在 store 中可見，key 為絕對路徑
apm md list
#   /Users/you/CLAUDE.md              1.2K bytes

# 不在 git repo 內時也能找到（從 cwd 往上找到 $HOME）
apm md list -u

# 從 store 刪除並移除 symlink
apm md remove ~/CLAUDE.md
```

## 設計重點

**遵循 XDG Base Directory 規範。** 設定檔存放於 `$XDG_CONFIG_HOME/apm`（預設 `~/.config/apm`），資料存放於 `$XDG_DATA_HOME/apm`（預設 `~/.local/share/apm`）。兩個路徑都尊重環境變數，非標準 home 配置也能正常運作。

**Symlink 而非複製。** Skills 和 CLAUDE.md 都以 symlink 方式連結，而非複製到目標位置。實際檔案存在 store 裡，symlink 只是指標。Skills 的更新透過 `git pull` 在 store 端完成；CLAUDE.md 的編輯也直接寫入 store，不需要額外同步步驟。

**CLAUDE.md 以 remote URL 為 key。** apm 不追蹤本機路徑，而是從 `git remote get-url origin` 推導 key（`https://github.com/org/repo` → `github.com_org_repo`）。這讓 store 記錄在 re-clone 到不同路徑後仍然有效，跨機器共用同一個 remote 也能找到對應記錄。

**用 `git ls-files` 做搜尋。** 掃描未納管的 CLAUDE.md（`apm md save -p`、`apm md list -u`）委派給 `git ls-files`，而非遞迴走訪檔案系統。在大型 repo 中速度更快，且自動尊重 `.gitignore`。

**路徑為 key 的獨立項目。** `save -f` 接受任意絕對路徑的 CLAUDE.md，以該路徑作為 store 的 key，而非 git remote。這讓 apm 能管理 repo 以外的檔案 — 最典型的就是 `~/CLAUDE.md`，即 Claude Code 的全域指令檔。

**MCP 委派給 claude CLI。** apm 不自行實作 MCP 註冊邏輯，直接委派給 `claude` CLI，只在 `packages.toml` 記錄設定以便重現。

## 檔案說明

| 路徑 | 用途 |
|------|------|
| `~/.config/apm/packages.toml` | 宣告的套件與 MCP servers，唯一事實來源 |
| `~/.config/apm/packages.lock` | 鎖定的 commit hash 與時間戳記 |
| `~/.local/share/apm/store/` | Git clone 存放目錄 |
| `~/.local/share/apm/claudemds/` | 儲存的 CLAUDE.md 檔案 — git 項目在 `<key>/`，路徑項目在 `file/` |
