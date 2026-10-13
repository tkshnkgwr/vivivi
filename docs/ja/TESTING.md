# vivivi: テスト方針・計画 (TESTING)

本書は、`vivivi` プロジェクトにおけるテスト戦略、単体テスト、結合テスト、および検証基準を定義します。

---

## 1. テスト戦略

エディタのコア（バッファ・Undo・Vi状態機械・特殊モード）は、UI表示がなくても単体テストで100%検証できるように設計されています。

### 主要テスト領域
1. **バッファ操作 (`vivivi-core`)**:
   - ギャップ移動、境界値挿入・削除、動的リサイズ、イテレータ
2. **Undo/Redo (`vivivi-core`)**:
   - 連続キー入力の自動マージ、改行時のマージ分割、`prohibit_merge`、トランザクションブロックの一括Undo
3. **Viエンジン (`vivivi-vi`)**:
   - モード遷移（Normal $\to$ Insert $\to$ Normal）、キーパース、レジスタ保存
4. **特殊モード (`vivivi-modes`)**:
   - Unicode罫線文字結合テーブルの正確性、インラインCSVカラム幅計算

---

## 2. テストコマンド

```powershell
# 1. ワークスペース全体の単体テスト実行
cargo test

# 2. 特定クレートのテスト実行
cargo test -p vivivi-core
cargo test -p vivivi-modes
cargo test -p vivivi-vi

# 3. テスト出力の詳細表示
cargo test -- --nocapture
```
