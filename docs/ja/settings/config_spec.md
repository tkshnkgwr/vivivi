# エディタ基本設定仕様書 (`config.toml`)

> [!IMPORTANT]
> 本書は、`vivivi`（TUI / GUI 共通）のエディタ基本設定ファイル `config.toml` の構造、設定項目、および初期起動時の自動配備（Auto Bootstrap）プロトコルの詳細仕様書です。
> 親仕様書: [設定管理ハブ (SETTINGS_DESIGN.md)](../SETTINGS_DESIGN.md)

---

## 1. 概要とディレクトリ構造

エディタの基本動作設定は、TOML形式のプレーンテキストファイルとして永続化されます。
設定は環境変数 `%USERPROFILE%`（Unix では `$HOME`）配下の `.vivivi/` ディレクトリに集約されます。

### ディレクトリ構成 (`%USERPROFILE%\.vivivi\`)

```
%USERPROFILE%\.vivivi\
├── config.toml                # エディタ基本設定・キーマップ・動作オプション (本書)
├── themes\                    # カラーテーマ専用ディレクトリ (themes_spec.md 参照)
├── types\                     # (将来) タイプ別設定 (CPP, Rust, Markdown等)
└── cache\                     # 履歴・スワップ・更新キャッシュ (window_state.md 参照)
```

---

## 2. 初期起動時の自動生成プロトコル (Auto Bootstrap)

エディタ起動時（TUI / GUI 共通）、設定ローダーは以下の処理を自律的に実行します。

1. **設定ディレクトリ存在チェック**:
   - `%USERPROFILE%\.vivivi\`（Unix では `~/.vivivi/` または `~/.config/vivivi/`）が存在しない場合、自動生成（`std::fs::create_dir_all`）。
2. **`config.toml` の自動作成**:
   - ファイルが存在しない場合、すべてのデフォルト値および解説コメントが記載された標準テンプレート `config.toml` を自動生成してディスクに保存。
3. **安全フォールバック**:
   - TOML パースエラーや不正な値が含まれる場合、エラー行を通知しつつ該当項目のみ内蔵デフォルト値で安全にフォールバック起動。

---

## 3. 設定項目一覧と TOML スキーマ

```toml
# vivivi Configuration File
version = "1.0"

# ==========================================
# 1. 一般 (General)
# ==========================================
[general]
default_encoding = "utf-8"     # "utf-8", "sjis", "euc-jp"
default_fileformat = "unix"    # "unix" (LF) または "dos" (CRLF)
clipboard = "unnamedplus"      # "none", "unnamed" (*), "unnamedplus" (+)
crash_recovery = true          # 異常終了時の自動スワップ保存
check_update = true            # 起動時の自動更新確認

# ==========================================
# 2. 表示・外観 (Appearance)
# ==========================================
[appearance]
theme = "tokyo-night"          # アクティブテーマ (themes/<name>.toml)
line_number = true             # 行番号表示 (nu)
relative_number = false        # 相対行番号表示 (rnu)
wrap = false                   # 右端折り返し (wrap)
gutter_markers = true          # 変更行ガター強調 (未保存:黄 / 保存済み:緑)

[appearance.gui]
font_family = "PlemolJP Console NF"
font_size = 15.0
line_height_multiplier = 1.35
show_minimap = true
minimap_width = 90.0
enable_gravity_drop = true     # 文字削除時の物理重力落下演出 (OpenViVi)
reactive_idle_sleep = true     # アイドル時CPU 0%省電力制御

# ==========================================
# 3. 不可視文字・空白 (Invisibles)
# ==========================================
[invisibles]
enabled = false                # 不可視文字表示 (:set list)
zenkaku_space = "　"           # 全角スペース (ワインレッド背景強調)
space = "·"                    # 半角スペース
tab = "▸---"                   # タブ文字
eol = "$"                      # 改行記号

# ==========================================
# 4. 入力・編集支援 (Editing & Assists)
# ==========================================
[editing]
tab_width = 4                  # タブ幅 (半角スペース換算)
expand_tab = false             # Tabキー押下時にスペースへ展開するか
ignorecase = true              # 検索時大文字小文字無視 (ic)
smartcase = true               # 大文字混在時は区別 (sc)
keisen_protect = true          # 罫線保護編集 (枠線内自動空白伸縮)
dabbrev_ignorecase = true      # 動的補完 (dabbrev) での大文字小文字無視
match_paren = true             # 対応括弧の強調表示 (%)

# ==========================================
# 5. 独自ツール・ファイル管理 (Tools & Views)
# ==========================================
[tools]
difflog_enabled = true         # 世代バックアップ (.difflog) 自動蓄積
difflog_max_generations = 50   # 世代保持上限数
outline_auto_open = false      # 起動時にアウトラインペインを開くか
outline_width = 30             # アウトラインペイン幅 (文字数)
code_folding_level = 0         # 初期折り畳みレベル (0: すべて展開)

# ==========================================
# 6. キーマップ・操作体系 (Keymaps)
# ==========================================
[keymap]
preset = "vi"                  # "vi" または "windows" (Shift+矢印, Ctrl+C/V)

[keymap.normal]
"<C-s>" = ":w<CR>"

[keymap.insert]
"jj" = "<Esc>"
```

---

## 4. 設定項目のカテゴリ別解説

| カテゴリ       | セクション名       | 役割・適用範囲                                               |
| :------------- | :----------------- | :----------------------------------------------------------- |
| **一般**       | `[general]`        | デフォルト文字コード、改行形式、クリップボード同期、自動更新 |
| **表示・外観** | `[appearance]`     | テーマ指定、行番号表示、相対行番号、折り返し、ガター強調     |
| **GUI固有**    | `[appearance.gui]` | フォント指定、フォントサイズ、ミニマップ、落下演出、省電力   |
| **不可視文字** | `[invisibles]`     | タブ、半角/全角スペース、改行記号の可視化スタイル            |
| **入力・編集** | `[editing]`        | タブ幅、スペース展開、検索の大文字小文字判定、罫線保護       |
| **独自ツール** | `[tools]`          | DiffLog世代バックアップ、アウトライン幅、コード折り畳み      |
| **キーマップ** | `[keymap]`         | プリセット体系 (Vi / Windows) および個別キーバインド定義     |
