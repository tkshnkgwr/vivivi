# vivivi Benchmarks (性能測定・極限ベンチマークスイート)

本ディレクトリおよび各クレートの `benches/` は、`vivivi` のデータ構造や編集アルゴリズムの処理速度を、ナノ秒（$10^{-9}$秒）〜マイクロ秒（$10^{-6}$秒）単位で統計的に測定・グラフ化する **Criterion ベンチマークスイート** です。

---

## 📌 実装済みベンチマーク一覧

| ベンチマークターゲット | 対象クレート   | 測定シナリオ・検証内容                                                                                   |
| :--------------------- | :------------- | :------------------------------------------------------------------------------------------------------- |
| `core_bench`           | `vivivi-core`  | 10万文字テキスト中央での1,000文字連続挿入における `GapBuffer` vs 標準 `String` の速度比較（約8.3倍高速） |
| `core_bench`           | `vivivi-core`  | `UndoMgr` における1,000ステップ連続編集・ブロック確定・一括巻き戻しのスループット測定                    |
| `core_bench`           | `vivivi-core`  | 文字コード（UTF-8 / Shift_JIS / EUC-JP）および改行コードの超高速早期自動判別レイテンシ測定               |
| `modes_bench`          | `vivivi-modes` | 1,000行の RFC 4180 CSV データに対するインライン列幅計算およびクォート透過フォーマット速度測定            |
| `modes_bench`          | `vivivi-modes` | Migemo ローマ字日本語検索における正規表現パターン展開および大規模テキスト走査レイテンシ測定              |
| `modes_bench`          | `vivivi-modes` | Unicode 罫線（U+2500〜U+254B）の十字・T字・角自動結合アルゴリズムおよび行末空白トリミング速度測定        |

---

## 🛠️ 採用フレームワーク（Criterion.rs）

統計的回帰分析および HTML / SVG レポート出力に対応した **`criterion = "0.5"`** をワークスペース標準として採用しています（測定結果は `target/criterion/` に出力されます）。

---

## 🚀 実行コマンド

```powershell
# ワークスペース全体のベンチマーク実行
cargo bench --workspace

# コアデータ構造（GapBuffer / UndoMgr / Encoding）のベンチマーク実行
cargo bench -p vivivi-core --bench core_bench

# 拡張モード（CSV / Migemo / Keisen / Trim）のベンチマーク実行
cargo bench -p vivivi-modes --bench modes_bench
```
