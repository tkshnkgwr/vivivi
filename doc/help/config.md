# 設定ファイルと永続カスタマイズ (Configuration)

`vivivi` は、ユーザーごとの編集スタイルやキーマップ、表示設定を `%USERPROFILE%\.vivivi\config.toml`（または `~/.vivivi/config.toml`）に永続保存し、起動時に自動ロードします。

---

## 📁 設定ファイルの配置場所

`vivivi` は起動時に以下の優先順序で設定ファイルを自動探索・ロードします。

1. `%USERPROFILE%\.vivivi\config.toml` (Windows) または `~/.vivivi/config.toml` (Unix)
2. `%USERPROFILE%\.config\vivivi\config.toml` または `~/.config/vivivi/config.toml`

ファイルが存在しない場合でも標準のデフォルト値で高速起動します。

---

## ⚙️ 設定例 (`config.toml`)

```toml
[editor]
# タブ幅（スペース換算数）
tab_stop = 4
# 行番号の表示 (true / false)
number = true
# 相対行番号の表示 (true / false)
relative_number = false
# 変更行ガター強調の表示 (true / false)
gutter = true
# 画面端での行折り返し表示 (true: 折り返す / false: 水平スクロール、デフォルト: false)
wrap = false
# 検索結果のハイライト表示 (true / false)
hlsearch = true
# 共有クリップボード連携 ("unnamed", "unnamedplus", "none")
clipboard = "unnamed"
# カラーテーマ ("default", "dark", "light", "monokai")
theme = "default"

[invisible]
# 不可視文字の表示 (true / false)
enabled = false
# 各不可視文字の表示記号（全角スペースは全角幅維持＋背景ハイライトがデフォルト）
zenkaku_space = "　"
space = "·"
tab = "▸---"
eol = "$"

[keymap.normal]
# ノーマルモードのキーリマップ (キー = "割り当て先")
"H" = "^"
"L" = "$"

[keymap.insert]
# インサートモードのキーリマップ
"jj" = "<Esc>"
```

---

## 🛠️ コマンドによる設定確認と動的再読込

エディタ起動中に設定を動的に変更・再読込できます。

| コマンド                                                         | 動作説明                                                                             |
| :--------------------------------------------------------------- | :----------------------------------------------------------------------------------- |
| `:config`                                                        | `config.toml` をディスクから即時再読込し、現在の設定を動的に反映します。             |
| `:set number` / `:set nonu`（`:se nu` / `:se nonu`）             | 行番号の表示／非表示を動的に切り替えます（`:se nu?` で状態確認）。                   |
| `:set relativenumber` / `:set nornu`（`:se rnu` / `:se nornu`）  | 相対行番号の表示／非表示を動的に切り替えます（`:se rnu?` で状態確認）。              |
| `:set wrap` / `:set nowrap`（`:se wrap` / `:se nowrap`）         | 画面端での行折り返し表示の有効／無効を動的に切り替えます（`:se wrap?` で状態確認）。 |
| `:set tabstop=4`（`:se ts=4`）                                   | タブ幅を動的に指定文字数へ変更します。                                               |
| `:set clipboard=unnamed`（`:se cb=unnamed`）                     | システム共有クリップボード連携を有効化します。                                       |
| `:set clipboard=none`（`:se cb=none`）                           | システム共有クリップボード連携を無効化（内部レジスタのみ）します。                   |
| `:set list` / `:set nolist`（`:se list` / `:se nolist`）         | 不可視文字表示の有効／無効を切り替えます。                                           |
| `:set gutter` / `:set nogutter`（`:se gutter` / `:se nogutter`） | 変更行ガター強調表示の有効／無効を切り替えます。                                     |
