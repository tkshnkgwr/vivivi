# vivivi: Release Guide (RELEASE)

This document describes the release workflow, version tagging, and artifact packaging procedures for `vivivi`.

---

## 1. Pre-Release Checklist

- [ ] All unit tests pass (`cargo test`)
- [ ] Clippy linter reports zero warnings (`cargo clippy --all-targets -- -D warnings`)
- [ ] Formatting is clean (`cargo fmt --check`)
- [ ] Package version is updated in `Cargo.toml`
- [ ] Changelogs updated in both `docs/ja/CHANGELOG.md` and `docs/en/CHANGELOG.md`

---

## 2. Release Build

```powershell
# Build optimized release binary
cargo build --release

# Verify executable
ls target/release/vivivi.exe
```
