# vivivi Examples (実例・サンプルコード置き場)

本ディレクトリは、`vivivi` のコア機能や特殊モードのロジックを、TUIエディタ全体を起動せずに単体で試せる **最小実行可能サンプルコード（Examples）** を配置したディレクトリです。

---

## 📌 配備済みサンプル一覧

| サンプル名 (ファイル名) | 対象クレート   | 目的・内容                                                                     | 実行コマンド                                 |
| :---------------------- | :------------- | :----------------------------------------------------------------------------- | :------------------------------------------- |
| `gap_buffer_demo.rs`    | `vivivi-core`  | `GapBuffer` の $O(1)$ 局所編集とメモリ移動の挙動を標準出力で可視化するデモ     | `cargo run --example gap_buffer_demo`        |
| `keisen_combine.rs`     | `vivivi-modes` | Unicode 罫線の十字自動結合・交差判定ロジックをコンソールで検証するミニツール   | `cargo run --example keisen_combine`         |
| `inline_csv_align.rs`   | `vivivi-modes` | カンマ・タブ区切りテキストを瞬時にインライン整列して出力するスタンドアロンCLI  | `cargo run --example inline_csv_align`       |
| `dabbrev_runner.rs`     | `vivivi-modes` | 動的単語補完（dabbrev）の前後方探索アルゴリズムの単体動作確認                  | `cargo run --example dabbrev_runner`         |

---

## 🚀 実行コマンド

```powershell
# 特定の example を実行するコマンド例
cargo run --example gap_buffer_demo

# Unicode 罫線自動結合の検証
cargo run --example keisen_combine

# CSV/TSV のインライン整列
cargo run --example inline_csv_align

# 動的単語補完 (dabbrev) の動作確認
cargo run --example dabbrev_runner
```
