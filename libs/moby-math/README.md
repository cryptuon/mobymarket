# moby-math

High-precision mathematical utilities for whale-scale cryptocurrency trading.

## Features

### Price Calculations
- **Whale-scale precision**: Handle amounts up to $1 billion with minimal rounding errors
- **Multi-token support**: Convert between USD and tokens with configurable decimal precision
- **Overflow protection**: Safe arithmetic operations with comprehensive error handling

### Slippage Modeling
- **Market impact calculation**: Progressive slippage models for different market conditions
- **Whale tier classification**: Automatic tier-based slippage estimation ($1M-$500M+)
- **TWAP cost estimation**: Time-weighted average price execution cost modeling
- **Optimal chunking**: Calculate optimal trade sizes to minimize market impact

### Utility Functions
- **Basis points calculation**: Standard financial calculations
- **Decimal normalization**: Convert between different token decimal standards
- **Volume-weighted pricing**: VWAP calculations for multiple price points
- **Gas cost estimation**: Ethereum transaction cost modeling for complex trades

## Usage

```rust
use moby_math::{Price, SlippageCalculator, SlippageParams};

// Calculate BTC amount for $100M purchase
let btc_price = Price::from_float(43_250.0, 6)?;
let whale_budget = 100_000_000 * Price::PRECISION; // $100M
let btc_amount = btc_price.divide_amount(whale_budget)?;

// Estimate slippage for large trade
let slippage_calc = SlippageCalculator::new(SlippageParams::default());
let market_liquidity = 500_000_000 * Price::PRECISION; // $500M liquidity
let slippage = slippage_calc.calculate_slippage(btc_amount, market_liquidity)?;

// Optimize execution with chunking
let chunks = slippage_calc.calculate_whale_optimal_chunks(
    btc_amount,
    market_liquidity,
    0.02 // 2% max impact per chunk
)?;

// Estimate TWAP execution cost
let twap_cost = slippage_calc.estimate_twap_execution_cost(
    btc_amount,
    market_liquidity,
    24 // 24-hour execution
)?;
```

## Whale Trading Scenarios

### Sovereign Wealth Fund ($500M ETH Purchase)
```rust
let eth_price = Price::from_float(2_500.0, 6)?;
let allocation = 500_000_000 * Price::PRECISION;
let eth_amount = eth_price.divide_amount(allocation)?;

// Calculate optimal execution strategy
let chunks = slippage_calc.calculate_whale_optimal_chunks(
    eth_amount,
    1_000_000_000 * Price::PRECISION, // $1B ETH liquidity
    0.015 // 1.5% max impact per chunk
)?;
```

### Multi-Asset Portfolio Diversification
```rust
let assets = vec![
    ("BTC", 43_250.0, 40.0), // 40% allocation
    ("ETH", 2_500.0, 30.0),  // 30% allocation
    ("SOL", 100.0, 20.0),    // 20% allocation
    ("AVAX", 35.0, 10.0),    // 10% allocation
];

let total_portfolio = 200_000_000 * Price::PRECISION;
// Calculate slippage for each asset allocation...
```

## Testing

The library includes comprehensive test coverage:

- **22 unit tests**: Core functionality and edge cases
- **7 integration tests**: Real-world whale trading scenarios
- **9 property tests**: Mathematical properties and invariants
- **Performance benchmarks**: Execution speed for high-frequency calculations

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test integration_tests
cargo test --test property_tests

# Run benchmarks
cargo bench
```

## Test Scenarios Covered

- **$100M BTC purchase** with market impact analysis
- **$500M sovereign wealth fund** ETH allocation
- **Multi-asset portfolio** diversification ($200M across 4 assets)
- **TWAP vs market order** comparison for $25M trades
- **Extreme market conditions** with thin liquidity
- **Precision preservation** for various price ranges and amounts

## Architecture

### Core Types

- `Price`: High-precision price representation with configurable decimals
- `SlippageCalculator`: Market impact and execution cost modeling
- `SlippageParams`: Configurable market depth and impact parameters
- `SlippageResult`: Detailed breakdown of slippage components

### Error Handling

All operations return `Result` types with descriptive error messages:
- `PriceError`: Price calculation and conversion errors
- `MathError`: General mathematical operation errors
- Overflow protection for all whale-scale calculations

### Performance

- **Zero-copy operations** where possible
- **Efficient decimal arithmetic** using `rust_decimal`
- **Minimal allocations** for high-frequency calculations
- **Property-based testing** ensures correctness across parameter ranges

## Dependencies

- `rust_decimal`: High-precision decimal arithmetic
- `serde`: Serialization support for all public types
- `thiserror`: Structured error handling
- `proptest`: Property-based testing framework
- `criterion`: Performance benchmarking

## License

MIT License - See LICENSE file for details.