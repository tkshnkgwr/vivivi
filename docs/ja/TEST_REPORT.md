# vivivi: テスト実行報告書 (TEST_REPORT)

本書は、`vivivi` ワークスペース（全6クレート）の単体テスト、公開API統合テスト、ドキュメンテーションテスト、および静的解析の実行結果を記録したレポートです。

---

## 1. 総合検証サマリー

- **実行日時**: 2026-11-20
- **対象バージョン**: `v0.9.22-beta`
- **Rust バージョン**: rustc 1.100.0 / Cargo 1.100.0 (2024 edition)
- **総合結果**: **全テスト合格 (774 passed, 0 failed / vivivi-app内 46 ignored doctests) / Clippy 警告 0 件 / Rustdoc 警告 0 件 / フォーマット合格**

---

## 2. クレート別テスト結果

| クレート名     | テスト数 |  成功   | 失敗  | 状態 | 主な検証内容                                                                                                                             |
| :------------- | :------: | :-----: | :---: | :--: | :--------------------------------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   130    |   130   |   0   | PASS | GapBuffer局所編集、Undoブロック・マージ、undofile永続化、文字コード/改行早期判別、括弧探索、全99件doctest                                |
| `vivivi-vi`    |    97    |   97    |   0   | PASS | モーション、オペレータ、矩形選択編集、レジスタ履歴、マクロ、ex置換、式置換、引用符オブジェクト、`:sort`、カウント飽和、全50件doctest     |
| `vivivi-modes` |   392    |   392   |   0   | PASS | RFC 4180 CSV、罫線結合、文字変換、BOX集計、Migemo、LCS Diff、Folding、Grep、Trim、Hex Editor、LSP、Tree-sitter AST、全218件doctest       |
| `moult-update` |    5     |    5    |   0   | PASS | GitHub Releases semver比較、SHA-256チェックサム解析、ZIP展開、Windowsインプレース・スワップ＆クリーンアップ、1件doctest                  |
| `vivivi-app`   |    91    |   91    |   0   | PASS | 二分木画面分割、マルチタブ、Windowsキー、外部プロセス、`:version`、カーソル設定、パニックログ、Hex/LSP/AST統合、4件doctest (46件 ignore) |
| `vivivi-gui`   |    59    |   59    |   0   | PASS | 5大テーマ、等幅グリッド・分割ペイン・ミニマップ・サイドバー、IME Preedit、落下パーティクル、2ペイン設定、2D力学グラフ、全15件doctest     |
| **合計**       | **774**  | **774** | **0** | PASS | **全項目合格 (774 passed, 0 failed / 46 ignored)**                                                                                       |

---

## 3. 静的解析 (Clippy) & ドキュメント & フォーマット結果

- **テスト**: `cargo test -j 4 --workspace --all-targets` / `cargo test -j 4 --workspace --doc` $\to$ **774 passed, 0 failed (46 ignored)**
- **Clippy**: `cargo clippy -j 4 --workspace --all-targets -- -D warnings` $\to$ **エラー・警告 0 件 (Clean)**
- **Rustdoc**: `cargo doc -j 4 --no-deps --workspace` $\to$ **リンク切れ・構文警告 0 件 (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **フォーマット違反 0 件 (Clean)**
