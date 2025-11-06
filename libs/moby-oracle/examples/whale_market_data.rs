//! # Whale Market Data Analysis Example
//!
//! This example demonstrates advanced whale trading analysis features including:
//! - Large order impact analysis
//! - Market manipulation detection
//! - Liquidity depth assessment
//! - Real-time whale activity monitoring
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example whale_market_data
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
    env_logger::init();

    println!("🐋 Moby Oracle - Whale Market Data Analysis");
    println!("===========================================");

    // Step 1: Analyze different whale order scenarios
    println!("\n📊 Analyzing Whale Order Scenarios:");
    await analyze_whale_order_sizes().await?;

    // Step 2: Demonstrate market manipulation detection
    println!("\n🕵️  Market Manipulation Detection:");
    await detect_market_manipulation().await?;

    // Step 3: Liquidity depth analysis
    println!("\n💧 Liquidity Depth Analysis:");
    await analyze_liquidity_depth().await?;

    // Step 4: MEV attack detection
    println!("\n⚡ MEV Attack Detection:");
    await demonstrate_mev_detection().await?;

    // Step 5: Real-time whale monitoring
    println!("\n📡 Real-time Whale Activity Monitoring:");
    await monitor_whale_activity().await?;

    // Step 6: Historical whale pattern analysis
    println!("\n📈 Historical Whale Pattern Analysis:");
    await analyze_whale_patterns().await?;

    println!("\n✅ Whale market data analysis completed!");
    Ok(())
}

/// Analyze different whale order sizes and their market impact
async fn analyze_whale_order_sizes() -> Result<()> {
    let order_sizes = vec![
        ("Small Whale", 1_000_000.0),    // $1M
        ("Medium Whale", 5_000_000.0),   // $5M
        ("Large Whale", 25_000_000.0),   // $25M
        ("Mega Whale", 100_000_000.0),   // $100M
    ];

    let config = AggregationConfig::default();

    for (whale_type, volume) in order_sizes {
        println!("\n   {} Order (${:.0}M):", whale_type, volume / 1_000_000.0);

        let whale_data = create_whale_order_data("ETH/USD", 2000.0, volume);
        let mut aggregator = Aggregator::new(config.clone());

        let result = aggregator.aggregate_prices("ETH/USD", whale_data).await?;

        println!("     Price: ${:.2}", result.price);
        println!("     Whale Detected: {}", result.whale_impact.whale_activity_detected);
        println!("     Price Impact: {:.1} bps", result.whale_impact.price_impact_bps);
        println!("     Liquidity Depth: ${:.1}M",
            result.whale_impact.liquidity_depth.to_string().parse::<f64>().unwrap() / 1_000_000.0);
        println!("     Volatility Score: {:.3}", result.whale_impact.volatility_score);

        if let Some(max_order) = result.whale_impact.max_order_size {
            println!("     Max Recommended Order: ${:.1}M",
                max_order.to_string().parse::<f64>().unwrap() / 1_000_000.0);
        }

        // Risk assessment
        let risk_level = if result.whale_impact.price_impact_bps > Decimal::from(200) {
            "🔴 HIGH RISK"
        } else if result.whale_impact.price_impact_bps > Decimal::from(50) {
            "🟡 MEDIUM RISK"
        } else {
            "🟢 LOW RISK"
        };
        println!("     Risk Level: {}", risk_level);
    }

    Ok(())
}

/// Create whale order data with specified volume
fn create_whale_order_data(symbol: &str, base_price: f64, total_volume: f64) -> Vec<DataPoint> {
    let sources = vec![
        (DataSource::Chainlink, 0.4),  // 40% of volume
        (DataSource::Pyth, 0.3),       // 30% of volume
        (DataSource::Band, 0.2),       // 20% of volume
        (DataSource::API3, 0.1),       // 10% of volume
    ];

    sources.into_iter().map(|(source, weight)| {
        let volume = total_volume * weight;
        let price_impact = (volume / 10_000_000.0) * 0.01; // 1% impact per $10M
        let adjusted_price = base_price * (1.0 + price_impact);

        DataPoint {
            source,
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(adjusted_price).unwrap(),
            timestamp: Utc::now() - chrono::Duration::seconds(rand::random::<u64>() % 30),
            confidence: 0.90 + (rand::random::<u64>() % 10) as f64 / 100.0,
            volume: Some(Decimal::from_f64_retain(volume).unwrap()),
            metadata: {
                let mut map = HashMap::new();
                map.insert("whale_order".to_string(), "true".to_string());
                map.insert("order_type".to_string(), "market".to_string());
                map
            },
        }
    }).collect()
}

/// Demonstrate market manipulation detection
async fn detect_market_manipulation() -> Result<()> {
    let scenarios = vec![
        ("Legitimate Trading", create_legitimate_data()),
        ("Price Manipulation", create_manipulated_data()),
        ("Wash Trading", create_wash_trading_data()),
        ("Pump and Dump", create_pump_dump_data()),
    ];

    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    for (scenario_name, data_points) in scenarios {
        println!("\n   Scenario: {}", scenario_name);

        let mut total_confidence = 0.0;
        let mut total_warnings = 0;
        let mut high_risk_count = 0;

        for data_point in data_points {
            let validation = validator.validate_data_point(&data_point).await?;

            total_confidence += validation.confidence_score;
            total_warnings += validation.security_warnings.len();

            if validation.fraud_risk >= FraudRiskLevel::High {
                high_risk_count += 1;
            }

            // Update source reputation
            validator.update_source_reputation(data_point.source, &validation);
        }

        let avg_confidence = total_confidence / 4.0; // 4 data points per scenario
        println!("     Average Confidence: {:.1}%", avg_confidence * 100.0);
        println!("     Total Warnings: {}", total_warnings);
        println!("     High Risk Alerts: {}", high_risk_count);

        let manipulation_detected = avg_confidence < 0.6 || total_warnings > 2;
        println!("     Manipulation Detected: {}", if manipulation_detected { "🚨 YES" } else { "✅ NO" });
    }

    Ok(())
}

fn create_legitimate_data() -> Vec<DataPoint> {
    let base_price = 2000.0;
    vec![
        create_normal_data_point(DataSource::Chainlink, base_price, 100_000.0),
        create_normal_data_point(DataSource::Pyth, base_price * 1.001, 120_000.0),
        create_normal_data_point(DataSource::Band, base_price * 0.999, 90_000.0),
        create_normal_data_point(DataSource::API3, base_price * 1.0005, 110_000.0),
    ]
}

fn create_manipulated_data() -> Vec<DataPoint> {
    let base_price = 2000.0;
    vec![
        create_normal_data_point(DataSource::Chainlink, base_price, 100_000.0),
        create_suspicious_data_point(DataSource::Pyth, base_price * 1.15, 5_000_000.0), // Massive price spike
        create_normal_data_point(DataSource::Band, base_price * 1.001, 90_000.0),
        create_normal_data_point(DataSource::API3, base_price * 0.999, 110_000.0),
    ]
}

fn create_wash_trading_data() -> Vec<DataPoint> {
    let base_price = 2000.0;
    // Artificially inflated volumes with minimal price movement
    vec![
        create_suspicious_data_point(DataSource::Chainlink, base_price, 10_000_000.0),
        create_suspicious_data_point(DataSource::Pyth, base_price * 1.0001, 15_000_000.0),
        create_suspicious_data_point(DataSource::Band, base_price * 0.9999, 12_000_000.0),
        create_suspicious_data_point(DataSource::API3, base_price * 1.0002, 8_000_000.0),
    ]
}

fn create_pump_dump_data() -> Vec<DataPoint> {
    let base_price = 2000.0;
    vec![
        create_suspicious_data_point(DataSource::Chainlink, base_price * 1.25, 3_000_000.0), // Pump
        create_suspicious_data_point(DataSource::Pyth, base_price * 1.30, 2_500_000.0),     // Peak
        create_suspicious_data_point(DataSource::Band, base_price * 0.85, 4_000_000.0),     // Dump
        create_normal_data_point(DataSource::API3, base_price * 0.95, 1_000_000.0),         // Recovery
    ]
}

fn create_normal_data_point(source: DataSource, price: f64, volume: f64) -> DataPoint {
    DataPoint {
        source,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from_f64_retain(price).unwrap(),
        timestamp: Utc::now(),
        confidence: 0.95,
        volume: Some(Decimal::from_f64_retain(volume).unwrap()),
        metadata: HashMap::new(),
    }
}

fn create_suspicious_data_point(source: DataSource, price: f64, volume: f64) -> DataPoint {
    let mut data_point = create_normal_data_point(source, price, volume);
    data_point.timestamp = Utc::now() - chrono::Duration::seconds(300); // Make it stale
    data_point.confidence = 0.70; // Lower confidence
    data_point.metadata.insert("suspicious".to_string(), "true".to_string());
    data_point
}

/// Analyze liquidity depth across different market conditions
async fn analyze_liquidity_depth() -> Result<()> {
    let market_conditions = vec![
        ("Bull Market", create_bull_market_data()),
        ("Bear Market", create_bear_market_data()),
        ("High Volatility", create_volatile_market_data()),
        ("Low Liquidity", create_low_liquidity_data()),
    ];

    let config = AggregationConfig::default();

    for (condition_name, data) in market_conditions {
        println!("\n   Market Condition: {}", condition_name);

        let mut aggregator = Aggregator::new(config.clone());
        let result = aggregator.aggregate_prices("ETH/USD", data).await?;

        let liquidity_depth = result.whale_impact.liquidity_depth.to_string().parse::<f64>().unwrap();
        let volatility = result.whale_impact.volatility_score;

        println!("     Liquidity Depth: ${:.1}M", liquidity_depth / 1_000_000.0);
        println!("     Volatility Score: {:.3}", volatility);
        println!("     Price Impact per $1M: {:.1} bps",
            (result.whale_impact.price_impact_bps.to_string().parse::<f64>().unwrap() / 10.0));

        let liquidity_rating = if liquidity_depth > 50_000_000.0 {
            "🟢 HIGH"
        } else if liquidity_depth > 20_000_000.0 {
            "🟡 MEDIUM"
        } else {
            "🔴 LOW"
        };

        println!("     Liquidity Rating: {}", liquidity_rating);

        // Trading recommendations
        if volatility > 0.3 {
            println!("     ⚠️  Recommendation: Use smaller order sizes");
        } else if liquidity_depth < 10_000_000.0 {
            println!("     ⚠️  Recommendation: Split large orders across time");
        } else {
            println!("     ✅ Recommendation: Normal trading conditions");
        }
    }

    Ok(())
}

fn create_bull_market_data() -> Vec<DataPoint> {
    // High volume, moderate price increases
    vec![
        create_normal_data_point(DataSource::Chainlink, 2100.0, 500_000.0),
        create_normal_data_point(DataSource::Pyth, 2110.0, 600_000.0),
        create_normal_data_point(DataSource::Band, 2095.0, 450_000.0),
        create_normal_data_point(DataSource::API3, 2105.0, 520_000.0),
    ]
}

fn create_bear_market_data() -> Vec<DataPoint> {
    // Lower volume, price declines
    vec![
        create_normal_data_point(DataSource::Chainlink, 1850.0, 200_000.0),
        create_normal_data_point(DataSource::Pyth, 1840.0, 180_000.0),
        create_normal_data_point(DataSource::Band, 1860.0, 220_000.0),
        create_normal_data_point(DataSource::API3, 1845.0, 190_000.0),
    ]
}

fn create_volatile_market_data() -> Vec<DataPoint> {
    // High price variance, mixed volumes
    vec![
        create_normal_data_point(DataSource::Chainlink, 2200.0, 800_000.0),
        create_normal_data_point(DataSource::Pyth, 1950.0, 750_000.0),
        create_normal_data_point(DataSource::Band, 2150.0, 900_000.0),
        create_normal_data_point(DataSource::API3, 1980.0, 700_000.0),
    ]
}

fn create_low_liquidity_data() -> Vec<DataPoint> {
    // Very low volumes, wider spreads
    vec![
        create_normal_data_point(DataSource::Chainlink, 2000.0, 25_000.0),
        create_normal_data_point(DataSource::Pyth, 2020.0, 30_000.0),
        create_normal_data_point(DataSource::Band, 1980.0, 20_000.0),
        create_normal_data_point(DataSource::API3, 2010.0, 28_000.0),
    ]
}

/// Demonstrate MEV attack detection capabilities
async fn demonstrate_mev_detection() -> Result<()> {
    let attack_scenarios = vec![
        ("Normal Trading", create_normal_trading_data()),
        ("Front-running Attack", create_frontrunning_data()),
        ("Sandwich Attack", create_sandwich_attack_data()),
        ("Flash Loan Attack", create_flashloan_data()),
    ];

    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    for (scenario, data_points) in attack_scenarios {
        println!("\n   Scenario: {}", scenario);

        let mut mev_detections = 0;
        let mut total_risk_score = 0.0;

        for data_point in data_points {
            let validation = validator.validate_data_point(&data_point).await?;

            // Check for MEV-related warnings
            for warning in &validation.security_warnings {
                match warning {
                    SecurityWarning::MEVAttackSuspected { attack_type, confidence } => {
                        mev_detections += 1;
                        total_risk_score += confidence;
                        println!("     🚨 MEV Attack Detected: {:?} ({:.1}% confidence)",
                            attack_type, confidence * 100.0);
                    }
                    _ => {}
                }
            }
        }

        if mev_detections == 0 {
            println!("     ✅ No MEV attacks detected");
        } else {
            let avg_confidence = total_risk_score / mev_detections as f64;
            println!("     MEV Detections: {}", mev_detections);
            println!("     Average Confidence: {:.1}%", avg_confidence * 100.0);
        }
    }

    Ok(())
}

fn create_normal_trading_data() -> Vec<DataPoint> {
    vec![
        create_normal_data_point(DataSource::Chainlink, 2000.0, 100_000.0),
        create_normal_data_point(DataSource::Pyth, 2001.0, 120_000.0),
    ]
}

fn create_frontrunning_data() -> Vec<DataPoint> {
    // Suspicious timing and large volume spikes
    let mut data = create_normal_data_point(DataSource::Chainlink, 2000.0, 5_000_000.0);
    data.timestamp = Utc::now(); // Exactly on block boundary (simplified)
    data.metadata.insert("block_position".to_string(), "first".to_string());
    vec![data]
}

fn create_sandwich_attack_data() -> Vec<DataPoint> {
    // Pre-trade, victim trade, post-trade pattern
    vec![
        {
            let mut data = create_normal_data_point(DataSource::Chainlink, 1995.0, 2_000_000.0);
            data.metadata.insert("sandwich_phase".to_string(), "pre".to_string());
            data
        },
        {
            let mut data = create_normal_data_point(DataSource::Pyth, 2005.0, 3_000_000.0);
            data.metadata.insert("sandwich_phase".to_string(), "post".to_string());
            data
        },
    ]
}

fn create_flashloan_data() -> Vec<DataPoint> {
    // Extremely large volume with immediate reversal
    vec![
        create_suspicious_data_point(DataSource::Chainlink, 2050.0, 50_000_000.0),
        create_suspicious_data_point(DataSource::Pyth, 2000.0, 50_000_000.0),
    ]
}

/// Monitor whale activity in real-time
async fn monitor_whale_activity() -> Result<()> {
    println!("   Starting 15-second whale monitoring simulation...");

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    let start_time = std::time::Instant::now();
    let mut whale_events = 0;

    while start_time.elapsed() < Duration::from_secs(15) {
        // Randomly generate whale or normal activity
        let is_whale_event = rand::random::<f64>() < 0.3; // 30% chance of whale activity

        let data = if is_whale_event {
            whale_events += 1;
            let volume = 2_000_000.0 + rand::random::<f64>() * 8_000_000.0; // $2-10M
            create_whale_order_data("ETH/USD", 2000.0, volume)
        } else {
            create_normal_market_data()
        };

        let result = aggregator.aggregate_prices("ETH/USD", data).await?;

        if result.whale_impact.whale_activity_detected {
            let volume_total = result.source_count as f64 * 1_000_000.0; // Rough estimate
            println!("     🐋 WHALE DETECTED: ${:.1}M volume, {:.1} bps impact",
                volume_total / 1_000_000.0,
                result.whale_impact.price_impact_bps);

            // Alert based on impact level
            if result.whale_impact.price_impact_bps > Decimal::from(100) {
                println!("       🔴 HIGH IMPACT ALERT");
            } else if result.whale_impact.price_impact_bps > Decimal::from(25) {
                println!("       🟡 Medium impact");
            }
        }

        sleep(Duration::from_millis(500)).await; // 2 Hz monitoring
    }

    println!("   Monitoring completed. Whale events detected: {}", whale_events);

    Ok(())
}

fn create_normal_market_data() -> Vec<DataPoint> {
    vec![
        create_normal_data_point(DataSource::Chainlink, 2000.0 + rand::random::<f64>() * 10.0 - 5.0,
                                50_000.0 + rand::random::<f64>() * 100_000.0),
        create_normal_data_point(DataSource::Pyth, 2000.0 + rand::random::<f64>() * 10.0 - 5.0,
                                75_000.0 + rand::random::<f64>() * 150_000.0),
    ]
}

/// Analyze historical whale trading patterns
async fn analyze_whale_patterns() -> Result<()> {
    println!("   Simulating historical whale pattern analysis...");

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    // Simulate historical data with whale patterns
    let mut whale_events = Vec::new();
    let mut normal_events = Vec::new();

    for i in 0..100 {
        let timestamp = Utc::now() - chrono::Duration::hours(i);

        if i % 10 == 0 { // Every 10th event is a whale event
            let volume = 1_000_000.0 + (i as f64 * 50_000.0);
            let mut data = create_whale_order_data("ETH/USD", 2000.0, volume);

            // Adjust timestamp
            for point in &mut data {
                point.timestamp = timestamp;
            }

            let result = aggregator.aggregate_prices("ETH/USD", data).await?;
            whale_events.push((timestamp, result.whale_impact.price_impact_bps, volume));
        } else {
            let mut data = create_normal_market_data();
            for point in &mut data {
                point.timestamp = timestamp;
            }
            let result = aggregator.aggregate_prices("ETH/USD", data).await?;
            normal_events.push(timestamp);
        }
    }

    // Analyze patterns
    let total_whale_impact: f64 = whale_events.iter()
        .map(|(_, impact, _)| impact.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();
    let avg_whale_impact = total_whale_impact / whale_events.len() as f64;

    let total_whale_volume: f64 = whale_events.iter()
        .map(|(_, _, volume)| volume)
        .sum();
    let avg_whale_volume = total_whale_volume / whale_events.len() as f64;

    println!("   Analysis Results:");
    println!("     Total Whale Events: {}", whale_events.len());
    println!("     Average Whale Volume: ${:.1}M", avg_whale_volume / 1_000_000.0);
    println!("     Average Price Impact: {:.1} bps", avg_whale_impact);

    // Pattern detection
    let high_impact_events = whale_events.iter()
        .filter(|(_, impact, _)| impact.to_string().parse::<f64>().unwrap_or(0.0) > 50.0)
        .count();

    println!("     High Impact Events (>50 bps): {}", high_impact_events);

    if high_impact_events > whale_events.len() / 3 {
        println!("     🚨 Pattern Alert: Frequent high-impact whale activity detected");
    } else {
        println!("     ✅ Pattern Normal: Whale activity within expected parameters");
    }

    // Show recent whale activity
    println!("   Recent Whale Events:");
    for (timestamp, impact, volume) in whale_events.iter().take(5) {
        println!("     {} - ${:.1}M volume, {:.1} bps impact",
            timestamp.format("%m/%d %H:%M"),
            volume / 1_000_000.0,
            impact.to_string().parse::<f64>().unwrap_or(0.0));
    }

    Ok(())
}