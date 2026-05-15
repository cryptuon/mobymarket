# CI/CD and Code Quality Setup

> **Automated testing, security scanning, and deployment pipeline for whale trading infrastructure**

## CI/CD Philosophy

### Quality Gates
- **No code merges without passing all tests**
- **Security scans must pass before deployment**
- **Performance regressions are blocking**
- **Code coverage must maintain >95%**
- **All dependencies must be audited**

### Deployment Strategy
- **Staging First**: All changes deployed to staging environment
- **Gradual Rollout**: Progressive deployment with monitoring
- **Instant Rollback**: Automated rollback on critical issues
- **Zero Downtime**: Blue-green deployments for minimal disruption

---

## 1. GitHub Actions Workflows

### Main CI Pipeline (`.github/workflows/ci.yml`)
```yaml
name: Continuous Integration

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  SOLANA_VERSION: 1.17.0
  ANCHOR_VERSION: 0.29.0

jobs:
  # Job 1: Code Quality Checks
  code-quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Check for unused dependencies
        run: |
          cargo install cargo-udeps --locked
          cargo udeps --all-targets

      - name: Audit dependencies
        run: |
          cargo install cargo-audit --locked
          cargo audit

      - name: Check code with cargo-deny
        run: |
          cargo install cargo-deny --locked
          cargo deny check

  # Job 2: Unit Tests
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --workspace --lib

      - name: Run doctests
        run: cargo test --workspace --doc

      - name: Generate code coverage
        run: |
          cargo install cargo-tarpaulin --locked
          cargo tarpaulin --out xml --output-dir coverage/

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: coverage/cobertura.xml
          flags: unit-tests
          name: unit-test-coverage

  # Job 3: Integration Tests
  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: password
          POSTGRES_DB: moby_market_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v4

      - name: Install Solana
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/v${{ env.SOLANA_VERSION }}/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH

      - name: Install Anchor
        run: |
          npm install -g @coral-xyz/anchor-cli@${{ env.ANCHOR_VERSION }}

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-integration-${{ hashFiles('**/Cargo.lock') }}

      - name: Build programs
        run: anchor build

      - name: Start Solana test validator
        run: |
          solana-test-validator --reset --quiet &
          sleep 10

      - name: Deploy programs to test validator
        run: anchor deploy --provider.cluster localnet

      - name: Run integration tests
        run: |
          export DATABASE_URL=postgresql://postgres:password@localhost:5432/moby_market_test
          export REDIS_URL=redis://localhost:6379
          anchor test --skip-build --skip-deploy

      - name: Upload integration test coverage
        uses: codecov/codecov-action@v3
        with:
          flags: integration-tests
          name: integration-test-coverage

  # Job 4: Security Scans
  security-scans:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Semgrep security scan
        uses: returntocorp/semgrep-action@v1
        with:
          config: >-
            p/security-audit
            p/rust
            p/solana

      - name: Run cargo-audit
        run: |
          cargo install cargo-audit --locked
          cargo audit --deny warnings

      - name: Scan for secrets
        uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: main
          head: HEAD

  # Job 5: Performance Tests
  performance-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install criterion
        run: cargo install cargo-criterion

      - name: Run benchmarks
        run: cargo bench --workspace

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/reports/benchmark.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true

  # Job 6: Build and Test Frontend
  frontend-tests:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ./app
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
          cache-dependency-path: app/package-lock.json

      - name: Install dependencies
        run: npm ci

      - name: Run ESLint
        run: npm run lint

      - name: Run TypeScript check
        run: npm run type-check

      - name: Run tests
        run: npm test -- --coverage

      - name: Build application
        run: npm run build

      - name: Upload frontend coverage
        uses: codecov/codecov-action@v3
        with:
          directory: app/coverage
          flags: frontend
          name: frontend-coverage

  # Job 7: SDK Tests
  sdk-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        sdk: [typescript, python, rust]
    steps:
      - uses: actions/checkout@v4

      - name: Test TypeScript SDK
        if: matrix.sdk == 'typescript'
        run: |
          cd sdk/typescript
          npm ci
          npm run lint
          npm test
          npm run build

      - name: Test Python SDK
        if: matrix.sdk == 'python'
        run: |
          cd sdk/python
          pip install -e .[test]
          pytest
          python -m build

      - name: Test Rust SDK
        if: matrix.sdk == 'rust'
        run: |
          cd sdk/rust
          cargo test
          cargo build --release
```

### Security Scanning Pipeline (`.github/workflows/security.yml`)
```yaml
name: Security Scans

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  # Advanced Security Scanning
  advanced-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Initialize CodeQL
        uses: github/codeql-action/init@v2
        with:
          languages: rust, javascript

      - name: Build for CodeQL
        run: |
          cargo build --workspace

      - name: Perform CodeQL Analysis
        uses: github/codeql-action/analyze@v2

  # Smart Contract Security
  contract-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Solana
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH

      - name: Install Anchor
        run: npm install -g @coral-xyz/anchor-cli

      - name: Build programs
        run: anchor build

      - name: Run Solana security analyzer
        run: |
          # Install security tools
          cargo install --git https://github.com/coral-xyz/sealevel-attacks sealevel-attacks

          # Run security analysis
          for program in programs/*/; do
            echo "Analyzing $program"
            sealevel-attacks "$program/src/lib.rs"
          done

      - name: Check for common vulnerabilities
        run: |
          # Custom security checks
          ./scripts/security-checks.sh

  # Dependency Vulnerability Scanning
  dependency-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Snyk security scan
        uses: snyk/actions/rust@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}

      - name: Run OSSAR analysis
        uses: github/ossar-action@v1
        id: ossar

      - name: Upload OSSAR results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: ${{ steps.ossar.outputs.sarifFile }}

  # License Compliance
  license-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check license compatibility
        run: |
          cargo install cargo-license
          cargo license --json > licenses.json
          ./scripts/check-license-compliance.sh
```

### Deployment Pipeline (`.github/workflows/deploy.yml`)
```yaml
name: Deploy

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  # Build and test before deployment
  pre-deploy-tests:
    uses: ./.github/workflows/ci.yml

  # Deploy to staging
  deploy-staging:
    needs: pre-deploy-tests
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - uses: actions/checkout@v4

      - name: Configure Solana CLI
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH
          solana config set --url devnet

      - name: Deploy to staging
        run: |
          echo "${{ secrets.STAGING_KEYPAIR }}" > staging-keypair.json
          solana config set --keypair staging-keypair.json
          anchor deploy --provider.cluster devnet

      - name: Run staging tests
        run: |
          export SOLANA_NETWORK=devnet
          npm run test:staging

      - name: Notify staging deployment
        uses: 8398a7/action-slack@v3
        with:
          status: ${{ job.status }}
          channel: '#deployments'
          webhook_url: ${{ secrets.SLACK_WEBHOOK }}

  # Deploy to mainnet (production)
  deploy-production:
    needs: deploy-staging
    runs-on: ubuntu-latest
    environment: production
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4

      - name: Configure Solana CLI for mainnet
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH
          solana config set --url mainnet-beta

      - name: Deploy to mainnet
        run: |
          echo "${{ secrets.MAINNET_KEYPAIR }}" > mainnet-keypair.json
          solana config set --keypair mainnet-keypair.json

          # Blue-green deployment
          ./scripts/blue-green-deploy.sh

      - name: Post-deployment verification
        run: |
          export SOLANA_NETWORK=mainnet-beta
          npm run test:production-smoke

      - name: Notify production deployment
        uses: 8398a7/action-slack@v3
        with:
          status: ${{ job.status }}
          channel: '#deployments'
          webhook_url: ${{ secrets.SLACK_WEBHOOK }}

  # Build and push Docker images
  build-images:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Log in to Container Registry
        uses: docker/login-action@v2
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v4
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}

      - name: Build and push Docker image
        uses: docker/build-push-action@v4
        with:
          context: .
          file: docker/Dockerfile.prod
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
```

---

## 2. Code Quality Tools

### Pre-commit Hooks (`.pre-commit-config.yaml`)
```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-toml
      - id: check-json
      - id: check-merge-conflict
      - id: check-added-large-files

  - repo: local
    hooks:
      - id: cargo-fmt
        name: Cargo Format
        entry: cargo fmt
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: Cargo Clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-test
        name: Cargo Test
        entry: cargo test --workspace --lib
        language: system
        types: [rust]
        pass_filenames: false

  - repo: https://github.com/psf/black
    rev: 23.3.0
    hooks:
      - id: black
        language_version: python3
        files: sdk/python/

  - repo: https://github.com/pre-commit/mirrors-eslint
    rev: v8.44.0
    hooks:
      - id: eslint
        files: \.(js|ts|tsx)$
        args: [--fix]
        additional_dependencies:
          - eslint@^8.0.0
          - '@typescript-eslint/eslint-plugin@^5.0.0'
          - '@typescript-eslint/parser@^5.0.0'
```

### Rust Quality Configuration (`.cargo/config.toml`)
```toml
[alias]
# Development aliases
dev-build = "build --workspace"
dev-test = "test --workspace"
dev-check = "check --workspace"

# Quality checks
quality = [
  "fmt",
  "clippy --all-targets --all-features -- -D warnings",
  "test --workspace",
  "audit",
]

# Security checks
security = [
  "audit",
  "deny check",
]

# Performance benchmarks
bench-all = "bench --workspace"

[build]
# Use lld linker for faster builds
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.'cfg(unix)']
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Clippy Configuration (`clippy.toml`)
```toml
# Lint configuration for high-quality Rust code
avoid-breaking-exported-api = false
cognitive-complexity-threshold = 30
type-complexity-threshold = 250
single-char-lifetime-names-threshold = 4
trivial-copy-size-limit = 64
pass-by-value-size-limit = 256
too-many-arguments-threshold = 7
too-many-lines-threshold = 100
large-type-threshold = 200
verbose-bit-mask-threshold = 1
literal-representation-threshold = 10
trivially-copy-pastes-size = 50
blacklisted-names = ["foo", "baz", "quux"]

# Enable pedantic lints for high code quality
enum-variant-names-threshold = 3
struct-excessive-bools = 3
fn-params-excessive-bools = 3
```

### ESLint Configuration (`.eslintrc.js`)
```javascript
module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint', 'react', 'react-hooks'],
  extends: [
    'eslint:recommended',
    '@typescript-eslint/recommended',
    'react-app',
    'react-app/jest',
  ],
  env: {
    browser: true,
    es6: true,
    node: true,
    jest: true,
  },
  parserOptions: {
    ecmaVersion: 2020,
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
  rules: {
    // TypeScript specific rules
    '@typescript-eslint/no-unused-vars': 'error',
    '@typescript-eslint/explicit-function-return-type': 'warn',
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/prefer-const': 'error',

    // React specific rules
    'react/jsx-uses-react': 'error',
    'react/jsx-uses-vars': 'error',
    'react-hooks/rules-of-hooks': 'error',
    'react-hooks/exhaustive-deps': 'warn',

    // General code quality
    'no-console': process.env.NODE_ENV === 'production' ? 'error' : 'warn',
    'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'warn',
    'prefer-const': 'error',
    'no-var': 'error',
  },
  overrides: [
    {
      files: ['**/*.test.ts', '**/*.test.tsx'],
      env: {
        jest: true,
      },
      rules: {
        '@typescript-eslint/no-explicit-any': 'off',
      },
    },
  ],
};
```

---

## 3. Testing and Coverage

### Test Coverage Configuration (`.tarpaulin.toml`)
```toml
[report]
out = ["Html", "Lcov", "Xml"]
output-dir = "coverage/"
skip-clean = false

[coverage]
exclude = [
    "tests/*",
    "examples/*",
    "benches/*",
    "target/*",
]

line-coverage = true
branch-coverage = true
forward-signals = true
run-types = ["Tests", "Doctests"]
timeout = 120
fail-under = 95
```

### Jest Configuration (`jest.config.js`)
```javascript
module.exports = {
  projects: [
    {
      displayName: 'frontend',
      testMatch: ['<rootDir>/app/**/*.test.{ts,tsx}'],
      setupFilesAfterEnv: ['<rootDir>/app/src/setupTests.ts'],
      testEnvironment: 'jsdom',
      collectCoverageFrom: [
        'app/src/**/*.{ts,tsx}',
        '!app/src/**/*.d.ts',
        '!app/src/index.tsx',
      ],
      coverageThreshold: {
        global: {
          branches: 90,
          functions: 90,
          lines: 90,
          statements: 90,
        },
      },
    },
    {
      displayName: 'sdk-typescript',
      testMatch: ['<rootDir>/sdk/typescript/**/*.test.ts'],
      testEnvironment: 'node',
      collectCoverageFrom: [
        'sdk/typescript/src/**/*.ts',
        '!sdk/typescript/src/**/*.d.ts',
      ],
      coverageThreshold: {
        global: {
          branches: 95,
          functions: 95,
          lines: 95,
          statements: 95,
        },
      },
    },
  ],
  collectCoverage: true,
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html'],
};
```

---

## 4. Security and Compliance

### Security Scanning Script (`scripts/security-checks.sh`)
```bash
#!/bin/bash

set -e

echo "🔒 Running comprehensive security checks..."

# Check for common Solana security issues
echo "📋 Checking for common Solana vulnerabilities..."

# Check for integer overflow/underflow
echo "🔢 Checking for unsafe math operations..."
if rg "\.wrapping_" programs/ --type rust; then
    echo "❌ Found wrapping math operations - use checked math instead"
    exit 1
fi

if rg "as u64|as i64" programs/ --type rust; then
    echo "⚠️  Found unsafe type conversions - review for overflow"
fi

# Check for missing access controls
echo "🔐 Checking access controls..."
if rg "pub fn " programs/ --type rust | rg -v "Context<" | rg -v "#\[access_control\]"; then
    echo "⚠️  Found public functions without proper access control context"
fi

# Check for hardcoded addresses
echo "🏠 Checking for hardcoded addresses..."
if rg "[1-9A-HJ-NP-Za-km-z]{32,44}" programs/ --type rust; then
    echo "⚠️  Found potential hardcoded addresses"
fi

# Check for TODO/FIXME in production code
echo "📝 Checking for unfinished code..."
if rg "TODO|FIXME|XXX|HACK" programs/ --type rust; then
    echo "⚠️  Found unfinished code markers"
fi

# Check for unsafe code blocks
echo "⚠️  Checking for unsafe code..."
if rg "unsafe" programs/ --type rust; then
    echo "❌ Found unsafe code blocks - manual review required"
fi

# Check for proper error handling
echo "🚨 Checking error handling..."
if rg "\.unwrap\(\)|\.expect\(" programs/ --type rust; then
    echo "❌ Found unwrap()/expect() calls - use proper error handling"
    exit 1
fi

# Check for proper logging
echo "📊 Checking event emission..."
program_files=$(find programs -name "lib.rs" -o -name "*.rs" | grep -v target)
for file in $program_files; do
    if grep -q "pub fn " "$file" && ! grep -q "emit!" "$file"; then
        echo "⚠️  $file: Public functions should emit events for monitoring"
    fi
done

echo "✅ Security checks completed"
```

### License Compliance Script (`scripts/check-license-compliance.sh`)
```bash
#!/bin/bash

set -e

echo "📄 Checking license compliance..."

# Generate license report
cargo license --json > licenses.json

# Check for GPL licenses (not compatible with commercial use)
if jq -r '.[] | select(.license | test("GPL")) | .name' licenses.json | grep -q .; then
    echo "❌ Found GPL licensed dependencies - not compatible"
    jq -r '.[] | select(.license | test("GPL")) | "\(.name): \(.license)"' licenses.json
    exit 1
fi

# Check for unknown licenses
if jq -r '.[] | select(.license == null or .license == "UNKNOWN") | .name' licenses.json | grep -q .; then
    echo "⚠️  Found dependencies with unknown licenses:"
    jq -r '.[] | select(.license == null or .license == "UNKNOWN") | .name' licenses.json
fi

# Approved licenses
approved_licenses=(
    "MIT"
    "Apache-2.0"
    "BSD-2-Clause"
    "BSD-3-Clause"
    "ISC"
    "CC0-1.0"
    "Unlicense"
    "MPL-2.0"
)

# Check for unapproved licenses
echo "📋 Checking for unapproved licenses..."
unapproved_found=false

while IFS= read -r license; do
    if [[ " ${approved_licenses[@]} " =~ " ${license} " ]]; then
        continue
    else
        if [ "$license" != "null" ] && [ -n "$license" ]; then
            echo "⚠️  Unapproved license found: $license"
            unapproved_found=true
        fi
    fi
done < <(jq -r '.[] | .license' licenses.json | sort -u)

if [ "$unapproved_found" = true ]; then
    echo "❌ Manual review required for unapproved licenses"
    exit 1
fi

echo "✅ All licenses are approved"
rm licenses.json
```

---

## 5. Performance Monitoring

### Benchmark Configuration (`benches/criterion_config.rs`)
```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use moby_math::price::Price;
use moby_math::slippage::SlippageCalculator;

fn price_calculation_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("price_calculations");

    // Test with different order sizes
    let order_sizes = vec![
        1_000_000,      // $1M
        10_000_000,     // $10M
        100_000_000,    // $100M
        1_000_000_000,  // $1B
    ];

    for size in order_sizes {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("price_multiply", size),
            &size,
            |b, &size| {
                let price = Price::from_float(150.25, 6).unwrap();
                b.iter(|| price.multiply_amount(size))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("slippage_calculation", size),
            &size,
            |b, &size| {
                let expected = size;
                let actual = size - (size / 1000); // 0.1% slippage
                b.iter(|| SlippageCalculator::calculate_slippage(expected, actual))
            },
        );
    }

    group.finish();
}

fn whale_trading_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("whale_scenarios");

    // Benchmark realistic whale trading scenarios
    group.bench_function("whale_otc_order_creation", |b| {
        b.iter(|| {
            // Simulate creating $50M SOL/USDC order
            let sol_price = Price::from_float(143.50, 6).unwrap();
            let usdc_amount = 50_000_000_000_000; // $50M
            let sol_amount = sol_price.divide_amount(usdc_amount).unwrap();
            let slippage = SlippageCalculator::calculate_slippage(usdc_amount, usdc_amount - 5000).unwrap();
            (sol_amount, slippage)
        })
    });

    group.finish();
}

criterion_group!(benches, price_calculation_benchmarks, whale_trading_scenarios);
criterion_main!(benches);
```

### Performance Test Script (`scripts/performance-tests.sh`)
```bash
#!/bin/bash

set -e

echo "🚀 Running performance tests..."

# Build in release mode
echo "🔨 Building release version..."
cargo build --release

# Run Rust benchmarks
echo "📊 Running Rust benchmarks..."
cargo bench --workspace

# Performance regression check
echo "📈 Checking for performance regressions..."
if [ -f "previous_benchmark.json" ]; then
    cargo bench --workspace -- --save-baseline current
    cargo bench --workspace -- --baseline previous --load-baseline current | tee perf_comparison.txt

    # Check if any benchmark regressed by more than 10%
    if grep -q "change:.*+[1-9][0-9]\." perf_comparison.txt; then
        echo "❌ Performance regression detected!"
        exit 1
    fi
fi

# Save current benchmarks for next comparison
cp target/criterion/*/base/benchmark.json previous_benchmark.json

echo "✅ Performance tests completed"
```

---

## 6. Monitoring and Alerting

### Prometheus Configuration (`docker/prometheus.yml`)
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'moby-market-api'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'solana-validator'
    static_configs:
      - targets: ['localhost:8899']
    metrics_path: /metrics

  - job_name: 'postgres'
    static_configs:
      - targets: ['localhost:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['localhost:9121']

rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Alert Rules (`docker/alert_rules.yml`)
```yaml
groups:
  - name: moby-market-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "95th percentile latency is {{ $value }} seconds"

      - alert: SlippageThresholdExceeded
        expr: avg_over_time(order_slippage_bps[5m]) > 100
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High slippage detected"
          description: "Average slippage is {{ $value }} basis points"

      - alert: LowLiquidity
        expr: total_liquidity_usd < 10000000
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Low liquidity warning"
          description: "Total liquidity is ${{ $value }}"
```

This comprehensive CI/CD setup ensures:
- **Automated quality checks** on every commit
- **Security scanning** for vulnerabilities
- **Performance monitoring** to catch regressions
- **Gradual deployments** with rollback capabilities
- **Comprehensive monitoring** of production systems

The pipeline prioritizes reliability and security while maintaining developer productivity with fast feedback loops.

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[{"content": "Create core library specifications and interfaces", "status": "completed", "activeForm": "Creating core library specifications and interfaces"}, {"content": "Set up comprehensive testing framework", "status": "completed", "activeForm": "Setting up comprehensive testing framework"}, {"content": "Create development environment setup guide", "status": "completed", "activeForm": "Creating development environment setup guide"}, {"content": "Document implementation order and dependencies", "status": "completed", "activeForm": "Documenting implementation order and dependencies"}, {"content": "Set up CI/CD and code quality tools", "status": "completed", "activeForm": "Setting up CI/CD and code quality tools"}]