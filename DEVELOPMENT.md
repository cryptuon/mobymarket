# Development Environment Setup

This guide will help you set up a complete development environment for the Moby Market whale trading infrastructure.

## Prerequisites

### System Requirements
- Rust 1.70+ with `wasm-pack` for WebAssembly compilation
- Node.js 18+ and npm/yarn for frontend tooling
- Solana CLI tools 1.16+ for blockchain development
- Anchor framework 0.28+ for Solana program development
- Docker and Docker Compose for local services

### Platform Support
- Linux (Ubuntu 20.04+, Debian 11+)
- macOS (Intel and Apple Silicon)
- Windows (with WSL2 recommended)

## Installation

### 1. Rust and Cargo

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add required targets
rustup target add wasm32-unknown-unknown
rustup component add clippy rustfmt

# Install cargo tools
cargo install cargo-watch cargo-nextest wasm-pack
```

### 2. Solana CLI Tools

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Add to PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Configure for local development
solana config set --url localhost
solana config set --keypair ~/.config/solana/id.json
```

### 3. Anchor Framework

```bash
# Install using AVM (Anchor Version Manager)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Verify installation
anchor --version
```

### 4. Node.js and Package Management

```bash
# Using Node Version Manager (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Install global tools
npm install -g yarn typescript ts-node
```

### 5. Docker and Services

```bash
# Install Docker (Ubuntu/Debian)
sudo apt update
sudo apt install docker.io docker-compose
sudo usermod -aG docker $USER

# macOS (using Homebrew)
brew install --cask docker

# Windows: Download Docker Desktop from docker.com
```

## Project Setup

### 1. Clone and Initialize

```bash
git clone <repository-url> moby-market
cd moby-market

# Install dependencies
cargo build
npm install
```

### 2. Environment Configuration

Create a `.env` file in the project root:

```bash
# Solana Configuration
SOLANA_RPC_URL=http://localhost:8899
ANCHOR_PROVIDER_URL=http://localhost:8899
ANCHOR_WALLET=~/.config/solana/id.json

# Oracle Configuration
PYTH_PROGRAM_ID=FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH
SWITCHBOARD_PROGRAM_ID=SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f

# Development Settings
RUST_LOG=debug
RUST_BACKTRACE=1
```

### 3. Local Blockchain Setup

Start a local Solana validator for testing:

```bash
# Terminal 1: Start validator
solana-test-validator \
  --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s ~/.local/share/solana/install/active_release/bin/spl_token_metadata.so \
  --reset \
  --quiet

# Terminal 2: Check validator status
solana cluster-version
solana balance
```

### 4. Build and Test

```bash
# Build all libraries
cargo build --workspace

# Run unit tests
cargo test --workspace

# Run integration tests
cargo nextest run

# Format code
cargo fmt --all

# Run lints
cargo clippy --workspace --all-targets -- -D warnings
```

## Development Workflow

### 1. Library Development

Each core library follows this structure:
```
libs/<library-name>/
├── Cargo.toml          # Package configuration
├── src/
│   ├── lib.rs          # Main library exports
│   ├── error.rs        # Error definitions
│   └── ...             # Feature modules
└── tests/              # Integration tests
```

### 2. Testing Strategy

```bash
# Unit tests (fast feedback)
cargo test --lib

# Integration tests (comprehensive)
cargo nextest run --workspace

# Benchmark tests
cargo bench

# Coverage reporting
cargo install cargo-tarpaulin
cargo tarpaulin --out html --output-dir coverage/
```

### 3. Code Quality

```bash
# Format check
cargo fmt --all --check

# Lint check
cargo clippy --workspace --all-targets -- -D warnings

# Security audit
cargo install cargo-audit
cargo audit

# Dependency check
cargo install cargo-outdated
cargo outdated
```

## IDE Setup

### VS Code (Recommended)

Install these extensions:
- `rust-analyzer` - Rust language support
- `ms-vscode.vscode-json` - JSON language support
- `bradlc.vscode-tailwindcss` - CSS support
- `esbenp.prettier-vscode` - Code formatting

Settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "rust-analyzer.inlayHints.enable": true
}
```

### Vim/Neovim

Add to your configuration:
```vim
" Install rust.vim and coc.nvim with coc-rust-analyzer
Plug 'rust-lang/rust.vim'
Plug 'neoclide/coc.nvim', {'branch': 'release'}

" Rust settings
let g:rustfmt_autosave = 1
let g:rust_clip_command = 'xclip -selection clipboard'
```

## Debugging

### 1. Rust Debugging

```bash
# Enable debug info
export RUST_BACKTRACE=full
export RUST_LOG=debug

# GDB debugging
cargo build
gdb target/debug/<binary_name>
```

### 2. Solana Program Debugging

```bash
# Program logs
solana logs --url localhost

# Account inspection
solana account <address> --output json

# Transaction inspection
solana confirm <signature> -v
```

### 3. Oracle Debugging

```bash
# Test oracle connections
cargo test oracle_integration -- --nocapture

# Price feed monitoring
curl -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["<oracle_address>"]}' \
  http://localhost:8899
```

## Performance Optimization

### 1. Compilation Speed

```bash
# Use mold linker (Linux)
sudo apt install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Use lld linker (cross-platform)
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Parallel compilation
export CARGO_BUILD_JOBS=$(nproc)
```

### 2. Development Builds

Add to `Cargo.toml`:
```toml
[profile.dev]
opt-level = 1
debug = true
split-debuginfo = "unpacked"
```

### 3. Testing Performance

```bash
# Parallel test execution
cargo nextest run --jobs $(nproc)

# Fast feedback loop
cargo watch -x "check --workspace"
```

## Troubleshooting

### Common Issues

1. **Anchor build fails**
   ```bash
   anchor clean
   rm -rf target/
   anchor build
   ```

2. **Solana validator won't start**
   ```bash
   rm -rf test-ledger/
   solana-test-validator --reset
   ```

3. **Oracle connection timeout**
   ```bash
   # Check network connectivity
   ping api.mainnet-beta.solana.com

   # Verify RPC endpoint
   curl -X POST -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
     $SOLANA_RPC_URL
   ```

4. **Cargo build issues**
   ```bash
   # Clear cache
   cargo clean
   rm -rf ~/.cargo/registry/cache/

   # Update toolchain
   rustup update
   ```

### Getting Help

- Documentation: `cargo doc --open`
- Community: [Solana Discord](https://discord.gg/solana)
- Issues: Create GitHub issues for bugs
- Development: Use `cargo run --example <name>` for examples

## Next Steps

After completing environment setup:
1. Review the [Architecture Documentation](ARCHITECTURE.md)
2. Explore the [API Reference](docs/api/)
3. Run the [Tutorial Examples](examples/)
4. Join our [Development Chat](https://discord.gg/moby-market)