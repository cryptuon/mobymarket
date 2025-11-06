//! Integration tests for the Moby Bridge system.
//!
//! These tests verify the end-to-end functionality of the bridge system
//! including cross-chain transfers, security validation, liquidity routing,
//! and system health monitoring.

use moby_bridge::{
    BridgeSystem, BridgeConfig, TransferRequest, PrivacyLevel, TransferPriority,
    chains::{ChainId, ChainConfig, ChainType, ChainStatus, implementations},
    security::{SecurityLevel, FraudDetector},
    liquidity::{LiquidityManager, LiquidityPool, PoolConfig},
    system::{SystemStatus, TransferStatus},
    error::BridgeError,
};
use std::collections::HashMap;
use tokio_test;
use chrono::Utc;
use rust_decimal::Decimal;

/// Test basic bridge system initialization
#[tokio::test]
async fn test_bridge_system_initialization() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");

    let config = BridgeConfig::default();
    bridge.initialize(config).await.expect("Failed to initialize bridge");

    let health = bridge.get_health_status().await.expect("Failed to get health status");
    assert_eq!(health.overall_status, SystemStatus::Healthy);
    assert!(!health.emergency_paused);
}

/// Test whale transfer with enhanced security
#[tokio::test]
async fn test_whale_transfer_flow() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");
    bridge.initialize(BridgeConfig::default()).await.expect("Failed to initialize");

    // Create a whale transfer request (> $1M threshold)
    let whale_request = TransferRequest {
        from_chain: "ethereum".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: 5_000_000_000_000, // $5M (assuming 6 decimals for USDC)
        recipient: "whale_recipient_solana_address".to_string(),
        sender: "whale_sender_ethereum_address".to_string(),
        privacy_level: PrivacyLevel::Enhanced,
        priority: TransferPriority::Whale,
        deadline: Some(Utc::now() + chrono::Duration::minutes(10)),
        metadata: HashMap::from([
            ("whale_verified".to_string(), "true".to_string()),
            ("institution".to_string(), "hedge_fund_alpha".to_string()),
        ]),
    };

    let transfer_id = bridge.initiate_transfer(whale_request).await
        .expect("Failed to initiate whale transfer");

    // Verify transfer was created
    let transfer_info = bridge.get_transfer(&transfer_id).await
        .expect("Failed to get transfer info");

    assert_eq!(transfer_info.request.priority, TransferPriority::Whale);
    assert_eq!(transfer_info.request.privacy_level, PrivacyLevel::Enhanced);
    assert!(transfer_info.request.amount > 1_000_000); // Above whale threshold

    // Wait a moment for processing to begin
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check that transfer is being processed
    let updated_info = bridge.get_transfer(&transfer_id).await
        .expect("Failed to get updated transfer info");

    // Transfer should be in validation or routing stage
    assert!(matches!(
        updated_info.status,
        TransferStatus::Pending | TransferStatus::Validating | TransferStatus::Routing
    ));
}

/// Test high-frequency transfer detection and throttling
#[tokio::test]
async fn test_high_frequency_transfer_detection() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");
    bridge.initialize(BridgeConfig::default()).await.expect("Failed to initialize");

    let base_request = TransferRequest {
        from_chain: "ethereum".to_string(),
        to_chain: "polygon".to_string(),
        token: "USDT".to_string(),
        amount: 10_000_000_000, // $10k
        recipient: "frequent_trader_polygon".to_string(),
        sender: "frequent_trader_ethereum".to_string(),
        privacy_level: PrivacyLevel::Public,
        priority: TransferPriority::High,
        deadline: None,
        metadata: HashMap::new(),
    };

    let mut transfer_ids = Vec::new();

    // Submit multiple rapid transfers
    for i in 0..5 {
        let mut request = base_request.clone();
        request.amount += i * 1000; // Slight variation to avoid exact duplicates

        let result = bridge.initiate_transfer(request).await;
        match result {
            Ok(id) => transfer_ids.push(id),
            Err(e) => {
                // Some transfers might be rejected due to fraud detection
                println!("Transfer {} rejected: {}", i, e);
            }
        }

        // Small delay between transfers
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // At least the first few transfers should succeed
    assert!(!transfer_ids.is_empty(), "No transfers succeeded");
    assert!(transfer_ids.len() >= 2, "Too few transfers succeeded");

    // Check transfer statuses
    for transfer_id in &transfer_ids {
        let info = bridge.get_transfer(transfer_id).await
            .expect("Failed to get transfer info");

        // All valid transfers should be processing
        assert!(!matches!(info.status, TransferStatus::Failed { .. }));
    }
}

/// Test cross-chain route optimization
#[tokio::test]
async fn test_cross_chain_route_optimization() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");

    // Initialize with route optimization enabled
    let mut config = BridgeConfig::default();
    config.liquidity_config.route_optimization_enabled = true;
    config.liquidity_config.whale_pools_enabled = true;

    bridge.initialize(config).await.expect("Failed to initialize");

    // Test different optimization scenarios
    let optimization_scenarios = vec![
        // Cost-optimized transfer
        TransferRequest {
            from_chain: "ethereum".to_string(),
            to_chain: "bsc".to_string(),
            token: "BUSD".to_string(),
            amount: 50_000_000_000, // $50k
            recipient: "cost_optimizer_bsc".to_string(),
            sender: "cost_optimizer_ethereum".to_string(),
            privacy_level: PrivacyLevel::Public,
            priority: TransferPriority::Low, // Cost-optimized
            deadline: Some(Utc::now() + chrono::Duration::hours(2)),
            metadata: HashMap::from([("optimization".to_string(), "cost".to_string())]),
        },
        // Speed-optimized transfer
        TransferRequest {
            from_chain: "polygon".to_string(),
            to_chain: "avalanche".to_string(),
            token: "AVAX".to_string(),
            amount: 100_000_000_000, // $100k
            recipient: "speed_optimizer_avalanche".to_string(),
            sender: "speed_optimizer_polygon".to_string(),
            privacy_level: PrivacyLevel::Confidential,
            priority: TransferPriority::Urgent, // Speed-optimized
            deadline: Some(Utc::now() + chrono::Duration::minutes(5)),
            metadata: HashMap::from([("optimization".to_string(), "speed".to_string())]),
        },
    ];

    for (i, scenario) in optimization_scenarios.into_iter().enumerate() {
        let transfer_id = bridge.initiate_transfer(scenario).await
            .expect(&format!("Failed to initiate optimization scenario {}", i));

        let transfer_info = bridge.get_transfer(&transfer_id).await
            .expect("Failed to get transfer info");

        // Verify transfer was accepted and is being processed
        assert!(matches!(
            transfer_info.status,
            TransferStatus::Pending | TransferStatus::Validating | TransferStatus::Routing
        ));

        println!("Optimization scenario {} initiated: {}", i, transfer_id);
    }
}

/// Test emergency pause and recovery
#[tokio::test]
async fn test_emergency_pause_recovery() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");

    let mut config = BridgeConfig::default();
    config.emergency_pause_enabled = true;
    bridge.initialize(config).await.expect("Failed to initialize");

    // Create a normal transfer
    let normal_request = TransferRequest {
        from_chain: "ethereum".to_string(),
        to_chain: "solana".to_string(),
        token: "ETH".to_string(),
        amount: 1_000_000_000_000, // $1k (assuming 18 decimals)
        recipient: "normal_user_solana".to_string(),
        sender: "normal_user_ethereum".to_string(),
        privacy_level: PrivacyLevel::Public,
        priority: TransferPriority::Normal,
        deadline: None,
        metadata: HashMap::new(),
    };

    // This transfer should succeed initially
    let transfer_id = bridge.initiate_transfer(normal_request.clone()).await
        .expect("Failed to initiate normal transfer");

    let transfer_info = bridge.get_transfer(&transfer_id).await
        .expect("Failed to get transfer info");
    assert!(matches!(transfer_info.status, TransferStatus::Pending));

    // TODO: Test emergency pause activation
    // This would require implementing emergency controls in the test
    // For now, we verify the system accepts transfers normally

    let health = bridge.get_health_status().await.expect("Failed to get health");
    assert_eq!(health.overall_status, SystemStatus::Healthy);
    assert!(!health.emergency_paused);
}

/// Test privacy-preserving transfers
#[tokio::test]
async fn test_privacy_preserving_transfers() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");
    bridge.initialize(BridgeConfig::default()).await.expect("Failed to initialize");

    let privacy_scenarios = vec![
        (PrivacyLevel::Public, "public_transfer"),
        (PrivacyLevel::Confidential, "confidential_amounts"),
        (PrivacyLevel::Anonymous, "anonymous_participants"),
        (PrivacyLevel::Enhanced, "full_privacy_zk"),
    ];

    for (privacy_level, scenario_name) in privacy_scenarios {
        let request = TransferRequest {
            from_chain: "ethereum".to_string(),
            to_chain: "solana".to_string(),
            token: "USDC".to_string(),
            amount: 25_000_000_000, // $25k
            recipient: format!("privacy_test_{}_solana", scenario_name),
            sender: format!("privacy_test_{}_ethereum", scenario_name),
            privacy_level: privacy_level.clone(),
            priority: TransferPriority::Normal,
            deadline: None,
            metadata: HashMap::from([
                ("privacy_test".to_string(), scenario_name.to_string()),
                ("compliance_verified".to_string(), "true".to_string()),
            ]),
        };

        let transfer_id = bridge.initiate_transfer(request).await
            .expect(&format!("Failed to initiate {} transfer", scenario_name));

        let transfer_info = bridge.get_transfer(&transfer_id).await
            .expect("Failed to get transfer info");

        assert_eq!(transfer_info.request.privacy_level, privacy_level);
        assert!(matches!(transfer_info.status, TransferStatus::Pending));

        println!("Privacy scenario {} initiated: {}", scenario_name, transfer_id);
    }
}

/// Test system health monitoring and metrics
#[tokio::test]
async fn test_system_health_monitoring() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");

    let mut config = BridgeConfig::default();
    config.monitoring_config.metrics_enabled = true;
    config.monitoring_config.health_check_interval = 1; // 1 second for testing

    bridge.initialize(config).await.expect("Failed to initialize");

    // Get initial health status
    let initial_health = bridge.get_health_status().await
        .expect("Failed to get initial health");

    assert_eq!(initial_health.overall_status, SystemStatus::Healthy);
    assert_eq!(initial_health.active_transfers, 0);

    // Create some transfers to change metrics
    let transfer_requests = vec![
        TransferRequest {
            from_chain: "ethereum".to_string(),
            to_chain: "polygon".to_string(),
            token: "MATIC".to_string(),
            amount: 1_000_000_000, // $1k
            recipient: "metrics_test_1_polygon".to_string(),
            sender: "metrics_test_1_ethereum".to_string(),
            privacy_level: PrivacyLevel::Public,
            priority: TransferPriority::Normal,
            deadline: None,
            metadata: HashMap::new(),
        },
        TransferRequest {
            from_chain: "solana".to_string(),
            to_chain: "avalanche".to_string(),
            token: "SOL".to_string(),
            amount: 5_000_000_000, // $5k
            recipient: "metrics_test_2_avalanche".to_string(),
            sender: "metrics_test_2_solana".to_string(),
            privacy_level: PrivacyLevel::Confidential,
            priority: TransferPriority::High,
            deadline: None,
            metadata: HashMap::new(),
        },
    ];

    let mut transfer_ids = Vec::new();
    for request in transfer_requests {
        let id = bridge.initiate_transfer(request).await
            .expect("Failed to initiate transfer for metrics test");
        transfer_ids.push(id);
    }

    // Wait for health check to update
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

    // Get updated health status
    let updated_health = bridge.get_health_status().await
        .expect("Failed to get updated health");

    // Should now show active transfers
    assert!(updated_health.active_transfers > 0);
    assert_eq!(updated_health.overall_status, SystemStatus::Healthy);

    // Get system metrics
    let metrics = bridge.get_system_metrics().await;
    assert!(metrics.avg_processing_time_seconds > 0);
    assert_eq!(metrics.uptime_percentage, 100.0);

    println!("Health check completed. Active transfers: {}", updated_health.active_transfers);
    println!("System metrics - Success rate: {}, Error rate: {}",
             metrics.success_rate, metrics.error_rate);
}

/// Test transfer cancellation scenarios
#[tokio::test]
async fn test_transfer_cancellation_scenarios() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");
    bridge.initialize(BridgeConfig::default()).await.expect("Failed to initialize");

    // Create a transfer that can be cancelled
    let cancellable_request = TransferRequest {
        from_chain: "ethereum".to_string(),
        to_chain: "bsc".to_string(),
        token: "BNB".to_string(),
        amount: 10_000_000_000, // $10k
        recipient: "cancellation_test_bsc".to_string(),
        sender: "cancellation_test_ethereum".to_string(),
        privacy_level: PrivacyLevel::Public,
        priority: TransferPriority::Normal,
        deadline: Some(Utc::now() + chrono::Duration::hours(1)),
        metadata: HashMap::from([("test_type".to_string(), "cancellation".to_string())]),
    };

    let transfer_id = bridge.initiate_transfer(cancellable_request).await
        .expect("Failed to initiate cancellable transfer");

    // Verify transfer exists and is pending
    let transfer_info = bridge.get_transfer(&transfer_id).await
        .expect("Failed to get transfer info");
    assert_eq!(transfer_info.status, TransferStatus::Pending);

    // Cancel the transfer
    let cancellation_result = bridge.cancel_transfer(
        &transfer_id,
        "User requested cancellation for testing".to_string()
    ).await;

    assert!(cancellation_result.is_ok(), "Failed to cancel transfer");

    // Verify transfer is now cancelled
    let cancelled_info = bridge.get_transfer(&transfer_id).await
        .expect("Failed to get cancelled transfer info");

    match cancelled_info.status {
        TransferStatus::Cancelled { reason, .. } => {
            assert!(reason.contains("User requested cancellation"));
        },
        other => panic!("Expected cancelled status, got {:?}", other),
    }

    // Test cancelling non-existent transfer
    let fake_id = "non-existent-transfer-id";
    let fake_cancel_result = bridge.cancel_transfer(fake_id, "Test".to_string()).await;

    assert!(fake_cancel_result.is_err());
    match fake_cancel_result.unwrap_err() {
        BridgeError::TransferNotFound { transfer_id } => {
            assert_eq!(transfer_id, fake_id);
        },
        other => panic!("Expected TransferNotFound error, got {:?}", other),
    }
}

/// Test multi-chain support and chain management
#[tokio::test]
async fn test_multi_chain_support() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");
    bridge.initialize(BridgeConfig::default()).await.expect("Failed to initialize");

    // Test transfers between different chain combinations
    let chain_combinations = vec![
        ("ethereum", "solana", "Cross-VM transfer"),
        ("polygon", "bsc", "EVM to EVM transfer"),
        ("avalanche", "ethereum", "L1 to L1 transfer"),
        ("solana", "polygon", "Solana to EVM transfer"),
    ];

    for (source, dest, description) in chain_combinations {
        let request = TransferRequest {
            from_chain: source.to_string(),
            to_chain: dest.to_string(),
            token: "USDC".to_string(),
            amount: 1_000_000_000, // $1k
            recipient: format!("multichain_test_{}_{}", dest, source),
            sender: format!("multichain_test_{}_{}", source, dest),
            privacy_level: PrivacyLevel::Public,
            priority: TransferPriority::Normal,
            deadline: None,
            metadata: HashMap::from([
                ("test_type".to_string(), "multi_chain".to_string()),
                ("description".to_string(), description.to_string()),
            ]),
        };

        let transfer_id = bridge.initiate_transfer(request).await
            .expect(&format!("Failed to initiate {} transfer", description));

        let transfer_info = bridge.get_transfer(&transfer_id).await
            .expect("Failed to get transfer info");

        assert_eq!(transfer_info.request.from_chain, source);
        assert_eq!(transfer_info.request.to_chain, dest);
        assert!(matches!(transfer_info.status, TransferStatus::Pending));

        println!("{}: {} -> {} (ID: {})", description, source, dest, transfer_id);
    }
}

/// Test fraud detection and security validation
#[tokio::test]
async fn test_fraud_detection_security() {
    let bridge = BridgeSystem::new().await.expect("Failed to create bridge system");

    let mut config = BridgeConfig::default();
    config.security_config.fraud_detection_enabled = true;
    config.security_config.compliance_enabled = true;

    bridge.initialize(config).await.expect("Failed to initialize");

    // Test potentially suspicious transfer patterns
    let suspicious_scenarios = vec![
        // Very large amount (potential fraud)
        TransferRequest {
            from_chain: "ethereum".to_string(),
            to_chain: "solana".to_string(),
            token: "USDC".to_string(),
            amount: 100_000_000_000_000, // $100M - very large
            recipient: "suspicious_large_amount_solana".to_string(),
            sender: "suspicious_large_amount_ethereum".to_string(),
            privacy_level: PrivacyLevel::Anonymous,
            priority: TransferPriority::Urgent,
            deadline: Some(Utc::now() + chrono::Duration::minutes(1)),
            metadata: HashMap::from([("risk_level".to_string(), "high".to_string())]),
        },
        // Normal transfer for comparison
        TransferRequest {
            from_chain: "polygon".to_string(),
            to_chain: "bsc".to_string(),
            token: "USDT".to_string(),
            amount: 5_000_000_000, // $5k - normal amount
            recipient: "normal_transfer_bsc".to_string(),
            sender: "normal_transfer_polygon".to_string(),
            privacy_level: PrivacyLevel::Public,
            priority: TransferPriority::Normal,
            deadline: None,
            metadata: HashMap::from([("risk_level".to_string(), "low".to_string())]),
        },
    ];

    for (i, scenario) in suspicious_scenarios.into_iter().enumerate() {
        let result = bridge.initiate_transfer(scenario.clone()).await;

        match result {
            Ok(transfer_id) => {
                println!("Scenario {} accepted: {}", i, transfer_id);

                let transfer_info = bridge.get_transfer(&transfer_id).await
                    .expect("Failed to get transfer info");

                // Even if accepted, it should be under validation
                assert!(matches!(
                    transfer_info.status,
                    TransferStatus::Pending | TransferStatus::Validating
                ));
            },
            Err(e) => {
                println!("Scenario {} rejected by fraud detection: {}", i, e);

                // High-risk transfers might be rejected
                assert!(scenario.amount > 50_000_000_000_000 ||
                        scenario.privacy_level == PrivacyLevel::Anonymous);
            }
        }
    }
}

/// Integration test helper functions
mod test_helpers {
    use super::*;

    /// Create a test bridge with standard configuration
    pub async fn create_test_bridge() -> BridgeSystem {
        let bridge = BridgeSystem::new().await.expect("Failed to create test bridge");

        let config = BridgeConfig {
            version: "test-0.1.0".to_string(),
            max_chains: 10,
            default_timeout_seconds: 300, // 5 minutes for tests
            whale_threshold: 1_000_000,
            emergency_pause_enabled: true,
            security_config: crate::security::SecurityConfig {
                default_security_level: SecurityLevel::Enhanced,
                fraud_detection_enabled: true,
                multisig_enabled: false, // Simplified for tests
                compliance_enabled: true,
                emergency_controls_enabled: true,
            },
            liquidity_config: crate::system::LiquidityConfig {
                auto_rebalancing_enabled: false, // Disabled for deterministic tests
                route_optimization_enabled: true,
                whale_pools_enabled: true,
                dynamic_fees_enabled: false, // Fixed fees for tests
            },
            monitoring_config: crate::system::MonitoringConfig {
                health_check_interval: 1,
                metrics_enabled: true,
                alert_thresholds: crate::system::AlertThresholds::default(),
                log_level: "debug".to_string(),
            },
        };

        bridge.initialize(config).await.expect("Failed to initialize test bridge");
        bridge
    }

    /// Create a standard test transfer request
    pub fn create_test_transfer(
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
            deadline: Some(Utc::now() + chrono::Duration::minutes(30)),
            metadata: HashMap::from([
                ("test_transfer".to_string(), "true".to_string()),
                ("amount_class".to_string(), if amount > 1_000_000 { "whale" } else { "normal" }.to_string()),
            ]),
        }
    }
}

/// Comprehensive end-to-end system test
#[tokio::test]
async fn test_comprehensive_bridge_system() {
    let bridge = test_helpers::create_test_bridge().await;

    // Test multiple concurrent transfers with different characteristics
    let test_transfers = vec![
        test_helpers::create_test_transfer("ethereum", "solana", 500_000, TransferPriority::Low),
        test_helpers::create_test_transfer("polygon", "bsc", 2_000_000, TransferPriority::Whale),
        test_helpers::create_test_transfer("avalanche", "ethereum", 10_000, TransferPriority::Normal),
        test_helpers::create_test_transfer("solana", "polygon", 750_000, TransferPriority::High),
    ];

    let mut transfer_ids = Vec::new();

    // Submit all transfers
    for (i, transfer) in test_transfers.into_iter().enumerate() {
        match bridge.initiate_transfer(transfer).await {
            Ok(id) => {
                transfer_ids.push(id);
                println!("Transfer {} initiated successfully", i);
            },
            Err(e) => {
                println!("Transfer {} failed: {}", i, e);
            }
        }
    }

    assert!(!transfer_ids.is_empty(), "No transfers were initiated successfully");

    // Wait for processing to begin
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check status of all transfers
    for (i, transfer_id) in transfer_ids.iter().enumerate() {
        let info = bridge.get_transfer(transfer_id).await
            .expect(&format!("Failed to get info for transfer {}", i));

        println!("Transfer {}: {:?}", i, info.status);

        // All transfers should be in some stage of processing
        assert!(!matches!(info.status, TransferStatus::Failed { .. }));
    }

    // Test system health after load
    let final_health = bridge.get_health_status().await
        .expect("Failed to get final health status");

    println!("Final system health: {:?}", final_health.overall_status);
    println!("Active transfers: {}", final_health.active_transfers);

    // System should still be healthy
    assert!(matches!(
        final_health.overall_status,
        SystemStatus::Healthy | SystemStatus::Degraded // Degraded is acceptable under load
    ));

    assert!(final_health.active_transfers > 0);

    println!("Comprehensive bridge system test completed successfully!");
}