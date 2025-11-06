# Development Environment Setup

> **Complete development environment for building whale trading infrastructure**

## Prerequisites

### System Requirements
```bash
# Minimum Requirements
- OS: Linux (Ubuntu 20.04+), macOS (12+), or Windows 11 with WSL2
- RAM: 16GB+ (32GB recommended for full testing)
- Storage: 100GB+ SSD space
- CPU: 8+ cores recommended
- Network: Stable internet for RPC calls and testing

# Recommended Development Machine
- RAM: 32-64GB
- Storage: 1TB+ NVMe SSD
- CPU: AMD Ryzen 9 / Intel i9 / Apple M2 Pro+
- Multiple monitors for trading interface development
```

### Required Software
```bash
# Core Development Tools
rustc >= 1.75.0
node >= 18.0.0
npm >= 9.0.0
git >= 2.30.0
docker >= 20.10.0
docker-compose >= 2.0.0

# Solana Toolchain
solana-cli >= 1.17.0
anchor-cli >= 0.29.0

# Optional but Recommended
vs-code (with Rust analyzer extension)
pgcli (PostgreSQL client)
redis-cli (Redis client)
k9s (Kubernetes dashboard)
```

---

## 1. Environment Setup

### Automated Setup Script
```bash
#!/bin/bash
# scripts/setup-dev-env.sh

set -e

echo "🚀 Setting up Moby Market development environment..."

# Check system requirements
check_system_requirements() {
    echo "📋 Checking system requirements..."

    # Check available RAM
    if [[ $(free -g | awk 'NR==2{printf "%.0f", $2}') -lt 16 ]]; then
        echo "❌ Error: Minimum 16GB RAM required"
        exit 1
    fi

    # Check available disk space
    if [[ $(df / | awk 'NR==2{print $4}') -lt 104857600 ]]; then # 100GB in KB
        echo "❌ Error: Minimum 100GB free disk space required"
        exit 1
    fi

    echo "✅ System requirements met"
}

# Install Rust and Cargo
install_rust() {
    echo "🦀 Installing Rust..."

    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi

    # Install required components
    rustup component add clippy rustfmt
    rustup toolchain install nightly
    rustup component add rustfmt --toolchain nightly

    # Install cargo tools
    cargo install cargo-audit cargo-udeps cargo-deny
    cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked

    echo "✅ Rust toolchain installed"
}

# Install Node.js and npm
install_node() {
    echo "📦 Installing Node.js..."

    if ! command -v node &> /dev/null; then
        # Install nvm
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
        source ~/.bashrc

        # Install latest LTS Node.js
        nvm install --lts
        nvm use --lts
    fi

    # Install global packages
    npm install -g typescript ts-node @types/node
    npm install -g @solana/cli

    echo "✅ Node.js and npm installed"
}

# Install Solana CLI
install_solana() {
    echo "☀️ Installing Solana CLI..."

    if ! command -v solana &> /dev/null; then
        sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
        export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    fi

    # Configure Solana CLI
    solana config set --url devnet
    solana config set --keypair ~/.config/solana/id.json

    # Generate keypair if it doesn't exist
    if [ ! -f ~/.config/solana/id.json ]; then
        solana-keygen new --outfile ~/.config/solana/id.json --no-bip39-passphrase
    fi

    echo "✅ Solana CLI installed and configured"
}

# Install Docker
install_docker() {
    echo "🐳 Installing Docker..."

    if ! command -v docker &> /dev/null; then
        # Install Docker (Ubuntu/Debian)
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh
        sudo usermod -aG docker $USER

        # Install Docker Compose
        sudo apt-get update
        sudo apt-get install docker-compose-plugin
    fi

    echo "✅ Docker installed"
}

# Clone repository and install dependencies
setup_project() {
    echo "📁 Setting up project..."

    # Clone repository (if not already cloned)
    if [ ! -d "moby-market" ]; then
        git clone https://github.com/moby-market/moby-market.git
        cd moby-market
    fi

    # Install Rust dependencies
    cargo build

    # Install Node.js dependencies
    npm install

    # Install frontend dependencies
    cd app && npm install && cd ..

    # Install SDK dependencies
    cd sdk/typescript && npm install && cd ../..

    echo "✅ Project dependencies installed"
}

# Setup development databases
setup_databases() {
    echo "🗄️ Setting up development databases..."

    # Start development services
    docker-compose -f docker/docker-compose.dev.yml up -d

    # Wait for services to be ready
    echo "⏳ Waiting for services to start..."
    sleep 30

    # Run database migrations
    npm run db:migrate

    # Seed test data
    npm run db:seed

    echo "✅ Development databases ready"
}

# Setup testing environment
setup_testing() {
    echo "🧪 Setting up testing environment..."

    # Install test validator
    if ! command -v solana-test-validator &> /dev/null; then
        cargo install solana-test-validator
    fi

    # Setup test accounts and tokens
    anchor test --skip-deploy

    echo "✅ Testing environment ready"
}

# Create development configuration
create_config() {
    echo "⚙️ Creating development configuration..."

    cat > .env.development << EOF
# Solana Configuration
SOLANA_NETWORK=devnet
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json

# Database Configuration
DATABASE_URL=postgresql://moby:password@localhost:5432/moby_market_dev
REDIS_URL=redis://localhost:6379

# Oracle Configuration
PYTH_RPC_URL=https://pythnet.rpc.pythnet.pyth.network
SWITCHBOARD_RPC_URL=https://api.devnet.solana.com

# API Configuration
API_PORT=3000
WS_PORT=3001

# Logging
LOG_LEVEL=debug
LOG_FORMAT=json

# Testing
TEST_DATABASE_URL=postgresql://moby:password@localhost:5432/moby_market_test
TEST_SOLANA_URL=http://localhost:8899
EOF

    echo "✅ Development configuration created"
}

# Main execution
main() {
    check_system_requirements
    install_rust
    install_node
    install_solana
    install_docker
    setup_project
    setup_databases
    setup_testing
    create_config

    echo ""
    echo "🎉 Development environment setup complete!"
    echo ""
    echo "Next steps:"
    echo "1. source ~/.bashrc (or restart terminal)"
    echo "2. cd moby-market"
    echo "3. npm run dev (start development server)"
    echo "4. npm test (run test suite)"
    echo ""
    echo "Happy coding! 🚀"
}

main "$@"
```

### Manual Setup Instructions

#### 1. Rust Toolchain Setup
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add required components
rustup component add clippy rustfmt
rustup toolchain install nightly

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked

# Install development tools
cargo install cargo-audit cargo-udeps cargo-deny cargo-tarpaulin
```

#### 2. Solana Development Setup
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Configure for development
solana config set --url devnet
solana-keygen new --outfile ~/.config/solana/id.json --no-bip39-passphrase

# Verify installation
solana --version
anchor --version
```

#### 3. Node.js and TypeScript Setup
```bash
# Install Node.js (using nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install --lts
nvm use --lts

# Install global packages
npm install -g typescript ts-node @types/node
npm install -g @solana/web3.js @coral-xyz/anchor
```

---

## 2. Project Structure Setup

### Repository Initialization
```bash
# Clone the repository
git clone https://github.com/moby-market/moby-market.git
cd moby-market

# Install dependencies
cargo build
npm install

# Setup git hooks
npm run prepare
```

### Project Structure
```
moby-market/
├── programs/                    # Solana programs
│   ├── moby-controller/        # Main controller program
│   ├── moby-otc/              # OTC trading program
│   ├── moby-executor/         # Algorithm execution program
│   ├── moby-privacy/          # Privacy program
│   └── moby-oracle/           # Oracle program
├── app/                        # Frontend application
│   ├── src/
│   │   ├── components/        # React components
│   │   ├── hooks/             # Custom React hooks
│   │   ├── utils/             # Utility functions
│   │   └── types/             # TypeScript definitions
│   ├── public/                # Static assets
│   └── package.json
├── sdk/                        # Client SDKs
│   ├── typescript/            # TypeScript SDK
│   │   ├── src/
│   │   ├── tests/
│   │   └── package.json
│   ├── python/                # Python SDK
│   └── rust/                  # Rust SDK
├── tests/                      # Test suites
│   ├── unit/                  # Unit tests
│   ├── integration/           # Integration tests
│   ├── performance/           # Performance tests
│   └── security/              # Security tests
├── scripts/                    # Utility scripts
│   ├── setup-dev-env.sh      # Environment setup
│   ├── deploy.sh              # Deployment script
│   └── test-runner.sh         # Test automation
├── docs/                       # Documentation
├── docker/                     # Docker configurations
│   ├── Dockerfile.dev         # Development container
│   ├── Dockerfile.prod        # Production container
│   └── docker-compose.dev.yml # Development services
├── .github/                    # GitHub workflows
│   └── workflows/
│       ├── ci.yml             # Continuous integration
│       ├── security.yml       # Security scanning
│       └── deploy.yml         # Deployment automation
├── Anchor.toml                 # Anchor configuration
├── Cargo.toml                  # Rust workspace
├── package.json                # Node.js configuration
├── .env.example                # Environment template
└── README.md                   # Project overview
```

---

## 3. Development Services

### Docker Development Environment (`docker/docker-compose.dev.yml`)
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: moby_market_dev
      POSTGRES_USER: moby
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init-db.sql

  postgres_test:
    image: postgres:15
    environment:
      POSTGRES_DB: moby_market_test
      POSTGRES_USER: moby
      POSTGRES_PASSWORD: password
    ports:
      - "5433:5432"
    tmpfs:
      - /var/lib/postgresql/data # Use tmpfs for faster test DB

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

  solana-test-validator:
    image: solanalabs/solana:latest
    ports:
      - "8899:8899"
      - "8900:8900"
    command: |
      sh -c "
        solana-test-validator \
          --bind-address 0.0.0.0 \
          --rpc-port 8899 \
          --rpc-bind-address 0.0.0.0 \
          --reset \
          --quiet
      "

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./docker/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./docker/grafana/dashboards:/etc/grafana/provisioning/dashboards

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:
```

### Starting Development Environment
```bash
# Start all development services
docker-compose -f docker/docker-compose.dev.yml up -d

# Check service status
docker-compose -f docker/docker-compose.dev.yml ps

# View logs
docker-compose -f docker/docker-compose.dev.yml logs -f

# Stop services
docker-compose -f docker/docker-compose.dev.yml down
```

---

## 4. IDE Configuration

### VS Code Configuration (`.vscode/settings.json`)
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.allTargets": false,
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,

  "typescript.preferences.importModuleSpecifier": "relative",
  "typescript.suggest.autoImports": true,
  "typescript.updateImportsOnFileMove.enabled": "always",

  "files.associations": {
    "*.rs": "rust",
    "Anchor.toml": "toml"
  },

  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true,
    "source.organizeImports": true
  },

  "solidity.packageDefaultDependenciesContractsDirectory": "contracts",
  "solidity.packageDefaultDependenciesDirectory": "node_modules",

  "files.exclude": {
    "**/target": true,
    "**/node_modules": true,
    "**/.git": true,
    "**/dist": true
  }
}
```

### VS Code Extensions (`.vscode/extensions.json`)
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "ms-vscode.vscode-typescript-next",
    "bradlc.vscode-tailwindcss",
    "ms-vscode.vscode-json",
    "redhat.vscode-yaml",
    "ms-vscode-remote.remote-containers",
    "ms-azuretools.vscode-docker",
    "github.copilot",
    "github.copilot-labs"
  ]
}
```

### Rust Configuration (`.cargo/config.toml`)
```toml
[alias]
c = "check"
t = "test"
r = "run"
b = "build"
br = "build --release"

[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.'cfg(unix)']
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[registries.crates-io]
protocol = "sparse"

[net]
retry = 2
git-fetch-with-cli = true
```

---

## 5. Development Workflow

### Daily Development Commands
```bash
# Start development environment
npm run dev:start

# Run development server
npm run dev

# Run tests
npm run test
npm run test:unit
npm run test:integration
npm run test:e2e

# Build programs
anchor build
anchor test

# Format code
npm run format
cargo fmt

# Lint code
npm run lint
cargo clippy

# Type checking
npm run type-check

# Build frontend
npm run build
```

### Git Workflow
```bash
# Feature development workflow
git checkout -b feature/new-feature
git add .
git commit -m "feat: add new feature"
git push origin feature/new-feature

# Create pull request
gh pr create --title "Add new feature" --body "Description of changes"

# Before merge - run full test suite
npm run test:all
npm run lint:all
npm run build:all
```

### Environment Variables (`.env.development`)
```bash
# Solana Configuration
SOLANA_NETWORK=devnet
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json

# Program IDs (will be set after deployment)
MOBY_CONTROLLER_PROGRAM_ID=
MOBY_OTC_PROGRAM_ID=
MOBY_EXECUTOR_PROGRAM_ID=
MOBY_PRIVACY_PROGRAM_ID=
MOBY_ORACLE_PROGRAM_ID=

# Database Configuration
DATABASE_URL=postgresql://moby:password@localhost:5432/moby_market_dev
TEST_DATABASE_URL=postgresql://moby:password@localhost:5433/moby_market_test
REDIS_URL=redis://localhost:6379

# API Configuration
API_PORT=3000
WS_PORT=3001
CORS_ORIGIN=http://localhost:3000

# Oracle Configuration
PYTH_RPC_URL=https://pythnet.rpc.pythnet.pyth.network
SWITCHBOARD_RPC_URL=https://api.devnet.solana.com

# Logging
LOG_LEVEL=debug
LOG_FORMAT=pretty

# Feature Flags
ENABLE_PRIVACY_FEATURES=true
ENABLE_CROSS_CHAIN=false
ENABLE_ADVANCED_ALGORITHMS=true

# Security
JWT_SECRET=your-jwt-secret-here
API_RATE_LIMIT=1000
```

---

## 6. Development Scripts

### Package.json Scripts
```json
{
  "scripts": {
    "dev": "concurrently \"npm run dev:programs\" \"npm run dev:app\"",
    "dev:programs": "anchor test --skip-deploy",
    "dev:app": "cd app && npm run dev",
    "dev:start": "docker-compose -f docker/docker-compose.dev.yml up -d",
    "dev:stop": "docker-compose -f docker/docker-compose.dev.yml down",

    "build": "anchor build && npm run build:app && npm run build:sdk",
    "build:app": "cd app && npm run build",
    "build:sdk": "cd sdk/typescript && npm run build",
    "build:programs": "anchor build",

    "test": "npm run test:programs && npm run test:app && npm run test:sdk",
    "test:programs": "anchor test",
    "test:app": "cd app && npm test",
    "test:sdk": "cd sdk/typescript && npm test",
    "test:unit": "cargo test --workspace",
    "test:integration": "cargo test --test integration_tests",
    "test:e2e": "npm run test:e2e:setup && npm run test:e2e:run",
    "test:performance": "cargo test --test performance_tests --release",
    "test:security": "cargo test --test security_tests",
    "test:all": "npm run test && npm run test:performance && npm run test:security",

    "lint": "npm run lint:rust && npm run lint:ts",
    "lint:rust": "cargo clippy --all-targets --all-features",
    "lint:ts": "eslint . --ext .ts,.tsx,.js,.jsx",
    "lint:fix": "npm run lint:rust -- --fix && npm run lint:ts -- --fix",

    "format": "cargo fmt && prettier --write .",
    "type-check": "tsc --noEmit && cd app && tsc --noEmit",

    "deploy:devnet": "anchor deploy --provider.cluster devnet",
    "deploy:mainnet": "anchor deploy --provider.cluster mainnet-beta",

    "db:migrate": "node scripts/migrate-db.js",
    "db:seed": "node scripts/seed-db.js",
    "db:reset": "npm run db:migrate && npm run db:seed",

    "prepare": "husky install"
  }
}
```

### Quick Start Script (`scripts/quick-start.sh`)
```bash
#!/bin/bash
# Quick start development environment

echo "🚀 Quick starting Moby Market development..."

# Check if dependencies are installed
if ! command -v anchor &> /dev/null; then
    echo "❌ Anchor CLI not found. Run setup script first."
    exit 1
fi

# Start development services
echo "📦 Starting development services..."
docker-compose -f docker/docker-compose.dev.yml up -d

# Wait for services
echo "⏳ Waiting for services to be ready..."
sleep 10

# Build programs
echo "🔨 Building Solana programs..."
anchor build

# Start test validator in background
echo "☀️ Starting Solana test validator..."
solana-test-validator --reset --quiet &
VALIDATOR_PID=$!

# Wait for validator to start
sleep 5

# Deploy programs to test validator
echo "🚀 Deploying programs to test validator..."
anchor deploy --provider.cluster localnet

# Run initial tests
echo "🧪 Running initial tests..."
anchor test --skip-deploy

# Start development server
echo "💻 Starting development server..."
npm run dev

# Cleanup on exit
trap "kill $VALIDATOR_PID; docker-compose -f docker/docker-compose.dev.yml down" EXIT
```

---

## 7. Troubleshooting

### Common Issues and Solutions

#### Rust/Anchor Issues
```bash
# Anchor build fails
anchor clean
rm -rf target/
anchor build

# Rust analyzer issues
rustup component add rust-src
rustup component add rust-analyzer

# Permission issues on Linux
sudo chown -R $USER:$USER ~/.cargo
sudo chown -R $USER:$USER ~/.config/solana
```

#### Solana Issues
```bash
# RPC connection issues
solana config set --url devnet
solana config set --url http://localhost:8899  # for local testing

# Keypair issues
solana-keygen recover --outfile ~/.config/solana/id.json

# Insufficient SOL for testing
solana airdrop 10  # devnet only
```

#### Database Issues
```bash
# PostgreSQL connection issues
docker-compose -f docker/docker-compose.dev.yml restart postgres

# Reset database
docker-compose -f docker/docker-compose.dev.yml down -v
docker-compose -f docker/docker-compose.dev.yml up -d
npm run db:reset
```

#### Performance Issues
```bash
# Slow builds
# Add to ~/.cargo/config.toml:
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Memory issues during testing
export RUST_MIN_STACK=8388608
ulimit -s 8192
```

### Getting Help
```bash
# Check system status
./scripts/health-check.sh

# View development logs
docker-compose -f docker/docker-compose.dev.yml logs -f

# Debug Solana programs
solana logs --url devnet

# Check anchor configuration
anchor --version
cat Anchor.toml
```

This development environment setup provides a robust foundation for building the whale trading infrastructure with proper tooling, testing, and automation in place.

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[{"content": "Create core library specifications and interfaces", "status": "completed", "activeForm": "Creating core library specifications and interfaces"}, {"content": "Set up comprehensive testing framework", "status": "completed", "activeForm": "Setting up comprehensive testing framework"}, {"content": "Create development environment setup guide", "status": "completed", "activeForm": "Creating development environment setup guide"}, {"content": "Document implementation order and dependencies", "status": "in_progress", "activeForm": "Documenting implementation order and dependencies"}, {"content": "Set up CI/CD and code quality tools", "status": "pending", "activeForm": "Setting up CI/CD and code quality tools"}]