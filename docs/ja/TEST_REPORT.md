# vivivi: テスト実行報告書 (TEST_REPORT)

本書は、`vivivi` ワークスペース（全6クレート）の単体テスト、公開API統合テスト、ドキュメンテーションテスト、および静的解析の実行結果を記録したレポートです。

---

## 1. 総合検証サマリー

- **実行日時**: 2026-11-11
- **対象バージョン**: `v0.9.17-beta`
- **Rust バージョン**: rustc 1.96.1 / Cargo 1.96.1 (2024 edition)
- **総合結果**: **全テスト合格 (724 passed, 0 failed) / Clippy 警告 0 件 / Rustdoc 警告 0 件 / フォーマット合格**

---

## 2. クレート別テスト結果

| クレート名     | テスト数 |  成功   | 失敗  | 状態 | 主な検証内容                                                                                                                    |
| :------------- | :------: | :-----: | :---: | :--: | :------------------------------------------------------------------------------------------------------------------------------ |
| `vivivi-core`  |   130    |   130   |   0   | PASS | GapBuffer局所編集、Undoブロック・マージ、undofile永続化、文字コード/改行早期判別、括弧探索、全99件doctest                       |
| `vivivi-vi`    |    90    |   90    |   0   | PASS | モーション、オペレータ、矩形選択編集、レジスタ履歴、マクロ、ex置換、式置換、引用符オブジェクト、全46件doctest                   |
| `vivivi-modes` |   386    |   386   |   0   | PASS | RFC 4180 CSV、罫線結合、文字変換、BOX集計、Migemo、DIFF、Folding、Grep、Trim、Hex Editor、LSP、Tree-sitter AST、全216件doctest  |
| `moult-update` |    5     |    5    |   0   | PASS | GitHub Releases semver比較、SHA-256チェックサム解析、ZIP展開、Windowsインプレース・スワップ＆クリーンアップ、doctest            |
| `vivivi-app`   |    76    |   76    |   0   | PASS | 二分木画面分割、マルチタブ、Windowsキー、外部プロセス(:r!, :[range]!)、Hex/LSP/Tree-sitter Exコマンド、TUI設定モーダル、doctest |
| `vivivi-gui`   |    37    |   37    |   0   | PASS | 5大テーマ、等幅グリッド描画、IME Preedit保護、ミニマップ、落下パーティクル、2ペイン設定、2D力学グラフ、Hex/LSP/AST統合、doctest |
| **合計**       | **724**  | **724** | **0** | PASS | **全項目合格 (724 passed, 0 failed)**                                                                                           |

---

## 3. 静的解析 (Clippy) & ドキュメント & フォーマット結果

- **Clippy**: `cargo clippy --workspace --all-targets` $\to$ **エラー・警告 0 件 (Clean)**
- **Rustdoc**: `cargo doc --workspace --no-deps` $\to$ **リンク切れ・構文警告 0 件 (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **フォーマット違反 0 件 (Clean)**
