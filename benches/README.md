# vivivi Benchmarks (性能測定・極限ベンチマーク置き場)

本ディレクトリは、`vivivi` のデータ構造や編集アルゴリズムの処理速度を、ナノ秒（$10^{-9}$秒）単位で厳密に測定・グラフ化するための **ベンチマーク（Benchmarks）** を配置するための将来用ディレクトリです（Phase 5 導入予定）。

---

## 📌 今後の追加予定ベンチマーク

| ベンチマーク名 | 測定対象 | 比較・検証内容 |
| :--- | :--- | :--- |
| `gap_buffer_bench.rs` | `vivivi-core` | 100万文字/10万行のテキストに対するランダム挿入・削除における、標準 `Vec<char>` と `GapBuffer` の速度比較（$O(1)$ vs $O(N)$） |
| `undo_bench.rs` | `vivivi-core` | 1万ステップのトランザクションUndo/Redo巻き戻し性能とメモリ消費量測定 |
| `csv_bench.rs` | `vivivi-modes` | 10万行の巨大CSV/TSVファイルに対するインライン列幅計算・クォート透過解析の処理速度 |
| `migemo_bench.rs` | `vivivi-modes` | 大規模バッファに対するMigemoローマ字日本語インクリメンタル検索の応答レイテンシ |

---

## 🛠️ 推奨フレームワーク（Criterion.rs）

Rustの標準ベンチマークハーネス、または統計的解析・HTMLグラフ出力に対応した **`criterion`** クレートを採用予定です。

```toml
# 今後 Cargo.toml に追加予定の依存設定例
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "gap_buffer_bench"
harness = false
```

---

## 🚀 実行コマンド（追加後の実行方法）

```powershell
# ワークスペース全体のベンチマーク実行
cargo bench

# 特定のベンチマークのみ実行
cargo bench --bench gap_buffer_bench
```
