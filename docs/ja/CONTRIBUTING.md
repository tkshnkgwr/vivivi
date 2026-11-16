# vivivi: コントリビューションガイド (CONTRIBUTING)

`vivivi` プロジェクトへの貢献を歓迎します！開発に参加する際は、以下のガイドラインに従ってください。

---

## 1. 開発フロー
1. 対象の課題について `git branch` でフィーチャーブランチを作成。
2. コード変更を行い、Rust標準のフォーマットを適用 (`cargo fmt`)。
3. すべての単体・統合・ドキュメントテストが通ることを確認 (`cargo test -j 4 --workspace --all-targets` / `cargo test -j 4 --workspace --doc`)。
4. Clippy の警告が 0 件であることを確認 (`cargo clippy -j 4 --workspace --all-targets -- -D warnings`)。
5. 明確なプレフィックス付きの日本語コミットメッセージでコミットを作成。

---

## 2. コーディング規約
- **Edition 2024**: 最新のRust文法と標準イディオムを使用してください。
- **ドキュメントコメント**: 公開API（`pub fn`, `pub struct`）には必ず `///` ドキュメントコメントを記述してください。
- **Markdownテーブル**: ドキュメント内の表は、生テキスト（gVim等）で整然と見えるよう垂直整列させてください。
