# apm

[Claude Code](https://claude.ai/code) 專案 `CLAUDE.md` 檔案的本機管理工具 — 儲存、還原，並跨 clone、跨機器共用。

## 安裝

**Homebrew（建議）**

```bash
brew tap zanets/tap
brew trust --tap zanets/tap
brew install apm
```

**從原始碼編譯**

```bash
cargo install --path .
```

## 概念

在共用 repo 上工作時，個人的 `CLAUDE.md` 無法 commit — 但每次 re-clone 都要重寫也很麻煩。`apm` 將它存在本機，並以 git remote URL 為 key，讓它跟著 repo 走，不管 clone 到哪裡都能還原。

```
~/.local/share/apm/claudemds/        # $XDG_DATA_HOME/apm
├── <key>/                           # 以 git remote URL 為 key
└── file/
    └── <encoded-path>/              # 以絕對路徑為 key
```

## 指令

```bash
apm new                    # 在當前目錄建立新的 CLAUDE.md，自動存入 store 並建立 symlink
apm save                   # 將 CLAUDE.md 移入 store，原位置建立 symlink
apm save -p                # 掃描 repo，逐一確認每個檔案是否存入
apm save -f <path>         # 以絕對路徑儲存獨立的 CLAUDE.md（不需要 git）
apm restore                # re-clone 後重建所有 symlink
apm list                   # 顯示所有已儲存的 CLAUDE.md
apm list -u                # 顯示目前 git repo 中未納管的 CLAUDE.md
apm remove <key>           # 從 store 刪除並清除對應 symlink
apm env                    # 顯示 apm 的儲存路徑
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
apm save -f ~/CLAUDE.md

# 現在在 store 中可見，key 為絕對路徑
apm list
#   /Users/you/CLAUDE.md              1.2K bytes

# 不在 git repo 內時也能找到（從 cwd 往上找到 $HOME）
apm list -u

# 從 store 刪除並移除 symlink
apm remove ~/CLAUDE.md
```

**使用情境：跨機器同步 store**

store 本身可以用 git 追蹤，讓同一份 CLAUDE.md 檔案跟著你到其他機器：

```bash
apm store init            # git init store（idempotent）
apm store sync            # add -A、commit、pull --rebase、push
apm store sync -m "msg"   # 自訂 commit 訊息
```

`sync` 會先 stage 並 commit 本機變更，接著在 push 前跑 `git pull --rebase`，把其他機器上的變更先拉下來。如果 rebase 遇到衝突，會直接中止並停在 rebase 進行中的狀態讓你手動解決（`cd` 進 store、修檔案、`git add <file>`、`git rebase --continue`），解決完再重跑 `apm store sync`。如果還沒設定 remote，`sync` 只會在本機 commit，並跳過 pull/push。

## 設計重點

**遵循 XDG Base Directory 規範。** 資料存放於 `$XDG_DATA_HOME/apm`（預設 `~/.local/share/apm`），尊重環境變數，非標準 home 配置也能正常運作。

**Symlink 而非複製。** 實際檔案存在 store 裡，symlink 只是指標。編輯直接寫入 store，不需要額外同步步驟。

**CLAUDE.md 以 remote URL 為 key。** apm 不追蹤本機路徑，而是從 `git remote get-url origin` 推導 key（`https://github.com/org/repo` → `github.com_org_repo`）。這讓 store 記錄在 re-clone 到不同路徑後仍然有效，跨機器共用同一個 remote 也能找到對應記錄。

**用 `git ls-files` 做搜尋。** 掃描未納管的 CLAUDE.md（`apm save -p`、`apm list -u`）委派給 `git ls-files`，而非遞迴走訪檔案系統。在大型 repo 中速度更快，且自動尊重 `.gitignore`。

**路徑為 key 的獨立項目。** `save -f` 接受任意絕對路徑的 CLAUDE.md，以該路徑作為 store 的 key，而非 git remote。這讓 apm 能管理 repo 以外的檔案 — 最典型的就是 `~/CLAUDE.md`，即 Claude Code 的全域指令檔。

**Store sync 只是薄薄一層 git 包裝。** `apm store sync` 就是對 claudemds store 執行 `git add -A` + commit + `git pull --rebase` + push，沒有自訂的合併邏輯。衝突會如實呈現為 git conflict 讓你自己解決，apm 不會嘗試自動合併。

## 檔案說明

| 路徑 | 用途 |
|------|------|
| `~/.local/share/apm/claudemds/` | 儲存的 CLAUDE.md 檔案 — git 項目在 `<key>/`，路徑項目在 `file/` |
