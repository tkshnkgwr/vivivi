# Rust Pre-Commit Quality Checks

Rust プロジェクト（vivivi）において git commit を実行する前に、必ず以下の検証手順を順番に実行し、すべてのエラーおよび警告が解消されていることを確認してください。

1. **フォーマット整形**:
   cargo fmt を実行してコードスタイルを Rust 標準に整える。
2. **静的解析・リント検証**:
   cargo clippy --all-targets を実行し、指摘された警告（0件）を解消する。
3. **コンパイル検証 & テスト**:
   cargo check --all-targets および cargo test を実行し、全ターゲット・全ワークスペースクレートでコンパイルおよび単体テストが通ることを確認する。
