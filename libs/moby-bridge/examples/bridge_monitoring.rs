//! Example: Bridge monitoring and health management
//!
//! This example demonstrates comprehensive monitoring capabilities of the
//! Moby Bridge system including health checks, metrics collection, alerting,
//! and emergency response procedures.

use moby_bridge::{
    BridgeSystem, BridgeConfig, TransferRequest, PrivacyLevel, TransferPriority,
    system::{TransferStatus, SystemStatus, AlertThresholds, HealthStatus},
    error::BridgeResult,
};
use std::collections::HashMap;
use chrono::{Utc, Duration};
use rust_decimal::Decimal;
use tokio::time::{sleep, interval, Duration as TokioDuration};

/// Custom event handler for monitoring
struct MonitoringEventHandler {
    alert_count: std::sync::Arc<std::sync::Mutex<u32>>,
}

impl MonitoringEventHandler {
    fn new() -> Self {
        Self {
            alert_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    fn get_alert_count(&self) -> u32 {
        *self.alert_count.lock().unwrap()
    }
}

#[moby_bridge::async_trait::async_trait]
impl moby_bridge::system::EventHandler for MonitoringEventHandler {
    async fn handle_event(&self, event: &moby_bridge::system::BridgeEvent) {
        match event {
            moby_bridge::system::BridgeEvent::TransferFailed { transfer_id, reason, timestamp } => {
                println!("🚨 ALERT: Transfer {} failed at {}: {}", transfer_id, timestamp, reason);
                let mut count = self.alert_count.lock().unwrap();
                *count += 1;
            }
            moby_bridge::system::BridgeEvent::EmergencyPause { reason, timestamp } => {
                println!("🔴 CRITICAL ALERT: Emergency pause activated at {}: {}", timestamp, reason);
                let mut count = self.alert_count.lock().unwrap();
                *count += 10; // High priority alert
            }
            moby_bridge::system::BridgeEvent::TransferCompleted { transfer_id, tx_hash, timestamp } => {
                println!("✅ Transfer {} completed at {} (TX: {})", transfer_id, timestamp, tx_hash);
            }
            _ => {
                // Handle other events as needed
            }
        }
    }
}

#[tokio::main]
async fn main() -> BridgeResult<()> {
    println!("📊 Moby Bridge - Monitoring & Health Management Example");
    println!("======================================================\n");

    // Initialize bridge with monitoring-optimized configuration
    println!("🔧 Initializing bridge with enhanced monitoring...");
    let bridge = BridgeSystem::new().await?;

    // Configure bridge for comprehensive monitoring
    let monitoring_config = BridgeConfig {
        version: "0.1.0".to_string(),
        max_chains: 50,
        default_timeout_seconds: 1800,
        whale_threshold: 1_000_000,
        emergency_pause_enabled: true,
        security_config: moby_bridge::system::SecurityConfig::default(),
        liquidity_config: moby_bridge::system::LiquidityConfig::default(),
        monitoring_config: moby_bridge::system::MonitoringConfig {
            health_check_interval: 5, // Check every 5 seconds for demo
            metrics_enabled: true,
            alert_thresholds: AlertThresholds {
                high_failure_rate: 0.05,     // 5% failure rate triggers alert
                high_latency_seconds: 300,   // 5 minutes max processing time
                low_liquidity_threshold: 50_000, // $50k minimum liquidity
                high_fraud_score: 0.7,       // 70% fraud score threshold
            },
            log_level: "debug".to_string(),
        },
    };

    bridge.initialize(monitoring_config).await?;

    // Set up event monitoring
    let event_handler = MonitoringEventHandler::new();
    bridge.add_event_handler(Box::new(event_handler)).await;

    println!("✅ Bridge monitoring system initialized\n");

    // Start health monitoring dashboard
    println!("🏥 Starting Health Monitoring Dashboard");
    println!("======================================\n");

    let bridge_clone = bridge.clone();
    let health_monitor = tokio::spawn(async move {
        let mut health_interval = interval(TokioDuration::from_secs(10));
        let mut iteration = 0;

        loop {
            health_interval.tick().await;
            iteration += 1;

            if let Ok(health) = bridge_clone.get_health_status().await {
                print_health_dashboard(&health, iteration).await;
            }

            // Stop after 30 iterations (5 minutes of monitoring)
            if iteration >= 30 {
                break;
            }
        }
    });

    // Start metrics collection
    let bridge_clone = bridge.clone();
    let metrics_collector = tokio::spawn(async move {
        let mut metrics_interval = interval(TokioDuration::from_secs(15));
        let mut metrics_history = Vec::new();

        loop {
            metrics_interval.tick().await;

            let metrics = bridge_clone.get_system_metrics().await;
            metrics_history.push((Utc::now(), metrics.clone()));

            // Keep only last 20 entries
            if metrics_history.len() > 20 {
                metrics_history.remove(0);
            }

            print_metrics_summary(&metrics, &metrics_history).await;

            // Stop collecting after 20 iterations
            if metrics_history.len() >= 20 {
                break;
            }
        }
    });

    // Generate test traffic for monitoring
    println!("🚀 Generating Test Traffic for Monitoring");
    println!("========================================\n");

    let test_scenarios = vec![
        // Normal transfers
        ("Normal ETH->SOL", create_test_transfer("ethereum", "solana", 1_000_000, TransferPriority::Normal)),
        ("Normal POL->BSC", create_test_transfer("polygon", "bsc", 5_000_000, TransferPriority::Normal)),

        // High priority transfers
        ("High ETH->AVAX", create_test_transfer("ethereum", "avalanche", 10_000_000, TransferPriority::High)),
        ("Urgent SOL->POL", create_test_transfer("solana", "polygon", 2_000_000, TransferPriority::Urgent)),

        // Whale transfers
        ("Whale ETH->BSC", create_test_transfer("ethereum", "bsc", 50_000_000, TransferPriority::Whale)),

        // Potentially problematic transfers (for monitoring demo)
        ("Large SOL->ETH", create_test_transfer("solana", "ethereum", 100_000_000, TransferPriority::Normal)),
    ];

    let mut test_transfer_ids = Vec::new();

    // Submit test transfers with delays
    for (name, transfer) in test_scenarios {
        println!("📝 Creating test transfer: {}", name);

        match bridge.initiate_transfer(transfer).await {
            Ok(id) => {
                test_transfer_ids.push((name, id));
                println!("   ✅ Transfer {} initiated", id);
            }
            Err(e) => {
                println!("   ❌ Transfer failed: {}", e);
            }
        }

        // Delay between transfers to spread out load
        sleep(TokioDuration::from_secs(2)).await;
    }

    println!("\n📈 Monitoring Transfer Progress");
    println!("==============================\n");

    // Monitor transfer progress
    let bridge_clone = bridge.clone();
    let transfer_monitor = tokio::spawn(async move {
        let mut monitoring_rounds = 0;

        while monitoring_rounds < 20 {
            let all_transfers = bridge_clone.list_transfers().await;

            let mut status_counts = HashMap::new();
            let mut total_volume = 0u64;
            let mut completed_count = 0;
            let mut failed_count = 0;

            for transfer in &all_transfers {
                // Count by status
                let status_key = format!("{:?}", transfer.status).split(' ').next().unwrap_or("Unknown").to_string();
                *status_counts.entry(status_key).or_insert(0) += 1;

                total_volume += transfer.request.amount;

                match transfer.status {
                    TransferStatus::Completed { .. } => completed_count += 1,
                    TransferStatus::Failed { .. } => failed_count += 1,
                    _ => {}
                }
            }

            println!("📊 Transfer Status Summary (Round {}):", monitoring_rounds + 1);
            for (status, count) in &status_counts {
                println!("   {}: {} transfers", status, count);
            }

            if all_transfers.len() > 0 {
                let success_rate = completed_count as f32 / all_transfers.len() as f32 * 100.0;
                let failure_rate = failed_count as f32 / all_transfers.len() as f32 * 100.0;

                println!("   Success Rate: {:.1}%", success_rate);
                println!("   Failure Rate: {:.1}%", failure_rate);
                println!("   Total Volume: ${:.2}M", total_volume as f64 / 1_000_000.0);
            }

            println!();

            monitoring_rounds += 1;
            sleep(TokioDuration::from_secs(8)).await;
        }
    });

    // Simulate emergency scenarios for monitoring
    let bridge_clone = bridge.clone();
    let emergency_simulator = tokio::spawn(async move {
        // Wait 1 minute before simulating issues
        sleep(TokioDuration::from_secs(60)).await;

        println!("🚨 Simulating Emergency Scenarios");
        println!("=================================\n");

        // Simulate high volume of transfers (stress test)
        println!("📈 Stress Test: High Volume Transfers");
        for i in 0..10 {
            let stress_transfer = create_test_transfer(
                "ethereum",
                "solana",
                1_000_000 + i * 100_000,
                TransferPriority::High,
            );

            if let Err(e) = bridge_clone.initiate_transfer(stress_transfer).await {
                println!("   Stress transfer {} failed: {}", i, e);
            }

            sleep(TokioDuration::from_millis(200)).await;
        }

        println!("   ✅ Stress test completed\n");

        // Wait for system to process
        sleep(TokioDuration::from_secs(30)).await;

        println!("📊 Post-Stress Test Health Check");
        if let Ok(health) = bridge_clone.get_health_status().await {
            print_detailed_health_report(&health).await;
        }
    });

    // Wait for all monitoring tasks to complete
    println!("⏳ Running monitoring for 5 minutes...\n");

    tokio::try_join!(health_monitor, metrics_collector, transfer_monitor, emergency_simulator)?;

    // Final system analysis
    println!("\n📋 Final System Analysis");
    println!("=======================\n");

    let final_health = bridge.get_health_status().await?;
    let final_metrics = bridge.get_system_metrics().await;

    print_final_monitoring_report(&final_health, &final_metrics).await;

    // Display monitoring recommendations
    println!("\n💡 Monitoring Recommendations");
    println!("============================\n");

    print_monitoring_recommendations(&final_health, &final_metrics).await;

    println!("\n🎯 Bridge monitoring example completed!");
    println!("   This example demonstrated:");
    println!("   • Real-time health monitoring");
    println!("   • Metrics collection and analysis");
    println!("   • Event-driven alerting");
    println!("   • Transfer progress tracking");
    println!("   • Emergency scenario simulation");
    println!("   • Performance analysis and recommendations");

    Ok(())
}

/// Create a test transfer request
fn create_test_transfer(
    from_chain: &str,
    to_chain: &str,
    amount: u64,
    priority: TransferPriority,
) -> TransferRequest {
    TransferRequest {
        from_chain: from_chain.to_string(),
        to_chain: to_chain.to_string(),
        token: "USDC".to_string(),
        amount,
        recipient: format!("test_recipient_{}_{}", to_chain, amount),
        sender: format!("test_sender_{}_{}", from_chain, amount),
        privacy_level: PrivacyLevel::Public,
        priority,
        deadline: Some(Utc::now() + Duration::minutes(30)),
        metadata: HashMap::from([
            ("test_type".to_string(), "monitoring_demo".to_string()),
            ("priority".to_string(), format!("{:?}", priority)),
        ]),
    }
}

/// Print health dashboard
async fn print_health_dashboard(health: &HealthStatus, iteration: u32) {
    println!("🏥 Health Dashboard - Update #{}", iteration);
    println!("   Overall Status: {:?}", health.overall_status);

    let status_emoji = match health.overall_status {
        SystemStatus::Healthy => "🟢",
        SystemStatus::Degraded => "🟡",
        SystemStatus::Paused => "🟠",
        SystemStatus::Critical => "🔴",
    };

    println!("   {} System Health: {:.1}%", status_emoji,
             if health.overall_status == SystemStatus::Healthy { 100.0 } else { 75.0 });
    println!("   Active Chains: {}", health.chain_count);
    println!("   Active Transfers: {}", health.active_transfers);
    println!("   Success Rate: {:.1}%", health.success_rate * 100.0);
    println!("   Avg Processing: {}s", health.avg_processing_time);
    println!("   Emergency Paused: {}", if health.emergency_paused { "🔴 YES" } else { "🟢 NO" });
    println!("   Last Updated: {}\n", health.last_updated.format("%H:%M:%S"));
}

/// Print metrics summary
async fn print_metrics_summary(
    metrics: &moby_bridge::system::SystemMetrics,
    history: &[(chrono::DateTime<Utc>, moby_bridge::system::SystemMetrics)],
) {
    println!("📊 Metrics Summary");
    println!("   Active Chains: {}", metrics.active_chains);
    println!("   Active Transfers: {}", metrics.active_transfers);
    println!("   Success Rate: {:.2}%", metrics.success_rate * 100.0);
    println!("   Error Rate: {:.2}%", metrics.error_rate * 100.0);
    println!("   Daily Volume: ${:.2}K", metrics.daily_volume.to_f64().unwrap_or(0.0) / 1000.0);
    println!("   Uptime: {:.1}%", metrics.uptime_percentage);

    if history.len() > 1 {
        let current = &history[history.len() - 1].1;
        let previous = &history[history.len() - 2].1;

        let volume_change = current.daily_volume - previous.daily_volume;
        let transfer_change = current.active_transfers as i32 - previous.active_transfers as i32;

        println!("   📈 Volume Change: ${:.2}K", volume_change.to_f64().unwrap_or(0.0) / 1000.0);
        println!("   📊 Transfer Change: {:+}", transfer_change);
    }

    println!();
}

/// Print detailed health report
async fn print_detailed_health_report(health: &HealthStatus) {
    println!("🔍 Detailed Health Report");
    println!("   System Status: {:?}", health.overall_status);
    println!("   Active Chains: {}", health.chain_count);
    println!("   Active Transfers: {}", health.active_transfers);
    println!("   Daily Volume: ${:.2}M", health.daily_volume.to_f64().unwrap_or(0.0) / 1_000_000.0);
    println!("   Success Rate: {:.2}%", health.success_rate * 100.0);
    println!("   Avg Processing Time: {} seconds", health.avg_processing_time);
    println!("   Emergency Status: {}", if health.emergency_paused { "PAUSED" } else { "NORMAL" });
    println!();
}

/// Print final monitoring report
async fn print_final_monitoring_report(
    health: &HealthStatus,
    metrics: &moby_bridge::system::SystemMetrics,
) {
    println!("📋 Final Monitoring Report");
    println!("   Overall System Health: {:?}", health.overall_status);
    println!("   Total Transfers Processed: Estimated based on metrics");
    println!("   Final Success Rate: {:.2}%", metrics.success_rate * 100.0);
    println!("   Final Error Rate: {:.2}%", metrics.error_rate * 100.0);
    println!("   Total Volume Processed: ${:.2}M", metrics.total_volume.to_f64().unwrap_or(0.0) / 1_000_000.0);
    println!("   Average Processing Time: {} seconds", metrics.avg_processing_time_seconds);
    println!("   System Uptime: {:.1}%", metrics.uptime_percentage);

    // System status assessment
    println!("\n   System Assessment:");
    match health.overall_status {
        SystemStatus::Healthy => println!("   ✅ System is operating normally"),
        SystemStatus::Degraded => println!("   ⚠️  System performance is degraded"),
        SystemStatus::Paused => println!("   🛑 System is paused"),
        SystemStatus::Critical => println!("   🚨 System requires immediate attention"),
    }
}

/// Print monitoring recommendations
async fn print_monitoring_recommendations(
    health: &HealthStatus,
    metrics: &moby_bridge::system::SystemMetrics,
) {
    println!("💡 Monitoring Recommendations:");

    if health.success_rate < 0.95 {
        println!("   🔧 Consider investigating low success rate ({:.1}%)", health.success_rate * 100.0);
    }

    if health.avg_processing_time > 600 {
        println!("   ⚡ Processing time is high ({}s) - optimize routing", health.avg_processing_time);
    }

    if metrics.error_rate > 0.05 {
        println!("   🚨 Error rate is elevated ({:.2}%) - review security settings", metrics.error_rate * 100.0);
    }

    if health.active_transfers > 100 {
        println!("   📈 High transfer volume ({}) - consider scaling", health.active_transfers);
    }

    println!("   ✅ Regular health checks every 30 seconds recommended");
    println!("   ✅ Set up automated alerting for critical metrics");
    println!("   ✅ Monitor liquidity levels across all chains");
    println!("   ✅ Track whale transfer patterns for optimization");
    println!("   ✅ Implement circuit breakers for emergency situations");
}