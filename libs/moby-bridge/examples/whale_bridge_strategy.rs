//! Example: Whale trading bridge strategy with optimized routing
//!
//! This example demonstrates how large whale traders can use the Moby Bridge
//! system for high-value cross-chain transfers with specialized routing,
//! enhanced security, and priority processing.

use moby_bridge::{
    BridgeSystem, BridgeConfig, TransferRequest, PrivacyLevel, TransferPriority,
    system::{TransferStatus, SystemStatus, SecurityConfig, LiquidityConfig},
    security::SecurityLevel,
    error::BridgeResult,
};
use std::collections::HashMap;
use chrono::Utc;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> BridgeResult<()> {
    println!("🐋 Moby Bridge - Whale Trading Strategy Example");
    println!("==============================================\n");

    // Initialize bridge system with whale-optimized configuration
    println!("🔧 Initializing whale-optimized bridge system...");
    let bridge = BridgeSystem::new().await?;

    // Configure bridge for whale trading
    let whale_config = BridgeConfig {
        version: "0.1.0".to_string(),
        max_chains: 50,
        default_timeout_seconds: 600, // 10 minutes for whale transfers
        whale_threshold: 1_000_000, // $1M threshold
        emergency_pause_enabled: true,
        security_config: SecurityConfig {
            default_security_level: SecurityLevel::Maximum,
            fraud_detection_enabled: true,
            multisig_enabled: true,
            compliance_enabled: true,
            emergency_controls_enabled: true,
        },
        liquidity_config: LiquidityConfig {
            auto_rebalancing_enabled: true,
            route_optimization_enabled: true,
            whale_pools_enabled: true,
            dynamic_fees_enabled: true,
        },
        monitoring_config: moby_bridge::system::MonitoringConfig {
            health_check_interval: 15, // More frequent monitoring
            metrics_enabled: true,
            alert_thresholds: moby_bridge::system::AlertThresholds {
                high_failure_rate: 0.01, // Lower tolerance for failures
                high_latency_seconds: 300, // 5 minutes max
                low_liquidity_threshold: 10_000_000, // $10M minimum
                high_fraud_score: 0.3, // More sensitive
            },
            log_level: "info".to_string(),
        },
    };

    bridge.initialize(whale_config).await?;
    println!("✅ Whale-optimized bridge system initialized\n");

    // Verify system is ready for whale operations
    let health = bridge.get_health_status().await?;
    println!("🏥 Whale Bridge Health Check:");
    println!("   Status: {:?}", health.overall_status);
    println!("   Active Chains: {}", health.chain_count);
    println!("   Success Rate: {:.2}%", health.success_rate * 100.0);
    println!("   Avg Processing Time: {} seconds", health.avg_processing_time);
    println!("   Emergency Paused: {}\n", health.emergency_paused);

    if health.overall_status != SystemStatus::Healthy {
        println!("⚠️  System not optimal for whale operations!");
        return Ok(());
    }

    // Demonstrate different whale trading scenarios
    println!("🎯 Whale Trading Scenarios");
    println!("==========================\n");

    // Scenario 1: Large institutional transfer with maximum privacy
    println!("📈 Scenario 1: Institutional Hedge Fund Transfer");
    println!("-----------------------------------------------");

    let institutional_transfer = TransferRequest {
        from_chain: "ethereum".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: 25_000_000_000_000, // $25M USDC
        recipient: "InstitutionalVaultSolana987654321".to_string(),
        sender: "0xInstitutionalVaultEthereum123456789".to_string(),
        privacy_level: PrivacyLevel::Enhanced, // Maximum privacy
        priority: TransferPriority::Whale,
        deadline: Some(Utc::now() + chrono::Duration::minutes(15)),
        metadata: HashMap::from([
            ("institution".to_string(), "Quantum Capital Management".to_string()),
            ("fund_type".to_string(), "hedge_fund".to_string()),
            ("compliance_id".to_string(), "QCM-2024-001".to_string()),
            ("risk_profile".to_string(), "conservative".to_string()),
            ("execution_strategy".to_string(), "privacy_focused".to_string()),
        ]),
    };

    println!("🏦 Institution: Quantum Capital Management");
    println!("💰 Amount: ${:.2}M", institutional_transfer.amount as f64 / 1_000_000_000_000.0);
    println!("🔒 Privacy: {:?}", institutional_transfer.privacy_level);
    println!("⚡ Priority: {:?}", institutional_transfer.priority);

    let institutional_id = bridge.initiate_transfer(institutional_transfer).await?;
    println!("✅ Institutional transfer initiated: {}\n", institutional_id);

    // Scenario 2: High-frequency whale arbitrage
    println!("📊 Scenario 2: Whale Arbitrage Opportunity");
    println!("------------------------------------------");

    let arbitrage_transfer = TransferRequest {
        from_chain: "polygon".to_string(),
        to_chain: "avalanche".to_string(),
        token: "AVAX".to_string(),
        amount: 5_000_000_000_000, // $5M AVAX
        recipient: "ArbitrageStrategyAvalanche456789".to_string(),
        sender: "0xArbitrageStrategyPolygon987654321".to_string(),
        privacy_level: PrivacyLevel::Confidential,
        priority: TransferPriority::Urgent, // Time-sensitive arbitrage
        deadline: Some(Utc::now() + chrono::Duration::minutes(3)), // Very tight deadline
        metadata: HashMap::from([
            ("strategy".to_string(), "cross_chain_arbitrage".to_string()),
            ("opportunity_id".to_string(), "ARB-POL-AVAX-240101".to_string()),
            ("expected_profit_bps".to_string(), "45".to_string()), // 0.45% profit
            ("max_slippage_bps".to_string(), "15".to_string()), // 0.15% max slippage
            ("execution_model".to_string(), "speed_optimized".to_string()),
        ]),
    };

    println!("⚡ Strategy: Cross-chain arbitrage");
    println!("💰 Amount: ${:.2}M", arbitrage_transfer.amount as f64 / 1_000_000_000_000.0);
    println!("⏱️  Deadline: {} minutes", 3);
    println!("📈 Expected Profit: 0.45%");

    let arbitrage_id = bridge.initiate_transfer(arbitrage_transfer).await?;
    println!("✅ Arbitrage transfer initiated: {}\n", arbitrage_id);

    // Scenario 3: Treasury diversification
    println!("🏛️  Scenario 3: Protocol Treasury Diversification");
    println!("------------------------------------------------");

    let treasury_transfer = TransferRequest {
        from_chain: "ethereum".to_string(),
        to_chain: "bsc".to_string(),
        token: "BNB".to_string(),
        amount: 15_000_000_000_000, // $15M BNB
        recipient: "ProtocolTreasuryBSC192837465".to_string(),
        sender: "0xProtocolTreasuryEthereum564738291".to_string(),
        privacy_level: PrivacyLevel::Public, // Transparent for governance
        priority: TransferPriority::High,
        deadline: Some(Utc::now() + chrono::Duration::hours(1)),
        metadata: HashMap::from([
            ("protocol".to_string(), "DeFi Protocol Alpha".to_string()),
            ("purpose".to_string(), "treasury_diversification".to_string()),
            ("governance_proposal".to_string(), "DPA-PROP-2024-15".to_string()),
            ("allocation_target".to_string(), "25_percent_bsc".to_string()),
            ("execution_model".to_string(), "cost_optimized".to_string()),
        ]),
    };

    println!("🏛️  Protocol: DeFi Protocol Alpha");
    println!("💰 Amount: ${:.2}M", treasury_transfer.amount as f64 / 1_000_000_000_000.0);
    println!("🎯 Purpose: Treasury diversification (25% to BSC)");
    println!("🗳️  Governance: DPA-PROP-2024-15");

    let treasury_id = bridge.initiate_transfer(treasury_transfer).await?;
    println!("✅ Treasury transfer initiated: {}\n", treasury_id);

    // Monitor all whale transfers
    println!("👀 Monitoring Whale Transfer Execution");
    println!("=====================================\n");

    let whale_transfers = vec![
        ("Institutional", institutional_id),
        ("Arbitrage", arbitrage_id),
        ("Treasury", treasury_id),
    ];

    // Track all transfers until completion or timeout
    let monitoring_start = Utc::now();
    let monitoring_timeout = chrono::Duration::minutes(10);
    let mut completed_transfers = Vec::new();

    while completed_transfers.len() < whale_transfers.len() {
        if Utc::now().signed_duration_since(monitoring_start) > monitoring_timeout {
            println!("⏱️  Monitoring timeout reached\n");
            break;
        }

        for (name, transfer_id) in &whale_transfers {
            if completed_transfers.contains(transfer_id) {
                continue;
            }

            let transfer_info = bridge.get_transfer(transfer_id).await?;

            match &transfer_info.status {
                TransferStatus::Completed { completed_at, tx_hash } => {
                    println!("🎉 {} Transfer Completed!", name);
                    println!("   Transfer ID: {}", transfer_id);
                    println!("   Completed: {}", completed_at);
                    println!("   TX Hash: {}", tx_hash);

                    if let Some(processing_time) = transfer_info.metrics.processing_time_seconds {
                        println!("   Processing Time: {} seconds", processing_time);
                    }

                    println!("   Total Fees: ${}", transfer_info.metrics.total_fees);

                    if let Some(route) = &transfer_info.route {
                        println!("   Route Cost: ${}", route.optimal_route.total_cost);
                        println!("   Route Reliability: {:.1}%",
                                route.optimal_route.reliability_score * 100.0);
                        println!("   Expected Slippage: {:.3}%",
                                route.optimal_route.expected_slippage * 100.0);
                    }

                    completed_transfers.push(transfer_id.clone());
                    println!();
                }
                TransferStatus::Failed { failed_at, reason, recoverable } => {
                    println!("❌ {} Transfer Failed!", name);
                    println!("   Transfer ID: {}", transfer_id);
                    println!("   Failed: {}", failed_at);
                    println!("   Reason: {}", reason);
                    println!("   Recoverable: {}", recoverable);

                    completed_transfers.push(transfer_id.clone());
                    println!();
                }
                TransferStatus::Executing => {
                    println!("⚡ {} transfer executing on blockchain...", name);
                }
                TransferStatus::Routing => {
                    println!("🗺️  {} transfer finding optimal route...", name);
                }
                TransferStatus::Validating => {
                    println!("🔍 {} transfer undergoing security validation...", name);
                }
                _ => {}
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    // Display whale trading analytics
    println!("📊 Whale Trading Analytics");
    println!("=========================\n");

    let system_metrics = bridge.get_system_metrics().await;

    println!("System Performance:");
    println!("   Active Transfers: {}", system_metrics.active_transfers);
    println!("   Success Rate: {:.2}%", system_metrics.success_rate * 100.0);
    println!("   Error Rate: {:.2}%", system_metrics.error_rate * 100.0);
    println!("   Avg Processing Time: {} seconds", system_metrics.avg_processing_time_seconds);
    println!("   Daily Volume: ${:.2}M", system_metrics.daily_volume.to_f64().unwrap_or(0.0) / 1_000_000.0);
    println!("   Total Volume: ${:.2}M", system_metrics.total_volume.to_f64().unwrap_or(0.0) / 1_000_000.0);

    // Calculate whale-specific metrics
    let mut total_whale_volume = 0.0;
    let mut total_whale_fees = Decimal::ZERO;
    let mut avg_whale_processing_time = 0.0;

    for (name, transfer_id) in &whale_transfers {
        if let Ok(info) = bridge.get_transfer(transfer_id).await {
            total_whale_volume += info.request.amount as f64 / 1_000_000_000_000.0; // Convert to millions
            total_whale_fees += info.metrics.total_fees;

            if let Some(processing_time) = info.metrics.processing_time_seconds {
                avg_whale_processing_time += processing_time as f64;
            }

            println!("\n{} Transfer Details:", name);
            println!("   Amount: ${:.2}M", info.request.amount as f64 / 1_000_000_000_000.0);
            println!("   Status: {:?}", info.status);
            println!("   Fees: ${}", info.metrics.total_fees);

            if let Some(processing_time) = info.metrics.processing_time_seconds {
                println!("   Processing Time: {} seconds", processing_time);
            }
        }
    }

    avg_whale_processing_time /= whale_transfers.len() as f64;

    println!("\nWhale Trading Summary:");
    println!("   Total Whale Volume: ${:.2}M", total_whale_volume);
    println!("   Total Whale Fees: ${}", total_whale_fees);
    println!("   Avg Whale Processing Time: {:.1} seconds", avg_whale_processing_time);
    println!("   Fee Rate: {:.4}%",
             (total_whale_fees.to_f64().unwrap_or(0.0) / (total_whale_volume * 1_000_000.0)) * 100.0);

    // Demonstrate whale-specific features
    println!("\n🐋 Whale Trading Features Demonstrated:");
    println!("   ✅ Enhanced privacy for institutional transfers");
    println!("   ✅ Ultra-fast execution for arbitrage opportunities");
    println!("   ✅ Cost-optimized routing for treasury operations");
    println!("   ✅ Maximum security validation for large amounts");
    println!("   ✅ Real-time monitoring and analytics");
    println!("   ✅ Compliance tracking and reporting");
    println!("   ✅ Priority processing for whale transactions");

    println!("\n🎯 Whale bridge strategy example completed!");
    println!("   This example showcased advanced features for:");
    println!("   • Institutional hedge fund transfers");
    println!("   • High-frequency arbitrage strategies");
    println!("   • Protocol treasury management");
    println!("   • Whale-optimized routing and security");

    Ok(())
}