# vivivi: テスト実行報告書 (TEST_REPORT)

本書は、`vivivi` ワークスペースの単体テストおよび静的解析の実行結果を記録したレポートです。

---

## 1. 総合検証サマリー

- **実行日時**: 2026-10-29
- **Rust バージョン**: rustc 1.96.1 / Cargo 1.96.1 (2024 edition)
- **総合結果**: **全テスト合格 (613 passed, 0 failed) / Clippy 警告 0 件 / フォーマット合格**

---

## 2. クレート別テスト結果

| クレート名     | テスト数 |  成功   | 失敗  |  状態   | 主な検証内容                                                                                                   |
| :------------- | :------: | :-----: | :---: | :-----: | :------------------------------------------------------------------------------------------------------------- |
| `vivivi-core`  |   114    |   114   |   0   | ✅ PASS | GapBuffer局所編集、Undoブロック・マージ、undofile永続化、文字コード/改行判別、括弧探索、doctest群              |
| `vivivi-vi`    |   123    |   123   |   0   | ✅ PASS | モーション、オペレータ、矩形選択編集、レジスタ履歴、マクロ、ex置換、式置換、引用符オブジェクト、doctest群      |
| `vivivi-modes` |   303    |   303   |   0   | ✅ PASS | RFC 4180 CSV、罫線全角結合、文字変換、BOX集計、アライン、Migemo、DIFF高速化、DiffLog、Folding、Grep、doctest群 |
| `vivivi-app`   |    57    |   57    |   0   | ✅ PASS | 二分木画面分割ツリー、マルチタブ、独立スクロール、未保存警告ガード、Ctrl+[全モード対応、doctest群              |
| `vivivi-gui`   |    16    |   16    |   0   | ✅ PASS | Tokyo Nightテーマ、等幅カスタム描画、IME Preedit連携、ミニマップ、削除文字落下パーティクル、doctest群          |
| **合計**       | **613**  | **613** | **0** | ✅ PASS | **全項目合格 (613 passed, 0 failed)**                                                                          |

---

## 3. 静的解析 (Clippy) & フォーマット結果

- **Clippy**: `cargo clippy --all-targets -- -D warnings` $\to$ **エラー・警告 0 件 (Clean)**
- **rustfmt**: `cargo fmt --check` $\to$ **フォーマット違反 0 件 (Clean)**
