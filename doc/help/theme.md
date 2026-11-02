# vivivi カラーテーマ設定 (Color Themes)

`vivivi`（ヴィヴィヴィ）は、ターミナル TUI およびデスクトップ GUI の両方で完全に一貫した外観・配色を提供する、モダンなテーマシステムを搭載しています。

---

## 🎨 内蔵 5 大プリセットテーマ

本エディタには、初回起動時から利用可能な 5 つの洗練されたテーマが組み込まれています。

| テーマ名       | 種別  | 概要                                                           |
| :------------- | :---- | :------------------------------------------------------------- |
| `tokyo-night`  | Dark  | デフォルトテーマ。青紫基調のモダンで目に優しいハイコントラスト |
| `openvivi`     | Dark  | OpenViVi (Qt) 伝統のダークネイビー調パレット                   |
| `vivi-classic` | Light | 往年の ViVi 3 / ViVi 4 を彷彿とさせるクラシックライト配色      |
| `vivivi-dark`  | Dark  | 深い黒を基調としたミニマル＆ストイックなダークテーマ           |
| `vivivi-light` | Light | クリーンでコントラスト明瞭なモダンライトテーマ                 |

---

## ⚡ テーマの切り替え・操作コマンド

コマンドライン（Ex コマンド）および設定画面から、いつでも瞬時にテーマを変更できます。

| 操作 / コマンド           | 説明                                                        |
| :------------------------ | :---------------------------------------------------------- |
| `:colorscheme <name>`     | テーマを動的に切り替えます（例: `:colorscheme openvivi`）   |
| `:colo <name>`            | `:colorscheme` の短縮表記                                   |
| `:colorscheme?` / `:colo` | 現在適用されているテーマ名を表示します                      |
| `:preferences` / `Ctrl+,` | 設定ダイアログを開き、「表示・外観」からテーマを選択します  |
| `:mkvivivirc`             | 現在のテーマを含むインメモリ設定を `config.toml` に永続保存 |

> [!NOTE]
> コマンドラインからの `:colorscheme` 変更は、Vim 伝統の「セッション限定の一時反映原則」に従います。
> 次回起動時にも永続させたい場合は、設定画面で「保存して閉じる」を押すか、`:mkvivivirc` を実行してください。

---

## 🛠️ 外部テーマ TOML によるカスタムテーマ作成

ユーザー独自のテーマを TOML ファイルとして作成・配備できます。
作成したテーマファイルは、エディタ再起動不要で即座に `:colorscheme <テーマ名>` として認識されます。

### 1. 配置ディレクトリ
- **Windows**: `%USERPROFILE%\.vivivi\themes\<テーマ名>.toml`
- **Linux / macOS**: `~/.vivivi/themes/<テーマ名>.toml`

### 2. テーマ TOML スキーマ例 (`my-custom.toml`)

```toml
[meta]
name = "my-custom"
author = "Your Name"
variant = "dark" # "dark" または "light"

[ui]
bg_editor = "#1a1b26"      # テキスト編集領域の背景色
bg_panel = "#16161e"       # サイドバー・タブバー・枠組みの背景色
bg_statusline = "#1f2335"  # ステータスラインの背景色
bg_selection = "#283457"   # 選択範囲の背景色
bg_hover = "#292e42"       # ホバー時ハイライト色
bg_active = "#414868"      # アクティブ項目背景色
fg_text = "#c0caf5"        # 通常テキスト文字色
fg_dimmed = "#565f89"      # 補助テキスト文字色
fg_linenumber = "#3b4261"  # 行番号文字色

[modes]
normal = "#7aa2f7"         # Normal モードバッジ色
insert = "#9ece6a"         # Insert モードバッジ色
visual = "#bb9af7"         # Visual モードバッジ色
command = "#e0af68"        # Command モードバッジ色
replace = "#f7768e"        # Replace モードバッジ色

[syntax]
keyword = "#bb9af7"        # キーワード (fn, let, if 等)
type_name = "#2ac3de"      # 型・構造体名 (String, Vec 等)
string = "#9ece6a"         # 文字列リテラル
number = "#ff9e64"         # 数値リテラル
comment = "#565f89"        # コメント
operator = "#89ddff"       # 記号・演算子
```

---

## ⚙️ `config.toml` によるデフォルトテーマ指定

エディタ起動時のデフォルトテーマは、`%USERPROFILE%\.vivivi\config.toml` 内の `[appearance]` セクションで指定します。

```toml
[appearance]
theme = "openvivi"          # 起動時のデフォルトテーマ名
line_number = true
wrap = false
```
