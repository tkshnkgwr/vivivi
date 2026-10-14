# vivivi AI連携機能 設計構想書 (AI Integration Architecture)

> [!NOTE]
> 本書は、ViVi 3 / ViVi 4 の伝統である「思考の速度で動く高速テキスト操作・編集予測」と、
> 現代の大規模言語モデル (LLM) による生成AI能力を融合させるためのアーキテクチャ設計書です。

---

## 🎯 1. ビジョンと基本思想

vivivi の最大の特徴は、**超軽量・超高速・キーボード完結** なテキスト編集体験にあります。
AI機能をエディタに組み込む際、VSCodeの重厚な拡張機能のような「エディタの起動や打鍵応答を1ミリ秒でも待たせる実装」は vivivi のアイデンティティに反します。

したがって、vivivi における AI 連携は以下の **3大原則** を厳格に遵守します：

1. **非同期・ノンブロッキングの徹底 (Zero-Latency UI)**:
   - LLMへの推論リクエストやネットワーク通信は、すべてバックグラウンド（別スレッド／非同期タスク）で実行。
   - タイピングやViコマンド実行を絶対にブロックしない。
2. **既存キラー編集アシストとの自然な共存・昇格 (Hybrid Intelligence)**:
   - ルールベースの高速機能（`dabbrev`、`LexEdit`、`DynamicMacro`、`abbrev`）を第1レイヤーとし、AIは第2レイヤー（意味的推論・先行予測）としてシームレスに重ね合わせる。
3. **ローカルファースト ＆ プライバシー重視 (Local First & Pluggable)**:
   - `llama.cpp` や `Ollama` などのローカル推論エンジンを標準サポート。
   - 外部クラウドAPI（Gemini API、Claude API、OpenAI API）も環境変数1つで差し替え可能な疎結合設計。

---

## 🏗️ 2. コア機能とアーキテクチャ

| 機能カテゴリ           | コマンド / キーバインド     | 連携エンジン / モデル       | 動作概要                                                       |
| :--------------------- | :-------------------------- | :-------------------------- | :------------------------------------------------------------- |
| AIインライン補完       | `Tab` / `Ctrl-Space`        | ローカル小モデル / 高速API  | カーソル前後の文脈から薄い文字色（ゴーストテキスト）で先読み補完 |
| AIテキストトランスフォーム | `:ai <指示>` / Visual選択   | 汎用推論モデル (Gemini等)   | 選択行や関数を自然言語指示に従いインプレースで書き換え・置換   |
| AIセマンティックLexEdit | `Ctrl-J` (拡張モード)       | 軽量推論 / 埋め込みモデル   | 置換・編集履歴の文脈から「次にやりたい編集」を高精度に推論提示 |
| AIコミットメッセージ   | `:ai commit`                | 汎用推論モデル              | `git diff` を解析しプロジェクト規約（日本語要約）でメッセージ生成 |
| AIコード説明・レビュー | `:ai explain` / `:ai review` | 対話・推論モデル            | アウトラインや選択コードの解説・潜在バグ指摘をペイン表示       |

---

## 🧩 3. 各機能の詳細仕様

### 3.1 AIインライン補完 (Ghost Text Completion)
- **トリガー**: 挿入モードでのタイピング休止（ディバウンス 200ms）または `Ctrl-Space`。
- **表示形態**: カーソル直後に暗めのグレー文字（薄色）で候補をインラインプレビュー。
- **操作**:
  - `Tab`: 提案を全文受け入れ
  - `Ctrl-Right`: 提案を単語単位で部分受け入れ
  - `Esc` / 他のキー入力: 提案を破棄して通常入力継続
- **ハイブリッド補完**:
  - `dabbrev`（同一バッファ内の単語）が完全一致する場合は即座に dabbrev を優先提示。
  - 単語の補完を超えた複数行コードや関数の続きを AI が薄色でアシスト。

### 3.2 AIテキストトランスフォーム (`:ai <instruction>`)
- **トリガー**:
  - ビジュアルモードで範囲選択して `:ai <instruction>`（例: `:ai 非同期関数にリファクタリングして`, `:ai 英語ドキュメントコメントを書いて`）
  - 通常モードで `:ai <instruction>`（現在行またはカレント関数・ブロックを対象）
- **UI挙動**:
  - 実行中は最下行ステータスに `AI処理中: '<指示>' ... (Escで中断)` と表示。
  - 推論完了後、変更箇所を一括置換（Undoスタックに1アクションとして積まれるため `u` で即座にロールバック可能）。

### 3.3 AIセマンティック LexEdit (字句編集予測の進化)
- **背景**:
  - 現在の ViVi 4 準拠 `LexEditEngine` は「単語の文字置換ルール（接頭辞・接尾辞の置換）」による決定論的アルゴリズムです。
- **AI連携**:
  - 「変数 `user_id` を `customer_id` に変えた」などの編集履歴をAIが解釈し、同一関数内の関連変数やログ文、SQLクエリ内の該当部分を自動検出して `Ctrl-J` で次候補としてサジェスト。

### 3.4 AIコミットメッセージ自動生成 (`:ai commit`)
- **トリガー**: `:ai commit` または `:aic`
- **動作**:
  1. バックグラウンドで `git diff --cached`（または `git diff`）を取得。
  2. プロジェクトのコミット規約（`Conventional Commits` 形式、プレフィックス、日本語概要）に合わせたプロンプトを構成。
  3. 生成されたコミットメッセージを一時バッファに展開し、ユーザーが確認・微修正して保存 (`:wq`) することでコミット完了。

---

## 🔌 4. バックエンド連携設計 (Provider Layer)

```text
+--------------------------------------------------------------+
|                        vivivi-app (TUI)                      |
|   - Ghost Text Renderer       - Status Bar Notification      |
|   - Key Event Interceptor     - Undo Transaction Manager     |
+--------------------------------------------------------------+
                               |
                               v (Non-blocking async channel)
+--------------------------------------------------------------+
|                        vivivi-ai (Crate)                     |
|   - Context Extractor (Cursor surroundings, AST outline)     |
|   - Debounce & Cancellation Engine                           |
|   - Prompt Templates (Commit, Refactor, Inline)              |
+--------------------------------------------------------------+
                               |
       +-----------------------+-----------------------+
       |                                               |
       v                                               v
[ Local LLM Provider ]                       [ Cloud API Provider ]
 - Ollama (http://localhost:11434)             - Google Gemini API (GEMINI_API_KEY)
 - llama.cpp server                            - Anthropic Claude API
 - ONNX Runtime local models                   - OpenAI Compatible API
```

### 環境変数設定例
```bash
# Gemini API を使用する場合
export GEMINI_API_KEY="your-api-key"
export VIVIVI_AI_PROVIDER="gemini"
export VIVIVI_AI_MODEL="gemini-2.5-flash"

# ローカル Ollama を使用する場合 (完全オフライン)
export VIVIVI_AI_PROVIDER="ollama"
export VIVIVI_AI_ENDPOINT="http://localhost:11434"
export VIVIVI_AI_MODEL="qwen2.5-coder:7b"
```

---

## 📅 5. 実装ロードマップ

1. **フェーズ 1 (基盤整備・現在完了)**:
   - `:ai` コマンドラインスロットの実装（引数・選択行範囲の解析とステータス表示）。
   - 設計構想の策定。
2. **フェーズ 2 (外部CLI/APIプロバイダ接続)**:
   - `vivivi-ai` モジュールまたはプラグインの導入。
   - `std::process::Command` または `reqwest` による非同期APIクライアント（Gemini / Ollama）。
   - `:ai commit` および `:ai <instruction>` の実動化。
3. **フェーズ 3 (エディタUI統合)**:
   - Ratatui 描画レイヤーへのゴーストテキスト（薄色インライン補完）レンダリング追加。
   - `Tab` / `Ctrl-Space` によるインライン受諾。
4. **フェーズ 4 (セマンティックLexEdit)**:
   - 編集ログとAST解析を組み合わせた高精度予測エンジン。
