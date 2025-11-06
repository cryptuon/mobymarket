//! Performance and load testing for the oracle system
//!
//! These tests verify that the oracle system can handle high-throughput scenarios
//! typical in whale trading operations, including stress testing and benchmarking.

use moby_oracle::*;
use moby_oracle::sources::*;
use moby_oracle::aggregation::*;
use moby_oracle::security::*;
use tokio_test;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use futures::future::join_all;

/// Helper to create large datasets for performance testing
fn create_large_dataset(size: usize, symbol: &str) -> Vec<DataPoint> {
    let mut data_points = Vec::with_capacity(size);
    let sources = [DataSource::Chainlink, DataSource::Pyth, DataSource::Band, DataSource::API3, DataSource::UMA];

    for i in 0..size {
        let source = sources[i % sources.len()];
        let base_price = 2000.0;
        let price_variation = (i as f64 % 100.0) / 100.0 * 50.0 - 25.0; // ±$25 variation
        let price = base_price + price_variation;

        data_points.push(DataPoint {
            source,
            symbol: symbol.to_string(),
            value: Decimal::from_f64_retain(price).unwrap(),
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
            confidence: 0.9 + (i as f64 % 10.0) / 100.0, // 0.90-0.99
            volume: Some(Decimal::from(50_000 + (i % 500_000))), // $50K-$550K
            metadata: HashMap::new(),
        });
    }

    data_points
}

/// Helper to measure execution time
async fn measure_time<F, T>(operation: F) -> (T, Duration)
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = operation.await;
    let duration = start.elapsed();
    (result, duration)
}

#[tokio::test]
async fn test_aggregation_performance_large_dataset() {
    const DATASET_SIZE: usize = 1000;

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    let large_dataset = create_large_dataset(DATASET_SIZE, "ETH/USD");

    let (result, duration) = measure_time(async {
        aggregator.aggregate_prices("ETH/USD", large_dataset).await
    }).await;

    assert!(result.is_ok(), "Large dataset aggregation should succeed");
    assert!(duration < Duration::from_millis(100),
        "Aggregation should complete under 100ms, took {:?}", duration);

    let aggregated = result.unwrap();
    assert_eq!(aggregated.source_count, DATASET_SIZE);
    assert!(aggregated.confidence > 0.5);
}

#[tokio::test]
async fn test_concurrent_aggregations() {
    const NUM_CONCURRENT: usize = 10;
    const DATASET_SIZE: usize = 100;

    let symbols = vec!["ETH/USD", "BTC/USD", "SOL/USD", "AVAX/USD", "MATIC/USD"];

    let tasks: Vec<_> = (0..NUM_CONCURRENT).map(|i| {
        let symbol = symbols[i % symbols.len()];
        async move {
            let config = AggregationConfig::default();
            let mut aggregator = Aggregator::new(config);
            let dataset = create_large_dataset(DATASET_SIZE, symbol);

            let start = Instant::now();
            let result = aggregator.aggregate_prices(symbol, dataset).await;
            let duration = start.elapsed();

            (result, duration)
        }
    }).collect();

    let (results, total_duration) = measure_time(async {
        join_all(tasks).await
    }).await;

    // All aggregations should succeed
    for (result, individual_duration) in &results {
        assert!(result.is_ok(), "Concurrent aggregation should succeed");
        assert!(individual_duration < &Duration::from_millis(50),
            "Individual aggregation should be fast: {:?}", individual_duration);
    }

    // Total time should be much less than sequential execution
    assert!(total_duration < Duration::from_millis(200),
        "Concurrent execution should be efficient: {:?}", total_duration);

    println!("Completed {} concurrent aggregations in {:?}", NUM_CONCURRENT, total_duration);
}

#[tokio::test]
async fn test_security_validation_throughput() {
    const NUM_VALIDATIONS: usize = 500;

    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    let test_data: Vec<_> = (0..NUM_VALIDATIONS).map(|i| {
        DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000 + (i % 100)), // Price range 2000-2099
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        }
    }).collect();

    let (results, duration) = measure_time(async {
        let mut validation_results = Vec::new();
        for data_point in test_data {
            let result = validator.validate_data_point(&data_point).await;
            validation_results.push(result);
        }
        validation_results
    }).await;

    // Check that all validations succeeded
    for result in &results {
        assert!(result.is_ok(), "Validation should succeed");
    }

    let throughput = NUM_VALIDATIONS as f64 / duration.as_secs_f64();
    assert!(throughput > 100.0, "Should validate at least 100 points/second, got {:.2}", throughput);

    println!("Security validation throughput: {:.2} validations/second", throughput);
}

#[tokio::test]
async fn test_whale_order_impact_calculation_performance() {
    const NUM_WHALE_ORDERS: usize = 100;

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    // Create whale-sized orders
    let whale_orders: Vec<_> = (0..NUM_WHALE_ORDERS).map(|i| {
        let volume = 1_000_000.0 + (i as f64 * 100_000.0); // $1M-$10M+ orders
        vec![
            DataPoint {
                source: DataSource::Chainlink,
                symbol: "ETH/USD".to_string(),
                value: Decimal::from(2000),
                timestamp: Utc::now(),
                confidence: 0.95,
                volume: Some(Decimal::from_f64_retain(volume).unwrap()),
                metadata: HashMap::new(),
            }
        ]
    }).collect();

    let (results, duration) = measure_time(async {
        let mut aggregation_results = Vec::new();
        for order_data in whale_orders {
            let result = aggregator.aggregate_prices("ETH/USD", order_data).await;
            aggregation_results.push(result);
        }
        aggregation_results
    }).await;

    // Verify all whale impact calculations completed
    for result in &results {
        assert!(result.is_ok(), "Whale impact calculation should succeed");
        let aggregated = result.as_ref().unwrap();
        assert!(aggregated.whale_impact.whale_activity_detected);
        assert!(aggregated.whale_impact.price_impact_bps > Decimal::ZERO);
    }

    let throughput = NUM_WHALE_ORDERS as f64 / duration.as_secs_f64();
    assert!(throughput > 50.0, "Should process at least 50 whale orders/second, got {:.2}", throughput);

    println!("Whale impact calculation throughput: {:.2} orders/second", throughput);
}

#[tokio::test]
async fn test_historical_data_performance() {
    const HISTORICAL_SIZE: usize = 10000;

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    // Build up historical data
    let (_, build_duration) = measure_time(async {
        for i in 0..HISTORICAL_SIZE {
            let data_point = vec![DataPoint {
                source: DataSource::Chainlink,
                symbol: "ETH/USD".to_string(),
                value: Decimal::from(2000 + (i % 100)),
                timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
                confidence: 0.95,
                volume: Some(Decimal::from(100_000)),
                metadata: HashMap::new(),
            }];

            let _ = aggregator.aggregate_prices("ETH/USD", data_point).await;
        }
    }).await;

    // Test querying historical data
    let (historical_data, query_duration) = measure_time(async {
        aggregator.get_historical_prices(Some(1000))
    }).await;

    assert_eq!(historical_data.len(), 1000);
    assert!(query_duration < Duration::from_millis(10),
        "Historical query should be fast: {:?}", query_duration);

    println!("Built {} historical records in {:?}", HISTORICAL_SIZE, build_duration);
    println!("Queried 1000 records in {:?}", query_duration);
}

#[tokio::test]
async fn test_outlier_filtering_performance() {
    const DATASET_SIZE: usize = 1000;
    const OUTLIER_PERCENTAGE: f64 = 0.2; // 20% outliers

    let mut dataset = create_large_dataset(DATASET_SIZE, "ETH/USD");

    // Inject outliers
    let num_outliers = (DATASET_SIZE as f64 * OUTLIER_PERCENTAGE) as usize;
    for i in 0..num_outliers {
        dataset[i].value = Decimal::from(10000); // Obvious outlier price
    }

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    let (result, duration) = measure_time(async {
        aggregator.aggregate_prices("ETH/USD", dataset).await
    }).await;

    assert!(result.is_ok(), "Outlier filtering should succeed");
    assert!(duration < Duration::from_millis(150),
        "Outlier filtering should be efficient: {:?}", duration);

    let aggregated = result.unwrap();

    // Should filter out outliers
    assert!(aggregated.source_count < DATASET_SIZE);
    assert!(aggregated.source_count >= DATASET_SIZE - num_outliers);

    println!("Filtered {} outliers from {} points in {:?}",
        num_outliers, DATASET_SIZE, duration);
}

#[tokio::test]
async fn test_memory_usage_stability() {
    const NUM_ITERATIONS: usize = 1000;
    const DATASET_SIZE: usize = 50;

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    let (_, duration) = measure_time(async {
        for i in 0..NUM_ITERATIONS {
            let dataset = create_large_dataset(DATASET_SIZE, "ETH/USD");
            let result = aggregator.aggregate_prices("ETH/USD", dataset).await;
            assert!(result.is_ok(), "Iteration {} should succeed", i);

            // Occasionally check that historical data doesn't grow unbounded
            if i % 100 == 0 {
                let historical_count = aggregator.get_historical_prices(None).len();
                assert!(historical_count <= 1000, "Historical data should be bounded at iteration {}", i);
            }
        }
    }).await;

    let throughput = NUM_ITERATIONS as f64 / duration.as_secs_f64();
    assert!(throughput > 10.0, "Should maintain throughput over iterations: {:.2}", throughput);

    // Check final historical data size
    let final_historical = aggregator.get_historical_prices(None).len();
    assert!(final_historical <= 1000, "Historical data should remain bounded: {}", final_historical);

    println!("Completed {} iterations in {:?} (throughput: {:.2}/sec)",
        NUM_ITERATIONS, duration, throughput);
}

#[tokio::test]
async fn test_high_frequency_updates() {
    const UPDATE_FREQUENCY: Duration = Duration::from_millis(10); // 100 Hz
    const TEST_DURATION: Duration = Duration::from_secs(5);

    let config = AggregationConfig::default();
    let mut aggregator = Aggregator::new(config);

    let start_time = Instant::now();
    let mut update_count = 0;

    while start_time.elapsed() < TEST_DURATION {
        let data_point = vec![DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000 + (update_count % 10)), // Small price variations
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        }];

        let result = aggregator.aggregate_prices("ETH/USD", data_point).await;
        assert!(result.is_ok(), "High-frequency update {} should succeed", update_count);

        update_count += 1;

        // Maintain update frequency
        tokio::time::sleep(UPDATE_FREQUENCY).await;
    }

    let actual_frequency = update_count as f64 / TEST_DURATION.as_secs_f64();
    let expected_frequency = 1.0 / UPDATE_FREQUENCY.as_secs_f64();

    // Allow 10% tolerance for timing variations
    assert!(actual_frequency >= expected_frequency * 0.9,
        "Should maintain high frequency updates: {:.2} Hz (expected {:.2} Hz)",
        actual_frequency, expected_frequency);

    println!("Maintained {:.2} Hz update rate for {} seconds ({} total updates)",
        actual_frequency, TEST_DURATION.as_secs(), update_count);
}

#[tokio::test]
async fn test_multi_symbol_concurrent_processing() {
    const NUM_SYMBOLS: usize = 20;
    const UPDATES_PER_SYMBOL: usize = 50;

    let symbols: Vec<String> = (0..NUM_SYMBOLS)
        .map(|i| format!("SYMBOL_{}/USD", i))
        .collect();

    let tasks: Vec<_> = symbols.into_iter().map(|symbol| {
        async move {
            let config = AggregationConfig::default();
            let mut aggregator = Aggregator::new(config);

            let start = Instant::now();
            for j in 0..UPDATES_PER_SYMBOL {
                let data_point = vec![DataPoint {
                    source: DataSource::Chainlink,
                    symbol: symbol.clone(),
                    value: Decimal::from(1000 + j * 10), // Increasing price
                    timestamp: Utc::now(),
                    confidence: 0.95,
                    volume: Some(Decimal::from(100_000)),
                    metadata: HashMap::new(),
                }];

                let result = aggregator.aggregate_prices(&symbol, data_point).await;
                assert!(result.is_ok(), "Update {} for {} should succeed", j, symbol);
            }

            (symbol, start.elapsed())
        }
    }).collect();

    let (results, total_duration) = measure_time(async {
        join_all(tasks).await
    }).await;

    // Verify all symbols processed successfully
    for (symbol, duration) in &results {
        assert!(duration < &Duration::from_secs(1),
            "Symbol {} should process quickly: {:?}", symbol, duration);
    }

    let total_updates = NUM_SYMBOLS * UPDATES_PER_SYMBOL;
    let throughput = total_updates as f64 / total_duration.as_secs_f64();

    assert!(throughput > 500.0,
        "Should handle multi-symbol processing efficiently: {:.2} updates/sec", throughput);

    println!("Processed {} symbols × {} updates = {} total updates in {:?} ({:.2} updates/sec)",
        NUM_SYMBOLS, UPDATES_PER_SYMBOL, total_updates, total_duration, throughput);
}

#[tokio::test]
async fn test_error_handling_performance() {
    const NUM_ERROR_CASES: usize = 1000;

    let config = SecurityConfig::default();
    let mut validator = SecurityValidator::new(config);

    // Create various error conditions
    let error_cases: Vec<_> = (0..NUM_ERROR_CASES).map(|i| {
        let mut data_point = DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        };

        // Introduce different error conditions
        match i % 4 {
            0 => data_point.value = Decimal::from(-100), // Invalid negative price
            1 => data_point.timestamp = Utc::now() - chrono::Duration::seconds(3600), // Very stale
            2 => data_point.confidence = 0.0, // Zero confidence
            3 => data_point.volume = Some(Decimal::from(0)), // Zero volume
            _ => {}
        }

        data_point
    }).collect();

    let (results, duration) = measure_time(async {
        let mut validation_results = Vec::new();
        for data_point in error_cases {
            let result = validator.validate_data_point(&data_point).await;
            validation_results.push(result);
        }
        validation_results
    }).await;

    // All validations should complete (even if they fail validation)
    for result in &results {
        assert!(result.is_ok(), "Validation should complete even for invalid data");
    }

    let throughput = NUM_ERROR_CASES as f64 / duration.as_secs_f64();
    assert!(throughput > 100.0,
        "Error handling should maintain performance: {:.2} validations/sec", throughput);

    // Count how many were properly flagged as invalid
    let invalid_count = results.iter()
        .filter(|r| !r.as_ref().unwrap().is_valid)
        .count();

    assert!(invalid_count > NUM_ERROR_CASES / 2,
        "Should properly detect invalid data: {}/{} flagged", invalid_count, NUM_ERROR_CASES);

    println!("Handled {} error cases in {:?} ({:.2}/sec), flagged {}/{} as invalid",
        NUM_ERROR_CASES, duration, throughput, invalid_count, NUM_ERROR_CASES);
}