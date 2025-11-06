//! # Moby DEX 🐋💱
//!
//! An advanced decentralized exchange infrastructure optimized for whale trading operations
//! with sophisticated automated market maker algorithms, MEV protection, and order optimization.
//!
//! This library provides comprehensive DEX functionality including:
//! - Advanced automated market maker (AMM) algorithms optimized for large trades
//! - High-performance order book with sophisticated matching engine
//! - Whale-specific trading strategies with slippage optimization
//! - MEV protection including sandwich attack prevention
//! - Multi-asset liquidity pools with concentrated liquidity support
//! - Real-time arbitrage detection and execution
//! - Privacy-preserving trading with zero-knowledge proofs integration
//! - Cross-chain trading support via bridge integration
//!
//! ## Features
//!
//! - **🐋 Whale Trading Optimization**: Specialized algorithms for large-volume trades
//! - **📈 Advanced AMM Models**: Constant product, constant sum, and concentrated liquidity
//! - **📚 High-Performance Order Book**: Sub-millisecond matching with priority queues
//! - **🛡️ MEV Protection**: Comprehensive protection against extractable value attacks
//! - **🔄 Real-time Arbitrage**: Automated cross-pool arbitrage detection and execution
//! - **💧 Dynamic Liquidity**: Intelligent liquidity provisioning and management
//! - **🔒 Privacy Integration**: Zero-knowledge proof trading via moby-privacy
//! - **🌉 Cross-Chain Support**: Multi-chain trading via moby-bridge integration
//!
//! ## Quick Start
//!
//! ```rust
//! use moby_dex::{DEXSystem, AMMPool, OrderBook, TradingStrategy};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize DEX system
//!     let dex = DEXSystem::new().await?;
//!
//!     // Create an AMM pool
//!     let pool = AMMPool::constant_product("ETH/USDC").await?;
//!
//!     // Execute whale trade with optimization
//!     let trade_result = dex.execute_whale_trade(
//!         pool.id(),
//!         1_000_000_000, // $1B trade
//!         TradingStrategy::MinimizeSlippage,
//!     ).await?;
//!
//!     println!("Trade executed: {} ETH for {} USDC",
//!         trade_result.amount_in, trade_result.amount_out);
//!
//!     Ok(())
//! }
//! ```

pub mod amm;
pub mod orderbook;
pub mod trading;
pub mod liquidity;
pub mod mev_protection;
pub mod arbitrage;
pub mod strategies;
pub mod system;
pub mod error;

#[cfg(feature = "mock")]
pub mod mock;

// Re-export core types
pub use system::{DEXSystem, DEXConfig};
pub use amm::{AMMPool, AMMType, PoolConfig, SwapResult};
pub use orderbook::{OrderBook, Order, OrderType, OrderSide, MatchResult};
pub use trading::{TradingEngine, TradingStrategy, TradeExecution, TradeResult};
pub use liquidity::{LiquidityProvider, LiquidityPool, LiquidityPosition, PositionManager};
pub use mev_protection::{MEVProtector, MEVDetection, ProtectionStrategy, MEVResult};
pub use arbitrage::{ArbitrageEngine, ArbitrageOpportunity, ArbitrageExecution, ArbitrageResult};
pub use strategies::{WhaleStrategy, TradingParameters, OrderOptimization, SlippageProtection};
pub use error::{DEXError, DEXResult};

/// Result type alias for DEX operations
pub type Result<T> = std::result::Result<T, DEXError>;

/// Current version of the DEX system
pub const DEX_VERSION: &str = "0.1.0";

/// Maximum number of supported trading pairs
pub const MAX_TRADING_PAIRS: usize = 10000;

/// Default trade fee (0.3%)
pub const DEFAULT_TRADE_FEE: f64 = 0.003;

/// Whale trade threshold ($1M)
pub const WHALE_TRADE_THRESHOLD: f64 = 1_000_000.0;

/// Maximum slippage tolerance (5%)
pub const MAX_SLIPPAGE_TOLERANCE: f64 = 0.05;

/// Default order book depth
pub const DEFAULT_ORDER_BOOK_DEPTH: usize = 1000;

/// MEV protection timeout (12 seconds - 1 block)
pub const MEV_PROTECTION_TIMEOUT_MS: u64 = 12000;

/// Minimum liquidity requirement for new pools
pub const MIN_POOL_LIQUIDITY: f64 = 10_000.0;

/// Maximum position size as percentage of pool (10%)
pub const MAX_POSITION_SIZE_PERCENTAGE: f64 = 0.10;

/// Arbitrage opportunity threshold (0.1%)
pub const ARBITRAGE_THRESHOLD: f64 = 0.001;