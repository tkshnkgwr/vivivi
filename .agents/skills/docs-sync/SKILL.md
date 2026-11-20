---
name: docs-sync
description: >-
  Use this skill when updating, auditing, or synchronizing project documentation (docs/ja, docs/en, README)
  after refactoring, module splitting, feature additions, or configuration/command changes in the vivivi project.
---

# Documentation Maintenance & Bilingual Sync Workflow

本スキルは、`vivivi` におけるリファクタリング・ファイル分割・新機能追加・設定変更などの後に、コードベースとドキュメント（`docs/ja/`・`docs/en/`・`README.md`・`README_JA.md`）の整合性を保つための整備手順書です。

> [!IMPORTANT]
> **Rust ビルド／検証の完全省略**:
> ドキュメント（`*.md`）のみを更新する場合は、`cargo check`、`cargo build`、`cargo test`、`cargo clippy`、`cargo fmt` は **すべて不要（実行しない）** です（[RULES.md](../../../RULES.md) 第4項準拠）。

---

## 1. コード変更とドキュメントの乖離スキャン

まず高速 CLI ツール（`git diff` / `fd` / `rg`）を用いて、コード側の最新状態とドキュメント間のズレを洗い出します。

```powershell
# 1. 直近のファイル構成・変更差分の確認
git status -s
git log -n 5 --stat

# 2. 現在のクレート・モジュール構成の一覧取得
fd -e rs . crates/

# 3. 削除・改名された旧ファイルパスがドキュメントに残っていないか検索
rg -n "旧ファイル名や旧シンボル名" docs/ README.md README_JA.md
```

---

## 2. 変更種別と更新対象ドキュメントの対応表

コードや設定の変更内容に応じて、該当する日英ドキュメントを対で更新します。

| 変更種別                                 | 主な更新対象ドキュメント（日・英ペア）                                           |
| :--------------------------------------- | :------------------------------------------------------------------------------- |
| クレート・モジュール構成変更、大規模分割 | `docs/{ja,en}/ARCHITECTURE.md`, `GUI_DESIGN.md`, `DEVELOPMENT.md`                |
| `config.toml`・GUI設定ダイアログ項目追加 | `docs/{ja,en}/SETTINGS_DESIGN.md`, `docs/ja/settings/*.md`, `USER_GUIDE.md`      |
| Exコマンド・キーバインド・操作仕様の追加 | `docs/{ja,en}/SPECIFICATION.md`, `USER_GUIDE.md`, `MANUAL_TEST_GUIDE.md`         |
| タスク進捗の完了・新規タスク発生         | `docs/{ja,en}/TODO.md`                                                           |
| バージョン更新・リリース相当の変更集約   | `docs/{ja,en}/CHANGELOG.md`, `README.md`, `README_JA.md`                         |
| テスト体系・新規テストシナリオの追加     | `docs/{ja,en}/TESTING.md`, `TEST_REPORT.md`                                      |

---

## 3. 記述・品質チェックリスト（`RULES.md` 準拠）

編集時および保存前に、以下を必ず確認してください。

- [ ] **日英バイリンガル同期**: `docs/ja/` を更新したら対応する `docs/en/`（および `README_JA.md` と `README.md`）も同時に更新したか。
- [ ] **Markdown テーブル垂直整列**: 作成した／編集した表組みのパイプ `|` とハイフン `-` が [markdown-table スキル](../markdown-table/SKILL.md) に沿って垂直整列されているか。
- [ ] **ローカル絶対パスの環境変数化**: `C:\Users\<名前>\...` のような生パスを含めず、`%USERPROFILE%`、`$HOME`、またはリポジトリ内相対パスを使用しているか。
- [ ] **ファイルリンク・存在確認**: 記載したファイルパスやモジュールパスが現在の `crates/` 構造に実在し、リンク切れがないか。
- [ ] **事実ベース記述**: 実装されていない挙動や未確認の数値を推測で書かず、コード・実測に基づく正確な内容になっているか。

---

## 4. 整備完了後の簡易検証（Markdown 専用）

Rust ツールチェーンは実行せず、テキスト検索のみでドキュメント品質を検証します。

```powershell
# 1. 生のローカル絶対パス（C:\Users\ 等）が混入していないか検査
rg -n -i "C:\\Users\\|C:/Users/" docs/ README.md README_JA.md

# 2. 削除・移動済みの旧ファイル参照が残っていないか検査
rg -n "確認したい旧パス" docs/ README.md README_JA.md
```
