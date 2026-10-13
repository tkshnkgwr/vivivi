# vivivi: Developer Guide (DEVELOPMENT)

This guide covers local environment setup, workspace building, testing, and quality verification workflows for `vivivi`.

---

## 1. Environment Setup

### Prerequisites
- **Rust Toolchain**: Rust 1.80+ (Latest stable recommended, Edition 2024 support)
- **Git**: Version control

```powershell
# Verify toolchain
rustc --version
cargo --version
```

### Clone Repository
```powershell
git clone https://github.com/tkshnkgwr/vivivi.git
cd vivivi
```

---

## 2. Workspace Build & Run

```powershell
# 1. Check all workspace targets
cargo check --all-targets

# 2. Run the application
cargo run

# 3. Release build
cargo build --release
```

---

## 3. Testing & Quality Checks

Run the following checks before committing code changes:

```powershell
# Run unit tests across all workspace crates
cargo test

# Clippy linter check (zero warnings)
cargo clippy --all-targets -- -D warnings

# Formatting check
cargo fmt --check
```
