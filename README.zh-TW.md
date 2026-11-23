# apm

管理 Codex 的本機指令檔，方便跨 clone 和跨機器重用。apm 將檔案存入中央 store，並在原位置建立 symlink。透過 symlink 編輯檔案會直接更新 store。

## 安裝

```sh
brew tap zanets/tap
brew trust --tap zanets/tap
brew install apm
```

從原始碼安裝可執行 `cargo install --path .`。

## 專案指令檔

```sh
apm new                 # 在目前目錄建立並納管 AGENTS.md
apm save                # 儲存未追蹤的 AGENTS.md 和 AGENTS.override.md
apm save -p             # 逐一確認再存入
apm list -u             # 顯示目前 repo 中未納管的 Codex 指令檔
apm restore             # 還原目前 clone 和獨立檔案的 symlink
apm list                # 列出已儲存的項目及其 key
apm remove <key>        # 刪除整個 store 項目，並移除對應的本機 symlink
apm env                 # 顯示儲存路徑
```

apm 以 Git 的 `origin` URL 推導 repo key。掃描涵蓋整個 repo，包含被 Git 忽略的檔案，並略過已追蹤檔案與 symlink。個人 symlink 可透過 `.git/info/exclude` 忽略。還原會略過 Git 已追蹤的路徑和任何已存在的檔案或 symlink。

## Codex 全域指令

用 `save -f` 納管 Codex 家目錄根層的 `AGENTS.md` 或 `AGENTS.override.md`。apm 會使用 `CODEX_HOME`，未設定時使用 `$HOME/.codex`。這類項目可跨機器還原。

```sh
apm save -f "${CODEX_HOME:-$HOME/.codex}/AGENTS.md"
```

另一台機器取得同一份 store 後，執行 `apm restore`，apm 會把指令連到那台機器的 Codex 家目錄。`apm list` 會顯示目的地，以及供 `apm remove` 使用的固定 key，例如 `codex:AGENTS.md`。其他獨立檔案和既有的絕對路徑項目仍會還原到原路徑。

## 從 Claude Code 遷移

舊的 `CLAUDE.md` 仍會出現在 `apm list`，也能以原檔名還原。請在 repo 內先預覽，再指定要匯入的檔案。

```sh
apm migrate
apm migrate --apply src/CLAUDE.md
```

先檢查來源內的 Claude 專用指令與設定。遷移會把指定的檔案複製到同層的 `AGENTS.md`，略過 Git 已追蹤或已存在的目標，並保留舊檔。匯入後可直接編輯 symlink 指向的 `AGENTS.md`，調整成 Codex 的工作方式。

若要遷移全域指令，請檢查舊檔，將調整後的內容寫入 `${CODEX_HOME:-$HOME/.codex}/AGENTS.md`，再用上述 `apm save -f` 納管。

## 同步 store

store 沿用 `${XDG_DATA_HOME:-$HOME/.local/share}/apm/claudemds/`，既有 Git 歷史與 remote 可繼續使用。

```sh
apm store init
apm store sync           # 暫存變更、commit、pull --rebase、push
apm store sync -m "更新專案指令"
```

沒有 `origin` 時只會在本機 commit。發生 rebase 衝突時，進入 `apm env` 顯示的 store 路徑處理，完成 rebase 後再執行 `apm store sync`。

## 在 Codex 自動同步

在 macOS 或 Linux 的 apm checkout 執行下列指令。需要 Python 3、Rust、Git，以及對 store 的 Git 讀寫權限。

```sh
python3 scripts/install-codex-hooks.py --store-url git@github.com:zanets/claudemds.git
```

安裝程式會建置 apm、在需要時 clone store，並根據本機路徑加入兩個 hooks，保留既有 hooks。請重新開啟 Codex，用 `/hooks` 審查並信任新設定。

啟動時會執行 `apm store project pull`，並將已納管的指令提供給本次工作階段。回合結束時，`apm store project push` 只會提交目前 repo 已連入且有修改的指令。Git 已追蹤的專案檔案、獨立項目和其他 store 檔案不會進入這些自動提交。同步使用 fast-forward，遇到歷史分岔或推送錯誤會回報。
