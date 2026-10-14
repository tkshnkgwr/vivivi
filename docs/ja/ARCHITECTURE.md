# vivivi: システムアーキテクチャ設計書 (ARCHITECTURE)

## 1. プロジェクト目的
本プロジェクトは、C++ / Qtベースで開発されていた高機能テキストエディタ「OpenViVi」の仕様をベースに、モダンかつ安全・高速なプログラミング言語である **Rust** で再構築するプロジェクトです。
また、有償版（ViVi 3 / ViVi 4）に実装されていた独自仕様（インラインCSV/TSVモード、罫線モード等）についても、ヘルプファイルや仕様を忠実に反映・移植します。

---

## 2. 参考元リソースの参照方針
- **OpenViVi ソースコード**: `%USERPROFILE%\Documents\参考元アプリ\openViVi`
  - テキストバッファ (`vivi/buffer/`)
  - Viエンジン・コマンド体系 (`vivi/vivi64/ViEngine.*`, `vivi/vivi64/TestViCommands.*`)
  - 罫線モード基礎実装 (`vivi/vivi64/keisen.cpp`)
- **有償版 ViVi 3 / ViVi 4**:
  - ViVi 3: `%USERPROFILE%\Documents\個人ツール\vivi3`
  - ViVi 4: `%USERPROFILE%\Documents\個人ツール\vivi4`
  - ヘルプファイル (`vivihelp.CHM`): `docs/references/vivi4_help/` 配下にデコンパイル・UTF-8化して配置。

---

## 3. クレート構成（Cargo Workspace）
エディタのコアロジックをUI（CLI / TUI / 将来のGUI）から完全に独立させ、高カバレッジな単体テストを可能にするため、マルチクレート構成（Cargo Workspace）を採用しています。

```mermaid
graph TD
    App["vivivi-app (CLI / TUI エントリポイント)"]
    Core["vivivi-core (GapBuffer・UndoMgr・カーソル・エンコーディング)"]
    Vi["vivivi-vi (Vi状態機械・コマンドパーサー・レジスタ・ex)"]
    Modes["vivivi-modes (インラインCSV・罫線描画)"]

    App --> Vi
    App --> Modes
    App --> Core
    Vi --> Core
    Modes --> Core
```

| クレート名     | 役割・責務                                                                                                                   | 依存クレート  |
| :------------- | :--------------------------------------------------------------------------------------------------------------------------- | :------------ |
| `vivivi-core`  | ギャップバッファ (`GapBuffer`)、行管理、カーソル、Undo/Redo (`UndoMgr`)、文字コード判別・変換 (`encoding`)、対応括弧 (`paren`) | なし          |
| `vivivi-vi`    | Vi状態機械、コマンドパーサー、レジスタ、モーション、矩形編集、独自vi拡張（fFtT・単語検索・増減・マーク・タグ）             | `vivivi-core` |
| `vivivi-modes` | キラー編集支援（罫線保護、インラインCSV、式置換、fFtT全角、アライン、リナンバ、文字変換、BOX集計、マクロ、Migemo等）       | `vivivi-core` |
| `vivivi-app`   | Ratatui TUIフロントエンド、マルチウィンドウ画面分割、キーボードマクロ記録・再生、クリップボード履歴、AI連携、実行エントリ点  | 全クレート    |
