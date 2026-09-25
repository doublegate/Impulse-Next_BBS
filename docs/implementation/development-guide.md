# Development Guide

**Project:** Impulse 7.1 BBS Modernization
**Last Updated:** 2025-11-22
**Target Audience:** Developers, Contributors

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Environment Setup](#development-environment-setup)
3. [Repository Structure](#repository-structure)
4. [Build System](#build-system)
5. [Code Style and Conventions](#code-style-and-conventions)
6. [Development Workflow](#development-workflow)
7. [Common Development Tasks](#common-development-tasks)
8. [Troubleshooting](#troubleshooting)
9. [Contributing Guidelines](#contributing-guidelines)

---

## Getting Started

### Prerequisites

**Required:**
- Rust 1.75+ (stable toolchain)
- Git 2.30+
- A Unix-like environment (Linux, macOS, WSL2 on Windows)

**Recommended:**
- Visual Studio Code with rust-analyzer extension
- Docker 24+ (for containerized testing)
- DOSBox (for testing against original Pascal build)

### Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd impulse-7.1

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development dependencies
rustup component add rustfmt clippy

# Build the project (when Rust conversion begins)
cargo build --workspace

# Run tests (when Rust conversion begins)
cargo test --workspace

# Verify code style
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### First-Time Setup Checklist

- [ ] Rust toolchain installed and updated
- [ ] Repository cloned and branches synchronized
- [ ] Editor/IDE configured with rust-analyzer
- [ ] Pre-commit hooks installed (if available)
- [ ] Documentation read (00-project-overview.md, 02-architecture.md)
- [ ] Build successful on local machine
- [ ] Test suite runs without errors

---

## Development Environment Setup

### Rust Toolchain Configuration

**Recommended `rust-toolchain.toml` (to be created in Sprint 1):**
```toml
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy", "rust-src"]
targets = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "x86_64-apple-darwin"]
profile = "default"
```

### IDE Configuration

#### Visual Studio Code

**Required Extensions:**
- `rust-lang.rust-analyzer` - Rust language server
- `vadimcn.vscode-lldb` - Debugger
- `tamasfe.even-better-toml` - TOML support
- `serayuzgur.crates` - Cargo.toml dependency management

**Recommended Settings (`.vscode/settings.json`):**
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "files.exclude": {
    "**/target": true,
    "**/*.tpu": true,
    "**/*.exe": true
  }
}
```

#### IntelliJ IDEA / CLion

- Install Rust plugin
- Enable Cargo check on save
- Configure rustfmt as code formatter

### Environment Variables

Create `.env` file in project root (do NOT commit to Git):

```bash
# Development configuration
RUST_LOG=debug
RUST_BACKTRACE=1

# BBS runtime configuration (for testing)
BBS_TELNET_PORT=2323
BBS_SSH_PORT=2222
BBS_DATA_DIR=./test-data
BBS_NODE_COUNT=4

# Database configuration (if needed in later sprints)
DATABASE_URL=sqlite://impulse.db

# Optional: Enable cargo features
CARGO_FEATURES=telnet,ssh,zmodem
```

### Platform-Specific Setup

#### Linux (Debian/Ubuntu)

```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    dosbox

# Optional: Cross-compilation support
sudo apt-get install -y gcc-mingw-w64
```

#### macOS

```bash
# Install Homebrew dependencies
brew install openssl dosbox

# Ensure Xcode Command Line Tools installed
xcode-select --install
```

#### Windows (WSL2 Recommended)

```bash
# Run in WSL2 Ubuntu
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Native Windows development requires:
# - Visual Studio 2022 Build Tools
# - Rust with MSVC toolchain
```

---

## Repository Structure

### High-Level Organization

```
impulse-7.1/
├── source/              # Original Pascal source code (reference)
├── imp71rel/            # Complete original Impulse 7.1 release
├── BP/                  # Borland Pascal 7.0 compiler (reference)
├── build.sh             # Pascal build script (reference)
├── docs/                # Project documentation (YOU ARE HERE)
├── ref-docs/            # Historical and reference documentation
├── to-dos/              # Sprint-by-sprint task tracking
├── Cargo.toml           # Rust workspace configuration (Sprint 1)
├── Cargo.lock           # Dependency lock file (Sprint 1)
├── .env                 # Local environment config (not in Git)
├── .gitignore           # Git ignore patterns
├── README.md            # Project overview and quick start
└── CHANGELOG.md         # Project history and version notes
```

### Rust Workspace Structure (Post-Sprint 1)

```
impulse-7.1/
├── crates/
│   ├── imp-core/        # Core types, traits, and utilities
│   ├── imp-records/     # Data structure definitions (Pascal record mappings)
│   ├── imp-file/        # Binary file I/O and parsing
│   ├── imp-common/      # Common utilities and helpers
│   ├── imp-ansi/        # ANSI/ASCII art rendering engine
│   ├── imp-telnet/      # Telnet protocol implementation
│   ├── imp-ssh/         # SSH protocol implementation
│   ├── imp-user/        # User management and authentication
│   ├── imp-message/     # Message base (JAM/Hudson/Squish)
│   ├── imp-file-area/   # File areas and transfer protocols
│   ├── imp-door/        # Door game integration
│   ├── imp-menu/        # Menu system and navigation
│   ├── imp-node/        # Multi-node coordination
│   └── imp-cli/         # Main executable (binary crate)
├── tests/               # Integration tests
├── benches/             # Performance benchmarks
├── fixtures/            # Test data and fixtures
└── tools/               # Development utilities
```

### Important Files to Know

| File/Directory | Purpose | Modify? |
|----------------|---------|---------|
| `Cargo.toml` (workspace root) | Workspace dependencies, metadata | Sprint 1+ |
| `crates/*/Cargo.toml` | Crate-specific dependencies | Per sprint |
| `crates/*/src/lib.rs` | Crate entry point | Per sprint |
| `crates/imp-cli/src/main.rs` | Main executable | Sprint 5+ |
| `docs/*.md` | Project documentation | Always (keep updated) |
| `to-dos/phase-*/sprint-*.md` | Sprint task tracking | Active sprint only |
| `source/*.pas` | Original Pascal (REFERENCE ONLY) | **NEVER** |
| `imp71rel/*` | Original BBS release (REFERENCE ONLY) | **NEVER** |

---

## Build System

### Cargo Workspace Configuration

The project uses Cargo workspace for managing multiple related crates.

**Root `Cargo.toml` (example from Sprint 1):**
```toml
[workspace]
members = [
    "crates/imp-core",
    "crates/imp-records",
    "crates/imp-file",
    "crates/imp-common",
    "crates/imp-ansi",
    # ... additional crates
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Impulse 7.1 Contributors"]
license = "Public Domain"

[workspace.dependencies]
# Shared dependencies across all crates
tokio = { version = "1.47", features = ["full"] }
kameo = "0.13"
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
binrw = "0.14"
crossterm = "0.28"
ratatui = "0.29"

# Development dependencies
proptest = "1.0"
criterion = "0.5"
```

### Common Build Commands

```bash
# Build entire workspace (all crates)
cargo build --workspace

# Build with optimizations
cargo build --workspace --release

# Build specific crate
cargo build -p imp-core

# Build with specific features
cargo build --workspace --features telnet,ssh

# Clean build artifacts
cargo clean

# Check code without building binaries (faster)
cargo check --workspace
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p imp-ansi

# Run specific test by name
cargo test --workspace test_ansi_parser

# Run with detailed output
cargo test --workspace -- --nocapture

# Run with parallelism control
cargo test --workspace -- --test-threads=1

# Run only doc tests
cargo test --workspace --doc
```

### Code Quality Tools

```bash
# Format code (must pass before commit)
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check

# Run Clippy lints (must pass before commit)
cargo clippy --all-targets --all-features -- -D warnings

# Run Clippy with pedantic lints (for extra strictness)
cargo clippy --all-targets --all-features -- -W clippy::pedantic

# Generate documentation
cargo doc --workspace --no-deps --open

# Check for outdated dependencies
cargo outdated

# Audit dependencies for security vulnerabilities
cargo audit
```

### Performance Benchmarking

```bash
# Run benchmarks (Sprint 4+)
cargo bench --workspace

# Run specific benchmark
cargo bench -p imp-ansi -- ansi_render

# Generate flamegraph (requires cargo-flamegraph)
cargo flamegraph --bench ansi_benchmark
```

### Cross-Platform Building

```bash
# Build for Linux
cargo build --target x86_64-unknown-linux-gnu

# Build for Windows (from Linux, requires mingw)
cargo build --target x86_64-pc-windows-gnu

# Build for macOS (from macOS)
cargo build --target x86_64-apple-darwin

# Using cross for easier cross-compilation
cargo install cross
cross build --target x86_64-pc-windows-gnu
cross build --target aarch64-unknown-linux-gnu
```

---

## Code Style and Conventions

### Rust Code Style

**We follow the official Rust style guide with these project-specific conventions:**

#### Naming Conventions

```rust
// Module names: snake_case
mod user_management;
mod file_transfer;

// Type names: PascalCase
struct UserRecord { }
enum ConnectionState { }
trait MessageBase { }

// Function names: snake_case
fn parse_jam_header() { }
fn render_ansi_sequence() { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_NODES: usize = 255;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

// Static variables: SCREAMING_SNAKE_CASE
static GLOBAL_CONFIG: OnceCell<Config> = OnceCell::new();
```

#### Code Organization

```rust
// Import order (enforced by rustfmt)
// 1. std/core
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates
use anyhow::{Context, Result};
use tokio::net::TcpListener;

// 3. Workspace crates
use imp_core::types::UserId;
use imp_records::UserRecord;

// 4. Local modules
use crate::config::Config;
use crate::utils::format_timestamp;
```

#### Documentation Comments

```rust
/// Parses a JAM message header from binary data.
///
/// # Arguments
///
/// * `data` - Raw byte slice containing JAM header
///
/// # Returns
///
/// * `Ok(JamHeader)` - Successfully parsed header
/// * `Err(ParseError)` - Invalid header format or checksum mismatch
///
/// # Examples
///
/// ```rust
/// use imp_message::jam::parse_jam_header;
///
/// let data = include_bytes!("../fixtures/jam_header.bin");
/// let header = parse_jam_header(data)?;
/// assert_eq!(header.signature, "JAM");
/// ```
///
/// # Safety
///
/// This function assumes `data` contains at least `JAM_HEADER_SIZE` bytes.
pub fn parse_jam_header(data: &[u8]) -> Result<JamHeader> {
    // Implementation
}
```

#### Error Handling

```rust
// Use Result<T> for recoverable errors
pub fn connect_to_node(node_id: u8) -> Result<Connection> {
    let addr = format!("127.0.0.1:{}", 3000 + node_id);
    TcpStream::connect(&addr)
        .with_context(|| format!("Failed to connect to node {}", node_id))
}

// Use custom error types with thiserror
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid header signature: expected {expected}, found {found}")]
    InvalidSignature { expected: String, found: String },

    #[error("Checksum mismatch: expected {expected:04x}, calculated {calculated:04x}")]
    ChecksumMismatch { expected: u16, calculated: u16 },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// Don't use unwrap() or expect() in production code
// Use ? operator or proper error handling
let config = Config::load()
    .context("Failed to load configuration")?;
```

#### Async Patterns

```rust
// Use async/await with Tokio runtime
#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:2323").await?;

    loop {
        let (socket, addr) = listener.accept().await?;

        // Spawn task for each connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Connection error from {}: {}", addr, e);
            }
        });
    }
}

// Prefer Select for timeout handling
async fn read_with_timeout(reader: &mut TcpStream) -> Result<Vec<u8>> {
    tokio::select! {
        result = reader.read_buf(&mut buf) => result?,
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            return Err(anyhow!("Read timeout"));
        }
    }
}
```

### Pascal to Rust Translation Patterns

**See `03-technical-details.md` for comprehensive mappings. Key patterns:**

```rust
// Pascal: array[1..255] of char
// Rust: [u8; 255] or Vec<u8>

// Pascal: record ... end
// Rust: struct with #[repr(C)] for binary compatibility

// Pascal: procedure MyProc(var x: Integer)
// Rust: fn my_proc(x: &mut i32)

// Pascal: unit MyUnit; interface ... implementation
// Rust: mod my_module { pub fn ... } or separate file

// Pascal: string[40]
// Rust: [u8; 40] or String with manual padding
```

### Git Commit Message Conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, no logic change)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Build system, dependencies, tooling
- `perf:` Performance improvements

**Examples:**
```
feat(ansi): implement color sequence parser

Add support for 256-color and 24-bit RGB ANSI sequences.
Includes comprehensive test suite covering edge cases.

Closes #42
```

```
fix(telnet): handle IAC command escaping correctly

Previously, IAC bytes in user data were not properly escaped,
causing protocol violations. Now doubles IAC bytes as per RFC 854.

Fixes #128
```

### Code Review Checklist

Before submitting a pull request:

- [ ] Code compiles without warnings (`cargo build --workspace`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] Clippy lints pass (`cargo clippy -- -D warnings`)
- [ ] Documentation updated (inline docs, README, guides)
- [ ] Commit messages follow conventions
- [ ] Branch rebased on latest main
- [ ] No sensitive data (keys, passwords) in commits
- [ ] Test coverage adequate (aim for 80%+ on new code)

---

## Development Workflow

### Branch Strategy

```
main (stable, deployable)
├── sprint/01-project-setup (active sprint)
├── sprint/02-core-types (upcoming)
├── feature/ansi-parser (individual feature)
├── bugfix/telnet-escape (bug fix)
└── docs/update-architecture (documentation)
```

**Branch Naming:**
- `sprint/XX-description` - Sprint work
- `feature/description` - New features
- `bugfix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring

### Development Cycle

```
1. Sync with main
   git checkout main
   git pull origin main

2. Create feature branch
   git checkout -b feature/jam-parser

3. Develop (test-driven)
   - Write failing test
   - Implement feature
   - Verify test passes
   - Refactor if needed

4. Commit frequently
   git add -p
   git commit -m "feat(message): add JAM header parser"

5. Push and create PR
   git push -u origin feature/jam-parser
   # Create PR on GitHub/GitLab

6. Address review feedback
   # Make changes
   git add .
   git commit -m "refactor: address review feedback"
   git push

7. Merge after approval
   # Squash and merge via UI
   git checkout main
   git pull
   git branch -d feature/jam-parser
```

### Test-Driven Development (TDD)

**Required for all new features:**

```rust
// 1. Write failing test first
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jam_header_valid() {
        let data = include_bytes!("../fixtures/valid_jam_header.bin");
        let result = parse_jam_header(data);

        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.signature, "JAM");
        assert_eq!(header.revision, 1);
    }
}

// 2. Run test (should fail)
// cargo test test_parse_jam_header_valid

// 3. Implement minimal code to pass
pub fn parse_jam_header(data: &[u8]) -> Result<JamHeader> {
    // ... implementation
}

// 4. Run test (should pass)
// cargo test test_parse_jam_header_valid

// 5. Refactor if needed (test should still pass)
```

### Continuous Integration Workflow

```yaml
# Automated on every push/PR
- cargo fmt --all -- --check
- cargo clippy --all-targets -- -D warnings
- cargo test --workspace
- cargo build --workspace --release
- cargo doc --workspace --no-deps
```

---

## Common Development Tasks

### Adding a New Crate to Workspace

```bash
# 1. Create crate structure
cargo new --lib crates/imp-newcrate

# 2. Add to workspace Cargo.toml
# [members] section: "crates/imp-newcrate",

# 3. Add dependencies to new crate's Cargo.toml
cd crates/imp-newcrate
cat >> Cargo.toml << 'EOF'

[dependencies]
imp-core = { path = "../imp-core" }
anyhow = { workspace = true }
EOF

# 4. Implement and test
# Edit crates/imp-newcrate/src/lib.rs

# 5. Verify builds
cargo build -p imp-newcrate
cargo test -p imp-newcrate
```

### Adding External Dependencies

```bash
# Add to workspace dependencies (if shared)
# Edit root Cargo.toml [workspace.dependencies]

# Or add to specific crate
cd crates/imp-ansi
cargo add regex --features unicode

# Audit new dependency
cargo audit
```

### Creating Integration Tests

```bash
# Create test file
mkdir -p tests
cat > tests/end_to_end.rs << 'EOF'
use imp_cli::BbsServer;
use tokio::net::TcpStream;

#[tokio::test]
async fn test_telnet_connection() {
    let server = BbsServer::new("127.0.0.1:2323").await.unwrap();
    tokio::spawn(async move { server.run().await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = TcpStream::connect("127.0.0.1:2323").await.unwrap();
    // ... test logic
}
EOF

# Run integration tests
cargo test --test end_to_end
```

### Profiling Performance

```bash
# Install flamegraph tooling
cargo install flamegraph

# Run with profiling
cargo flamegraph --bin imp-cli -- --demo-mode

# Generate callgrind data
valgrind --tool=callgrind ./target/release/imp-cli

# Analyze with kcachegrind
kcachegrind callgrind.out.*
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo run

# Run with full backtrace
RUST_BACKTRACE=full cargo run

# Debug with lldb (macOS/Linux)
rust-lldb target/debug/imp-cli

# Debug with gdb (Linux)
rust-gdb target/debug/imp-cli
```

### Working with Legacy Pascal Code

```bash
# Build original Pascal version (reference)
./build.sh

# Run in DOSBox for testing
dosbox imp71rel/IMP.EXE

# Extract binary structures
hexdump -C imp71rel/USERS.DAT | less

# Compare behavior with Rust implementation
# (useful for ensuring compatibility)
```

---

## Troubleshooting

### Common Build Issues

**Issue: `cargo build` fails with linker errors**
```bash
# Linux: Install build tools
sudo apt-get install build-essential pkg-config libssl-dev

# macOS: Install Xcode tools
xcode-select --install

# Windows: Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
```

**Issue: `cargo test` hangs or fails sporadically**
```bash
# Run tests sequentially
cargo test -- --test-threads=1

# Increase timeout for async tests
cargo test -- --test-threads=1 --nocapture
```

**Issue: Out of date lock file**
```bash
# Update dependencies
cargo update

# Or rebuild from scratch
cargo clean && cargo build
```

### Rust-Specific Issues

**Issue: Borrow checker errors**
- Read: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
- Consider using `Rc<RefCell<T>>` or `Arc<Mutex<T>>` for shared mutable state
- Use lifetime annotations where needed

**Issue: Async runtime not initialized**
```rust
// Ensure main is annotated with #[tokio::main]
#[tokio::main]
async fn main() {
    // async code here
}
```

**Issue: Module not found**
```rust
// Ensure mod declaration in parent module
// In lib.rs or main.rs:
mod my_module;

// Or for multi-file modules:
mod my_module {
    mod submodule;
}
```

### Getting Help

1. **Check documentation**: `cargo doc --open`
2. **Search issues**: GitHub/GitLab issue tracker
3. **Ask in discussions**: Project discussion forum
4. **Rust community**: https://users.rust-lang.org/
5. **Async Rust**: https://tokio.rs/tokio/tutorial

---

## Contributing Guidelines

### Before Contributing

1. Read `00-project-overview.md` for project vision
2. Review `02-architecture.md` for design decisions
3. Check `to-dos/` for current sprint priorities
4. Look for "good first issue" labels in issue tracker

### Contribution Process

1. **Open an issue** (or claim existing issue)
2. **Discuss approach** with maintainers
3. **Fork and branch** from latest main
4. **Develop with tests** (TDD approach)
5. **Ensure quality** (fmt, clippy, tests pass)
6. **Submit PR** with clear description
7. **Address feedback** from code review
8. **Celebrate merge!**

### Code of Conduct

- Be respectful and inclusive
- Assume positive intent
- Focus on constructive feedback
- Help newcomers learn
- Credit others' contributions

### Recognition

Contributors will be acknowledged in:
- `CHANGELOG.md` for significant contributions
- Project README
- Release notes

---

## Additional Resources

### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### Project-Specific
- [03-technical-details.md](../architecture/technical-details.md) - Implementation details
- [05-testing-strategy.md](../testing/testing-strategy.md) - Testing approach
- [ref-docs/rust-conversion-technical.md](../ref-docs/rust-conversion-technical.md) - Conversion guide

### Tools
- [Rust Playground](https://play.rust-lang.org/) - Test snippets online
- [Compiler Explorer](https://godbolt.org/) - Analyze generated assembly
- [crates.io](https://crates.io/) - Find dependencies

---

**Questions?** Open an issue or discussion in the project repository.

**Happy coding!**
