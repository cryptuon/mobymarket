//! # Moby Market Platform 🐋💰
//!
//! The unified whale trading platform that integrates all Moby Market components
//! into a cohesive, high-value trading system for institutional and whale traders.
//!
//! ## Value Proposition
//!
//! This platform delivers real value by:
//!
//! 1. **🎯 Optimized Whale Trading**: Execute large trades with minimal slippage across multiple DEXs
//! 2. **🔒 Privacy Protection**: Zero-knowledge proofs hide trading strategies from competitors
//! 3. **🌉 Cross-Chain Arbitrage**: Capture opportunities across different blockchains
//! 4. **📊 Advanced Analytics**: Real-time market intelligence and whale activity monitoring
//! 5. **💰 Revenue Generation**: Multiple fee streams and yield optimization strategies
//! 6. **🏛️ Decentralized Governance**: Community-driven protocol improvements and fee sharing
//!
//! ## Core Features
//!
//! - **Intelligent Order Routing**: Automatically find best execution across AMMs and order books
//! - **MEV Protection**: Shield trades from sandwich attacks and front-running
//! - **Privacy Pools**: Execute trades without revealing positions or strategies
//! - **Cross-Chain Optimization**: Seamlessly trade across Ethereum, Polygon, Avalanche, etc.
//! - **Yield Optimization**: Automatically compound returns and optimize liquidity provision
//! - **Risk Management**: Real-time position monitoring and automated stop-losses
//!
//! ## Revenue Streams
//!
//! 1. **Trading Fees**: Competitive fees on all trades with volume discounts
//! 2. **Premium Features**: Advanced analytics, priority execution, custom strategies
//! 3. **Liquidity Provision**: Earn fees by providing liquidity to our pools
//! 4. **Cross-Chain Services**: Fees for bridge operations and cross-chain trades
//! 5. **Data Services**: Real-time whale tracking and market intelligence APIs
//! 6. **Governance Tokens**: Revenue sharing through protocol governance participation
//!
//! ## Quick Start
//!
//! ```rust
//! use moby_market_platform::{MobyMarket, TradingStrategy, PrivacyLevel};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the complete platform
//!     let moby = MobyMarket::new().await?;
//!
//!     // Execute a private whale trade with cross-chain optimization
//!     let trade_result = moby.execute_whale_trade()
//!         .amount(10_000_000) // $10M trade
//!         .pair("ETH/USDC")
//!         .privacy_level(PrivacyLevel::Full)
//!         .strategy(TradingStrategy::MinimizeSlippage)
//!         .cross_chain_enabled(true)
//!         .mev_protection(true)
//!         .execute()
//!         .await?;
//!
//!     println!("Trade executed: {} profit, {} slippage",
//!         trade_result.profit, trade_result.slippage);
//!
//!     Ok(())
//! }
//! ```

pub mod platform;
pub mod integration;
pub mod strategies;
pub mod analytics;
pub mod api;
pub mod revenue;
pub mod monitoring;
pub mod clients;

// Re-export core components
pub use platform::{MobyMarket, PlatformConfig, PlatformState};
pub use integration::{ComponentManager, CrossComponentCommunication, EventBus};
pub use strategies::{
    TradingStrategy, WhaleStrategy, ArbitrageStrategy, YieldStrategy,
    StrategyEngine, StrategyResult, StrategyParameters
};
pub use analytics::{
    MarketAnalytics, WhaleTracking, RealTimeMonitoring,
    AnalyticsEngine, MarketIntelligence, TradingSignals
};
pub use api::{ApiServer, ApiClient, WebSocketServer, RestApiHandler};
pub use revenue::{RevenueEngine, FeeStructure, YieldOptimizer, ProfitDistribution};
pub use monitoring::{MetricsCollector, HealthMonitor, PerformanceTracker, AlertManager};

// Re-export from component libraries for convenience
pub use moby_privacy::{PrivacyEngine, ZKProofSystem, PrivacyLevel};
pub use moby_governance::{GovernanceEngine, ProposalSystem, VotingMechanism};
pub use moby_bridge::{BridgeEngine, CrossChainManager, ChainSupport};
pub use moby_oracle::{OracleEngine, PriceFeedManager, DataAggregation};
pub use moby_dex::{DEXEngine, AMMManager, OrderBookManager};

/// Current platform version
pub const PLATFORM_VERSION: &str = "0.1.0";

/// Platform name
pub const PLATFORM_NAME: &str = "Moby Market";

/// Default configuration values
pub const DEFAULT_MAX_TRADE_SIZE: f64 = 100_000_000.0; // $100M
pub const DEFAULT_MIN_TRADE_SIZE: f64 = 1_000.0; // $1K
pub const DEFAULT_WHALE_THRESHOLD: f64 = 1_000_000.0; // $1M
pub const DEFAULT_MAX_SLIPPAGE: f64 = 0.05; // 5%
pub const DEFAULT_PLATFORM_FEE: f64 = 0.002; // 0.2%

/// Supported blockchain networks
pub const SUPPORTED_CHAINS: &[&str] = &[
    "ethereum",
    "polygon",
    "avalanche",
    "arbitrum",
    "optimism",
    "bsc",
    "fantom",
];

/// Trading pair categories
pub const MAJOR_PAIRS: &[&str] = &[
    "ETH/USDC", "BTC/USDC", "ETH/BTC",
    "AVAX/USDC", "MATIC/USDC", "BNB/USDC"
];

pub const STABLE_PAIRS: &[&str] = &[
    "USDC/USDT", "DAI/USDC", "FRAX/USDC"
];

/// Error type for the platform
pub type Result<T> = std::result::Result<T, PlatformError>;

/// Platform-wide error types
#[derive(thiserror::Error, Debug)]
pub enum PlatformError {
    #[error("Component initialization failed: {component}")]
    ComponentInitializationFailed { component: String },

    #[error("Cross-component communication error: {message}")]
    CrossComponentError { message: String },

    #[error("Strategy execution failed: {strategy} - {reason}")]
    StrategyExecutionFailed { strategy: String, reason: String },

    #[error("Revenue optimization error: {details}")]
    RevenueOptimizationError { details: String },

    #[error("Analytics processing error: {operation}")]
    AnalyticsError { operation: String },

    #[error("API request failed: {endpoint} - {status}")]
    ApiRequestFailed { endpoint: String, status: u16 },

    #[error("Monitoring alert: {severity} - {message}")]
    MonitoringAlert { severity: String, message: String },

    #[error("Configuration error: {parameter} = {value}")]
    ConfigurationError { parameter: String, value: String },

    #[error("Database error: {operation}")]
    DatabaseError { operation: String },

    #[error("Network error: {chain} - {details}")]
    NetworkError { chain: String, details: String },

    #[error("Privacy operation failed: {operation}")]
    PrivacyError { operation: String },

    #[error("Governance action failed: {action}")]
    GovernanceError { action: String },

    #[error("Bridge operation failed: {from_chain} -> {to_chain}")]
    BridgeError { from_chain: String, to_chain: String },

    #[error("Oracle data error: {source} - {reason}")]
    OracleError { source: String, reason: String },

    #[error("DEX operation failed: {dex} - {operation}")]
    DEXError { dex: String, operation: String },

    #[error("Authentication failed: {user_id}")]
    AuthenticationFailed { user_id: String },

    #[error("Authorization denied: {operation} for {user_id}")]
    AuthorizationDenied { operation: String, user_id: String },

    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimitExceeded { limit: u32, window: String },

    #[error("Internal platform error: {details}")]
    InternalError { details: String },
}

/// Convert component library errors to platform errors
impl From<moby_privacy::PrivacyError> for PlatformError {
    fn from(err: moby_privacy::PrivacyError) -> Self {
        PlatformError::PrivacyError {
            operation: err.to_string(),
        }
    }
}

impl From<moby_governance::GovernanceError> for PlatformError {
    fn from(err: moby_governance::GovernanceError) -> Self {
        PlatformError::GovernanceError {
            action: err.to_string(),
        }
    }
}

impl From<moby_bridge::BridgeError> for PlatformError {
    fn from(err: moby_bridge::BridgeError) -> Self {
        PlatformError::BridgeError {
            from_chain: "unknown".to_string(),
            to_chain: "unknown".to_string(),
        }
    }
}

impl From<moby_oracle::OracleError> for PlatformError {
    fn from(err: moby_oracle::OracleError) -> Self {
        PlatformError::OracleError {
            source: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<moby_dex::DEXError> for PlatformError {
    fn from(err: moby_dex::DEXError) -> Self {
        PlatformError::DEXError {
            dex: "unknown".to_string(),
            operation: err.to_string(),
        }
    }
}