# vivivi: Release Guide (RELEASE)

This document describes the release workflow, version tagging, and artifact packaging procedures for `vivivi`.

---

## 1. Pre-Release Checklist

- [ ] All workspace unit, integration, and doc tests pass (`cargo test -j 4 --workspace --all-targets` and `cargo test -j 4 --workspace --doc`)
- [ ] Clippy linter reports zero warnings (`cargo clippy -j 4 --workspace --all-targets -- -D warnings`)
- [ ] Formatting is clean (`cargo fmt --check`)
- [ ] Package version is updated in `Cargo.toml`
- [ ] Changelogs updated in both `docs/ja/CHANGELOG.md` and `docs/en/CHANGELOG.md`

---

## 2. Release Build

```powershell
# Build optimized release binaries (TUI & GUI)
cargo build --release --workspace

# Verify executables
ls target/release/vivivi.exe, target/release/vivivi-gui.exe
```
