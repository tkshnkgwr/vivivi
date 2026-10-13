# vivivi: システムアーキテクチャ設計書

## 1. プロジェクト目的
本プロジェクトは、C++ / Qtベースで開発されていた高機能テキストエディタ「OpenViVi」の仕様をベースに、モダンかつ安全・高速なプログラミング言語である **Rust** で再構築するプロジェクトです。
また、有償版（ViVi 3 / ViVi 4）に実装されていた独自仕様（インラインCSV/TSVモード、罫線モード等）についても、ヘルプファイルや仕様を忠実に反映・移植します。

---

## 2. 参考元リソースの参照方針
- **OpenViVi ソースコード**: `C:\Users\632792\Documents\参考元アプリ\openViVi`
  - テキストバッファ (`vivi/buffer/`)
  - Viエンジン・コマンド体系 (`vivi/vivi64/ViEngine.*`, `vivi/vivi64/TestViCommands.*`)
  - 罫線モード基礎実装 (`vivi/vivi64/keisen.cpp`)
- **有償版 ViVi 3 / ViVi 4**:
  - ViVi 3: `C:\Users\632792\Documents\個人ツール\vivi3`
  - ViVi 4: `C:\Users\632792\Documents\個人ツール\vivi4`
  - ヘルプファイル (`vivihelp.CHM`): `docs/references/vivi4_help/` および `docs/references/vivi3_help/` 配下にデコンパイル・UTF-8化して配置。`rg` による高速仕様検索が可能。

---

## 3. クレート構成（Cargo Workspace）
エディタのコアロジックをUI（CLI / TUI / 将来のGUI）から完全に独立させ、高カバレッジな単体テストを可能にするため、マルチクレート構成（Cargo Workspace）を採用しています。

```mermaid
graph TD
    App[vivivi-app (CLI / TUI エントリポイント)]
    Core[vivivi-core (バッファ・Undo・カーソル・エンコーディング)]
    Vi[vivivi-vi (Vi状態機械・コマンド・レジスタ・ex)]
    Modes[vivivi-modes (インラインCSV・罫線描画)]

    App --> Vi
    App --> Modes
    App --> Core
    Vi --> Core
    Modes --> Core
```

### (1) `vivivi-core`
- **責務**: テキストバッファ、行管理、カーソル位置、Undo/Redo、文字コード変換、高速検索。
- **主要モジュール**:
  - `buffer`: `TextBuffer` (将来的にGapBuffer最適化)
  - `cursor`: カーソル位置、選択範囲（アンカー）
  - `line_mgr`: 行インデックス、折り返し、行メタデータ
  - `undo`: トランザクション対応Undoブロック管理
  - `encoding`: UTF-8, Shift_JIS, EUC-JP, UTF-16LE/BE 対応
  - `search`: 高速文字列検索

### (2) `vivivi-vi`
- **責務**: Viコマンドエンジン。
- **主要モジュール**:
  - `mode`: Normal, Insert, Replace, Visual, VisualBlock, CommandLine
  - `command`: モーション（`h, j, k, l, w, b, e, $...`）、オペレータ（`d, y, c...`）
  - `engine`: キー入力イベントからViCommandへの変換状態機械
  - `register`: クリップボード、無名レジスタ、名前付きレジスタ (`"a`〜`"z`)

### (3) `vivivi-modes`
- **責務**: ViVi特有の高度な編集モード群。
- **主要モジュール**:
  - `csv`: インラインCSV/TSVモード（自動カラム幅計算、クォート処理、ヘッダ行認識）
  - `keisen`: Unicode罫線（U+2500〜U+254B）の交点結合・描画・消去

### (4) `vivivi-app`
- **責務**: フロントエンドおよびアプリケーション本体。
