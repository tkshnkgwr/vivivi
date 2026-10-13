---
name: vault-memo
description: >-
  Use this skill when the user asks to save discussion, design ideas, or technical notes
  into Obsidian Vault (e.g., "一時メモにして", "Vaultにメモして", "これ一時メモにして", "Vaultにメモ").
---

# Vault 一時メモ保存スキル (vault-memo)

vivivi プロジェクトでの対話において、有益な知見・設計アイデア・検討ログが出た際に、Obsidian Vault の受信箱（`01.FleetingNote`）へ一時メモとして自動保存するためのスキルです。

---

## 🎯 トリガーフレーズ

以下の指示があった際に自律的に適用されます：
- **「一時メモにして」** / **「これ一時メモにして」**
- **「Vaultにメモして」** / **「Vaultにメモ」**
- （スラッシュコマンド: `/vault-memo`）

---

## 📋 実行プロトコル

### ステップ 1: 会話内容の抽出・構造化
1. **対象範囲の判定**:
   - ボスが「ここでの○○に関して」と特定テーマを指定した場合はその内容。
   - 指定がない場合は、直前の対話で議論・決定された技術的知見（GapBufferやUndoMgrの構造、OpenViVi再構築の設計等）を網羅的に抽出する。
2. **手厚い情報密度の確保**:
   - 単なる数行の要約で終わらせず、背景・文脈・具体的なコード片やコマンド・検討した理由をしっかり残し、後からVault側で読み返した際に当時の文脈が完全に復元できる「骨太な一時メモ」とする。

### ステップ 2: 一時メモテンプレートの適用
Vaultの `ZZ.TemplateNote/一時メモテンプレート.md` の仕様に準拠し、以下のフォーマットで作成する：

```markdown
---
title: "<決定したタイトル>"
date: YYYY-MM-DD
tags:
  - AI
  - memo
  - <トピックに応じたタグ（例: rust, vivivi, openvivi, gapbuffer, architecture 等）>
---

# <タイトル>

## 1. 概要・発端
<背景や問題意識、議論の発端>

## 2. 検討内容・技術的詳細
<具体的な設計、コード片、比較検討、分かったこと>

## 3. 決定事項・今後の課題
<結論、採用した方針、次のアクション>
```

### ステップ 3: ファイルの保存
- **保存先パス**:
  - `%USERPROFILE%\Documents\ObsidianVault\01.FleetingNote\<タイトル>.md`
  - ※ 環境変数 `%USERPROFILE%`（または `$env:USERPROFILE`）を展開し、絶対パスで保存する。
- **ファイル命名規則**:
  - 内容が一目でわかる具体的かつ簡潔な名前にする（例: `YYYYMMDD_<トピック名>.md` または `<トピック名>.md`）。

### ステップ 4: ボスへの完了報告
- 保存したファイルのパス（ファイル名）と、記録した主要要点を簡潔に報告する。
