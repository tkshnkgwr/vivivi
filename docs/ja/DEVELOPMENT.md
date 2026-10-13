# vivivi: 開発者ガイド (DEVELOPMENT)

本書は、`vivivi` のローカル開発環境のセットアップ、ワークスペースビルド、テスト実行、およびコード品質検証手順について説明します。

---

## 1. 開発環境のセットアップ

### 前提条件
- **Rust ツールチェーン**: Rust 1.80 以上 (最新 stable 推奨、Edition 2024 対応)
- **Git**: バージョン管理

```powershell
# ツールチェーンの確認
rustc --version
cargo --version
```

### リポジトリのクローン
```powershell
git clone https://github.com/tkshnkgwr/vivivi.git
cd vivivi
```

---

## 2. ワークスペースのビルド & 実行

```powershell
# 1. ワークスペース全クレートのチェック
cargo check --all-targets

# 2. アプリケーションの実行
cargo run

# 3. リリースビルド
cargo build --release
```

---

## 3. テスト & 品質検証

コード変更時は以下の手順で品質検証を行います。

```powershell
# 全クレートの単体テスト実行
cargo test

# Clippy 静的リント解析（警告ゼロを担保）
cargo clippy --all-targets -- -D warnings

# フォーマット検証
cargo fmt --check
```
