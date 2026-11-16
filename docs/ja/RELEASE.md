# vivivi: リリース手順書 (RELEASE)

本書は、`vivivi` のリリース手順、バージョンタグ付与、およびビルド成果物のパッケージング手順を定めます。

---

## 1. リリース前チェックリスト

- [ ] すべてのテストが成功すること (`cargo test -j 4 --workspace --all-targets` および `cargo test -j 4 --workspace --doc`)
- [ ] Clippy の警告が 0 件であること (`cargo clippy -j 4 --workspace --all-targets -- -D warnings`)
- [ ] コードフォーマットが整っていること (`cargo fmt --check`)
- [ ] `Cargo.toml` のバージョン番号が更新されていること
- [ ] `docs/ja/CHANGELOG.md` および `docs/en/CHANGELOG.md` に更新内容が記載されていること

---

## 2. リリースビルド手順

```powershell
# 最適化リリースバイナリ（TUI & GUI）のビルド
cargo build --release --workspace

# 生成された実行ファイルの確認
ls target/release/vivivi.exe, target/release/vivivi-gui.exe
```
