//! # Oracle System Monitoring Example
//!
//! This example demonstrates comprehensive monitoring and health checking
//! of the oracle system including:
//! - Source health monitoring
//! - Performance metrics tracking
//! - Reputation management
//! - Circuit breaker functionality
//! - System diagnostics
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example oracle_monitoring
//! ```

use moby_oracle::*;
use moby_oracle::sources::*;
use moby_oracle::aggregation::*;
use moby_oracle::security::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::Utc;
use rust_decimal::Decimal;
use tokio::time::sleep;

/// System health status
#[derive(Debug, Clone)]
struct SystemHealth {
    pub overall_status: HealthStatus,
    pub active_sources: usize,
    pub circuit_breakers_active: usize,
    pub avg_latency_ms: u64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
}

/// Health status levels
#[derive(Debug, Clone, PartialEq)]
enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Down,
}

/// Performance metrics
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    pub aggregations_per_second: f64,
    pub validations_per_second: f64,
    pub average_response_time: Duration,
    pub success_rate: f64,
    pub memory_usage_mb: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("🔍 Moby Oracle - System Monitoring & Health Checking");
    println!("===================================================");

    // Step 1: Initialize monitoring system
    println!("\n🏥 Initializing Health Monitoring System:");
    let mut monitor = OracleMonitor::new().await?;

    // Step 2: Source health checking
    println!("\n📊 Source Health Assessment:");
    await check_source_health(&mut monitor).await?;

    // Step 3: Performance metrics collection
    println!("\n⚡ Performance Metrics Collection:");
    await collect_performance_metrics(&mut monitor).await?;

    // Step 4: Reputation management
    println!("\n🏆 Source Reputation Management:");
    await demonstrate_reputation_management(&mut monitor).await?;

    // Step 5: Circuit breaker functionality
    println!("\n🔌 Circuit Breaker Demonstration:");
    await demonstrate_circuit_breakers(&mut monitor).await?;

    // Step 6: Real-time monitoring
    println!("\n📡 Real-time System Monitoring:");
    await realtime_monitoring(&mut monitor).await?;

    // Step 7: Generate health report
    println!("\n📋 System Health Report:");
    await generate_health_report(&monitor).await?;

    println!("\n✅ Oracle monitoring demonstration completed!");
    Ok(())
}

/// Oracle monitoring system
struct OracleMonitor {
    security_validator: SecurityValidator,
    aggregator: Aggregator,
    start_time: Instant,
    total_operations: u64,
    successful_operations: u64,
    response_times: Vec<Duration>,
    error_count: u64,
}

impl OracleMonitor {
    async fn new() -> Result<Self> {
        Ok(Self {
            security_validator: SecurityValidator::new(SecurityConfig::default()),
            aggregator: Aggregator::new(AggregationConfig::default()),
            start_time: Instant::now(),
            total_operations: 0,
            successful_operations: 0,
            response_times: Vec::new(),
            error_count: 0,
        })
    }

    async fn perform_health_check(&mut self, source: DataSource) -> Result<SourceHealth> {
        let start = Instant::now();

        // Simulate source health check
        let health = match source {
            DataSource::Chainlink => SourceHealth {
                is_healthy: true,
                last_successful_fetch: Some(Utc::now()),
                consecutive_failures: 0,
                average_response_time: Duration::from_millis(120),
                error_rate: 0.02,
                status_message: "Operational".to_string(),
            },
            DataSource::Pyth => SourceHealth {
                is_healthy: true,
                last_successful_fetch: Some(Utc::now() - chrono::Duration::seconds(30)),
                consecutive_failures: 1,
                average_response_time: Duration::from_millis(85),
                error_rate: 0.05,
                status_message: "Minor delays".to_string(),
            },
            DataSource::Band => SourceHealth {
                is_healthy: false,
                last_successful_fetch: Some(Utc::now() - chrono::Duration::minutes(15)),
                consecutive_failures: 5,
                average_response_time: Duration::from_millis(800),
                error_rate: 0.25,
                status_message: "Connection issues".to_string(),
            },
            DataSource::API3 => SourceHealth {
                is_healthy: true,
                last_successful_fetch: Some(Utc::now() - chrono::Duration::seconds(5)),
                consecutive_failures: 0,
                average_response_time: Duration::from_millis(95),
                error_rate: 0.01,
                status_message: "Excellent performance".to_string(),
            },
            DataSource::UMA => SourceHealth {
                is_healthy: false,
                last_successful_fetch: Some(Utc::now() - chrono::Duration::hours(2)),
                consecutive_failures: 15,
                average_response_time: Duration::from_millis(2000),
                error_rate: 0.80,
                status_message: "Service degraded".to_string(),
            },
        };

        let response_time = start.elapsed();
        self.response_times.push(response_time);
        self.total_operations += 1;

        if health.is_healthy {
            self.successful_operations += 1;
        } else {
            self.error_count += 1;
        }

        Ok(health)
    }

    fn get_system_health(&self) -> SystemHealth {
        let uptime = self.start_time.elapsed().as_secs();
        let error_rate = if self.total_operations > 0 {
            self.error_count as f64 / self.total_operations as f64
        } else {
            0.0
        };

        let avg_latency = if !self.response_times.is_empty() {
            self.response_times.iter().sum::<Duration>().as_millis() as u64
                / self.response_times.len() as u64
        } else {
            0
        };

        let overall_status = if error_rate > 0.5 {
            HealthStatus::Critical
        } else if error_rate > 0.2 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        SystemHealth {
            overall_status,
            active_sources: 3, // Simulated
            circuit_breakers_active: 1, // Simulated
            avg_latency_ms: avg_latency,
            error_rate,
            uptime_seconds: uptime,
        }
    }

    fn get_performance_metrics(&self) -> PerformanceMetrics {
        let uptime_secs = self.start_time.elapsed().as_secs_f64();
        let aggregations_per_second = if uptime_secs > 0.0 {
            self.successful_operations as f64 / uptime_secs
        } else {
            0.0
        };

        let avg_response_time = if !self.response_times.is_empty() {
            self.response_times.iter().sum::<Duration>() / self.response_times.len() as u32
        } else {
            Duration::from_millis(0)
        };

        let success_rate = if self.total_operations > 0 {
            self.successful_operations as f64 / self.total_operations as f64
        } else {
            0.0
        };

        PerformanceMetrics {
            aggregations_per_second,
            validations_per_second: aggregations_per_second * 1.2, // Estimate
            average_response_time: avg_response_time,
            success_rate,
            memory_usage_mb: 45.6, // Simulated
        }
    }
}

/// Check health of all oracle sources
async fn check_source_health(monitor: &mut OracleMonitor) -> Result<()> {
    let sources = vec![
        DataSource::Chainlink,
        DataSource::Pyth,
        DataSource::Band,
        DataSource::API3,
        DataSource::UMA,
    ];

    for source in sources {
        let health = monitor.perform_health_check(source).await?;

        println!("   {:?}:", source);
        println!("     Status: {}", if health.is_healthy { "🟢 Healthy" } else { "🔴 Unhealthy" });
        println!("     Response Time: {:?}", health.average_response_time);
        println!("     Error Rate: {:.1}%", health.error_rate * 100.0);
        println!("     Consecutive Failures: {}", health.consecutive_failures);
        println!("     Message: {}", health.status_message);

        if let Some(last_fetch) = health.last_successful_fetch {
            let age = Utc::now().signed_duration_since(last_fetch);
            println!("     Last Success: {} ago", format_duration(age));
        }

        // Health recommendations
        if !health.is_healthy {
            if health.consecutive_failures > 10 {
                println!("     🚨 Recommendation: Disable source temporarily");
            } else if health.error_rate > 0.1 {
                println!("     ⚠️  Recommendation: Reduce weight in aggregation");
            }
        }

        println!();
    }

    Ok(())
}

/// Collect and display performance metrics
async fn collect_performance_metrics(monitor: &mut OracleMonitor) -> Result<()> {
    println!("   Running performance test suite...");

    // Simulate load testing
    let test_data = create_test_data_points(50);

    let load_test_start = Instant::now();
    for (i, data_point) in test_data.iter().enumerate() {
        let operation_start = Instant::now();

        // Simulate validation
        let _validation = monitor.security_validator.validate_data_point(data_point).await?;

        // Simulate aggregation if we have enough data
        if i >= 2 {
            let aggregation_data = test_data[i-2..=i].to_vec();
            let _result = monitor.aggregator.aggregate_prices("ETH/USD", aggregation_data).await?;
        }

        let operation_time = operation_start.elapsed();
        monitor.response_times.push(operation_time);
        monitor.total_operations += 1;
        monitor.successful_operations += 1;

        // Brief pause to simulate realistic timing
        sleep(Duration::from_millis(10)).await;
    }

    let total_test_time = load_test_start.elapsed();
    let metrics = monitor.get_performance_metrics();

    println!("   Performance Results:");
    println!("     Test Duration: {:?}", total_test_time);
    println!("     Operations/Second: {:.1}", metrics.aggregations_per_second);
    println!("     Average Response Time: {:?}", metrics.average_response_time);
    println!("     Success Rate: {:.1}%", metrics.success_rate * 100.0);
    println!("     Memory Usage: {:.1} MB", metrics.memory_usage_mb);

    // Performance benchmarks
    if metrics.aggregations_per_second < 10.0 {
        println!("     ⚠️  Performance Warning: Low throughput");
    } else if metrics.aggregations_per_second > 100.0 {
        println!("     ✅ Excellent performance");
    } else {
        println!("     ✅ Performance within normal range");
    }

    if metrics.average_response_time > Duration::from_millis(500) {
        println!("     ⚠️  Latency Warning: High response times");
    }

    Ok(())
}

/// Demonstrate source reputation management
async fn demonstrate_reputation_management(monitor: &mut OracleMonitor) -> Result<()> {
    println!("   Simulating reputation changes over time...");

    let sources = [DataSource::Chainlink, DataSource::Pyth, DataSource::Band];

    // Simulate various scenarios affecting reputation
    for round in 1..=5 {
        println!("\n   Round {}:", round);

        for &source in &sources {
            // Create test data with varying quality
            let quality_factor = match (source, round) {
                (DataSource::Chainlink, _) => 0.95,           // Consistently good
                (DataSource::Pyth, r) if r <= 2 => 0.85,      // Starts poor, improves
                (DataSource::Pyth, _) => 0.93,
                (DataSource::Band, r) if r >= 4 => 0.60,      // Degrades over time
                (DataSource::Band, _) => 0.90,
                _ => 0.80,
            };

            let data_point = create_test_data_point(source, quality_factor);
            let validation = monitor.security_validator.validate_data_point(&data_point).await?;

            // Update reputation
            monitor.security_validator.update_source_reputation(source, &validation);

            // Get current reputation
            if let Some(reputation) = monitor.security_validator.get_source_reputation(&source) {
                println!("     {:?}: Score {:.2}, Failures: {}, Circuit Breaker: {}",
                    source,
                    reputation.reputation_score,
                    reputation.failure_count,
                    if reputation.circuit_breaker_active { "🔴 ACTIVE" } else { "🟢 OFF" });

                // Reputation-based recommendations
                if reputation.reputation_score < 0.3 {
                    println!("       🚨 Action: Remove from aggregation pool");
                } else if reputation.reputation_score < 0.6 {
                    println!("       ⚠️  Action: Reduce aggregation weight");
                } else if reputation.reputation_score > 0.9 {
                    println!("       ✅ Status: Trusted source");
                }
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Demonstrate circuit breaker functionality
async fn demonstrate_circuit_breakers(monitor: &mut OracleMonitor) -> Result<()> {
    println!("   Testing circuit breaker activation and recovery...");

    let test_source = DataSource::Band;

    // Simulate failures to trigger circuit breaker
    println!("\n   Simulating consecutive failures:");
    for failure_count in 1..=6 {
        let bad_data = create_bad_data_point(test_source);
        let validation = monitor.security_validator.validate_data_point(&bad_data).await?;

        monitor.security_validator.update_source_reputation(test_source, &validation);

        if let Some(reputation) = monitor.security_validator.get_source_reputation(&test_source) {
            println!("     Failure {}: Score {:.2}, Circuit Breaker: {}",
                failure_count,
                reputation.reputation_score,
                if reputation.circuit_breaker_active { "🔴 ACTIVE" } else { "🟢 OFF" });

            if reputation.circuit_breaker_active && failure_count >= 5 {
                println!("     🚨 Circuit breaker activated after {} failures", failure_count);
                break;
            }
        }
    }

    // Simulate manual reset
    println!("\n   Testing manual circuit breaker reset:");
    monitor.security_validator.reset_circuit_breaker(test_source);

    if let Some(reputation) = monitor.security_validator.get_source_reputation(&test_source) {
        println!("     After reset: Circuit Breaker: {}",
            if reputation.circuit_breaker_active { "🔴 ACTIVE" } else { "🟢 OFF" });
        println!("     ✅ Circuit breaker successfully reset");
    }

    // Test recovery with good data
    println!("\n   Testing recovery with good data:");
    for recovery_round in 1..=3 {
        let good_data = create_test_data_point(test_source, 0.95);
        let validation = monitor.security_validator.validate_data_point(&good_data).await?;

        monitor.security_validator.update_source_reputation(test_source, &validation);

        if let Some(reputation) = monitor.security_validator.get_source_reputation(&test_source) {
            println!("     Recovery {}: Score {:.2}",
                recovery_round,
                reputation.reputation_score);
        }
    }

    Ok(())
}

/// Real-time monitoring simulation
async fn realtime_monitoring(monitor: &mut OracleMonitor) -> Result<()> {
    println!("   Starting 10-second real-time monitoring...");

    let monitoring_start = Instant::now();
    let mut status_checks = 0;

    while monitoring_start.elapsed() < Duration::from_secs(10) {
        // Perform system health check
        let health = monitor.get_system_health();

        status_checks += 1;

        if status_checks % 5 == 0 { // Report every 5th check
            println!("     Status Check #{}: {} (Latency: {}ms, Error Rate: {:.1}%)",
                status_checks,
                format_health_status(&health.overall_status),
                health.avg_latency_ms,
                health.error_rate * 100.0);

            // Alert on issues
            match health.overall_status {
                HealthStatus::Critical => {
                    println!("       🚨 CRITICAL ALERT: System requires immediate attention");
                }
                HealthStatus::Warning => {
                    println!("       ⚠️  WARNING: Performance degradation detected");
                }
                _ => {}
            }
        }

        // Simulate some operations
        let test_data = create_test_data_point(DataSource::Chainlink, 0.9);
        let _validation = monitor.security_validator.validate_data_point(&test_data).await?;

        sleep(Duration::from_millis(200)).await; // 5 Hz monitoring
    }

    println!("   Real-time monitoring completed. Total status checks: {}", status_checks);

    Ok(())
}

/// Generate comprehensive health report
async fn generate_health_report(monitor: &OracleMonitor) -> Result<()> {
    let health = monitor.get_system_health();
    let metrics = monitor.get_performance_metrics();

    println!("╭─────────────────────────────────────────────────────────╮");
    println!("│                  ORACLE HEALTH REPORT                  │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ Overall Status: {:43} │", format_health_status(&health.overall_status));
    println!("│ Uptime: {:47} │", format_uptime(health.uptime_seconds));
    println!("│ Active Sources: {:39} │", health.active_sources);
    println!("│ Circuit Breakers Active: {:31} │", health.circuit_breakers_active);
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│                   PERFORMANCE METRICS                  │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ Operations/Second: {:36.1} │", metrics.aggregations_per_second);
    println!("│ Average Response Time: {:32?} │", metrics.average_response_time);
    println!("│ Success Rate: {:41.1}% │", metrics.success_rate * 100.0);
    println!("│ Error Rate: {:43.1}% │", health.error_rate * 100.0);
    println!("│ Average Latency: {:35}ms │", health.avg_latency_ms);
    println!("│ Memory Usage: {:38.1} MB │", metrics.memory_usage_mb);
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│                      RECOMMENDATIONS                   │");
    println!("├─────────────────────────────────────────────────────────┤");

    // Generate recommendations
    let mut recommendations = Vec::new();

    if health.error_rate > 0.1 {
        recommendations.push("Investigate high error rate");
    }
    if metrics.average_response_time > Duration::from_millis(200) {
        recommendations.push("Optimize response times");
    }
    if health.circuit_breakers_active > 0 {
        recommendations.push("Address circuit breaker issues");
    }
    if metrics.aggregations_per_second < 10.0 {
        recommendations.push("Scale system for higher throughput");
    }
    if health.active_sources < 3 {
        recommendations.push("Add more data sources for redundancy");
    }

    if recommendations.is_empty() {
        println!("│ ✅ System is operating within normal parameters        │");
    } else {
        for (i, recommendation) in recommendations.iter().enumerate() {
            if i == 0 {
                println!("│ • {:53} │", recommendation);
            } else {
                println!("│ • {:53} │", recommendation);
            }
        }
    }

    println!("╰─────────────────────────────────────────────────────────╯");

    Ok(())
}

// Helper functions

fn create_test_data_points(count: usize) -> Vec<DataPoint> {
    let sources = [DataSource::Chainlink, DataSource::Pyth, DataSource::Band, DataSource::API3];
    (0..count).map(|i| {
        let source = sources[i % sources.len()];
        create_test_data_point(source, 0.9)
    }).collect()
}

fn create_test_data_point(source: DataSource, quality: f64) -> DataPoint {
    DataPoint {
        source,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(2000),
        timestamp: if quality > 0.8 { Utc::now() } else { Utc::now() - chrono::Duration::seconds(300) },
        confidence: quality,
        volume: Some(Decimal::from(100_000)),
        metadata: HashMap::new(),
    }
}

fn create_bad_data_point(source: DataSource) -> DataPoint {
    DataPoint {
        source,
        symbol: "ETH/USD".to_string(),
        value: Decimal::from(-100), // Invalid negative price
        timestamp: Utc::now() - chrono::Duration::seconds(600), // Stale
        confidence: 0.1, // Very low confidence
        volume: Some(Decimal::from(0)), // No volume
        metadata: HashMap::new(),
    }
}

fn format_health_status(status: &HealthStatus) -> String {
    match status {
        HealthStatus::Healthy => "🟢 HEALTHY".to_string(),
        HealthStatus::Warning => "🟡 WARNING".to_string(),
        HealthStatus::Critical => "🔴 CRITICAL".to_string(),
        HealthStatus::Down => "⚫ DOWN".to_string(),
    }
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        format!("{}m", total_seconds / 60)
    } else if total_seconds < 86400 {
        format!("{}h", total_seconds / 3600)
    } else {
        format!("{}d", total_seconds / 86400)
    }
}