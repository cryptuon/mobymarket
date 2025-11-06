//! Integration tests for the moby-oracle library
//!
//! These tests verify the complete oracle system functionality including:
//! - End-to-end price feed aggregation
//! - Multi-source data integration
//! - Security validation workflows
//! - Whale trading scenarios
//! - Error handling and recovery

use moby_oracle::*;
use moby_oracle::sources::*;
use moby_oracle::aggregation::*;
use moby_oracle::security::*;
use tokio_test;
use std::time::Duration;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Test helper to create mock data points
fn create_mock_data_point(source: DataSource, symbol: &str, price: f64, volume: Option<f64>) -> DataPoint {
    DataPoint {
        source,
        symbol: symbol.to_string(),
        value: Decimal::from_f64_retain(price).unwrap(),
        timestamp: Utc::now(),
        confidence: 0.95,
        volume: volume.map(|v| Decimal::from_f64_retain(v).unwrap()),
        metadata: HashMap::new(),
    }
}

/// Test helper to create aggregation config
fn create_test_aggregation_config() -> AggregationConfig {
    let mut config = AggregationConfig::default();
    config.min_sources = 2;
    config.max_deviation = Decimal::from_f64_retain(0.1).unwrap(); // 10%
    config
}

#[tokio::test]
async fn test_end_to_end_price_aggregation() {
    // Create test data from multiple sources
    let data_points = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0)),
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2010.0, Some(150_000.0)),
        create_mock_data_point(DataSource::Band, "ETH/USD", 1990.0, Some(80_000.0)),
    ];

    // Set up aggregator
    let config = create_test_aggregation_config();
    let mut aggregator = Aggregator::new(config);

    // Perform aggregation
    let result = aggregator.aggregate_prices("ETH/USD", data_points).await;
    assert!(result.is_ok(), "Aggregation should succeed");

    let aggregated = result.unwrap();
    assert_eq!(aggregated.symbol, "ETH/USD");
    assert_eq!(aggregated.source_count, 3);
    assert!(aggregated.confidence > 0.5);
    assert_eq!(aggregated.price, Decimal::from(2000)); // Median of [1990, 2000, 2010]
}

#[tokio::test]
async fn test_security_validation_workflow() {
    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    // Test with good data
    let good_data = create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0));
    let validation_result = validator.validate_data_point(&good_data).await;

    assert!(validation_result.is_ok());
    let result = validation_result.unwrap();
    assert!(result.is_valid);
    assert!(result.confidence_score > 0.7);
    assert_eq!(result.fraud_risk, FraudRiskLevel::VeryLow);

    // Update reputation
    validator.update_source_reputation(DataSource::Chainlink, &result);
    let reputation = validator.get_source_reputation(&DataSource::Chainlink);
    assert!(reputation.is_some());
    assert!(reputation.unwrap().reputation_score > 0.9);
}

#[tokio::test]
async fn test_whale_trading_scenario() {
    // Create whale-sized volume data
    let whale_data = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(2_000_000.0)), // $2M
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2005.0, Some(1_500_000.0)), // $1.5M
        create_mock_data_point(DataSource::Band, "ETH/USD", 1998.0, Some(3_000_000.0)), // $3M
    ];

    let config = create_test_aggregation_config();
    let mut aggregator = Aggregator::new(config);

    let result = aggregator.aggregate_prices("ETH/USD", whale_data).await;
    assert!(result.is_ok());

    let aggregated = result.unwrap();

    // Verify whale impact analysis
    assert!(aggregated.whale_impact.whale_activity_detected);
    assert!(aggregated.whale_impact.price_impact_bps > Decimal::ZERO);
    assert!(aggregated.whale_impact.liquidity_depth > Decimal::from(1_000_000));
    assert!(aggregated.whale_impact.max_order_size.is_some());
}

#[tokio::test]
async fn test_multi_source_consensus() {
    // Test consensus aggregation strategy
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::Consensus {
        min_sources: 2,
        threshold: Decimal::from_f64_retain(0.02).unwrap(), // 2% threshold
    };
    config.min_sources = 2;

    let mut aggregator = Aggregator::new(config);

    // Data points with tight consensus
    let consensus_data = vec![
        create_mock_data_point(DataSource::Chainlink, "BTC/USD", 50000.0, Some(500_000.0)),
        create_mock_data_point(DataSource::Pyth, "BTC/USD", 50100.0, Some(600_000.0)),
        create_mock_data_point(DataSource::Band, "BTC/USD", 49950.0, Some(450_000.0)),
        create_mock_data_point(DataSource::API3, "BTC/USD", 52000.0, Some(300_000.0)), // Outlier
    ];

    let result = aggregator.aggregate_prices("BTC/USD", consensus_data).await;
    assert!(result.is_ok());

    let aggregated = result.unwrap();

    // Should achieve consensus with 3 sources (excluding outlier)
    assert_eq!(aggregated.source_count, 3);
    assert!(aggregated.confidence > 0.8);

    // Price should be close to consensus group average
    let expected_price = (50000.0 + 50100.0 + 49950.0) / 3.0;
    let price_diff = (aggregated.price.to_string().parse::<f64>().unwrap() - expected_price).abs();
    assert!(price_diff < 100.0); // Within $100
}

#[tokio::test]
async fn test_weighted_aggregation() {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::WeightedAverage;

    // Set custom weights
    config.source_weights.insert(DataSource::Chainlink, 0.5);
    config.source_weights.insert(DataSource::Pyth, 0.3);
    config.source_weights.insert(DataSource::Band, 0.2);

    let mut aggregator = Aggregator::new(config);

    let weighted_data = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0)),
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2100.0, Some(150_000.0)),
        create_mock_data_point(DataSource::Band, "ETH/USD", 1900.0, Some(80_000.0)),
    ];

    let result = aggregator.aggregate_prices("ETH/USD", weighted_data).await;
    assert!(result.is_ok());

    let aggregated = result.unwrap();

    // Expected: (2000*0.5 + 2100*0.3 + 1900*0.2) = 2010
    assert_eq!(aggregated.price, Decimal::from(2010));
    assert!(aggregated.confidence > 0.8);
}

#[tokio::test]
async fn test_time_weighted_average() {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::TimeWeightedAverage {
        window: Duration::from_secs(300), // 5 minutes
    };

    let mut aggregator = Aggregator::new(config);

    // Create data points with different timestamps
    let mut twap_data = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0)),
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2020.0, Some(150_000.0)),
    ];

    // Adjust timestamps
    twap_data[0].timestamp = Utc::now() - chrono::Duration::seconds(120); // 2 minutes ago
    twap_data[1].timestamp = Utc::now() - chrono::Duration::seconds(60);  // 1 minute ago

    let result = aggregator.aggregate_prices("ETH/USD", twap_data).await;
    assert!(result.is_ok());

    let aggregated = result.unwrap();
    assert!(aggregated.confidence > 0.5);

    // TWAP should be between the two prices
    let price_value = aggregated.price.to_string().parse::<f64>().unwrap();
    assert!(price_value >= 2000.0 && price_value <= 2020.0);
}

#[tokio::test]
async fn test_fraud_detection() {
    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    // Create suspicious data point
    let mut suspicious_data = create_mock_data_point(
        DataSource::Chainlink,
        "ETH/USD",
        10000.0, // Unrealistically high price
        Some(5_000_000.0) // Very large volume
    );

    // Make it stale
    suspicious_data.timestamp = Utc::now() - chrono::Duration::seconds(600); // 10 minutes old

    let validation_result = validator.validate_data_point(&suspicious_data).await;
    assert!(validation_result.is_ok());

    let result = validation_result.unwrap();

    // Should be flagged as invalid or high risk
    assert!(!result.is_valid || result.fraud_risk >= FraudRiskLevel::High);
    assert!(!result.security_warnings.is_empty());

    // Check for specific warnings
    let has_stale_warning = result.security_warnings.iter().any(|w| matches!(w, SecurityWarning::StaleDataDetected { .. }));
    assert!(has_stale_warning);
}

#[tokio::test]
async fn test_circuit_breaker_functionality() {
    let mut config = SecurityConfig::default();
    config.circuit_breaker_threshold = 2; // Low threshold for testing

    let mut validator = SecurityValidator::new(config);

    // Simulate multiple validation failures
    for i in 0..3 {
        let bad_validation = ValidationResult {
            is_valid: false,
            confidence_score: 0.2,
            checks_performed: vec![],
            security_warnings: vec![],
            fraud_risk: FraudRiskLevel::High,
            integrity_verified: false,
            validated_at: Utc::now(),
        };

        validator.update_source_reputation(DataSource::Chainlink, &bad_validation);
    }

    // Circuit breaker should be active
    let reputation = validator.get_source_reputation(&DataSource::Chainlink);
    assert!(reputation.is_some());
    assert!(reputation.unwrap().circuit_breaker_active);

    // Test manual reset
    validator.reset_circuit_breaker(DataSource::Chainlink);
    let reputation_after_reset = validator.get_source_reputation(&DataSource::Chainlink);
    assert!(!reputation_after_reset.unwrap().circuit_breaker_active);
}

#[tokio::test]
async fn test_volume_weighted_average() {
    let mut config = AggregationConfig::default();
    config.strategy = AggregationStrategy::VolumeWeightedAverage;

    let mut aggregator = Aggregator::new(config);

    let vwap_data = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0)),
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2100.0, Some(200_000.0)), // Higher volume
        create_mock_data_point(DataSource::Band, "ETH/USD", 1900.0, Some(50_000.0)),   // Lower volume
    ];

    let result = aggregator.aggregate_prices("ETH/USD", vwap_data).await;
    assert!(result.is_ok());

    let aggregated = result.unwrap();

    // VWAP should be weighted toward the high-volume price (2100)
    let vwap_price = aggregated.price.to_string().parse::<f64>().unwrap();
    assert!(vwap_price > 2000.0); // Should be above simple average due to volume weighting
    assert!(aggregated.confidence > 0.8);
}

#[tokio::test]
async fn test_insufficient_sources_handling() {
    let mut config = AggregationConfig::default();
    config.min_sources = 5; // Require 5 sources

    let aggregator = Aggregator::new(config);

    // Provide only 2 sources
    let insufficient_data = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0)),
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2010.0, Some(150_000.0)),
    ];

    let result = aggregator.aggregate_prices("ETH/USD", insufficient_data).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        OracleError::InsufficientDataSources { required, available } => {
            assert_eq!(required, 5);
            assert_eq!(available, 2);
        }
        _ => panic!("Expected InsufficientDataSources error"),
    }
}

#[tokio::test]
async fn test_outlier_filtering() {
    let config = create_test_aggregation_config();
    let mut aggregator = Aggregator::new(config);

    // Include one obvious outlier
    let outlier_data = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0)),
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2010.0, Some(150_000.0)),
        create_mock_data_point(DataSource::Band, "ETH/USD", 1990.0, Some(80_000.0)),
        create_mock_data_point(DataSource::API3, "ETH/USD", 5000.0, Some(50_000.0)), // Outlier
    ];

    let result = aggregator.aggregate_prices("ETH/USD", outlier_data).await;
    assert!(result.is_ok());

    let aggregated = result.unwrap();

    // Should filter outlier and aggregate remaining 3 sources
    assert_eq!(aggregated.source_count, 3);
    assert_eq!(aggregated.price, Decimal::from(2000)); // Median without outlier
}

#[tokio::test]
async fn test_mev_attack_detection() {
    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    // Create data point that looks like MEV attack
    let mev_data = create_mock_data_point(
        DataSource::Chainlink,
        "ETH/USD",
        2000.0,
        Some(10_000_000.0) // Extremely large volume
    );

    let validation_result = validator.validate_data_point(&mev_data).await;
    assert!(validation_result.is_ok());

    let result = validation_result.unwrap();

    // Should detect MEV risk
    let mev_check = result.checks_performed.iter()
        .find(|c| c.check_name == "mev_detection");
    assert!(mev_check.is_some());

    // May have MEV warning
    let has_mev_warning = result.security_warnings.iter()
        .any(|w| matches!(w, SecurityWarning::MEVAttackSuspected { .. }));

    // Either the check should fail or there should be a warning
    assert!(!mev_check.unwrap().passed || has_mev_warning);
}

#[tokio::test]
async fn test_historical_consistency_validation() {
    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    // Add some historical data first
    for i in 0..5 {
        let historical_price = AggregatedPrice {
            price: Decimal::from(2000 + i * 10), // Gradual price increase
            symbol: "ETH/USD".to_string(),
            timestamp: Utc::now() - chrono::Duration::seconds(i * 60),
            confidence: 0.9,
            source_count: 3,
            contributing_sources: vec![DataSource::Chainlink, DataSource::Pyth, DataSource::Band],
            deviation: None,
            strategy: AggregationStrategy::Median,
            quality_metrics: QualityMetrics {
                price_variance: Decimal::from_f64_retain(0.01).unwrap(),
                max_spread: Decimal::from_f64_retain(20.0).unwrap(),
                outlier_rate: 0.0,
                avg_latency_ms: 100,
                source_reliability: HashMap::new(),
            },
            whale_impact: WhaleImpactAnalysis {
                price_impact_bps: Decimal::from_f64_retain(10.0).unwrap(),
                liquidity_depth: Decimal::from(1_000_000),
                volatility_score: 0.1,
                whale_activity_detected: false,
                max_order_size: None,
            },
        };

        validator.historical_data.push(historical_price);
    }

    // Test consistent price
    let consistent_price = AggregatedPrice {
        price: Decimal::from(2050), // Consistent with trend
        symbol: "ETH/USD".to_string(),
        timestamp: Utc::now(),
        confidence: 0.9,
        source_count: 3,
        contributing_sources: vec![DataSource::Chainlink, DataSource::Pyth, DataSource::Band],
        deviation: Some(Decimal::from_f64_retain(0.01).unwrap()),
        strategy: AggregationStrategy::Median,
        quality_metrics: QualityMetrics {
            price_variance: Decimal::from_f64_retain(0.01).unwrap(),
            max_spread: Decimal::from_f64_retain(20.0).unwrap(),
            outlier_rate: 0.0,
            avg_latency_ms: 100,
            source_reliability: HashMap::new(),
        },
        whale_impact: WhaleImpactAnalysis {
            price_impact_bps: Decimal::from_f64_retain(10.0).unwrap(),
            liquidity_depth: Decimal::from(1_000_000),
            volatility_score: 0.1,
            whale_activity_detected: false,
            max_order_size: None,
        },
    };

    let validation_result = validator.validate_aggregated_price(&consistent_price).await;
    assert!(validation_result.is_ok());

    let result = validation_result.unwrap();
    assert!(result.is_valid);

    let consistency_check = result.checks_performed.iter()
        .find(|c| c.check_name == "historical_consistency");
    assert!(consistency_check.is_some());
    assert!(consistency_check.unwrap().passed);
}

#[tokio::test]
async fn test_price_manipulation_detection() {
    let mut config = SecurityConfig::default();
    config.max_price_deviation = 0.05; // 5% max deviation

    let mut validator = SecurityValidator::new(config);

    // Add historical data with stable prices around $2000
    for _ in 0..3 {
        let stable_price = AggregatedPrice {
            price: Decimal::from(2000),
            symbol: "ETH/USD".to_string(),
            timestamp: Utc::now() - chrono::Duration::seconds(300),
            confidence: 0.9,
            source_count: 3,
            contributing_sources: vec![DataSource::Chainlink, DataSource::Pyth, DataSource::Band],
            deviation: None,
            strategy: AggregationStrategy::Median,
            quality_metrics: QualityMetrics {
                price_variance: Decimal::from_f64_retain(0.01).unwrap(),
                max_spread: Decimal::from_f64_retain(10.0).unwrap(),
                outlier_rate: 0.0,
                avg_latency_ms: 100,
                source_reliability: HashMap::new(),
            },
            whale_impact: WhaleImpactAnalysis {
                price_impact_bps: Decimal::from_f64_retain(5.0).unwrap(),
                liquidity_depth: Decimal::from(1_000_000),
                volatility_score: 0.05,
                whale_activity_detected: false,
                max_order_size: None,
            },
        };

        validator.historical_data.push(stable_price);
    }

    // Test with manipulated price (20% higher)
    let manipulated_data = create_mock_data_point(
        DataSource::Chainlink,
        "ETH/USD",
        2400.0, // 20% above normal
        Some(100_000.0)
    );

    let validation_result = validator.validate_data_point(&manipulated_data).await;
    assert!(validation_result.is_ok());

    let result = validation_result.unwrap();

    // Should flag as potential price manipulation
    let price_check = result.checks_performed.iter()
        .find(|c| c.check_name == "price_range");
    assert!(price_check.is_some());
    assert!(!price_check.unwrap().passed);

    // Should have price manipulation warning
    let has_manipulation_warning = result.security_warnings.iter()
        .any(|w| matches!(w, SecurityWarning::PriceManipulationSuspected { .. }));

    // Either low confidence or manipulation warning
    assert!(result.confidence_score < 0.7 || has_manipulation_warning);
}

#[tokio::test]
async fn test_cross_source_correlation() {
    let config = create_test_aggregation_config();
    let mut aggregator = Aggregator::new(config);

    // Create well-correlated data
    let correlated_data = vec![
        create_mock_data_point(DataSource::Chainlink, "ETH/USD", 2000.0, Some(100_000.0)),
        create_mock_data_point(DataSource::Pyth, "ETH/USD", 2005.0, Some(150_000.0)),     // Very close
        create_mock_data_point(DataSource::Band, "ETH/USD", 1998.0, Some(80_000.0)),      // Very close
    ];

    let result = aggregator.aggregate_prices("ETH/USD", correlated_data).await;
    assert!(result.is_ok());

    let aggregated = result.unwrap();

    // Validate the aggregated price
    let security_config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(security_config);

    let validation_result = validator.validate_aggregated_price(&aggregated).await;
    assert!(validation_result.is_ok());

    let validation = validation_result.unwrap();

    // Should pass correlation check due to tight price clustering
    let correlation_check = validation.checks_performed.iter()
        .find(|c| c.check_name == "cross_source_correlation");
    assert!(correlation_check.is_some());
    assert!(correlation_check.unwrap().passed);
}