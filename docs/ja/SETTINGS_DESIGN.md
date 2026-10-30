# vivivi: 設定管理＆設定画面・ダイアログ設計仕様書 (SETTINGS_DESIGN.md)

> [!IMPORTANT]
> 本書は、`vivivi`（TUI / GUI 両エディション共通）における設定永続化アーキテクチャ、外部設定ファイル分割設計、初期起動時の自動生成仕様、および将来実装する設定ダイアログ・設定画面の設計仕様書です。

---

## 1. 全体アーキテクチャと設定ファイル分割設計

従来の ViVi ではレジスタやバイナリプロパティに保存されていた設定を、現代の Rust / エディタ標準として **TOML 形式** で平文永続化します。
特に、肥大化しやすい **「カラーテーマ設定（構文色・UI各部配色）」** はエディタの基本動作設定と分離し、独立したファイルとして安全に追加・変更・共有・配布できるようにします。

### 1.1 ディレクトリ構成 (`%USERPROFILE%\.vivivi\`)

```
%USERPROFILE%\.vivivi\
├── config.toml                # エディタ基本設定・キーマップ・動作オプション
├── themes\                    # カラーテーマ専用ディレクトリ
│   ├── tokyo-night.toml       # デフォルトダークテーマ (Tokyo Night)
│   ├── openvivi.toml          # OpenViVi 復刻テーマ (Qt版の洗練されたダーク)
│   ├── vivi-classic.toml      # ViVi 3/4 伝統のクラシック配色 (白背景/灰背景)
│   ├── vivivi-dark.toml       # vivivi 標準モダンダークテーマ
│   └── vivivi-light.toml      # vivivi 標準モダンライトテーマ
├── types\                     # (将来) タイプ別設定 (CPP, Rust, Markdown等)
│   ├── rust.toml
│   └── markdown.toml
└── cache\                     # 履歴・スワップ・更新キャッシュ (非設定情報)
    ├── recent.json            # 最近使ったファイル履歴
    ├── bookmarks.json         # グローバルマーク
    └── update_check.json      # 自動更新チェックキャッシュ
```

---

## 2. 初期起動時の自動生成プロトコル (Auto Bootstrap)

エディタ起動時（TUI / GUI 共通）、設定読み込み処理において以下の判定を自律的に行います。

1. **設定ディレクトリ存在チェック**:
   - `%USERPROFILE%\.vivivi\`（Unix では `~/.vivivi/` または `~/.config/vivivi/`）が存在しない場合、自動生成（`std::fs::create_dir_all`）。
2. **`config.toml` の自動作成**:
   - ファイルが存在しない場合、すべてのデフォルト値および解説コメントが記載された標準テンプレート `config.toml` を自動生成して保存。
3. **`themes/` ディレクトリおよび標準テーマの自動配備**:
   - `themes/` 配下に標準テーマ（`tokyo-night.toml`, `openvivi.toml`, `vivi-classic.toml`, `vivivi-dark.toml`, `vivivi-light.toml`）が存在しない場合、バイナリ内蔵のデフォルト定義を展開して自動書き出し。
   - ユーザーは任意の `<theme-name>.toml` を同ディレクトリに配置するだけで、再起動なし（またはコマンド一発）で新しいテーマを認識・切替可能。

---

## 3. 設定ファイル仕様 (`config.toml`)

エディタの基本動作、表示オプション、ファイルI/O、キーマップを管理します。

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

## 4. テーマ設定ファイル仕様 (`themes/<name>.toml`)

テーマカラー設定は、エディタ本体、UIパネル、Viモードバッジ、およびシンタックスハイライトの各要素を **16進数カラーコード (`#RRGGBB` / `#RRGGBBAA`)** で定義します。

```toml
[meta]
name = "Tokyo Night"
author = "folke / vivivi port"
variant = "dark" # "dark" または "light"

[ui]
# 3段階背景コントラスト
bg_editor = "#1a1b26"       # 中央エディタ領域
bg_panel = "#16161e"        # サイドバー・タブバー・メニューバー
bg_statusline = "#1f2335"   # ステータスバー・枠線
bg_selection = "#283457"    # 選択範囲ハイライト
bg_hover = "#292e42"        # ボタン・項目ホバー
bg_active = "#414868"       # アクティブ・押下状態

fg_text = "#c0caf5"         # 標準テキスト
fg_dimmed = "#565f89"       # コメント・非アクティブ文字
fg_linenumber = "#3b4261"   # 行番号・ガター
fg_cursor = "#c0caf5"       # カーソル前景色
bg_cursor = "#7aa2f7"       # カーソル背景色

[modes]
# 各 Vi モード表示バッジ背景色
normal = "#7aa2f7"          # 青
insert = "#9ece6a"          # 緑
visual = "#bb9af7"          # 紫
command = "#e0af68"         # 黄
replace = "#f7768e"         # 赤
keisen = "#7dcfff"          # シアン

[gutter]
change_modified = "#e0af68" # 未保存変更行ガター（黄）
change_saved = "#9ece6a"    # 保存済み変更行ガター（緑）

[syntax]
# 構文ハイライト色定義
keyword = "#bb9af7"         # fn, struct, let, if, return 等
type = "#7dcfff"            # String, usize, i32, Option 等
function = "#7aa2f7"        # 関数名・メソッド名
string = "#9ece6a"          # "文字列リテラル"
number = "#ff9e64"          # 123, 0xFF, 3.14 等
comment = "#565f89"         # // コメント, /* ブロック */
macro = "#7dcfff"           # println!, vec! 等
operator = "#89ddff"        # +, -, =, =>, :: 等
```

---

## 5. 設定画面（GUI / TUI）のUI設計仕様

### 5.1 GUI 設定パネル（egui: 左右2ペイン・モーダルダイアログ）

ViVi の洗練された設定プロパティシートを踏襲し、フラットで使いやすいダイアログを提供します。

```
+--------------------------------------------------------------------------+
|  設定 (Preferences) - [Tokyo Night]                             [X] 閉じる|
+-------------------+------------------------------------------------------+
| [検索: _________] | [表示・外観 (Appearance)]                            |
|                   |                                                      |
| ▶ 1. 一般 (Gen)   |  カラーテーマ                 : [Tokyo Night        ▼]|
| ▼ 2. 表示・外観   |  行番号表示 (Line Numbers)    : [x] 有効              |
|    - 外観・テーマ |  相対行番号 (Relative Num)    : [ ] 無効              |
|    - ガター       |  右端折り返し (Word Wrap)     : [ ] 無効 (nowrap)     |
| ▶ 3. 不可視文字   |  変更行ガター強調             : [x] 有効              |
| ▶ 4. 入力・編集   |                                                      |
| ▶ 5. 独自ツール   | [GUI / フォント]                                     |
| ▶ 6. キーマップ   |  フォントファミリー           : [PlemolJP Console NF▼]|
|                   |  フォントサイズ               : [ 15.0 ] [▲][▼]      |
|                   |  文字削除落下アニメーション   : [x] 有効 (OpenViVi演出)|
|                   |  ミニマップ表示               : [x] 有効 (幅: 90px)   |
|                   |                                                      |
+-------------------+------------------------------------------------------+
| [デフォルトに戻す]                      [キャンセル]  [適用]  [保存して閉じる] |
+--------------------------------------------------------------------------+
```

### 5.2 テーマ選択・リアルタイムプレビューUI

- **テーマ切替ドロップダウン**:
  - メニューバー `[表示(V)] -> [カラーテーマ(T)] -> [OpenViVi / ViViClassic / TokyoNight / ViviviDark / ViviviLight]`
  - または設定画面内のテーマリストからワンクリックで切替。
- **ホットリロード**:
  - テーマファイル（`.toml`）を変更して保存すると、エディタを再起動することなく画面全体の配色が即座にリアクティブ更新。

### 5.3 TUI 設定画面（Ratatui: 中央フローティングウィンドウ）

- `:preferences` または `:set`（引数なし）でポップアップ表示。
- `j`/`k` で項目移動、`h`/`l` でカテゴリ開閉、`Space` でトグル切り替え、`Enter` で値入力・確定。
- Vi キーバインドに完全準拠したキーボード完結の操作感。

---

## 6. コマンドライン（`:set`, `:colorscheme`）実行と永続化の境界ルール

コマンドライン（Exコマンド）からの設定変更やテーマ変更と、ファイル永続化の境界について以下のルールを定めます。

1. **Exコマンドによる設定・テーマ変更は「一時反映（セッション限定）」を原則とする**:
   - **エディタ設定**: キャプチャ撮影時の一時的な `:set nonumber` (`:se nonu`)、一時的な折り返し表示 `:set wrap`、他者コード閲覧時の一時的な `:set tabstop=2` (`:se ts=2`) など。
   - **カラーテーマ**: 気分転換や屋外での一時的な `:colorscheme high-contrast` (`:colo ...`) や `:set theme=openvivi` など。
   - これらは**現在のセッション（起動中）のインメモリ状態のみを変更**し、`config.toml` には書き込まない。
   - エディタを再起動すれば、いつでも `config.toml` で定義された安全な基本設定・常用テーマにクリーンに戻る。
2. **現在の設定・テーマを恒久保存（永続化）する方法**:
   - **方法A（Ex保存コマンド）**: Vim 互換の **`:mkvivivirc`**（または `:wconfig`）を実行する。
     - 現在のインメモリ設定値（タブ幅、行番号、アクティブテーマ等）を取りまとめ、`%USERPROFILE%\.vivivi\config.toml` へ一発で安全に上書き永続化する。
   - **方法B（設定画面）**: GUI / TUI 設定ダイアログを開いて設定を変更し、`[保存して閉じる]` を押下する。
3. **外部ファイル再読込**:
   - **`:source`**（または `:so`）コマンドにより、エディタを再起動することなくディスク上の最新 `config.toml` を再ロードして画面へ即座に反映できる。

---

## 7. 実装計画とロードマップ

1. **ステップ 1: 外部テーマ TOML パーサー＆ローダーの実装 (`vivivi-gui::theme`)**
   - `themes/<name>.toml` の構造体定義（serde Serialize/Deserialize）。
   - `%USERPROFILE%\.vivivi\themes\` からのファイル読み込みと動的テーマ生成。
2. **ステップ 2: 初期起動ブートストラップ処理の実装 (`vivivi-app::config`)**
   - `%USERPROFILE%\.vivivi\` および `themes/` の自動作成・内蔵5大テーマの自動書き出し。
3. **ステップ 3: GUI メニューバーからの動的テーマ切り替え接続**
   - メニュー選択時にアクティブテーマを変更し、`Visuals` とエディタ描画色を即時再描画。
4. **ステップ 4: GUI / TUI 設定ダイアログのUI実装**
   - 設定項目（タブ幅、フォント、行番号等）をインタラクティブに変更・保存できる専用モーダルの構築。
