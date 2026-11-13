---
name: rust-verification
description: >-
  Use this skill when verifying changes to Rust source code in the vivivi project,
  running cargo check/test/clippy/fmt, or determining whether verification can be skipped for markdown/documentation-only edits.
---

# Rust Verification & Quality Check Workflow

本スキルは、`vivivi` におけるコード変更時の品質検証（ビルド、単体テスト、静的解析）およびドキュメント変更時の事前検証省略基準を定めた手順書です。

## 1. 検証省略・軽量化・並列数制御の基準

- **対象が Markdown のみ (`*.md` のみ)**:
  - Rust のビルドやテスト（`cargo check`, `cargo test` 等）は **すべて省略** し、迅速に応答・反映します。
- **並列数の標準基準（マシン負荷抑制）**:
  - ユーザー（ボス）から別段の指定がない限り、すべての Cargo ビルド／チェック／テスト／Clippy コマンドには **`-j 4`** を付与して並列数を制限します（Pコアや16GBメモリを圧迫しないための標準設定）。
  - **「フルコアでBuildして」等の全コア指示がある場合**: `-j 4` を外して Cargo 既定の全論理コア並列で実行します。
  - **`-j 2` / `-j 6` 等を明示指定された場合**: 指定値を優先します。
- **軽量チェック指示（「チェックだけにして」「軽めで」等がある場合）**:
  - テスト・Docテスト・Clippy は省略し、`cargo fmt` と `cargo check -j 4`（変更が単一クレートに閉じる場合は `-p <クレート名>` 指定可）のみを実行します。
- **通常のソースコード (`*.rs`)・設定 (`Cargo.toml`) 変更時**:
  - 下の手順を実行します（`Cargo.toml` が `default-members` を絞っているため、ワークスペース全体を対象にする場合は `--workspace` を付与します）。

## 2. コード検証手順

コード変更後、標準時は **`-j 4`** 付きで以下のコマンドを順次実行してエラーや警告がないことを確認します（フルコア指示時は `-j 4` を省略）。

```powershell
# 1. フォーマット整形 & 高速構文・型チェック
cargo fmt
cargo check -j 4 --workspace --all-targets

# 2. テスト実行（内部単体テスト・公開API統合テスト(tests/)・Doc-testの全通過を確認）
cargo test -j 4 --workspace --all-targets
cargo test -j 4 --workspace --doc

# 3. Clippy 静的リント解析（警告ゼロを担保）
cargo clippy -j 4 --workspace --all-targets -- -D warnings

# 4. コードフォーマット検証
cargo fmt --check
```

## 3. トラブルシューティング & 修正手順

- **フォーマット違反 (`cargo fmt --check` 失敗時)**:
  - `cargo fmt` を実行して自動整形を適用します。
- **テスト・リントエラー時**:
  - エラー出力を確認し、該当コードまたはテストコードを修正して再検証します。
