# Git Commit & Push Policy

## 厳格な制約事項 (Strict Constraints)
- **ユーザー（ボス）の明示的な指示がない限り、git commit および git push は絶対に実行しないこと。**
- ファイルの作成・編集、テスト、ビルド完了後であっても、自動的にコミットやプッシュを行わず、作業結果の報告にとどめること。
- ユーザーから「コミットして」「プッシュして」等の明確な指示を受けた場合のみ、コミット・プッシュを実行すること。
- （コミットを行う際は、既存ルールに従い cargo fmt / cargo clippy / cargo check / cargo test を事前にパスさせること）

## コミットメッセージ規約 (Commit Message Convention)
- コミットコメントの決定はAI（Gemini / 大賢者）に任せる。作業内容から適切な要約メッセージを自動決定すること。
- **接頭語（プレフィックス）は英語** を使用する（例: feat:, fix:, chore:, ci:, docs:, style:, refactor:, test:）。
- **説明文・文章は日本語** で分かりやすく簡潔に記述すること。
