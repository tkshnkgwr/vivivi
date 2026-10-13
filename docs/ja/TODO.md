# vivivi: 開発ロードマップ & タスク一覧 (TODO)

## 1. 完了済みタスク (Completed)
- [x] 初期開発環境・マルチクレートワークスペース構成の整備
- [x] `vivivi-core`: OpenViVi準拠の `GapBuffer<T>` 実装
- [x] `vivivi-core`: トランザクション対応 `UndoMgr` 実装（Undoブロック・連続タイピングマージ・Save Point）
- [x] `vivivi-core`: `GapBuffer` と `UndoMgr` を統合した `TextBuffer` 実装
- [x] エージェント運用基盤（`.agents/`, `GEMINI.md`, `RULES.md`）の配備
- [x] 日英バイリンガルドキュメント体系の構築（`docs/ja/`, `docs/en/`）

---

## 2. 進行中・予定タスク (Upcoming)

### フェーズ 1: Viコマンドエンジンの本格移植 (`vivivi-vi`)
- [x] 基本カーソルモーション (`h`, `j`, `k`, `l`, `w`, `b`, `e`, `0`, `$`, `^`, `gg`, `G`)
- [x] オペレータと削除・変更・ヤンク (`d`, `c`, `y` + motion / `dd`, `yy`, `cc`)
- [x] 貼り付け (`p`, `P`) とレジスタ管理 (`"a`〜`"z`, クリップボード)
- [x] 検索コマンド (`/`, `?`, `n`, `N`)
- [x] 基本exコマンド (`:w`, `:q`, `:wq`, `:s`, `:%s`)

### フェーズ 2: 独自拡張モードの実装 (`vivivi-modes`)
- [x] 罫線モードのキーハンドリング（`Ctrl+矢印` で描画・自動結合、`Shift+矢印` で消去）
- [x] 罫線内文字列の左右・中央揃えアルゴリズム
- [x] インラインCSV/TSVモードのカラム整列ビュー計算
- [x] CSVモードでのクォート透過編集とカーソル行生表示

### フェーズ 3: フロントエンドの実装 (`vivivi-app`)
- [ ] ターミナルUI（Ratatui + crossterm）フロントエンドの先行実装（対話的Vi操作・リアルタイム描画）
- [ ] ファイルI/Oと文字コード自動判別（Shift_JIS, UTF-8, EUC-JP）の統合
- [ ] デスクトップGUI（egui / eframe）フロントエンドの実装（リッチなグラフィック罫線・フォントレンダリング）
