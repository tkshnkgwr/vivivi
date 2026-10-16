# vivivi: 変更履歴 (CHANGELOG)

本プロジェクトのすべての主要な変更を記録します。フォーマットは [Keep a Changelog](https://keepachangelog.com/) に準拠し、バージョン番号は [セマンティックバージョニング (SemVer)](https://semver.org/) を採用しています。

---

## [0.9.0-beta] - 2026-10-16 (Testing / Dogfooding)

### Changed
- **機能凍結（Feature Freeze）とベータ版テストへの移行**:
  - 全計画機能（Phase 1〜4.8）の実装完了およびモジュール分割リファクタリングの完了に伴い、開発フェーズを「α版（機能構築）」から「β版（実用検証・熟成）」へ移行。
  - ボスの日常実務におけるドッグフーディングおよび手動テストガイド（`MANUAL_TEST_GUIDE.md`）に基づく品質・安定性・操作感の検証を開始。

---

## [0.9.0-alpha] - 2026-10-16 (Alpha Final)

### Added
- **Phase 4.8 高度エディタ機能 & 堅牢化の全面実装**:
  - **マルチタブ管理 (Tab Bar UI)**:
    - 独立した画面分割構造・カレントバッファ・カーソル位置を保持する `TabPage` 実装。
    - 2つ以上のタブオープン時に画面最上部にタブバーを自動描画し、アクティブタブを強調表示。
    - `gt`, `gT`, `[count]gt`, `:tabnew [file]`, `:tabclose` (`:tabc`), `:tabnext`, `:tabprev`, `:tabonly` の完全サポート。
  - **矩形選択一括編集 (Visual Block Operations)**:
    - `Ctrl-v` 矩形選択に対する `I<text>Esc`（左端一括挿入）、`A<text>Esc`（右端一括追加）、`c<text>Esc`（一括置換）、`r<char>`（一括塗りつぶし）、`d` / `x`（一括削除と文字詰め）を実装。
  - **設定ファイルと永続カスタマイズ (`config.toml` / `vivivirc`)**:
    - `%USERPROFILE%\.vivivi\config.toml`（または `~/.vivivi/config.toml`）からの自動設定ロード。
    - エディタ設定（`tab_stop`, `number`, `relative_number`, `gutter`, `hlsearch`, `clipboard`, `theme`）、不可視文字設定、キーマップ（Normal/Insert）の永続化。
    - 起動中の `:config` 再読込コマンド、および `:set number`, `:set relativenumber`, `:set tabstop`, `:set clipboard` コマンドの実装。
  - **文字コード・改行明示変換 (:e ++enc, :set fenc, :set ff)**:
    - `:e ++enc=sjis [file]`, `:e ++enc=euc-jp [file]`: 文字コードを指定した再読込。
    - `:set fenc=utf-8 / sjis / euc-jp`: ファイル保存文字コードの動的指定。
    - `:set ff=unix / dos`: 保存時の改行コード（LF / CRLF）変換指定。
  - **クラッシュリカバリ・スワップファイル (.swp)**:
    - 20打鍵ごとおよびEscキー復帰時におけるバックグラウンドスワップ自動保存（`.<filename>.swp` 形式）。
    - 異常終了後の次回起動時におけるスワップファイル検出と警告プロンプト表示。
    - `:recover`（未保存データの完全復元）および `:swap`（スワップ詳細情報確認）コマンドの実装。
    - 保存時および未変更終了時におけるスワップファイルの安全な自動消去。
  - **システム共有クリップボード連携 ("+y, "*p)**:
    - `arboard` クレートを用いた `SystemClipboard` プロバイダの実装。
    - `"+y`, `"*y` によるOSクリップボードへの直接ヤンク、`"+p`, `"*p` による直接貼り付け。
    - `:set clipboard=unnamed / unnamedplus / none` による通常ヤンク/プットとOSクリップボードの透過同期。
  - **シンタックスハイライト対応言語の拡充 (全18言語対応)**:
    - Go (`.go`), Ruby (`.rb`), TOML (`.toml`), PowerShell (`.ps1`, `.psm1`, `.psd1`), YAML (`.yaml`, `.yml`), XML (`.xml`), Windows Batch (`.bat`, `.cmd`) の構文解析トークナイザーとステータスバーバッジ表示を実装。
  - **内蔵ヘルプシステムの拡充**:
    - `doc/help/tab.md`, `doc/help/config.md`, `doc/help/recovery.md`, `doc/help/clipboard.md` を新設。
    - `doc/help/index.md`, `doc/help/syntax.md`, `doc/help/visual.md`, `doc/help/basic.md` を最新機能に同期。

### Changed / Refactored
- **大規模モジュール分割リファクタリング（肥大化ソースコードの責務分離・可読性向上）**:
  - 1,000行を超えて肥大化していた全モジュールを対象に、公開APIや機能の完全な後方互換性を維持したまま責務ごとにサブモジュールへ構造分割：
    - `vivivi-app/src/tui/app.rs` (3,440行) $\to$ `tui/app/` (`mod.rs`, `tabs.rs`, `windows.rs`, `files.rs`, `tools.rs`, `tests.rs`): アプリ状態管理、タブ制御、画面分割、ファイルI/O、内蔵ツールを完全分離。
    - `vivivi-app/src/tui/event.rs` (1,696行) $\to$ `tui/event/` (`mod.rs`, `modes.rs`, `ex.rs`): イベントループディスパッチ、モード別キーハンドラ、exコマンド実行ループを分離。
    - `vivivi-modes/src/syntax.rs` (1,563行) $\to$ `syntax/` (`mod.rs`, `keywords.rs`, `tests.rs`): 構文解析ロジックと言語キーワード定義テーブルを分離。
    - `vivivi-modes/src/keisen.rs` (1,240行) $\to$ `keisen/` (`mod.rs`, `draw.rs`, `protect.rs`, `tests.rs`): 罫線ステートマシン、描画ロジック、文字保護ロジックを分離。
    - `vivivi-vi/src/engine/mod.rs` (1,182行) $\to$ `engine/` (`mod.rs`, `exec.rs`): コア状態機械とコマンド実行ディスパッチを分離。
    - `vivivi-core/src/undo.rs` (960行) $\to$ `undo/` (`mod.rs`, `serialize.rs`, `tests.rs`): トランザクション管理本体とundofile永続化を分離。
  - テスト専用ファイルを除く全プロダクションコードで1,000行未満（最大800〜900行）を達成。全354件超のテスト合格。

---

## [0.8.0-alpha] - 2026-10-15

### Added
- **Vim互換ウィンドウ階層ツリーレイアウト (`window.rs`, `app.rs`)**:
  - 二分木構造 `WindowTreeNode` による再帰的分割管理。
  - 幾何学的近傍判定による `Ctrl-W h / j / k / l` および矢印キーでの直感的なウィンドウ移動。
  - ウィンドウごとの独立スクロールオフセットおよび独立バッファ表示。
- **表示幅（Visual Column）基準の水平スクロール (`editor.rs`, `app.rs`)**:
  - 全角文字（2幅）やタブ文字を含む80文字超長行での滑らかな自動追従とターミナル物理カーソル同期。
- **アウトライン表示の操作性向上 & キー操作統一 (`outline.rs`, `event.rs`)**:
  - `F2`（OpenViVi準拠）と `F8`（モダン準拠）の両対応、起動直後の自動フォーカス、`Tab`/`Esc` 相互切替。
  - 多階層見出し対応（H1〜H6）および言語自動判定（`detect_language_from_content`）、カーソル追従自動縦スクロール。
- **Vim互換 `Insert` キーによるモード切替・トグル (`event.rs`)**:
  - NormalモードでInsert、InsertモードでReplace、ReplaceモードでInsertへとトグル遷移。
- **Migemo（ローマ字日本語検索）独自MIT辞書の内蔵 & 約物記号対応 (`migemo.rs`)**:
  - クリーンな独自MIT辞書内蔵による約物記号（`kakko`, `yajirushi`, `maru`, `ten` 等）一括マッチ。
  - 外部公式SKK辞書の自動探索・透過マルチエンコーディング読み込み。
- **実機手動テストガイドの整備 (`docs/ja/MANUAL_TEST_GUIDE.md`, `docs/en/MANUAL_TEST_GUIDE.md`)**:
  - 画面分割、罫線保護、アウトライン、水平スクロール、Migemo、文字コード等の包括的テスト手順書を作成。

---

## [0.7.0-alpha] - 2026-10-14

### Added
- **TUI/UX編集・視覚支援機能 (Phase 4.6)**:
  - **変更行ガター強調表示 (Change Markers)**: 行番号右隣に未保存変更行（黄色の `▌`）および保存済み変更行（緑色の `▌`）を表示（`:set gutter`）。
  - **最近使ったファイル (MRU) & お気に入り (Favorites) モーダル**: `F7` または `:recent` で一覧表示、ワンキーオープン、お気に入り登録。
  - **不可視文字・空白文字の可視化**: `:set list` で全角スペース（赤色 `□`）、タブ（`▸---`）、改行記号（`$`）を表示。
- **罫線モード角結合アルゴリズムの刷新**:
  - 上下左右の隣接セルの接続状況を幾何学的に検査し、不要な突き出し成分を自動消去して綺麗な角（`┌`, `┐`, `└`, `┘`）を形成。
- **モード別カーソル形状連動 (Terminal Cursor Style)**:
  - Normal/Visual: ブロック（`SteadyBlock`）、Insert: 縦棒（`SteadyBar`）、Replace: 下線（`SteadyUnderScore`）。
- **置換モード (`R`) のTUI編集ハンドリング**:
  - カーソル下文字の上書き、行末自動拡張、Undo統合。
- **検索ハイライト (`hlsearch`) & 消去 (`:noh`, `Esc` 2回連打)**:
  - 一致箇所のリアルタイム色反転、`:noh` コマンド、ノーマルモードでのダブルEsc消去。
- **罫線モードのキー修飾キーガード**:
  - `Ctrl`/`Alt`（描画）または `Shift`（消去）時のみ罫線操作とし、素の矢印キーや `h/j/k/l` は通常カーソル移動として保護。

---

## [0.6.0-alpha] - 2026-10-14

### Added
- **ツール機能・ファイル管理 TUI 統合 (Phase 4.5)**:
  - **世代バックアップ (DiffLog / `.difflog`)**: 保存ごとの差分蓄積、任意世代復元、行別更新世代表示（簡易blame）。
  - **永続Undo (Persistent Undo / `undofile`)**: 終了後もUndoスタックを保持し次回起動時に自動復元。
  - **印刷プレビュー & 自己完結型 PDF 1.4 直接出力 (`print.rs`)**: A4ページ分割、プレビュー表示、PDF直接出力。
  - **テンプレート自動読み込み (`.template.<ext>`)**: 新規作成時のマクロ展開テンプレート自動挿入。
  - **コード内 ToDo 抽出 (`todo.rs`)**: ソース内コメントの ToDo/FIXME 自動収集と該当行ワンキージャンプ。
  - **コード折り畳みエンジン (`folding.rs`)**: インデント階層解析と開閉コマンド（`zc`, `zo`, `za`, `zM`, `zR`, `z<lvl>`）。
  - **プロジェクト内高速 Grep 検索 (`grep.rs`)**: 再帰走査、正規表現マッチ、QuickFixセッション（`:cnext`, `:cprev`）。
  - **2ペイン ディレクトリビュー (ファイラー / `filer.rs`)**: 左右独立ペイン、Tab切替、1キーファイル操作。
  - **プロジェクト管理 (`project.rs`)**: 統合ツリー管理、一括tags生成（`:maketags`）。

---

## [0.5.0-alpha] - 2026-10-14

### Added
- **式置換エンジン (`expr_subst.rs`)**:
  - `:s/pattern/expression/e` での動的数式評価（四則演算、`$0`〜`$9` キャプチャ展開、`$i`/`$n` 連番、`$line` 行番号）。
- **ViVi独自vi拡張コマンド・ナビゲーション (Phase 4.4)**:
  - **全角文字透過マッチ (`fftt.rs`)**: 半角英数記号入力時に対応する全角文字（括弧・数字・記号）へ自動マッチ。
  - **単語前後検索**: `*`, `#` 完全一致、`g*`, `g#` 部分一致、`g<` 前方一致、`g=` 後方一致。
  - **数値・英字増減**: `#+`, `##`, `#-` によるインクリメント/デクリメント、および `#!` による真偽値トグル。
  - **直前編集復帰**: `@`, `'@`, `` `@ `` による直前編集位置ジャンプ、および `=<char>` による前行文字列複製。
  - **マーク管理**: ローカル `ma`〜`mz`、永続グローバル `mA`〜`mZ`、ジャンプ `'a`、一覧 `:marks`。
  - **タグジャンプ**: `Ctrl-]`, `Ctrl-t`, `Ctrl-[`, `:tag`, `:tags`（`tags` ファイル＆AST構文解析フォールバック）。

---

## [0.4.0-alpha] - 2026-10-14

### Added
- **Gvim常用機能・実務レイアウト (Phase 4.1〜4.3)**:
  - **画面分割・マルチビュー**: `:split`, `:vsplit`, `:close`, `:only`, `Ctrl-w` ウィンドウキーバインド。
  - **BOX選択・矩形編集の完全対応**: `Ctrl-v` による矩形選択、ヤンク（`y`）、削除（`d`, `x`）、一括挿入（`I`, `A`）、スマート矩形ペースト。
  - **アウトライン解析サイドペイン**: Markdown見出しツリー、Rust関数の関数・構造体ツリー解析と即時ジャンプ。
  - **文字変換 ex コマンド**: `:%zenkaku`, `:%hankaku`, `:%hiragana`, `:%katakana`, `:%toupper`, `:%tolower` 等の一括変換。
  - **テキストアライン・リナンバ・行編集支援**: `:align <target>`, `:renumber [start] [step]`, `:insbeg`, `:insend`, `:begend`。
  - **高速入力・マクロ機能**:
    - 動的単語補完 (dabbrev / `Ctrl-l`, `Ctrl-m`)
    - 静的略語展開 (abbrev / `Ctrl-;`, `Alt-;`, `F4`, `:abbrev`)
    - ダイナミックマクロ (増井式リピート / `Alt-m`, `Ctrl-Alt-m`, `:dmacro`)
    - 字句編集予測 (LexEdit / `Ctrl-j`, `:lexedit`)
    - Zen-Coding (Emmet / `Ctrl-,`, `Alt-,`, `:zen`)
    - キーボードマクロ (`Ctrl-Alt-r` / `Ctrl-Alt-p`, `F9` / `F10`, `:macro`)
  - **BOX選択数値集計**: 矩形範囲の合計・平均・標準偏差計算（`:calc`, `:sum`, `:total`）。
  - **クリップボード履歴**: `F6` / `:cbhist` ダイアログ。
  - **内蔵ヘルプビューア**: `F1`, `:help [topic]`。

---

## [0.3.0-alpha] - 2026-10-13

### Added
- **TUIフロントエンドの実装 (`vivivi-app`)**:
  - `ratatui` (v0.29) と `crossterm` (v0.28) によるフルスクリーン端末制御。
  - 代替スクリーンとRawモード制御、パニック耐性復元フック。
  - エディタバッファ描画、リアルタイムステータスバー、コマンドライン入力UI。
- **文字コード・改行自動判別モジュール (`encoding.rs`)**:
  - BOM判定、UTF-8, Shift_JIS/CP932, EUC-JP, UTF-16LE/BE 対応。
  - CRLF, LF, CR 改行コードの自動判別と相互変換。
- **対応括弧探索モジュール (`paren.rs`)**:
  - `()`, `{}`, `[]`, `<>` の文字列・コメントスキップ探索、`%` ジャンプ。
- **基本シンタックスハイライト (`syntax.rs`)**:
  - Rust, C/C++, Python, JS, JSON のキーワード・型・文字列・コメント色分け。

---

## [0.2.0-alpha] - 2026-10-13

### Added
- **Viコマンドエンジン基礎 (`vivivi-vi`)**:
  - モード定義（Normal, Insert, Visual, VisualLine, CommandLine）。
  - カーソルモーション（`h/j/k/l`, `w/b/e`, `0/^/$`, `gg/G`）。
  - オペレータと削除・変更・ヤンク・貼り付け（`d/c/y/p/P`, `dd/yy/cc`, `D/C/Y`, `x/X`）。
  - レジスタ管理（無名レジスタ, `"a`〜`"z`, 大文字アペンド, 行/文字指向判定）。
  - パターン検索（`/`, `?`, `n`, `N`）および基本exコマンド（`:w`, `:q`, `:wq`, `:s`, `:%s`）。
  - ステートフルな `CommandParser`。
- **独自拡張モード基礎 (`vivivi-modes`)**:
  - Unicode罫線モード（`Ctrl+矢印` で描画・十字自動結合、`Shift+矢印` で消去）。
  - 罫線枠内文字列の整列アルゴリズム（左・中央・右揃え）。
  - インラインCSV/TSVモード（カラム幅自動整列、クォート透過編集、アクティブ行の生テキスト表示）。

---

## [0.1.0-alpha] - 2026-10-13

### Added
- **コアデータ構造基盤 (`vivivi-core`)**:
  - OpenViVi準拠の `GapBuffer<T>` 実装（$O(1)$ 局所編集・動的容量倍増・ギャップスライド）。
  - トランザクション対応 `UndoMgr` 実装（Undoブロック、連続タイピング自動マージ、保存点追跡）。
  - `GapBuffer` と `UndoMgr` を統合した `TextBuffer` 実装。

---

## [0.0.1-alpha] - 2026-10-13

### Added
- **初期開発環境整備**:
  - マルチクレートワークスペース構成の整備（`vivivi-core`, `vivivi-vi`, `vivivi-modes`, `vivivi-app`）。
  - エージェント運用基盤（`.agents/`, `GEMINI.md`, `RULES.md`）の導入。
  - 日英バイリンガルドキュメント体系の立ち上げ（`docs/ja/`, `docs/en/`）。
