//! Test library for Moby Bridge integration tests.

// Re-export the main library for tests
pub use moby_bridge::*;

/// Common test utilities
pub mod test_utils {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;

    /// Create a mock transfer request for testing
    pub fn mock_transfer_request(
        from_chain: &str,
        to_chain: &str,
        amount: u64,
    ) -> system::TransferRequest {
        system::TransferRequest {
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            token: "USDC".to_string(),
            amount,
            recipient: format!("mock_recipient_{}", to_chain),
            sender: format!("mock_sender_{}", from_chain),
            privacy_level: system::PrivacyLevel::Public,
            priority: system::TransferPriority::Normal,
            deadline: Some(Utc::now() + chrono::Duration::minutes(30)),
            metadata: HashMap::new(),
        }
    }

    /// Create a test bridge configuration
    pub fn test_bridge_config() -> system::BridgeConfig {
        system::BridgeConfig {
            version: "test-0.1.0".to_string(),
            max_chains: 10,
            default_timeout_seconds: 300,
            whale_threshold: 1_000_000,
            emergency_pause_enabled: true,
            security_config: system::SecurityConfig::default(),
            liquidity_config: system::LiquidityConfig::default(),
            monitoring_config: system::MonitoringConfig::default(),
        }
    }
}