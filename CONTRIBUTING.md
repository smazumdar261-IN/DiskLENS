# Contributing to DiskLens

Thank you for your interest in contributing to **DiskLens**! We welcome contributions to make DiskLens faster, safer, and more helpful for the Linux community.

---

## 🛠️ Development Setup

### 1. Prerequisites
- **Rust**: Latest stable compiler (1.75+)
- **System Dependencies**:
  ```bash
  sudo apt install build-essential libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev
  ```

### 2. Building & Running
```bash
# Clone repository
git clone https://github.com/smazumdar261-IN/DiskLens.git
cd DiskLens

# Run tests
cargo test --tests

# Run release build
cargo run --release
```

---

## 📋 Guidelines

1. **Safety First**: Any filesystem modification must respect FreeDesktop.org safe-trash semantics. Permanent deletion must always prompt for user confirmation.
2. **Immediate-Mode Purity**: Never block the UI thread with synchronous disk I/O or unbounded loop computations. Use background channels for all long-running tasks.
3. **Format & Quality**: Ensure code is formatted with `cargo fmt --check` and clean of compiler warnings before submitting PRs.
4. **License**: DiskLens is licensed under the **GNU General Public License v3.0**. All contributions are subject to GPL-3.0.
