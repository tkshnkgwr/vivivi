---
name: rust-verification
description: >-
  Use this skill when verifying changes to Rust source code in the vivivi project,
  running cargo check/test/clippy/fmt, or determining whether verification can be skipped for markdown/documentation-only edits.
---

# Rust Verification & Quality Check Workflow

本スキルは、`vivivi` におけるコード変更時の品質検証（ビルド、単体テスト、静的解析）およびドキュメント変更時の事前検証省略基準を定めた手順書です。

## 1. 検証省略の判断基準（迅速対応）

- **対象が Markdown のみ (`*.md` のみ)**:
  - Rust のビルドやテスト（`cargo check`, `cargo test` 等）は **すべて省略** し、迅速に応答・反映します。
- **対象にソースコード (`*.rs` 等) や設定 (`Cargo.toml`) が含まれる場合**:
  - 以下の検証手順を必ず実行します。

## 2. コード検証手順

コード変更後、以下のコマンドを順次実行してエラーや警告がないことを確認します。

```powershell
# 1. 高速構文・型チェック
cargo check --all-targets

# 2. 単体テスト実行（全テスト通過を確認）
cargo test

# 3. Clippy 静的リント解析（警告ゼロを担保）
cargo clippy --all-targets -- -D warnings

# 4. コードフォーマット検証
cargo fmt --check
```

## 3. トラブルシューティング & 修正手順

- **フォーマット違反 (`cargo fmt --check` 失敗時)**:
  - `cargo fmt` を実行して自動整形を適用します。
- **テスト・リントエラー時**:
  - エラー出力を確認し、該当コードまたはテストコードを修正して再検証します。
