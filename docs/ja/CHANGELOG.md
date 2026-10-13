# vivivi: 変更履歴 (CHANGELOG)

本プロジェクトのすべての主要な変更を記録します。フォーマットは [Keep a Changelog](https://keepachangelog.com/) に準拠しています。

---

## [Unreleased]

### Added
- **`vivivi-core`**:
  - OpenViVi準拠の `GapBuffer<T>` 実装（$O(1)$ 局所編集・動的容量倍増・ギャップスライド）。
  - トランザクション対応 `UndoMgr` 実装（Undoブロック、連続タイピング自動マージ、保存点追跡）。
  - `GapBuffer` と `UndoMgr` を統合した `TextBuffer` 実装。
- **`vivivi-vi`**:
  - Viモード定義（Normal, Insert, Replace, Visual, VisualLine, VisualBlock, CommandLine）。
  - 基本コマンド・モーション・オペレータ定義。
  - 各種カーソルモーション実装（`h`, `j`, `k`, `l`, `w`, `b`, `e`, `W`, `B`, `E`, `0`, `^`, `$`, `gg`, `G`）。
  - オペレータ実行機構（`d`, `y`, `c` + motion / `dd`, `yy`, `cc`, `D`, `C`, `Y`, `x`, `X`）。
  - レジスタ管理システム（無名レジスタ、`"a`〜`"z`、大文字アペンド、行/文字指向判定、`p`, `P` 貼り付け）。
  - 前方・後方パターン検索（`/`, `?`, `n`, `N`, `*`, `#`、ラップアラウンド対応）。
  - exコマンド実行機構（`:w`, `:q`, `:s`, `:%s/before/after/g` 正規表現置換、`:行番号` ジャンプ）。
  - ステートフルな `CommandParser`（カウント数値、レジスタ指定、2打鍵プレフィックスのリアルタイム解析）。
- **`vivivi-modes`**:
  - ViVi 4準拠のインラインCSV/TSVモード定義。
  - OpenViVi `keisen.cpp` からのUnicode罫線自動結合テーブル移植。
  - 罫線モードのキー操作描画・消去・自動結合ロジック（`step`）の実装（進行方向・直角方向の線判定、改行・余白の自動拡張補完）。
  - 罫線枠内文字列の整列アルゴリズム（左揃え、中央揃え、右揃え）の実装。
  - インラインCSV表示フォーマッター（`format_line`）およびクォート透過表示・アクティブ行の生テキスト表示。
  - 生バッファ内のセル文字範囲特定（`cell_range_in_raw_line`）の実装。
- **エージェント・開発基盤**:
  - `.agents/`（ルール・スキル群）、`GEMINI.md`、`RULES.md` のセットアップ。
  - 多言語ドキュメント体系（`docs/ja/`, `docs/en/`）の構築。
