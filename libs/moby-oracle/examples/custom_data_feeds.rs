//! # Custom Data Feeds Example
//!
//! This example demonstrates how to create and integrate custom data sources
//! and feeds into the moby-oracle system, including:
//! - Custom source provider implementation
//! - Real-time data streaming
//! - Custom aggregation strategies
//! - Feed configuration and management
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example custom_data_feeds
//! ```

use moby_oracle::*;
use moby_oracle::sources::*;
use moby_oracle::aggregation::*;
use moby_oracle::security::*;
use std::collections::HashMap;
use std::time::Duration;
use chrono::Utc;
use rust_decimal::Decimal;
use tokio::time::sleep;
use async_trait::async_trait;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("🔧 Moby Oracle - Custom Data Feeds Example");
    println!("==========================================");

    // Step 1: Create custom data source providers
    println!("\n🏗️  Creating Custom Data Source Providers:");
    let custom_providers = create_custom_providers().await?;

    // Step 2: Implement custom aggregation strategy
    println!("\n⚙️  Implementing Custom Aggregation Strategy:");
    await demonstrate_custom_aggregation().await?;

    // Step 3: Set up real-time data streaming
    println!("\n📡 Setting Up Real-time Data Streaming:");
    await demonstrate_realtime_streaming(&custom_providers).await?;

    // Step 4: Create custom feed configurations
    println!("\n📊 Custom Feed Configuration:");
    await demonstrate_feed_configuration().await?;

    // Step 5: Advanced custom validation
    println!("\n🔍 Custom Validation Rules:");
    await demonstrate_custom_validation().await?;

    // Step 6: Multi-asset feed management
    println!("\n🎯 Multi-Asset Feed Management:");
    await demonstrate_multi_asset_feeds().await?;

    println!("\n✅ Custom data feeds example completed!");
    Ok(())
}

/// Custom data source for cryptocurrency exchange
struct CryptoExchangeProvider {
    exchange_name: String,
    api_endpoint: String,
    supported_pairs: Vec<String>,
}

#[async_trait]
impl SourceProvider for CryptoExchangeProvider {
    fn source_type(&self) -> DataSource {
        DataSource::Chainlink // Using Chainlink as placeholder for custom source
    }

    async fn fetch_price(&self, symbol: &str) -> OracleResult<DataPoint> {
        // Simulate API call to exchange
        sleep(Duration::from_millis(50 + rand::random::<u64>() % 100)).await;

        if !self.supported_pairs.contains(&symbol.to_string()) {
            return Err(OracleError::UnsupportedSymbol {
                symbol: symbol.to_string(),
                source: self.source_type(),
            });
        }

        // Simulate price data from exchange
        let base_price = match symbol {
            "ETH/USD" => 2000.0,
            "BTC/USD" => 50000.0,
            "SOL/USD" => 100.0,
            _ => 100.0,
        };

        let price_variation = (rand::random::<f64>() - 0.5) * 0.02; // ±1% variation
        let final_price = base_price * (1.0 + price_variation);

        let volume = 50_000.0 + rand::random::<f64>() * 500_000.0;

        let mut metadata = HashMap::new();
        metadata.insert("exchange".to_string(), self.exchange_name.clone());
        metadata.insert("pair_type".to_string(), "spot".to_string());
        metadata.insert("bid_ask_spread".to_string(), "0.02".to_string());

        Ok(DataPoint {
            source: self.source_type(),
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(final_price).unwrap(),
            timestamp: Utc::now(),
            confidence: 0.90 + rand::random::<f64>() * 0.08, // 90-98% confidence
            volume: Some(Decimal::from_f64_retain(volume).unwrap()),
            metadata,
        })
    }

    async fn health_check(&self) -> OracleResult<SourceHealth> {
        // Simulate health check
        let is_healthy = rand::random::<f64>() > 0.1; // 90% chance of being healthy

        Ok(SourceHealth {
            is_healthy,
            last_successful_fetch: Some(Utc::now() - chrono::Duration::seconds(30)),
            consecutive_failures: if is_healthy { 0 } else { 2 },
            average_response_time: Duration::from_millis(75),
            error_rate: if is_healthy { 0.02 } else { 0.15 },
            status_message: if is_healthy {
                "Exchange API operational".to_string()
            } else {
                "Experiencing minor delays".to_string()
            },
        })
    }

    async fn get_supported_symbols(&self) -> OracleResult<Vec<String>> {
        Ok(self.supported_pairs.clone())
    }
}

/// Custom DeFi protocol provider
struct DeFiProtocolProvider {
    protocol_name: String,
    chain: String,
}

#[async_trait]
impl SourceProvider for DeFiProtocolProvider {
    fn source_type(&self) -> DataSource {
        DataSource::Pyth // Using Pyth as placeholder for custom DeFi source
    }

    async fn fetch_price(&self, symbol: &str) -> OracleResult<DataPoint> {
        // Simulate on-chain price fetch
        sleep(Duration::from_millis(200 + rand::random::<u64>() % 300)).await;

        let base_price = match symbol {
            "ETH/USD" => 2000.0,
            "BTC/USD" => 50000.0,
            "SOL/USD" => 100.0,
            _ => return Err(OracleError::UnsupportedSymbol {
                symbol: symbol.to_string(),
                source: self.source_type(),
            }),
        };

        // DeFi protocols often have slightly different prices due to AMM mechanics
        let amm_premium = 0.001 + rand::random::<f64>() * 0.004; // 0.1-0.5% premium
        let final_price = base_price * (1.0 + amm_premium);

        let liquidity = 1_000_000.0 + rand::random::<f64>() * 10_000_000.0;

        let mut metadata = HashMap::new();
        metadata.insert("protocol".to_string(), self.protocol_name.clone());
        metadata.insert("chain".to_string(), self.chain.clone());
        metadata.insert("pool_liquidity".to_string(), liquidity.to_string());
        metadata.insert("fee_tier".to_string(), "0.3".to_string());

        Ok(DataPoint {
            source: self.source_type(),
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(final_price).unwrap(),
            timestamp: Utc::now(),
            confidence: 0.85 + rand::random::<f64>() * 0.10, // 85-95% confidence
            volume: Some(Decimal::from_f64_retain(liquidity / 10.0).unwrap()), // Volume as fraction of liquidity
            metadata,
        })
    }

    async fn health_check(&self) -> OracleResult<SourceHealth> {
        Ok(SourceHealth {
            is_healthy: true,
            last_successful_fetch: Some(Utc::now() - chrono::Duration::seconds(15)),
            consecutive_failures: 0,
            average_response_time: Duration::from_millis(250),
            error_rate: 0.05,
            status_message: format!("{} protocol on {} functioning normally", self.protocol_name, self.chain),
        })
    }

    async fn get_supported_symbols(&self) -> OracleResult<Vec<String>> {
        Ok(vec!["ETH/USD".to_string(), "BTC/USD".to_string(), "SOL/USD".to_string()])
    }
}

/// Create custom provider instances
async fn create_custom_providers() -> Result<HashMap<String, Box<dyn SourceProvider>>> {
    let mut providers: HashMap<String, Box<dyn SourceProvider>> = HashMap::new();

    // Create crypto exchange providers
    let binance = Box::new(CryptoExchangeProvider {
        exchange_name: "Binance".to_string(),
        api_endpoint: "https://api.binance.com".to_string(),
        supported_pairs: vec!["ETH/USD".to_string(), "BTC/USD".to_string(), "SOL/USD".to_string()],
    });

    let coinbase = Box::new(CryptoExchangeProvider {
        exchange_name: "Coinbase".to_string(),
        api_endpoint: "https://api.coinbase.com".to_string(),
        supported_pairs: vec!["ETH/USD".to_string(), "BTC/USD".to_string()],
    });

    // Create DeFi protocol providers
    let uniswap = Box::new(DeFiProtocolProvider {
        protocol_name: "Uniswap V3".to_string(),
        chain: "Ethereum".to_string(),
    });

    let curve = Box::new(DeFiProtocolProvider {
        protocol_name: "Curve Finance".to_string(),
        chain: "Ethereum".to_string(),
    });

    providers.insert("binance".to_string(), binance);
    providers.insert("coinbase".to_string(), coinbase);
    providers.insert("uniswap".to_string(), uniswap);
    providers.insert("curve".to_string(), curve);

    println!("   ✅ Created {} custom data providers", providers.len());

    // Test each provider
    for (name, provider) in &providers {
        println!("   Testing provider: {}", name);

        let health = provider.health_check().await?;
        println!("     Health: {} - {}",
            if health.is_healthy { "🟢" } else { "🔴" },
            health.status_message);

        let symbols = provider.get_supported_symbols().await?;
        println!("     Supported symbols: {:?}", symbols);

        // Test price fetch
        if !symbols.is_empty() {
            let price_data = provider.fetch_price(&symbols[0]).await?;
            println!("     Sample price for {}: ${:.2}", symbols[0], price_data.value);
        }

        println!();
    }

    Ok(providers)
}

/// Custom aggregation strategy: Liquidity-Weighted Average
struct LiquidityWeightedStrategy;

impl LiquidityWeightedStrategy {
    async fn aggregate_with_liquidity(&self, data_points: Vec<DataPoint>) -> Result<AggregatedPrice> {
        if data_points.is_empty() {
            return Err(OracleError::NoDataPointsProvided);
        }

        let mut total_weighted_price = Decimal::ZERO;
        let mut total_liquidity = Decimal::ZERO;

        for point in &data_points {
            // Extract liquidity from metadata or use volume as proxy
            let liquidity = if let Some(liq_str) = point.metadata.get("pool_liquidity") {
                Decimal::from_f64_retain(liq_str.parse::<f64>().unwrap_or(0.0)).unwrap_or_default()
            } else {
                point.volume.unwrap_or(Decimal::from(100_000)) * Decimal::from(10) // Volume * 10 as liquidity proxy
            };

            total_weighted_price += point.value * liquidity;
            total_liquidity += liquidity;
        }

        let final_price = if total_liquidity > Decimal::ZERO {
            total_weighted_price / total_liquidity
        } else {
            // Fallback to simple average
            let sum: Decimal = data_points.iter().map(|p| p.value).sum();
            sum / Decimal::from(data_points.len())
        };

        // Calculate quality metrics
        let prices: Vec<f64> = data_points.iter()
            .map(|p| p.value.to_string().parse::<f64>().unwrap_or(0.0))
            .collect();

        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        let quality_metrics = QualityMetrics {
            price_variance: Decimal::from_f64_retain(variance).unwrap_or_default(),
            max_spread: Decimal::from_f64_retain(
                prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)) -
                prices.iter().fold(f64::INFINITY, |a, &b| a.min(b))
            ).unwrap_or_default(),
            outlier_rate: 0.0,
            avg_latency_ms: 150,
            source_reliability: HashMap::new(),
        };

        let whale_impact = WhaleImpactAnalysis {
            price_impact_bps: Decimal::from_f64_retain(5.0).unwrap(),
            liquidity_depth: total_liquidity,
            volatility_score: variance.sqrt().min(1.0),
            whale_activity_detected: total_liquidity > Decimal::from(5_000_000),
            max_order_size: Some(total_liquidity / Decimal::from(20)), // 5% of total liquidity
        };

        Ok(AggregatedPrice {
            price: final_price,
            symbol: data_points[0].symbol.clone(),
            timestamp: Utc::now(),
            confidence: 0.85 + (data_points.len() as f64 * 0.02).min(0.1), // Higher confidence with more sources
            source_count: data_points.len(),
            contributing_sources: data_points.iter().map(|p| p.source).collect(),
            deviation: None,
            strategy: AggregationStrategy::Custom { name: "LiquidityWeighted".to_string() },
            quality_metrics,
            whale_impact,
        })
    }
}

/// Demonstrate custom aggregation strategy
async fn demonstrate_custom_aggregation() -> Result<()> {
    let strategy = LiquidityWeightedStrategy;

    // Create test data with different liquidity levels
    let high_liquidity_data = DataPoint {
        source: DataSource::Pyth,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(2000),
        timestamp: Utc::now(),
        confidence: 0.95,
        volume: Some(Decimal::from(500_000)),
        metadata: {
            let mut map = HashMap::new();
            map.insert("pool_liquidity".to_string(), "10000000".to_string()); // $10M liquidity
            map
        },
    };

    let medium_liquidity_data = DataPoint {
        source: DataSource::Chainlink,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(2010),
        timestamp: Utc::now(),
        confidence: 0.92,
        volume: Some(Decimal::from(200_000)),
        metadata: {
            let mut map = HashMap::new();
            map.insert("pool_liquidity".to_string(), "2000000".to_string()); // $2M liquidity
            map
        },
    };

    let low_liquidity_data = DataPoint {
        source: DataSource::Band,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(2020),
        timestamp: Utc::now(),
        confidence: 0.88,
        volume: Some(Decimal::from(50_000)),
        metadata: {
            let mut map = HashMap::new();
            map.insert("pool_liquidity".to_string(), "500000".to_string()); // $500K liquidity
            map
        },
    };

    let data_points = vec![high_liquidity_data, medium_liquidity_data, low_liquidity_data];

    println!("   Input data:");
    for (i, point) in data_points.iter().enumerate() {
        let liquidity = point.metadata.get("pool_liquidity").unwrap();
        println!("     Source {}: ${:.2} (Liquidity: ${})",
            i+1, point.value, liquidity);
    }

    let result = strategy.aggregate_with_liquidity(data_points).await?;

    println!("\n   Liquidity-Weighted Aggregation Result:");
    println!("     Final Price: ${:.2}", result.price);
    println!("     Total Liquidity: ${:.1}M",
        result.whale_impact.liquidity_depth.to_string().parse::<f64>().unwrap() / 1_000_000.0);
    println!("     Confidence: {:.1}%", result.confidence * 100.0);
    println!("     Strategy: {:?}", result.strategy);

    // Compare with simple average
    let simple_avg: Decimal = vec![
        Decimal::from(2000),
        Decimal::from(2010),
        Decimal::from(2020),
    ].iter().sum::<Decimal>() / Decimal::from(3);

    println!("\n   Comparison:");
    println!("     Simple Average: ${:.2}", simple_avg);
    println!("     Liquidity-Weighted: ${:.2}", result.price);
    println!("     Difference: ${:.2}", (result.price - simple_avg).abs());

    Ok(())
}

/// Demonstrate real-time data streaming
async fn demonstrate_realtime_streaming(providers: &HashMap<String, Box<dyn SourceProvider>>) -> Result<()> {
    println!("   Starting 8-second real-time streaming simulation...");

    let symbols = vec!["ETH/USD", "BTC/USD"];
    let provider_names: Vec<_> = providers.keys().collect();

    let streaming_start = Instant::now();

    while streaming_start.elapsed() < Duration::from_secs(8) {
        for symbol in &symbols {
            // Fetch from multiple providers concurrently
            let mut data_points = Vec::new();

            for provider_name in &provider_names {
                if let Some(provider) = providers.get(*provider_name) {
                    match provider.fetch_price(symbol).await {
                        Ok(data_point) => {
                            data_points.push(data_point);
                        }
                        Err(e) => {
                            println!("     ⚠️  Error from {}: {}", provider_name, e);
                        }
                    }
                }
            }

            if !data_points.is_empty() {
                // Quick aggregation
                let avg_price: Decimal = data_points.iter()
                    .map(|p| p.value)
                    .sum::<Decimal>() / Decimal::from(data_points.len());

                let timestamp = Utc::now();
                println!("     {} - {}: ${:.2} ({} sources)",
                    timestamp.format("%H:%M:%S%.3f"),
                    symbol,
                    avg_price,
                    data_points.len());

                // Detect significant price movements
                if symbol == "ETH/USD" {
                    let price_f64 = avg_price.to_string().parse::<f64>().unwrap_or(0.0);
                    if (price_f64 - 2000.0).abs() > 20.0 { // >$20 deviation
                        println!("       🚨 Significant price movement detected!");
                    }
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        sleep(Duration::from_millis(500)).await; // 1 Hz per symbol
    }

    println!("   Real-time streaming completed");

    Ok(())
}

/// Demonstrate feed configuration
async fn demonstrate_feed_configuration() -> Result<()> {
    // Create different feed configurations for different use cases
    let configurations = vec![
        ("Conservative Whale Trading", create_conservative_config()),
        ("High-Frequency Trading", create_hft_config()),
        ("Cross-Chain Arbitrage", create_arbitrage_config()),
        ("Risk Management", create_risk_management_config()),
    ];

    for (config_name, config) in configurations {
        println!("\n   Configuration: {}", config_name);
        println!("     Strategy: {:?}", config.strategy);
        println!("     Min Sources: {}", config.min_sources);
        println!("     Max Data Age: {:?}", config.max_data_age);
        println!("     Max Deviation: {:.1}%",
            config.max_deviation.to_string().parse::<f64>().unwrap_or(0.0) * 100.0);
        println!("     Outlier Threshold: {}σ", config.outlier_threshold);

        // Show source weights if any
        if !config.source_weights.is_empty() {
            println!("     Source Weights:");
            for (source, weight) in &config.source_weights {
                println!("       {:?}: {:.1}%", source, weight * 100.0);
            }
        }

        // Show whale-specific settings
        println!("     Whale Volume Threshold: ${:.1}M",
            config.whale_config.whale_volume_threshold.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0);

        // Demonstrate with test data
        let mut aggregator = Aggregator::new(config);
        let test_data = create_sample_feed_data();

        match aggregator.aggregate_prices("ETH/USD", test_data).await {
            Ok(result) => {
                println!("     ✅ Test Result: ${:.2} (confidence: {:.1}%)",
                    result.price, result.confidence * 100.0);
            }
            Err(e) => {
                println!("     ❌ Test Failed: {}", e);
            }
        }
    }

    Ok(())
}

fn create_conservative_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::Consensus {
        min_sources: 4,
        threshold: Decimal::from_f64_retain(0.01).unwrap(), // 1% threshold
    };
    config.min_sources = 4;
    config.max_deviation = Decimal::from_f64_retain(0.02).unwrap(); // 2% max deviation
    config.outlier_threshold = 1.5; // Conservative outlier detection
    config.whale_config.whale_volume_threshold = Decimal::from(500_000); // $500K threshold
    config
}

fn create_hft_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::WeightedAverage;
    config.min_sources = 2; // Minimum for speed
    config.max_data_age = Duration::from_secs(10); // Very fresh data
    config.max_deviation = Decimal::from_f64_retain(0.001).unwrap(); // 0.1% max deviation
    config.outlier_threshold = 3.0; // More lenient for speed

    // Favor fast, reliable sources
    config.source_weights.insert(DataSource::Chainlink, 0.4);
    config.source_weights.insert(DataSource::Pyth, 0.4);
    config.source_weights.insert(DataSource::API3, 0.2);

    config
}

fn create_arbitrage_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::VolumeWeightedAverage;
    config.min_sources = 3;
    config.max_deviation = Decimal::from_f64_retain(0.05).unwrap(); // 5% for cross-chain differences
    config.whale_config.whale_volume_threshold = Decimal::from(2_000_000); // $2M threshold
    config
}

fn create_risk_management_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::Median; // Most conservative
    config.min_sources = 5; // Require many sources
    config.max_deviation = Decimal::from_f64_retain(0.01).unwrap(); // 1% max deviation
    config.outlier_threshold = 1.0; // Very strict outlier detection
    config.whale_config.whale_volume_threshold = Decimal::from(100_000); // $100K threshold
    config
}

fn create_sample_feed_data() -> Vec<DataPoint> {
    vec![
        DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        },
        DataPoint {
            source: DataSource::Pyth,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2005),
            timestamp: Utc::now(),
            confidence: 0.92,
            volume: Some(Decimal::from(150_000)),
            metadata: HashMap::new(),
        },
        DataPoint {
            source: DataSource::Band,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(1995),
            timestamp: Utc::now(),
            confidence: 0.90,
            volume: Some(Decimal::from(80_000)),
            metadata: HashMap::new(),
        },
    ]
}

/// Demonstrate custom validation rules
async fn demonstrate_custom_validation() -> Result<()> {
    let mut validator = create_custom_validator();

    let test_scenarios = vec![
        ("Normal DeFi Trade", create_normal_defi_data()),
        ("Suspicious MEV Activity", create_mev_suspicious_data()),
        ("Cross-Chain Price Discrepancy", create_cross_chain_data()),
        ("Flash Loan Attack Pattern", create_flash_loan_data()),
    ];

    for (scenario_name, data_point) in test_scenarios {
        println!("\n   Scenario: {}", scenario_name);

        let validation = validator.validate_data_point(&data_point).await?;

        println!("     Validation Result:");
        println!("       Valid: {}", validation.is_valid);
        println!("       Confidence: {:.1}%", validation.confidence_score * 100.0);
        println!("       Fraud Risk: {:?}", validation.fraud_risk);
        println!("       Checks Performed: {}", validation.checks_performed.len());

        if !validation.security_warnings.is_empty() {
            println!("       Security Warnings:");
            for warning in &validation.security_warnings {
                match warning {
                    SecurityWarning::MEVAttackSuspected { attack_type, confidence } => {
                        println!("         🚨 MEV Attack: {:?} ({:.1}% confidence)", attack_type, confidence * 100.0);
                    }
                    SecurityWarning::PriceManipulationSuspected { deviation_percentage, .. } => {
                        println!("         ⚠️  Price Manipulation: {:.1}% deviation", deviation_percentage);
                    }
                    SecurityWarning::VolumeAnomalyDetected { reported_volume, .. } => {
                        println!("         📊 Volume Anomaly: ${:.1}M",
                            reported_volume.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0);
                    }
                    _ => {
                        println!("         ⚠️  Other warning detected");
                    }
                }
            }
        }

        // Update source reputation
        validator.update_source_reputation(data_point.source, &validation);
    }

    Ok(())
}

fn create_custom_validator() -> SecurityValidator {
    let mut config = SecurityConfig::default();

    // Customize for DeFi/DEX environments
    config.max_price_deviation = 0.03; // 3% for DEX environments
    config.mev_detection_sensitivity = 0.4; // More sensitive MEV detection
    config.whale_security.mev_protection_enabled = true;
    config.whale_security.large_order_threshold = Decimal::from(1_000_000);

    SecurityValidator::new(config)
}

fn create_normal_defi_data() -> DataPoint {
    DataPoint {
        source: DataSource::Pyth,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(2005),
        timestamp: Utc::now(),
        confidence: 0.90,
        volume: Some(Decimal::from(150_000)),
        metadata: {
            let mut map = HashMap::new();
            map.insert("protocol".to_string(), "Uniswap V3".to_string());
            map.insert("pool_fee".to_string(), "0.3".to_string());
            map.insert("chain".to_string(), "ethereum".to_string());
            map
        },
    }
}

fn create_mev_suspicious_data() -> DataPoint {
    DataPoint {
        source: DataSource::Chainlink,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(2050), // Significant price jump
        timestamp: Utc::now(),
        confidence: 0.85,
        volume: Some(Decimal::from(5_000_000)), // Large volume
        metadata: {
            let mut map = HashMap::new();
            map.insert("transaction_type".to_string(), "frontrun".to_string());
            map.insert("gas_price".to_string(), "500".to_string()); // High gas
            map.insert("block_position".to_string(), "1".to_string()); // First in block
            map
        },
    }
}

fn create_cross_chain_data() -> DataPoint {
    DataPoint {
        source: DataSource::Band,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(1900), // Significant discount
        timestamp: Utc::now(),
        confidence: 0.88,
        volume: Some(Decimal::from(200_000)),
        metadata: {
            let mut map = HashMap::new();
            map.insert("chain".to_string(), "polygon".to_string());
            map.insert("bridge_liquidity".to_string(), "low".to_string());
            map.insert("arbitrage_opportunity".to_string(), "5.0".to_string()); // 5% arbitrage
            map
        },
    }
}

fn create_flash_loan_data() -> DataPoint {
    DataPoint {
        source: DataSource::API3,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(1800), // Temporary price depression
        timestamp: Utc::now(),
        confidence: 0.75,
        volume: Some(Decimal::from(25_000_000)), // Massive volume
        metadata: {
            let mut map = HashMap::new();
            map.insert("transaction_type".to_string(), "flash_loan".to_string());
            map.insert("loan_amount".to_string(), "50000000".to_string()); // $50M flash loan
            map.insert("attack_vector".to_string(), "oracle_manipulation".to_string());
            map
        },
    }
}

/// Demonstrate multi-asset feed management
async fn demonstrate_multi_asset_feeds() -> Result<()> {
    let assets = vec!["ETH/USD", "BTC/USD", "SOL/USD"];
    let mut feed_managers = HashMap::new();

    // Create specialized feed manager for each asset
    for asset in &assets {
        let config = match *asset {
            "ETH/USD" => create_eth_specific_config(),
            "BTC/USD" => create_btc_specific_config(),
            "SOL/USD" => create_sol_specific_config(),
            _ => AggregationConfig::default(),
        };

        feed_managers.insert(asset.to_string(), Aggregator::new(config));
    }

    println!("   Managing {} asset feeds simultaneously", assets.len());

    // Simulate simultaneous price updates
    for round in 1..=5 {
        println!("\n   Update Round {}:", round);

        for asset in &assets {
            let data = create_asset_specific_data(asset, round);

            if let Some(aggregator) = feed_managers.get_mut(*asset) {
                match aggregator.aggregate_prices(asset, data).await {
                    Ok(result) => {
                        println!("     {}: ${:.2} (conf: {:.1}%, sources: {})",
                            asset, result.price, result.confidence * 100.0, result.source_count);

                        if result.whale_impact.whale_activity_detected {
                            println!("       🐋 Whale activity: {:.1} bps impact",
                                result.whale_impact.price_impact_bps);
                        }
                    }
                    Err(e) => {
                        println!("     {}: ❌ Error - {}", asset, e);
                    }
                }
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    // Show final statistics
    println!("\n   Final Feed Statistics:");
    for asset in &assets {
        if let Some(aggregator) = feed_managers.get(asset) {
            let historical = aggregator.get_historical_prices(Some(5));
            println!("     {}: {} historical price points", asset, historical.len());

            if !historical.is_empty() {
                let latest = historical.last().unwrap();
                let oldest = historical.first().unwrap();
                let price_change = ((latest.price - oldest.price) / oldest.price)
                    .to_string().parse::<f64>().unwrap_or(0.0) * 100.0;

                println!("       Price change: {:.2}%", price_change);
                println!("       Avg confidence: {:.1}%",
                    historical.iter().map(|p| p.confidence).sum::<f64>() / historical.len() as f64 * 100.0);
            }
        }
    }

    Ok(())
}

fn create_eth_specific_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::VolumeWeightedAverage; // Good for ETH due to high liquidity
    config.min_sources = 3;
    config.whale_config.whale_volume_threshold = Decimal::from(2_000_000); // $2M for ETH
    config
}

fn create_btc_specific_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::Median; // Conservative for BTC
    config.min_sources = 4; // Require more sources for BTC
    config.whale_config.whale_volume_threshold = Decimal::from(5_000_000); // $5M for BTC
    config
}

fn create_sol_specific_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::WeightedAverage;
    config.min_sources = 2; // Fewer sources available for SOL
    config.max_deviation = Decimal::from_f64_retain(0.08).unwrap(); // 8% for more volatile asset
    config.whale_config.whale_volume_threshold = Decimal::from(500_000); // $500K for SOL
    config
}

fn create_asset_specific_data(asset: &str, round: usize) -> Vec<DataPoint> {
    let (base_price, volatility) = match asset {
        "ETH/USD" => (2000.0, 0.02),
        "BTC/USD" => (50000.0, 0.03),
        "SOL/USD" => (100.0, 0.05),
        _ => (100.0, 0.02),
    };

    let price_change = (round as f64 - 3.0) * volatility; // Simulate price movement
    let current_price = base_price * (1.0 + price_change);

    vec![
        DataPoint {
            source: DataSource::Chainlink,
            symbol: asset.to_string(),
            value: Decimal::from_f64_retain(current_price).unwrap(),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000 + round * 50_000)),
            metadata: HashMap::new(),
        },
        DataPoint {
            source: DataSource::Pyth,
            symbol: asset.to_string(),
            value: Decimal::from_f64_retain(current_price * 1.001).unwrap(),
            timestamp: Utc::now(),
            confidence: 0.90,
            volume: Some(Decimal::from(150_000 + round * 30_000)),
            metadata: HashMap::new(),
        },
    ]
}

use std::time::Instant;