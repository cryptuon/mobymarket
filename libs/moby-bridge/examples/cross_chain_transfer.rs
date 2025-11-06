//! Example: Basic cross-chain transfer using Moby Bridge
//!
//! This example demonstrates how to perform a simple cross-chain transfer
//! from Ethereum to Solana using the Moby Bridge system.

use moby_bridge::{
    BridgeSystem, BridgeConfig, TransferRequest, PrivacyLevel, TransferPriority,
    system::{TransferStatus, SystemStatus},
    error::BridgeResult,
};
use std::collections::HashMap;
use chrono::Utc;

#[tokio::main]
async fn main() -> BridgeResult<()> {
    println!("🌉 Moby Bridge - Cross-Chain Transfer Example");
    println!("============================================\n");

    // Initialize the bridge system
    println!("🔧 Initializing bridge system...");
    let bridge = BridgeSystem::new().await?;

    // Configure the bridge with default settings
    let config = BridgeConfig::default();
    bridge.initialize(config).await?;

    println!("✅ Bridge system initialized successfully\n");

    // Check system health before proceeding
    let health = bridge.get_health_status().await?;
    println!("🏥 System Health Check:");
    println!("   Status: {:?}", health.overall_status);
    println!("   Active Chains: {}", health.chain_count);
    println!("   Emergency Paused: {}\n", health.emergency_paused);

    if health.overall_status != SystemStatus::Healthy {
        println!("⚠️  System is not healthy, proceeding with caution...\n");
    }

    // Create a cross-chain transfer request
    println!("📝 Creating transfer request...");
    let transfer_request = TransferRequest {
        from_chain: "ethereum".to_string(),
        to_chain: "solana".to_string(),
        token: "USDC".to_string(),
        amount: 1_000_000_000, // $1,000 USDC (6 decimals)
        recipient: "DemoRecipientSolanaAddress123456789".to_string(),
        sender: "0xDemoSenderEthereumAddress123456789".to_string(),
        privacy_level: PrivacyLevel::Public,
        priority: TransferPriority::Normal,
        deadline: Some(Utc::now() + chrono::Duration::minutes(30)),
        metadata: HashMap::from([
            ("purpose".to_string(), "Demo transfer".to_string()),
            ("user_id".to_string(), "demo_user_001".to_string()),
        ]),
    };

    println!("✅ Transfer request created:");
    println!("   From: {} -> To: {}", transfer_request.from_chain, transfer_request.to_chain);
    println!("   Token: {}", transfer_request.token);
    println!("   Amount: ${}", transfer_request.amount as f64 / 1_000_000.0);
    println!("   Privacy: {:?}", transfer_request.privacy_level);
    println!("   Priority: {:?}\n", transfer_request.priority);

    // Initiate the transfer
    println!("🚀 Initiating cross-chain transfer...");
    let transfer_id = bridge.initiate_transfer(transfer_request).await?;

    println!("✅ Transfer initiated successfully!");
    println!("   Transfer ID: {}\n", transfer_id);

    // Monitor transfer progress
    println!("👀 Monitoring transfer progress...");
    let mut last_status = TransferStatus::Pending;
    let start_time = Utc::now();
    let timeout = chrono::Duration::minutes(5);

    loop {
        // Check if we've exceeded timeout
        if Utc::now().signed_duration_since(start_time) > timeout {
            println!("⏱️  Transfer monitoring timeout reached");
            break;
        }

        // Get current transfer status
        let transfer_info = bridge.get_transfer(&transfer_id).await?;

        // Only print status updates when status changes
        if std::mem::discriminant(&transfer_info.status) != std::mem::discriminant(&last_status) {
            match &transfer_info.status {
                TransferStatus::Pending => {
                    println!("📋 Transfer status: Pending - Queued for processing");
                }
                TransferStatus::Validating => {
                    println!("🔍 Transfer status: Validating - Security and fraud checks in progress");
                }
                TransferStatus::Routing => {
                    println!("🗺️  Transfer status: Routing - Finding optimal path");
                }
                TransferStatus::Executing => {
                    println!("⚡ Transfer status: Executing - Processing on blockchain");
                }
                TransferStatus::Completed { completed_at, tx_hash } => {
                    println!("🎉 Transfer status: Completed!");
                    println!("   Completed at: {}", completed_at);
                    println!("   Transaction hash: {}", tx_hash);

                    // Display transfer metrics
                    if let Some(processing_time) = transfer_info.metrics.processing_time_seconds {
                        println!("   Total processing time: {} seconds", processing_time);
                    }
                    println!("   Total fees: ${}", transfer_info.metrics.total_fees);
                    println!("   Retries: {}", transfer_info.metrics.retry_count);
                    break;
                }
                TransferStatus::Failed { failed_at, reason, recoverable } => {
                    println!("❌ Transfer status: Failed");
                    println!("   Failed at: {}", failed_at);
                    println!("   Reason: {}", reason);
                    println!("   Recoverable: {}", recoverable);

                    if *recoverable {
                        println!("💡 This transfer may be retryable");
                    }
                    break;
                }
                TransferStatus::Cancelled { cancelled_at, reason } => {
                    println!("🚫 Transfer status: Cancelled");
                    println!("   Cancelled at: {}", cancelled_at);
                    println!("   Reason: {}", reason);
                    break;
                }
            }

            last_status = transfer_info.status.clone();
        }

        // Wait before next check
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // Display final transfer information
    println!("\n📊 Final Transfer Summary:");
    let final_info = bridge.get_transfer(&transfer_id).await?;

    println!("   Transfer ID: {}", final_info.transfer_id);
    println!("   Created: {}", final_info.created_at);
    println!("   Last Updated: {}", final_info.updated_at);
    println!("   Final Status: {:?}", final_info.status);

    if let Some(route) = &final_info.route {
        println!("   Route Cost: ${}", route.optimal_route.total_cost);
        println!("   Route Time: {} seconds", route.optimal_route.estimated_time_seconds);
        println!("   Route Reliability: {:.1}%", route.optimal_route.reliability_score * 100.0);
    }

    // Show system metrics after transfer
    println!("\n📈 System Metrics After Transfer:");
    let metrics = bridge.get_system_metrics().await;
    println!("   Active Transfers: {}", metrics.active_transfers);
    println!("   Success Rate: {:.1}%", metrics.success_rate * 100.0);
    println!("   Average Processing Time: {} seconds", metrics.avg_processing_time_seconds);
    println!("   Daily Volume: ${}", metrics.daily_volume);

    // Demonstrate transfer listing
    println!("\n📋 Listing Recent Transfers:");
    let all_transfers = bridge.list_transfers().await;
    println!("   Total transfers in system: {}", all_transfers.len());

    for (i, transfer) in all_transfers.iter().take(5).enumerate() {
        println!("   {}. {} ({} -> {}) - {:?}",
                i + 1,
                transfer.transfer_id,
                transfer.request.from_chain,
                transfer.request.to_chain,
                transfer.status);
    }

    println!("\n🎯 Cross-chain transfer example completed!");
    println!("   This example demonstrated:");
    println!("   • Bridge system initialization");
    println!("   • Health status monitoring");
    println!("   • Transfer request creation");
    println!("   • Transfer execution and monitoring");
    println!("   • Status tracking and metrics");

    Ok(())
}