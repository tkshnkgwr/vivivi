---
name: markdown-table
description: >-
  Use this skill when creating or editing Markdown tables in documents or notes for the vivivi project,
  ensuring strict vertical alignment of pipes and hyphens for plain-text editors like gVim.
---

# Markdown Table Vertical Alignment Guide

本スキルは、Markdown 内でテーブル（表組み）を作成・編集する際に、プレビュー画面だけでなく生テキスト（gVim 等）で閲覧した際にも一目で構造がわかるよう、文字幅を考慮した垂直整列を行うための手順書です。

## 1. 整列ルール

- **パイプ `|` とハイフン `-` の位置を完全に垂直整列** させます。
- 全角文字（日本語など）は環境によって半角2文字分として扱われる場合が多いため、視覚的な揃えやすさを考慮して適切にパディングします。
- ヘッダー区切り行のハイフン `-` は、列幅に合わせて十分な数を配置します。

## 2. 良い例と悪い例

### 良い例（整然と垂直整列されている）
```markdown
| クレート名     | 役割・責務                                                   | 依存クレート  |
| :------------- | :----------------------------------------------------------- | :------------ |
| `vivivi-core`  | テキストバッファ、行管理、カーソル、Undo/Redo、エンコーディング | なし          |
| `vivivi-vi`    | Vi状態機械、コマンドパーサー、レジスタ、exコマンド           | `vivivi-core` |
| `vivivi-modes` | インラインCSV/TSVモード、Unicode罫線描画・結合モード         | `vivivi-core` |
| `vivivi-app`   | アプリケーション本体・実行エントリポイント                   | 全クレート    |
```

### 悪い例（列位置がバラバラで生テキストで構造が掴めない）
```markdown
| クレート名 | 役割・責務 | 依存クレート |
| --- | --- | --- |
| `vivivi-core` | テキストバッファ、行管理、カーソル、Undo/Redo、エンコーディング | なし |
| `vivivi-vi` | Vi状態機械、コマンドパーサー、レジスタ、exコマンド | `vivivi-core` |
| `vivivi-modes` | インラインCSV/TSVモード、Unicode罫線描画・結合モード | `vivivi-core` |
| `vivivi-app` | アプリケーション本体・実行エントリポイント | 全クレート |
```

## 3. 作成・編集後のチェックリスト

- [ ] 生テキスト（エディタ）で開いた際に列の境界（`|`）がまっすぐ揃っているか。
- [ ] 各セル内の前後に適切な半角スペースのパディングが入っているか。
- [ ] テキストが極端に長すぎて画面外に折り返されていないか。
