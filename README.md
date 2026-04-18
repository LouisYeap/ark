# Ark 🔐

> Secure offline password vault with TUI, written in Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Release](https://github.com/LouisYeap/ark/actions/workflows/release.yml/badge.svg)](https://github.com/LouisYeap/ark/actions/workflows/release.yml)
[![Test](https://github.com/LouisYeap/ark/actions/workflows/test.yml/badge.svg)](https://github.com/LouisYeap/ark/actions/workflows/test.yml)

---

## ✨ Features

- **🔒 Secure Local Storage** — AES-256-GCM encryption with Argon2id key derivation
- **🖥️ Terminal UI** — Built with Ratatui for a smooth TUI experience
- **📋 Clipboard Support** — Copy passwords to clipboard with auto-clear
- **🔍 Search & Filter** — Quickly find credentials
- **🗂️ Categories** — Organize passwords by category
- **⚡ Fast** — Pure Rust, zero dependencies at runtime

---

## 📦 Installation

### Pre-built Binaries

Download from [Releases](https://github.com/LouisYeap/ark/releases) — Windows, macOS, and Linux binaries available.

### From Source

```bash
cargo install --git https://github.com/LouisYeap/ark.git
```

### Build Manually

```bash
git clone https://github.com/LouisYeap/ark.git
cd ark
cargo build --release
./target/release/ark
```

---

## 🚀 Quick Start

```bash
# First launch — create your master password
ark

# Add an entry
# Navigate with arrow keys, Enter to select
# Type to search
# Tab to switch fields
# Ctrl+C to copy password
# Ctrl+Q to quit
```

---

## 🔒 Security

| Feature | Implementation |
|---------|---------------|
| Key Derivation | Argon2id |
| Encryption | AES-256-GCM |
| Randomness | `rand` crate (ChaCha12 RNG) |
| Storage | Encrypted JSON (local file) |

Your vault never leaves your machine. No cloud, no network, no tracking.

---

## 📁 Project Structure

```
ark/
├── src/
│   ├── main.rs        # Entry point
│   ├── cli/           # TUI rendering (ratatui)
│   ├── domain/        # Core domain models
│   ├── crypto/        # Encryption/decryption
│   ├── storage/       # File I/O
│   └── service/       # Business logic
├── Cargo.toml
└── README.md
```

---

## 🧪 Running Tests

```bash
cargo test
```

---

## 📥 Downloads

Pre-built binaries for Windows, macOS, and Linux are available in [Releases](https://github.com/LouisYeap/ark/releases).

| Platform | Architecture |
|----------|-------------|
| Windows | `x86_64-pc-windows-msvc` |
| macOS | `x86_64-apple-darwin`, `aarch64-apple-darwin` |
| Linux | `x86_64-unknown-linux-musl` |

---

## 📄 License

MIT License
