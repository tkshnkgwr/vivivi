# vivivi Examples (実例・サンプルコード置き場)

本ディレクトリは、`vivivi` のコア機能や特殊モードのロジックを、TUIエディタ全体を起動せずに単体で試せる **最小実行可能サンプルコード（Examples）** を配置するための将来用ディレクトリです。

---

## 📌 今後の追加予定サンプル

| サンプル名 (ファイル名) | 対象クレート | 目的・内容 |
| :--- | :--- | :--- |
| `gap_buffer_demo.rs` | `vivivi-core` | `GapBuffer` の $O(1)$ 局所編集とメモリ移動の挙動を標準出力で可視化するデモ |
| `keisen_combine.rs` | `vivivi-modes` | Unicode罫線の十字自動結合・交差判定ロジックをコンソールで検証するミニツール |
| `inline_csv_align.rs` | `vivivi-modes` | カンマ区切りテキストを瞬時にインライン整列して出力するスタンドアロンCLI |
| `dabbrev_runner.rs` | `vivivi-modes` | 動的単語補完（dabbrev）の前後方探索アルゴリズムの単体動作確認 |

---

## 🚀 実行コマンド（追加後の実行方法）

```powershell
# 特定の example を実行するコマンド例
cargo run --example gap_buffer_demo

# リリースビルドで高速実行
cargo run --release --example inline_csv_align
```
