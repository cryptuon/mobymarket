# Moby Oracle 🐋📊

A comprehensive decentralized oracle infrastructure designed specifically for whale trading operations, providing reliable, secure price feeds and advanced data aggregation with anti-manipulation features.

## 🌟 Features

- **📊 Multi-Source Aggregation**: Combine data from multiple oracle networks (Chainlink, Pyth, Band Protocol, API3, UMA)
- **🐋 Whale Trading Focus**: Specialized handling for large-volume market data and whale activity detection
- **🔒 Cryptographic Security**: Advanced data integrity verification and fraud detection systems
- **⚡ High Performance**: Optimized for sub-second data updates with concurrent processing
- **🔄 Real-time Streaming**: Live market data with WebSocket connections and adaptive feed management
- **📈 Historical Data**: Time-series data storage and analytics with configurable retention
- **🛡️ MEV Protection**: Comprehensive MEV attack detection and prevention mechanisms
- **🏛️ Decentralized Governance**: Community-driven oracle parameter management and source curation

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
moby-oracle = "0.1.0"
```

### Basic Usage

```rust
use moby_oracle::{OracleSystem, PriceFeedConfig, DataSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize oracle system
    let oracle = OracleSystem::new().await?;

    // Configure price feed
    let feed_config = PriceFeedConfig {
        symbol: "ETH/USD".to_string(),
        sources: vec![DataSource::Chainlink, DataSource::Pyth],
        update_frequency: std::time::Duration::from_secs(1),
        deviation_threshold: 0.01, // 1%
    };

    // Start price feed
    let feed_id = oracle.create_price_feed(feed_config).await?;

    // Get latest price
    let price = oracle.get_latest_price(&feed_id).await?;
    println!("ETH/USD: ${}", price.value);

    Ok(())
}
```

### Advanced Whale Trading Example

```rust
use moby_oracle::{
    aggregation::{Aggregator, AggregationConfig, AggregationStrategy},
    security::{SecurityValidator, SecurityConfig},
    sources::{DataPoint, DataSource}
};
use rust_decimal::Decimal;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure for whale trading
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::VolumeWeightedAverage;
    config.whale_config.whale_volume_threshold = Decimal::from(1_000_000); // $1M

    let mut aggregator = Aggregator::new(config);
    let mut validator = SecurityValidator::new(SecurityConfig::default());

    // Create whale-sized order data
    let whale_data = vec![
        DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(5_000_000)), // $5M volume
            metadata: std::collections::HashMap::new(),
        }
    ];

    // Validate and aggregate
    for data_point in &whale_data {
        let validation = validator.validate_data_point(data_point).await?;
        println!("Validation: {} (confidence: {:.1}%)",
            validation.is_valid, validation.confidence_score * 100.0);
    }

    let result = aggregator.aggregate_prices("ETH/USD", whale_data).await?;

    if result.whale_impact.whale_activity_detected {
        println!("🐋 Whale activity detected!");
        println!("Price impact: {:.1} basis points", result.whale_impact.price_impact_bps);
        println!("Liquidity depth: ${:.1}M",
            result.whale_impact.liquidity_depth.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0);
    }

    Ok(())
}
```

## 📚 Core Components

### Data Sources (`sources.rs`)

The oracle supports multiple data source providers:

- **Chainlink**: Industry-standard decentralized oracle network
- **Pyth**: High-frequency financial market data
- **Band Protocol**: Cross-chain data oracle platform
- **API3**: First-party oracle solutions
- **UMA**: Optimistic oracle system

Each source provides:
- Real-time price feeds
- Health monitoring
- Automatic failover
- Rate limiting and retry logic

### Aggregation Strategies (`aggregation.rs`)

Multiple aggregation methods for different use cases:

- **Median**: Robust against outliers, good for volatile markets
- **Weighted Average**: Source reliability-based weighting
- **TWAP**: Time-weighted average price for trend analysis
- **VWAP**: Volume-weighted average price for whale trading
- **Consensus**: Require agreement between sources within threshold
- **Custom**: Implement your own aggregation logic

### Security & Validation (`security.rs`)

Comprehensive security features:

- **Data Integrity Verification**: Cryptographic signature validation
- **Fraud Detection**: Anomaly detection and manipulation alerts
- **MEV Attack Protection**: Front-running and sandwich attack detection
- **Circuit Breakers**: Automatic source disconnection on failures
- **Reputation System**: Dynamic source credibility scoring

### Whale Trading Analytics

Specialized features for large-volume operations:

- **Whale Activity Detection**: Automatic identification of large orders
- **Price Impact Analysis**: Estimate market impact of large trades
- **Liquidity Depth Assessment**: Real-time liquidity availability
- **Slippage Protection**: Advanced slippage calculation and warnings
- **Order Size Optimization**: Recommended order splitting strategies

## 🔧 Configuration

### Aggregation Configuration

```rust
use moby_oracle::aggregation::{AggregationConfig, AggregationStrategy, WhaleAggregationConfig};
use rust_decimal::Decimal;
use std::time::Duration;

let config = AggregationConfig {
    strategy: AggregationStrategy::Consensus {
        min_sources: 3,
        threshold: Decimal::from_f64_retain(0.01).unwrap(), // 1%
    },
    fallback_strategy: Some(AggregationStrategy::Median),
    max_data_age: Duration::from_secs(300), // 5 minutes
    min_sources: 3,
    max_deviation: Decimal::from_f64_retain(0.05).unwrap(), // 5%
    outlier_threshold: 2.0, // 2 standard deviations
    whale_config: WhaleAggregationConfig {
        whale_volume_threshold: Decimal::from(1_000_000),
        liquidity_depth_levels: 10,
        volatility_window: Duration::from_secs(3600),
        // ... other whale-specific settings
    },
    // ... other configuration options
};
```

### Security Configuration

```rust
use moby_oracle::security::{SecurityConfig, WhaleSecurityConfig, FraudRiskThresholds};
use rust_decimal::Decimal;
use std::time::Duration;

let config = SecurityConfig {
    max_price_deviation: 0.05, // 5%
    max_data_age: Duration::from_secs(300),
    min_correlation: 0.7,
    mev_detection_sensitivity: 0.6,
    circuit_breaker_threshold: 5,
    reputation_decay_rate: 0.001,
    fraud_risk_thresholds: FraudRiskThresholds {
        very_low: 0.1,
        low: 0.25,
        medium: 0.5,
        high: 0.75,
    },
    whale_security: WhaleSecurityConfig {
        large_order_threshold: Decimal::from(1_000_000),
        price_impact_threshold: 0.02,
        mev_protection_enabled: true,
        slippage_protection: 0.005,
    },
};
```

## 📊 Examples

The library includes comprehensive examples:

- **`price_feed_aggregation.rs`**: Basic price feed setup and aggregation strategies
- **`whale_market_data.rs`**: Advanced whale trading analytics and detection
- **`oracle_monitoring.rs`**: System health monitoring and performance tracking
- **`custom_data_feeds.rs`**: Custom source implementation and feed configuration
- **`oracle_governance.rs`**: Decentralized governance and parameter management

Run examples with:

```bash
cargo run --example price_feed_aggregation
cargo run --example whale_market_data
cargo run --example oracle_monitoring
cargo run --example custom_data_feeds
cargo run --example oracle_governance
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Unit tests
cargo test unit_tests

# Integration tests
cargo test integration_tests

# Performance tests
cargo test performance_tests

# All tests
cargo test
```

### Test Coverage

- **Unit Tests**: Individual component testing
- **Integration Tests**: End-to-end oracle workflows
- **Performance Tests**: Load testing and benchmarking
- **Security Tests**: Fraud detection and validation
- **Whale Trading Tests**: Large volume scenario testing

## 🔒 Security Features

### MEV Protection

The oracle includes sophisticated MEV (Maximal Extractable Value) attack detection:

- **Front-running Detection**: Identify suspicious timing patterns
- **Sandwich Attack Prevention**: Detect coordinated price manipulation
- **Flash Loan Attack Recognition**: Monitor for unusual volume spikes
- **Oracle Manipulation Alerts**: Cross-reference with on-chain data

### Data Validation

Multi-layered validation ensures data integrity:

- **Signature Verification**: Cryptographic proof of data authenticity
- **Timestamp Validation**: Ensure data freshness and prevent replay attacks
- **Cross-Source Correlation**: Detect inconsistencies across providers
- **Statistical Outlier Detection**: Filter anomalous price movements

### Circuit Breakers

Automatic protection mechanisms:

- **Source Isolation**: Disconnect unreliable data sources
- **Rate Limiting**: Prevent spam and DoS attacks
- **Emergency Shutdown**: Manual override for critical situations
- **Reputation Tracking**: Dynamic source credibility assessment

## 🌊 Whale Trading Features

### Activity Detection

Sophisticated algorithms identify whale trading patterns:

- **Volume Thresholds**: Configurable detection levels
- **Pattern Recognition**: Identify characteristic whale behaviors
- **Cross-Market Analysis**: Monitor activity across multiple venues
- **Real-time Alerts**: Immediate notification of whale movements

### Impact Analysis

Comprehensive market impact assessment:

- **Price Impact Modeling**: Linear, square-root, and order-book models
- **Liquidity Depth Analysis**: Real-time liquidity assessment
- **Slippage Calculation**: Advanced slippage prediction
- **Order Optimization**: Recommended execution strategies

### Risk Management

Advanced risk assessment for large orders:

- **Market Condition Analysis**: Volatility and liquidity scoring
- **Timing Recommendations**: Optimal execution windows
- **Size Limitations**: Dynamic order size recommendations
- **Cross-Exchange Coordination**: Multi-venue execution strategies

## 🏛️ Governance

### Decentralized Management

Community-driven oracle governance:

- **Parameter Voting**: Adjust oracle settings through community consensus
- **Source Management**: Add or remove data sources via governance
- **Security Updates**: Community-approved security parameter changes
- **Emergency Procedures**: Rapid response to critical situations

### Reputation System

Merit-based governance participation:

- **Performance Tracking**: Monitor oracle operator performance
- **Stake-based Voting**: Weighted voting based on stake and reputation
- **Delegation Mechanisms**: Delegate voting power to trusted operators
- **Penalty Systems**: Slashing for malicious or negligent behavior

## 🔧 Custom Implementation

### Custom Data Sources

Implement your own data source providers:

```rust
use moby_oracle::sources::{SourceProvider, DataPoint, SourceHealth};
use async_trait::async_trait;

struct MyCustomSource;

#[async_trait]
impl SourceProvider for MyCustomSource {
    fn source_type(&self) -> DataSource {
        DataSource::API3 // Use appropriate enum value
    }

    async fn fetch_price(&self, symbol: &str) -> OracleResult<DataPoint> {
        // Implement your price fetching logic
        // ...
    }

    async fn health_check(&self) -> OracleResult<SourceHealth> {
        // Implement health checking
        // ...
    }

    async fn get_supported_symbols(&self) -> OracleResult<Vec<String>> {
        // Return supported trading pairs
        // ...
    }
}
```

### Custom Aggregation Strategies

Create specialized aggregation methods:

```rust
use moby_oracle::aggregation::{AggregationStrategy, AggregatedPrice};

// Implement custom liquidity-weighted aggregation
pub async fn liquidity_weighted_aggregation(
    data_points: Vec<DataPoint>
) -> Result<AggregatedPrice> {
    // Your custom aggregation logic
    // Weight prices by available liquidity
    // ...
}
```

### Custom Validation Rules

Add domain-specific validation:

```rust
use moby_oracle::security::{ValidationResult, SecurityWarning};

pub async fn custom_defi_validation(
    data_point: &DataPoint
) -> Result<ValidationResult> {
    // Custom validation for DeFi protocols
    // Check for specific DeFi attack patterns
    // Validate against on-chain data
    // ...
}
```

## 📈 Performance

### Benchmarks

Typical performance metrics:

- **Aggregation Throughput**: 1000+ operations/second
- **Validation Speed**: 500+ validations/second
- **Memory Usage**: ~50MB base + ~10KB per active feed
- **Latency**: Sub-100ms for most operations
- **Concurrent Feeds**: 100+ simultaneous price feeds

### Optimization Tips

1. **Source Selection**: Use geographically distributed sources
2. **Caching Strategy**: Implement appropriate cache TTLs
3. **Batch Operations**: Group related operations when possible
4. **Resource Monitoring**: Monitor memory and CPU usage
5. **Network Optimization**: Use persistent connections where possible

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/moby-market/moby-oracle.git
cd moby-oracle

# Install dependencies
cargo build

# Run tests
cargo test

# Run examples
cargo run --example price_feed_aggregation
```

### Code Style

- Follow Rust standard formatting with `cargo fmt`
- Ensure all tests pass with `cargo test`
- Add documentation for public APIs
- Include examples for new features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Chainlink](https://chain.link/) for oracle infrastructure inspiration
- [Pyth Network](https://pyth.network/) for high-frequency data concepts
- [Band Protocol](https://bandprotocol.com/) for cross-chain oracle architecture
- [API3](https://api3.org/) for first-party oracle innovations
- [UMA](https://umaproject.org/) for optimistic oracle mechanisms

## 📞 Support

- **Documentation**: [docs.moby-market.com/oracle](https://docs.moby-market.com/oracle)
- **Discord**: [Moby Market Community](https://discord.gg/moby-market)
- **Issues**: [GitHub Issues](https://github.com/moby-market/moby-oracle/issues)
- **Email**: oracle@moby-market.com

---

Built with 🐋 by the Moby Market team for the next generation of whale trading infrastructure.