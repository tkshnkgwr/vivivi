# 設定ファイル・設定画面・永続カスタマイズ (Configuration)

`vivivi` は、ユーザーごとの編集スタイル、外観、キーマップ、ツール設定を `%USERPROFILE%\.vivivi\config.toml`（または `~/.vivivi/config.toml`）に永続保存し、起動時に自動ロードします。
また、GUI（egui）および TUI（Ratatui）の両方で直感的に操作できる **専用設定画面（Preferences Dialog）** を搭載しています。

---

## 🖥️ 設定ダイアログ (Preferences Dialog)

コマンドライン、メニューバー、またはキーボードショートカットから、いつでも設定画面を呼び出すことができます。

| UI 環境 | 起動方法                                                          | 操作方法・特徴                                                 |
| :------ | :---------------------------------------------------------------- | :------------------------------------------------------------- |
| **GUI** | `Ctrl+,` または `:preferences`, メニュー「Edit」→「Preferences」 | 左右2ペイン構成。インクリメンタル検索、リアルタイム適用、保存  |
| **TUI** | `:preferences` / `:pref` / `:settings` / `:dialog`                | 画面中央フローティングパネル。Viキーバインド完全準拠で操作完結 |

### TUI 設定ダイアログのキー操作
- **`j` / `k` / `↓` / `↑`**: カテゴリまたは設定項目間のカーソル移動
- **`h` / `l` / `Tab`**: 左（カテゴリ一覧）と右（設定項目）のペインフォーカス切り替え
- **`Space` / `Enter`**: 設定値のトグル・順次変更（文字コード、行番号、折り返し、不可視文字、テーマ等を即時エディタ反映）
- **`s`**: 現在の設定を `%USERPROFILE%\.vivivi\config.toml` へ即時保存して閉じる
- **`Esc` / `q`**: 設定ダイアログを閉じる（保存せずメモリ状態を維持）

---

## 📁 設定ファイルの配置場所と自動生成

初回起動時に以下のディレクトリおよび日本語コメント解説付きの初期設定テンプレートが自動生成されます。

1. **Windows**: `%USERPROFILE%\.vivivi\config.toml`
2. **Linux / macOS**: `~/.vivivi/config.toml` (または `~/.config/vivivi/config.toml`)

---

## ⚙️ 6大カテゴリ設定仕様 (`config.toml`)

`vivivi` の設定は、以下の 6 つの主要カテゴリに体系化されています。

```toml
# ==========================================
# 1. 全般設定 (General)
# ==========================================
[general]
default_encoding = "utf-8"       # 起動時の既定文字コード ("utf-8", "sjis", "euc-jp")
default_newline = "lf"           # 既定の改行コード ("lf", "crlf")
clipboard = "unnamedplus"        # クリップボード連携 ("unnamedplus", "unnamed", "none")
backup = false                   # 保存時バックアップ作成の有無

# ==========================================
# 2. 表示・外観設定 (Appearance)
# ==========================================
[appearance]
theme = "tokyo-night"            # カラーテーマ名 ("tokyo-night", "openvivi", "vivi-classic", 等)
line_number = true               # 行番号の表示 (true / false)
relative_number = false          # 相対行番号の表示 (true / false)
gutter_markers = true            # 変更行ガター強調の表示 (true / false)
wrap = false                     # 画面端での行折り返し表示 (false: 水平スクロール)

[appearance.gui]
font_size = 15.0                 # GUI フォントサイズ (pt)
line_height_multiplier = 1.4     # 行間倍率 (1.3〜1.5)
show_minimap = true              # ミニマップの表示
enable_gravity_drop = true       # 文字削除時の重力落下演出
cursor_blink = false             # カーソル点滅の有無

# ==========================================
# 3. 不可視文字表示設定 (Invisibles)
# ==========================================
[invisibles]
enabled = false                  # 不可視文字表示の初期有効化 (true / false)
zenkaku_space = "　"             # 全角スペースの表示記号（全角幅維持＋赤色背景ハイライト）
space = "·"                      # 半角スペースの表示記号
tab = "▸---"                     # タブ文字の表示記号
eol = "$"                        # 改行記号の表示記号

# ==========================================
# 4. 入力・編集支援設定 (Editing)
# ==========================================
[editing]
tab_width = 4                    # タブ幅（半角スペース換算数）
expand_tab = false               # Tabキー押下時にスペースへ展開するか
ignorecase = true                # 検索時に大文字小文字を無視 (ic)
smartcase = true                 # 大文字混在時は厳密に区別 (sc)
keisen_protect = true            # 罫線保護編集（枠線内の文字入力・削除時に右側空白を自動伸縮）
dabbrev_ignorecase = true        # 動的補完 (dabbrev) での大文字小文字無視
match_paren = true               # 対応括弧の強調表示 (%)

# ==========================================
# 5. 独自ツール・ファイル管理設定 (Tools)
# ==========================================
[tools]
difflog_enabled = true           # 世代バックアップ (.difflog) 自動蓄積
difflog_max_generations = 50     # 世代保持上限数
outline_auto_open = false        # 起動時にアウトラインペインを開くか
outline_width = 30               # アウトラインペイン幅 (文字数)
code_folding_level = 0           # 初期折り畳みレベル (0: すべて展開)

# ==========================================
# 6. キーマップ・操作体系 (Keymaps)
# ==========================================
[keymap]
preset = "vi"                    # プリセット体系 ("vi" または "windows")

[keymap.normal]
"<C-s>" = ":w<CR>"

[keymap.insert]
"jj" = "<Esc>"
```

---

## ⚡ 設定操作コマンド & 一時反映原則

`vivivi` は Vim 伝統の **「セッション一時反映原則」** を採用しています。
コマンドラインから実行した `:set` や `:colorscheme` は現在のメモリ上のみを変更し、設定ファイルは自動的には上書きされません。

| コマンド                              | 動作説明                                                         |
| :------------------------------------ | :--------------------------------------------------------------- |
| `:preferences` / `:pref` / `Ctrl+,`   | GUI / TUI 設定ダイアログを起動します。                           |
| `:mkvivivirc [path]` / `:wconfig`     | 現在のインメモリ設定値（一時変更を含む）を設定ファイルへ永続保存 |
| `:source [path]` / `:so [path]`       | ディスク上の TOML 設定ファイルを再読込、または Ex スクリプト実行 |
| `:config`                             | `config.toml` をディスクから即時再読込します。                   |
| `:set all`                            | 現在有効なすべてのオプション一覧を表示します。                   |
| `:set number` / `:set nonu`           | 行番号表示の有効／無効切替（`:se nu?` で状態確認）               |
| `:set relativenumber` / `:set nornu`  | 相対行番号表示の有効／無効切替（`:se rnu?` で状態確認）          |
| `:set wrap` / `:set nowrap`           | 画面端折り返し／水平スクロールの切替（`:se wrap?` で状態確認）   |
| `:set list` / `:set nolist`           | 不可視文字（全角空白□/タブ/改行$）表示切替                      |
| `:set minimap` / `:set nominimap`     | ミニマップの表示／非表示切替（GUI）                              |
| `:set particles` / `:set noparticles` | 文字削除落下演出の有効／無効切替（GUI）                          |
| `:set bg=dark` / `:set bg=light`      | 背景明暗の即時切り替え                                           |
| `:set tabstop=4` / `:set ts=4`        | タブ幅の指定                                                     |
| `:set clipboard=unnamedplus`          | OSクリップボード共有の有効化                                     |
| `:colorscheme <name>` / `:colo`       | カラーテーマの動的切り替え（`:colo?` で現在テーマ表示）          |
