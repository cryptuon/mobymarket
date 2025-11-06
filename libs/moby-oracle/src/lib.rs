//! # Moby Oracle 🐋📊
//!
//! A comprehensive decentralized oracle infrastructure designed specifically for whale trading operations.
//!
//! This library provides all the necessary components for reliable, secure price feeds and data aggregation including:
//! - Multi-source price feed aggregation (Chainlink, Pyth, Band Protocol, API3, UMA)
//! - High-precision financial data processing for whale trading operations
//! - Real-time market data streaming and historical data management
//! - Advanced security with cryptographic verification and fraud detection
//! - Customizable data feeds and oracle governance mechanisms
//! - Performance optimization for low-latency trading operations
//!
//! ## Features
//!
//! - **📊 Multi-Source Aggregation**: Combine data from multiple oracle networks
//! - **🐋 Whale Trading Focus**: Specialized handling for large-volume market data
//! - **🔒 Cryptographic Security**: Verify data integrity and prevent manipulation
//! - **⚡ High Performance**: Optimized for sub-second data updates
//! - **🔄 Real-time Streaming**: Live market data with WebSocket connections
//! - **📈 Historical Data**: Time-series data storage and analytics
//! - **🛡️ Fraud Detection**: Anomaly detection and outlier filtering
//! - **🏛️ Governance**: Decentralized oracle parameter management
//!
//! ## Quick Start
//!
//! ```rust
//! use moby_oracle::{OracleSystem, PriceFeedConfig, DataSource};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize oracle system
//!     let oracle = OracleSystem::new().await?;
//!
//!     // Configure price feed
//!     let feed_config = PriceFeedConfig {
//!         symbol: "ETH/USD".to_string(),
//!         sources: vec![DataSource::Chainlink, DataSource::Pyth],
//!         update_frequency: std::time::Duration::from_secs(1),
//!         deviation_threshold: 0.01, // 1%
//!     };
//!
//!     // Start price feed
//!     let feed_id = oracle.create_price_feed(feed_config).await?;
//!
//!     // Get latest price
//!     let price = oracle.get_latest_price(&feed_id).await?;
//!     println!("ETH/USD: ${}", price.value);
//!
//!     Ok(())
//! }
//! ```

pub mod sources;
pub mod aggregation;
pub mod feeds;
pub mod security;
pub mod storage;
pub mod governance;
pub mod monitoring;
pub mod streaming;
pub mod analytics;
pub mod system;
pub mod error;

#[cfg(feature = "mock")]
pub mod mock;

// Re-export core types
pub use system::{OracleSystem, OracleConfig};
pub use feeds::{PriceFeed, PriceFeedConfig, PriceData, FeedStatus};
pub use sources::{DataSource, SourceProvider, SourceConfig, DataPoint};
pub use aggregation::{AggregationStrategy, AggregatedPrice, Aggregator};
pub use security::{SecurityValidator, ValidationResult, DataIntegrity};
pub use storage::{DataStorage, TimeSeriesData, HistoricalQuery};
pub use governance::{OracleGovernance, GovernanceProposal, VotingSystem};
pub use monitoring::{OracleMonitor, HealthStatus, PerformanceMetrics};
pub use streaming::{DataStream, StreamConfig, RealtimeData};
pub use analytics::{MarketAnalytics, TradingSignals, WhaleMetrics};
pub use error::{OracleError, OracleResult};

/// Result type alias for oracle operations
pub type Result<T> = std::result::Result<T, OracleError>;

/// Current version of the oracle system
pub const ORACLE_VERSION: &str = "0.1.0";

/// Maximum number of supported data sources
pub const MAX_DATA_SOURCES: usize = 100;

/// Default price feed update frequency (1 second)
pub const DEFAULT_UPDATE_FREQUENCY_MS: u64 = 1000;

/// Default price deviation threshold (0.5%)
pub const DEFAULT_DEVIATION_THRESHOLD: f64 = 0.005;

/// Maximum historical data retention (1 year)
pub const MAX_HISTORICAL_RETENTION_DAYS: u32 = 365;

/// Default aggregation timeout (5 seconds)
pub const DEFAULT_AGGREGATION_TIMEOUT_MS: u64 = 5000;

/// Whale trading volume threshold
pub const WHALE_VOLUME_THRESHOLD: f64 = 1_000_000.0; // $1M