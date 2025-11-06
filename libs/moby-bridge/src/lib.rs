//! # Moby Bridge 🐋🌉
//!
//! A comprehensive cross-chain bridge infrastructure designed specifically for whale trading operations.
//!
//! This library provides all the necessary components for secure, efficient cross-chain transfers including:
//! - Multi-chain protocol support (Ethereum, Solana, Polygon, BSC, Avalanche)
//! - High-value transaction optimization for whale traders
//! - Privacy-preserving bridge mechanisms
//! - Advanced security with multi-signature validation
//! - Liquidity aggregation across chains
//! - Real-time monitoring and analytics
//!
//! ## Features
//!
//! - **🌉 Multi-Chain Support**: Seamless integration across major blockchains
//! - **🐋 Whale Optimization**: Specialized handling for large-value transfers
//! - **🔒 Privacy Bridges**: Zero-knowledge proof integration for private transfers
//! - **🛡️ Security First**: Multi-signature validation and fraud prevention
//! - **⚡ High Performance**: Optimized for low latency and high throughput
//! - **🔄 Liquidity Management**: Cross-chain liquidity aggregation and routing
//! - **📊 Analytics**: Real-time monitoring and bridge health metrics
//! - **🚨 Emergency Controls**: Circuit breakers and emergency pause mechanisms
//!
//! ## Quick Start
//!
//! ```rust
//! use moby_bridge::{BridgeSystem, ChainConfig, TransferRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize bridge system
//!     let bridge = BridgeSystem::new().await?;
//!
//!     // Create a cross-chain transfer
//!     let transfer = TransferRequest {
//!         from_chain: "ethereum".to_string(),
//!         to_chain: "solana".to_string(),
//!         token: "USDC".to_string(),
//!         amount: 10_000_000, // $10M transfer
//!         recipient: "recipient_address".to_string(),
//!         privacy_level: PrivacyLevel::Enhanced,
//!     };
//!
//!     // Execute transfer
//!     let transfer_id = bridge.initiate_transfer(transfer).await?;
//!     println!("✅ Cross-chain transfer initiated: {}", transfer_id);
//!
//!     Ok(())
//! }
//! ```

pub mod chains;
pub mod protocols;
pub mod security;
pub mod liquidity;
pub mod privacy;
pub mod monitoring;
pub mod relayer;
pub mod validator;
pub mod system;
pub mod error;

#[cfg(feature = "mock")]
pub mod mock;

// Re-export core types
pub use system::{BridgeSystem, BridgeConfig};
pub use chains::{Chain, ChainId, ChainConfig, ChainStatus};
pub use protocols::{Protocol, ProtocolType, MessageType};
pub use security::{SecurityLevel, ValidationResult, Signature};
pub use liquidity::{LiquidityPool, LiquidityProvider, RouteOptimization};
pub use privacy::{PrivacyLevel, PrivacyBridge, ZkProof};
pub use monitoring::{BridgeMonitor, HealthStatus, Metrics};
pub use relayer::{Relayer, RelayerNetwork, RelayerStatus};
pub use validator::{Validator, ValidatorSet, Consensus};
pub use error::{BridgeError, BridgeResult};

/// Result type alias for bridge operations
pub type Result<T> = std::result::Result<T, BridgeError>;

/// Current version of the bridge system
pub const BRIDGE_VERSION: &str = "0.1.0";

/// Maximum number of supported chains
pub const MAX_SUPPORTED_CHAINS: usize = 50;

/// Default transfer timeout (30 minutes)
pub const DEFAULT_TRANSFER_TIMEOUT: u64 = 30 * 60;

/// Minimum validators required for consensus
pub const MIN_VALIDATORS: usize = 3;

/// Maximum validators for efficient consensus
pub const MAX_VALIDATORS: usize = 21;

/// Default bridge fee (0.1%)
pub const DEFAULT_BRIDGE_FEE: f64 = 0.001;

/// Whale threshold (transfers above this amount get special handling)
pub const WHALE_THRESHOLD: u64 = 1_000_000; // $1M

/// Maximum single transfer amount (for security)
pub const MAX_TRANSFER_AMOUNT: u64 = 100_000_000; // $100M