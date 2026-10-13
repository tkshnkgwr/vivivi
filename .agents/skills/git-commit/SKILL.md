---
name: git-commit
description: >-
  Use this skill when the boss (user) explicitly instructs to commit or push changes to Git in the vivivi project.
  Guides commit preparation, Japanese message formatting, and safe git execution.
---

# Safe Git Commit & Push Workflow

本スキルは、ボスから明示的に「コミットして」「Pushして」と指示された際にのみ発火し、安全かつ規約に沿って Git コミット・Push を行うための手順書です。

> [!CAUTION]
> **自動コミットの厳禁**:
> AI の自律判断で git add, git commit, git push を行うことは RULES.md で固く禁じられています。
> 本スキルは、ボスから直接指示があった場合にのみ手順に従って実行してください。

## 1. コミット前チェックリスト

コミットを実行する前に、以下を必ず確認します。

1. **変更ファイルの確認**: `git status` で意図しないファイル（一時ファイルや不要なログ等）が含まれていないか確認。
2. **差分の確認**: `git diff` で変更内容が過不足なく安全であることを確認。
3. **コード検証**: ソースコード（`*.rs`）に変更がある場合、`rust-verification` スキルに沿ってテスト・フォーマット・Clippyが通過しているか確認。

## 2. コミットメッセージ規約

コミットメッセージは以下の形式で記述します。

```text
<prefix>: <日本語による明確な変更要約>

[必要に応じて詳細な説明（日本語）]
```

### プレフィックス一覧
- `feat:` 新機能・新モジュールの追加
- `fix:` バグ修正
- `docs:` ドキュメントのみの変更
- `refactor:` リファクタリング（機能変更なし）
- `test:` テストの追加・修正
- `chore:` ビルド構成、CI、雑務

### 記入例
- `feat: OpenViVi準拠のGapBufferとトランザクション対応UndoMgrの実装`
- `docs: ギャップバッファおよびアンドゥマネージャ詳細仕様書の作成`
- `fix: 連続入力マージ時の空ブロック誤プッシュを修正`

## 3. コミット & Push 手順

```powershell
# 1. 対象ファイルをステージング
git add <対象ファイル>

# 2. 規約に沿った日本語メッセージでコミット
git commit -m "<prefix>: <要約>"

# 3. ボスから Push の指示もある場合のみ Push 実行
git push
```
