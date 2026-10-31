# カラーテーマ仕様書 (`themes/<name>.toml`)

> [!IMPORTANT]
> 本書は、`vivivi`（TUI / GUI 共通）におけるカラーテーマファイル形式、セマンティックカラー定義、およびフロントエンド（Ratatui / egui）への共通マッピング仕様書です。
> 親仕様書: [設定管理ハブ (SETTINGS_DESIGN.md)](../SETTINGS_DESIGN.md)

---

## 1. 概要と共通テーマ設計思想

`vivivi` では、**「1つのテーマファイル（`.toml`）で TUI と GUI の両方の世界観を完全に統一する」** アーキテクチャを採用しています。

- **TrueColor (24bit RGB) 統一**: 現代のターミナルおよび egui はどちらも 24bit RGB（`#RRGGBB`）をネイティブサポートしているため、同一のカラーパレットを共有可能。
- **セマンティック抽象化**: 色コードを直接ハードコードせず、「エディタ背景」「選択色」「キーワード色」などの役割名で定義。
- **独立管理**: テーマファイルは `%USERPROFILE%\.vivivi\themes\` ディレクトリに個別の `.toml` ファイルとして配置され、ユーザーによる自作や追加配布が容易。

---

## 2. テーマ TOML スキーマ

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

## 3. TUI / GUI 各フロントエンドへの共通マッピング

| セクション | 役割                     | GUI (egui) へのマッピング            | TUI (Ratatui) へのマッピング          |
| :--------- | :----------------------- | :----------------------------------- | :------------------------------------ |
| `[ui]`     | エディタ背景・UI各部     | `Color32::from_rgb` / `Visuals`      | `ratatui::style::Color::Rgb` / 背景色 |
| `[modes]`  | Vi モードバッジ配色      | タブバー・左下ステータスバッジの背景 | ステータスラインのモード文字列背景    |
| `[gutter]` | 行変更追跡マーカー       | ガターの垂直バー色                   | ガター境界文字（`│`）の前景色         |
| `[syntax]` | 構文シンタックスハイライト | `egui::RichText` の描画カラー        | `ratatui::text::Span` の `Style`      |

---

## 4. 内蔵5大プリセットテーマ

初期起動時に `%USERPROFILE%\.vivivi\themes\` へ以下の5つの標準テーマが自動展開されます。

| テーマファイル名    | テーマ名称         | スタイル | 特徴                                                       |
| :------------------ | :----------------- | :------- | :--------------------------------------------------------- |
| `tokyo-night.toml`  | **Tokyo Night**    | Dark     | モダンな青藍系ダークテーマ。高コントラストで視認性抜群     |
| `openvivi.toml`     | **OpenViVi**       | Dark     | Qt版 OpenViVi 復刻テーマ。黒基調に深みのあるアクセント     |
| `vivi-classic.toml` | **ViVi Classic**   | Light    | ViVi 3/4 伝統のクラシック白背景配色。明瞭なエディタ表示    |
| `vivivi-dark.toml`  | **vivivi Dark**    | Dark     | vivivi 独自設計のミニマル・フラットダークテーマ            |
| `vivivi-light.toml` | **vivivi Light**   | Light    | 目に優しいウォーム系ペーパーホワイト調のライトテーマ       |

---

## 5. ホットリロード仕様

- **監視と即時反映**: テーマファイル（`.toml`）が外部エディタで保存された際、または設定画面・`:colorscheme` コマンドで切り替えられた際、エディタを再起動することなく画面全体の配色が即座にリアクティブ更新されます。
