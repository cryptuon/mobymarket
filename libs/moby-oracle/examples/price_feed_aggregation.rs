//! # Price Feed Aggregation Example
//!
//! This example demonstrates how to set up and use the moby-oracle system for
//! aggregating price feeds from multiple oracle sources with different strategies.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example price_feed_aggregation
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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("🐋 Moby Oracle - Price Feed Aggregation Example");
    println!("================================================");

    // Step 1: Create mock data sources for demonstration
    println!("\n📊 Creating mock price data from multiple sources...");

    let eth_usd_data = create_mock_price_data("ETH/USD", 2000.0);
    let btc_usd_data = create_mock_price_data("BTC/USD", 50000.0);

    // Step 2: Set up aggregation with different strategies
    println!("\n🔄 Testing different aggregation strategies:");

    // Test median aggregation
    println!("\n1. Median Aggregation:");
    await test_aggregation_strategy(
        "ETH/USD",
        eth_usd_data.clone(),
        AggregationStrategy::Median
    ).await?;

    // Test weighted average aggregation
    println!("\n2. Weighted Average Aggregation:");
    await test_weighted_aggregation("ETH/USD", eth_usd_data.clone()).await?;

    // Test time-weighted average
    println!("\n3. Time-Weighted Average Aggregation:");
    await test_twap_aggregation("ETH/USD", eth_usd_data.clone()).await?;

    // Test consensus aggregation
    println!("\n4. Consensus Aggregation:");
    await test_consensus_aggregation("ETH/USD", eth_usd_data.clone()).await?;

    // Step 3: Demonstrate whale trading scenario
    println!("\n🐋 Whale Trading Scenario:");
    await demonstrate_whale_trading("BTC/USD", btc_usd_data).await?;

    // Step 4: Show security validation
    println!("\n🔒 Security Validation:");
    await demonstrate_security_validation().await?;

    // Step 5: Real-time feed simulation
    println!("\n⚡ Real-time Feed Simulation:");
    await simulate_realtime_feeds().await?;

    println!("\n✅ Price feed aggregation example completed successfully!");
    Ok(())
}

/// Create mock price data from multiple sources
fn create_mock_price_data(symbol: &str, base_price: f64) -> Vec<DataPoint> {
    vec![
        DataPoint {
            source: DataSource::Chainlink,
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(base_price).unwrap(),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(150_000)),
            metadata: {
                let mut map = HashMap::new();
                map.insert("feed_id".to_string(), "chainlink_eth_usd".to_string());
                map.insert("network".to_string(), "ethereum".to_string());
                map
            },
        },
        DataPoint {
            source: DataSource::Pyth,
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(base_price * 1.002).unwrap(), // +0.2%
            timestamp: Utc::now() - chrono::Duration::seconds(5),
            confidence: 0.92,
            volume: Some(Decimal::from(200_000)),
            metadata: {
                let mut map = HashMap::new();
                map.insert("feed_id".to_string(), "pyth_eth_usd".to_string());
                map.insert("network".to_string(), "solana".to_string());
                map
            },
        },
        DataPoint {
            source: DataSource::Band,
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(base_price * 0.998).unwrap(), // -0.2%
            timestamp: Utc::now() - chrono::Duration::seconds(3),
            confidence: 0.90,
            volume: Some(Decimal::from(120_000)),
            metadata: {
                let mut map = HashMap::new();
                map.insert("feed_id".to_string(), "band_eth_usd".to_string());
                map.insert("network".to_string(), "cosmos".to_string());
                map
            },
        },
        DataPoint {
            source: DataSource::API3,
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(base_price * 1.001).unwrap(), // +0.1%
            timestamp: Utc::now() - chrono::Duration::seconds(8),
            confidence: 0.88,
            volume: Some(Decimal::from(80_000)),
            metadata: {
                let mut map = HashMap::new();
                map.insert("feed_id".to_string(), "api3_eth_usd".to_string());
                map.insert("network".to_string(), "ethereum".to_string());
                map
            },
        },
    ]
}

/// Test a specific aggregation strategy
async fn test_aggregation_strategy(
    symbol: &str,
    data: Vec<DataPoint>,
    strategy: AggregationStrategy,
) -> Result<()> {
    let mut config = AggregationConfig::default();
    config.strategy = strategy.clone();
    config.min_sources = 3;

    let mut aggregator = Aggregator::new(config);

    let result = aggregator.aggregate_prices(symbol, data).await?;

    println!("   Strategy: {:?}", strategy);
    println!("   Price: ${:.2}", result.price);
    println!("   Confidence: {:.2}%", result.confidence * 100.0);
    println!("   Sources: {}", result.source_count);
    println!("   Variance: ${:.2}", result.quality_metrics.price_variance);

    if result.whale_impact.whale_activity_detected {
        println!("   🐋 Whale activity detected!");
        println!("   Price impact: {:.1} bps", result.whale_impact.price_impact_bps);
    }

    Ok(())
}

/// Test weighted average aggregation with custom weights
async fn test_weighted_aggregation(symbol: &str, data: Vec<DataPoint>) -> Result<()> {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::WeightedAverage;

    // Set custom source weights
    config.source_weights.insert(DataSource::Chainlink, 0.4);   // 40% weight
    config.source_weights.insert(DataSource::Pyth, 0.3);        // 30% weight
    config.source_weights.insert(DataSource::Band, 0.2);        // 20% weight
    config.source_weights.insert(DataSource::API3, 0.1);        // 10% weight

    let mut aggregator = Aggregator::new(config);
    let result = aggregator.aggregate_prices(symbol, data).await?;

    println!("   Strategy: Weighted Average");
    println!("   Weights: Chainlink(40%), Pyth(30%), Band(20%), API3(10%)");
    println!("   Price: ${:.2}", result.price);
    println!("   Confidence: {:.2}%", result.confidence * 100.0);

    Ok(())
}

/// Test time-weighted average price (TWAP)
async fn test_twap_aggregation(symbol: &str, mut data: Vec<DataPoint>) -> Result<()> {
    // Adjust timestamps to create time spacing
    for (i, point) in data.iter_mut().enumerate() {
        point.timestamp = Utc::now() - chrono::Duration::seconds((i as i64 + 1) * 30);
    }

    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::TimeWeightedAverage {
        window: Duration::from_secs(300), // 5-minute window
    };

    let mut aggregator = Aggregator::new(config);
    let result = aggregator.aggregate_prices(symbol, data).await?;

    println!("   Strategy: Time-Weighted Average (5min window)");
    println!("   TWAP Price: ${:.2}", result.price);
    println!("   Confidence: {:.2}%", result.confidence * 100.0);

    Ok(())
}

/// Test consensus aggregation
async fn test_consensus_aggregation(symbol: &str, data: Vec<DataPoint>) -> Result<()> {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::Consensus {
        min_sources: 3,
        threshold: Decimal::from_f64_retain(0.005).unwrap(), // 0.5% threshold
    };

    let mut aggregator = Aggregator::new(config);
    let result = aggregator.aggregate_prices(symbol, data).await?;

    println!("   Strategy: Consensus (min 3 sources, 0.5% threshold)");
    println!("   Consensus Price: ${:.2}", result.price);
    println!("   Participating Sources: {}", result.source_count);
    println!("   Confidence: {:.2}%", result.confidence * 100.0);

    Ok(())
}

/// Demonstrate whale trading scenario with large volumes
async fn demonstrate_whale_trading(symbol: &str, mut data: Vec<DataPoint>) -> Result<()> {
    // Simulate whale-sized volumes
    for point in &mut data {
        point.volume = Some(Decimal::from(5_000_000)); // $5M volume each
    }

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    let result = aggregator.aggregate_prices(symbol, data).await?;

    println!("   Symbol: {}", symbol);
    println!("   Total Volume: ${:.1}M",
        result.source_count as f64 * 5.0); // $5M per source

    if result.whale_impact.whale_activity_detected {
        println!("   🐋 WHALE ACTIVITY DETECTED!");
        println!("   Price Impact: {:.1} basis points", result.whale_impact.price_impact_bps);
        println!("   Liquidity Depth: ${:.1}M",
            result.whale_impact.liquidity_depth.to_string().parse::<f64>().unwrap() / 1_000_000.0);
        println!("   Volatility Score: {:.2}", result.whale_impact.volatility_score);

        if let Some(max_order) = result.whale_impact.max_order_size {
            println!("   Recommended Max Order: ${:.1}M",
                max_order.to_string().parse::<f64>().unwrap() / 1_000_000.0);
        }
    }

    Ok(())
}

/// Demonstrate security validation features
async fn demonstrate_security_validation() -> Result<()> {
    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    // Test with good data
    println!("\n   Testing with legitimate data:");
    let good_data = DataPoint {
        source: DataSource::Chainlink,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(2000),
        timestamp: Utc::now(),
        confidence: 0.95,
        volume: Some(Decimal::from(100_000)),
        metadata: HashMap::new(),
    };

    let validation = validator.validate_data_point(&good_data).await?;
    println!("     ✅ Valid: {}", validation.is_valid);
    println!("     Confidence: {:.2}%", validation.confidence_score * 100.0);
    println!("     Fraud Risk: {:?}", validation.fraud_risk);
    println!("     Checks Passed: {}/{}",
        validation.checks_performed.iter().filter(|c| c.passed).count(),
        validation.checks_performed.len());

    // Test with suspicious data
    println!("\n   Testing with suspicious data:");
    let suspicious_data = DataPoint {
        source: DataSource::Chainlink,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(10000), // Unrealistic price
        timestamp: Utc::now() - chrono::Duration::seconds(600), // Stale data
        confidence: 0.95,
        volume: Some(Decimal::from(10_000_000)), // Very large volume
        metadata: HashMap::new(),
    };

    let suspicious_validation = validator.validate_data_point(&suspicious_data).await?;
    println!("     ⚠️  Valid: {}", suspicious_validation.is_valid);
    println!("     Confidence: {:.2}%", suspicious_validation.confidence_score * 100.0);
    println!("     Fraud Risk: {:?}", suspicious_validation.fraud_risk);
    println!("     Security Warnings: {}", suspicious_validation.security_warnings.len());

    for warning in &suspicious_validation.security_warnings {
        match warning {
            SecurityWarning::StaleDataDetected { age_seconds, .. } => {
                println!("       - Stale data: {} seconds old", age_seconds);
            }
            SecurityWarning::PriceManipulationSuspected { deviation_percentage, .. } => {
                println!("       - Price manipulation: {:.1}% deviation", deviation_percentage);
            }
            SecurityWarning::MEVAttackSuspected { attack_type, confidence } => {
                println!("       - MEV attack suspected: {:?} ({:.1}% confidence)",
                    attack_type, confidence * 100.0);
            }
            _ => {
                println!("       - Other security warning detected");
            }
        }
    }

    Ok(())
}

/// Simulate real-time price feeds
async fn simulate_realtime_feeds() -> Result<()> {
    println!("   Starting 10-second real-time simulation...");

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    let start_time = std::time::Instant::now();
    let mut update_count = 0;

    while start_time.elapsed() < Duration::from_secs(10) {
        // Generate slightly varying price data
        let price_variation = (update_count as f64 % 20.0 - 10.0) / 100.0; // ±10 cents
        let base_price = 2000.0 + price_variation;

        let data = vec![
            DataPoint {
                source: DataSource::Chainlink,
                symbol: "ETH/USD".to_string(),
                value: Decimal::from_f64_retain(base_price).unwrap(),
                timestamp: Utc::now(),
                confidence: 0.95,
                volume: Some(Decimal::from(75_000 + (update_count % 50_000))),
                metadata: HashMap::new(),
            },
            DataPoint {
                source: DataSource::Pyth,
                symbol: "ETH/USD".to_string(),
                value: Decimal::from_f64_retain(base_price * 1.001).unwrap(),
                timestamp: Utc::now(),
                confidence: 0.92,
                volume: Some(Decimal::from(100_000 + (update_count % 30_000))),
                metadata: HashMap::new(),
            },
        ];

        let result = aggregator.aggregate_prices("ETH/USD", data).await?;

        if update_count % 10 == 0 { // Print every 10th update
            println!("     Update {}: ${:.2} (confidence: {:.1}%)",
                update_count, result.price, result.confidence * 100.0);
        }

        update_count += 1;
        sleep(Duration::from_millis(100)).await; // 10 Hz updates
    }

    println!("   Completed {} real-time updates", update_count);

    // Show historical data
    let historical = aggregator.get_historical_prices(Some(5));
    println!("   Last 5 prices:");
    for (i, price) in historical.iter().rev().enumerate() {
        println!("     {}: ${:.2} @ {}",
            i + 1, price.price, price.timestamp.format("%H:%M:%S"));
    }

    Ok(())
}

/// Helper function to demonstrate error handling
#[allow(dead_code)]
async fn demonstrate_error_handling() -> Result<()> {
    println!("\n🚨 Error Handling Demonstration:");

    let config = AggregationConfig {
        min_sources: 5, // Require 5 sources
        ..Default::default()
    };

    let aggregator = Aggregator::new(config);

    // Try to aggregate with insufficient sources
    let insufficient_data = vec![
        DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        }
    ];

    match aggregator.aggregate_prices("ETH/USD", insufficient_data).await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => {
            println!("   ✅ Correctly handled error: {}", e);
            match e {
                OracleError::InsufficientDataSources { required, available } => {
                    println!("     Required: {}, Available: {}", required, available);
                }
                _ => {}
            }
        }
    }

    Ok(())
}