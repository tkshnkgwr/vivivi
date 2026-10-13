# vivivi: 変更履歴 (CHANGELOG)

本プロジェクトのすべての主要な変更を記録します。フォーマットは [Keep a Changelog](https://keepachangelog.com/) に準拠しています。

---

## [Unreleased]

### Added
- **`vivivi-core`**:
  - OpenViVi準拠の `GapBuffer<T>` 実装（$O(1)$ 局所編集・動的容量倍増・ギャップスライド）。
  - トランザクション対応 `UndoMgr` 実装（Undoブロック、連続タイピング自動マージ、保存点追跡）。
  - `GapBuffer` と `UndoMgr` を統合した `TextBuffer` 実装。
  - 文字コード自動判別・相互変換モジュール（`encoding.rs`）の実装（BOM判定、UTF-8, Shift_JIS/CP932, EUC-JP, UTF-16LE/BE 対応、CRLF/LF/CR 改行コードの自動判別と相互変換）。
  - 対応括弧探索モジュール（`paren.rs`）の実装（`()`, `{}`, `[]`, `<>` の文字列・コメントスキップ探索、カーソル位置対応ペア検出）。
- **`vivivi-vi`**:
  - Viモード定義（Normal, Insert, Replace, Visual, VisualLine, VisualBlock, CommandLine）。
  - 基本コマンド・モーション・オペレータ定義。
  - 各種カーソルモーション実装（`h`, `j`, `k`, `l`, `w`, `b`, `e`, `W`, `B`, `E`, `0`, `^`, `$`, `gg`, `G`、対応括弧ジャンプ `%`）。
  - オペレータ実行機構（`d`, `y`, `c` + motion / `dd`, `yy`, `cc`, `D`, `C`, `Y`, `x`, `X`）。
  - レジスタ管理システム（無名レジスタ、`"a`〜`"z`、大文字アペンド、行/文字指向判定、`p`, `P` 貼り付け）。
  - **BOX選択・矩形編集の完全対応（VisualBlock）**:
    - `Ctrl-v` による矩形選択、矩形ヤンク（`y`）、矩形削除（`d`, `x`）および `RegisterContent::BlockWise` 格納。
    - 矩形一括挿入（`I`, `A`）: 矩形先頭/末尾からInsertモードへ突入し、`Esc` 復帰時に選択全行の同列位置へ入力文字列を一括反映。
    - スマート矩形ペースト（`p`, `P`）: 行末スペース自動補填による矩形貼り付け。
    - ビジュアル選択範囲の大文字・小文字反転（`~`）。
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
  - 構文解析・シンタックスハイライト・トークナイザー（`syntax.rs`）の実装（Rust, C/C++, Python, JS/TS, JSON, Markdown の予約語・型・文字列・数値・コメント色分け）。
  - **文字変換モジュール（`conv.rs`）の実装**:
    - 全角英数記号 ⇄ 半角英数記号、ひらがな ⇄ 全角カタカナ、全角カタカナ ⇄ 半角カタカナ（濁点結合・分解対応）、大文字 ⇄ 小文字、ケース反転。
  - **アウトライン解析モジュール（`outline.rs`）の実装**:
    - Markdown（`#` 〜 `######` 見出し階層）、Rust（`fn`, `struct`, `impl`, `enum`, `trait`, `mod`）、C/C++、Pythonのシンボルツリー解析。
- **`vivivi-app`**:
  - `ratatui` (v0.29) と `crossterm` (v0.28) によるフルスクリーンTUIフロントエンド実装。
  - 代替スクリーン（Alternate Screen）とRawモードによる端末制御およびパニック耐性復元フック。
  - エディタバッファ（行番号・カーソル行ハイライト・スクロール・CSV整列ビュー対応）の描画。
  - シンタックスハイライト描画および対応括弧ペアのハイライト描画。
  - **画面分割・マルチビュー（Split / Vsplit）の実装**:
    - 水平分割（`:split` / `:sp`）、垂直分割（`:vsplit` / `:vs`）、ウィンドウ閉鎖（`:close` / `:c`）、単一ウィンドウ化（`:only`）。
    - Gvim互換の `Ctrl-w` ウィンドウキーバインド（`Ctrl-w w`, `Ctrl-w s`, `Ctrl-w v`, `Ctrl-w c`, `Ctrl-w o`, `Ctrl-w h/j/k/l`）。
    - 複数エディタウィンドウの上下・左右並行分割描画およびアクティブウィンドウ枠線表示。
  - **アウトライン解析サイドペインの実装**:
    - `F8` キーまたは `:outline` によるアウトライン表示トグル。
    - `Ctrl-o` でのサイドペインフォーカス、`j`/`k` 選択と `Enter` による該当行即時ジャンプ。
  - **文字変換 ex コマンドの実装**:
    - `:%zenkaku`, `:%hankaku`, `:%hiragana`, `:%katakana`, `:%hankata`, `:%zenkata`, `:%toupper`, `:%tolower` によるバッファ全文一括変換。
  - Visual / VisualLine / VisualBlock 選択範囲のリアルタイム背景色ハイライト描画。
  - リアルタイム・ステータスバー（Viモード・罫線/CSVバッジ・ファイル名・変更フラグ・文字コード/改行/言語バッジ・行/列・進捗%）の描画。
  - ファイル保存時の元の文字コード・改行コード維持保存。
  - コマンドライン入力UI（`:w`, `:q`, `:wq`, `:align`, `:%s` 等の対話実行）。
  - ネイティブターミナルカーソルのハードウェア点滅連動。
  - `--demo`（コンソールデモ）フラグのサポート。
- **リポジトリ・ライセンス・開発基盤**:
  - リポジトリルートに [LICENSE](file:///c:/Users/ABC/OneDrive/ドキュメント/自作/vivivi/LICENSE)（MIT License）を配備。
  - `.agents/`（ルール・スキル群）、`GEMINI.md`、`RULES.md` のセットアップ。
  - 多言語ドキュメント体系（`docs/ja/`, `docs/en/`）の構築。

### Changed / Refactored
- **モジュール分割・ファイル肥大化解消リファクタリング**:
  - 1,000行を超えていた `vivivi-vi` の `engine.rs` (1,441行) を機能責務ごとに `engine/` ディレクトリ配下（`motion.rs`, `operate.rs`, `visual.rs`, `ex.rs`, `mod.rs`, `tests.rs`）へ分割（各ファイル150〜350行へスリム化）。
  - `vivivi-app` の `tui/ui.rs` (514行) を `ui/` ディレクトリ配下（`editor.rs`, `outline.rs`, `status.rs`, `mod.rs`）へ分割し、今後のGrepペイン・DIFF画面等の拡張性を強化。
  - `vivivi-app` のウィンドウ管理（`SplitAxis`, `EditorWindow`）を `tui/window.rs` へ分離。
  - プロジェクト全体の全ソースコードが500行以下に収まり、保守性と可読性を向上。

