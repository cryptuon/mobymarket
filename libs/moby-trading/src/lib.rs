// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! # Moby Trading
//!
//! Core trading engine and order management system for whale trading infrastructure.
//!
//! This library provides:
//! - Advanced order management with multiple execution strategies
//! - TWAP (Time-Weighted Average Price) execution
//! - VWAP (Volume-Weighted Average Price) execution
//! - Market making and liquidity provision
//! - Risk management and position sizing
//! - Order matching engine
//! - Fee calculation and optimization
//!
//! ## Features
//!
//! - **Multi-Strategy Execution**: Support for market, limit, TWAP, VWAP, and smart execution
//! - **Whale-Optimized**: Specialized algorithms for large order execution
//! - **Risk Management**: Comprehensive risk controls and position limits
//! - **Privacy Support**: Integration with zero-knowledge proof systems
//! - **Cross-Chain**: Support for multi-chain trading operations
//!
//! ## Quick Start
//!
//! ```rust
//! use moby_trading::{TradingEngine, OrderRequest, ExecutionStrategy};
//! use moby_types::{WhaleAmount, TradingTier};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize trading engine
//! let mut engine = TradingEngine::new();
//!
//! // Create a large whale order
//! let order = OrderRequest::new(
//!     WhaleAmount::from_dollars(50_000_000), // $50M order
//!     ExecutionStrategy::Twap { duration_minutes: 60 },
//!     TradingTier::LargeWhale,
//! );
//!
//! // Execute the order
//! let execution_plan = engine.plan_execution(&order).await?;
//! let result = engine.execute_order(order, execution_plan).await?;
//! # Ok(())
//! # }
//! ```

pub mod engine;
pub mod execution;
pub mod matching;
pub mod orders;
pub mod strategies;
pub mod risk;
pub mod fees;
pub mod market_making;
pub mod analytics;
pub mod error;

// Re-export core types
pub use engine::TradingEngine;
pub use execution::{ExecutionEngine, ExecutionPlan, ExecutionResult};
pub use matching::{OrderMatcher, MatchingEngine};
pub use orders::{OrderManager, OrderRequest, OrderUpdate};
pub use strategies::{ExecutionStrategy, TwapStrategy, VwapStrategy, SmartStrategy};
pub use risk::{RiskManager, PositionLimits, RiskAssessment};
pub use fees::{FeeCalculator, FeeStructure, VolumeDiscount};
pub use market_making::{MarketMaker, LiquidityProvider, SpreadManager};
pub use analytics::{TradingAnalytics, PerformanceMetrics, MarketImpactAnalysis};
pub use error::{TradingError, TradingResult};

// Re-export from dependencies
pub use moby_math::{Price, Amount, Percentage, SlippageCalculator};
pub use moby_types::{
    WhaleAmount, TradingTier, OrderType, OrderStatus, ExecutionStrategy as TypesExecutionStrategy,
    WhaleOrder, OtcTrade, TwapExecution
};
pub use moby_oracle::{PriceOracle, OraclePrice, PriceAggregator};

/// Trading engine configuration
#[derive(Debug, Clone)]
pub struct TradingConfig {
    /// Maximum slippage tolerance for market orders
    pub max_slippage_bps: u16,

    /// Default TWAP execution window in minutes
    pub default_twap_duration: u32,

    /// Maximum order size as percentage of market cap
    pub max_order_size_pct: u16,

    /// Enable privacy mode by default
    pub privacy_enabled: bool,

    /// Risk management settings
    pub risk_config: RiskConfig,

    /// Fee configuration
    pub fee_config: FeeConfig,
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 100, // 1%
            default_twap_duration: 60, // 1 hour
            max_order_size_pct: 500, // 5%
            privacy_enabled: false,
            risk_config: RiskConfig::default(),
            fee_config: FeeConfig::default(),
        }
    }
}

/// Risk management configuration
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// Maximum daily trading volume per trader
    pub max_daily_volume: u64,

    /// Maximum position size per token
    pub max_position_size: u64,

    /// Maximum number of open orders
    pub max_open_orders: u32,

    /// Enable real-time risk monitoring
    pub real_time_monitoring: bool,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_daily_volume: 1_000_000_000 * moby_math::Price::PRECISION, // $1B
            max_position_size: 100_000_000 * moby_math::Price::PRECISION, // $100M
            max_open_orders: 50,
            real_time_monitoring: true,
        }
    }
}

/// Fee configuration
#[derive(Debug, Clone)]
pub struct FeeConfig {
    /// Base trading fee in basis points
    pub base_fee_bps: u16,

    /// Privacy fee premium in basis points
    pub privacy_fee_bps: u16,

    /// OTC trading fee in basis points
    pub otc_fee_bps: u16,

    /// Cross-chain fee in basis points
    pub cross_chain_fee_bps: u16,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            base_fee_bps: 30, // 0.3%
            privacy_fee_bps: 10, // 0.1% additional
            otc_fee_bps: 20, // 0.2%
            cross_chain_fee_bps: 50, // 0.5%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_config_defaults() {
        let config = TradingConfig::default();
        assert_eq!(config.max_slippage_bps, 100);
        assert_eq!(config.default_twap_duration, 60);
        assert!(!config.privacy_enabled);
    }

    #[test]
    fn test_risk_config_defaults() {
        let config = RiskConfig::default();
        assert_eq!(config.max_daily_volume, 1_000_000_000 * moby_math::Price::PRECISION);
        assert_eq!(config.max_open_orders, 50);
        assert!(config.real_time_monitoring);
    }

    #[test]
    fn test_fee_config_defaults() {
        let config = FeeConfig::default();
        assert_eq!(config.base_fee_bps, 30);
        assert_eq!(config.privacy_fee_bps, 10);
        assert_eq!(config.otc_fee_bps, 20);
    }
}