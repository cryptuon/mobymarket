// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Error types for the trading engine

use thiserror::Error;
use moby_types::MobyError;

/// Result type for trading operations
pub type TradingResult<T> = Result<T, TradingError>;

/// Comprehensive error types for trading operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TradingError {
    // Order-related errors
    #[error("Invalid order: {message}")]
    InvalidOrder { message: String },

    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: u64 },

    #[error("Order already exists: {order_id}")]
    OrderAlreadyExists { order_id: u64 },

    #[error("Order cannot be modified in current state: {current_state}")]
    OrderNotModifiable { current_state: String },

    #[error("Order size {size} exceeds maximum allowed {max_size}")]
    OrderSizeExceeded { size: u64, max_size: u64 },

    // Execution errors
    #[error("Execution failed: {reason}")]
    ExecutionFailed { reason: String },

    #[error("Insufficient liquidity for order size {size}")]
    InsufficientLiquidity { size: u64 },

    #[error("Slippage {actual_bps}bps exceeds tolerance {max_bps}bps")]
    SlippageExceeded { actual_bps: u16, max_bps: u16 },

    #[error("Execution timeout after {duration_ms}ms")]
    ExecutionTimeout { duration_ms: u64 },

    #[error("Market is closed or suspended")]
    MarketClosed,

    // Risk management errors
    #[error("Risk limit exceeded: {limit_type}")]
    RiskLimitExceeded { limit_type: String },

    #[error("Daily volume limit {current} exceeds maximum {limit}")]
    DailyVolumeLimitExceeded { current: u64, limit: u64 },

    #[error("Position size {current} exceeds maximum {limit}")]
    PositionSizeLimitExceeded { current: u64, limit: u64 },

    #[error("Maximum number of open orders {limit} reached")]
    MaxOpenOrdersReached { limit: u32 },

    #[error("Trader tier {tier} not authorized for order size {size}")]
    TierNotAuthorized { tier: String, size: u64 },

    // Market data errors
    #[error("Price feed unavailable for token {token}")]
    PriceFeedUnavailable { token: String },

    #[error("Stale price data: {age_seconds} seconds old")]
    StalePriceData { age_seconds: u64 },

    #[error("Price deviation {deviation_bps}bps exceeds threshold {threshold_bps}bps")]
    PriceDeviationExceeded { deviation_bps: u16, threshold_bps: u16 },

    #[error("Market data inconsistent across sources")]
    InconsistentMarketData,

    // Liquidity errors
    #[error("Insufficient liquidity pool reserves")]
    InsufficientPoolReserves,

    #[error("Liquidity ratio {ratio} below minimum {minimum}")]
    LiquidityRatioTooLow { ratio: u16, minimum: u16 },

    #[error("Market impact {impact_bps}bps exceeds threshold {threshold_bps}bps")]
    MarketImpactTooHigh { impact_bps: u16, threshold_bps: u16 },

    // Fee errors
    #[error("Fee calculation failed: {reason}")]
    FeeCalculationFailed { reason: String },

    #[error("Insufficient funds to cover fees: required {required}, available {available}")]
    InsufficientFundsForFees { required: u64, available: u64 },

    // Strategy errors
    #[error("Invalid execution strategy: {strategy}")]
    InvalidExecutionStrategy { strategy: String },

    #[error("TWAP execution failed: {reason}")]
    TwapExecutionFailed { reason: String },

    #[error("VWAP execution failed: {reason}")]
    VwapExecutionFailed { reason: String },

    #[error("Smart routing failed: no viable execution path")]
    SmartRoutingFailed,

    // OTC errors
    #[error("OTC counterparty not found: {counterparty}")]
    OtcCounterpartyNotFound { counterparty: String },

    #[error("OTC trade negotiation failed: {reason}")]
    OtcNegotiationFailed { reason: String },

    #[error("OTC settlement failed: {reason}")]
    OtcSettlementFailed { reason: String },

    // Privacy errors
    #[error("Privacy proof generation failed: {reason}")]
    PrivacyProofFailed { reason: String },

    #[error("Privacy proof verification failed")]
    PrivacyVerificationFailed,

    #[error("Privacy pool insufficient for order size {size}")]
    PrivacyPoolInsufficient { size: u64 },

    // Cross-chain errors
    #[error("Cross-chain bridge unavailable for network {network}")]
    CrossChainBridgeUnavailable { network: String },

    #[error("Cross-chain verification failed: {reason}")]
    CrossChainVerificationFailed { reason: String },

    #[error("Cross-chain settlement timeout")]
    CrossChainSettlementTimeout,

    // Configuration errors
    #[error("Invalid configuration: {field}")]
    InvalidConfiguration { field: String },

    #[error("Required configuration missing: {field}")]
    MissingConfiguration { field: String },

    // System errors
    #[error("System temporarily unavailable")]
    SystemUnavailable,

    #[error("Rate limit exceeded: {operations} operations in {window_seconds} seconds")]
    RateLimitExceeded { operations: u32, window_seconds: u32 },

    #[error("Internal error: {message}")]
    Internal { message: String },

    // Integration errors
    #[error("Oracle error: {0}")]
    Oracle(#[from] moby_oracle::OracleError),

    #[error("Math error: {0}")]
    Math(#[from] moby_math::MathError),

    #[error("Core type error: {0}")]
    CoreType(#[from] MobyError),
}

impl TradingError {
    /// Create a new invalid order error
    pub fn invalid_order(message: impl Into<String>) -> Self {
        Self::InvalidOrder { message: message.into() }
    }

    /// Create a new execution failed error
    pub fn execution_failed(reason: impl Into<String>) -> Self {
        Self::ExecutionFailed { reason: reason.into() }
    }

    /// Create a new risk limit exceeded error
    pub fn risk_limit_exceeded(limit_type: impl Into<String>) -> Self {
        Self::RiskLimitExceeded { limit_type: limit_type.into() }
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into() }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Temporary system issues
            Self::SystemUnavailable
            | Self::ExecutionTimeout { .. }
            | Self::MarketClosed
            | Self::PriceFeedUnavailable { .. }
            | Self::StalePriceData { .. }
            | Self::CrossChainSettlementTimeout => true,

            // Rate limiting
            Self::RateLimitExceeded { .. } => true,

            // Liquidity issues (may resolve)
            Self::InsufficientLiquidity { .. }
            | Self::InsufficientPoolReserves
            | Self::LiquidityRatioTooLow { .. } => true,

            // Non-recoverable errors
            Self::InvalidOrder { .. }
            | Self::OrderNotFound { .. }
            | Self::OrderAlreadyExists { .. }
            | Self::InvalidExecutionStrategy { .. }
            | Self::PrivacyVerificationFailed
            | Self::InvalidConfiguration { .. }
            | Self::MissingConfiguration { .. } => false,

            // Risk limits (policy dependent)
            Self::RiskLimitExceeded { .. }
            | Self::DailyVolumeLimitExceeded { .. }
            | Self::PositionSizeLimitExceeded { .. }
            | Self::MaxOpenOrdersReached { .. }
            | Self::TierNotAuthorized { .. } => false,

            // Context dependent
            _ => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical system errors
            Self::Internal { .. }
            | Self::SystemUnavailable
            | Self::CrossChainVerificationFailed { .. } => ErrorSeverity::Critical,

            // High severity - trading execution issues
            Self::ExecutionFailed { .. }
            | Self::SlippageExceeded { .. }
            | Self::MarketImpactTooHigh { .. }
            | Self::OtcSettlementFailed { .. }
            | Self::PrivacyProofFailed { .. } => ErrorSeverity::High,

            // Medium severity - risk and validation
            Self::RiskLimitExceeded { .. }
            | Self::InvalidOrder { .. }
            | Self::InsufficientLiquidity { .. }
            | Self::PriceDeviationExceeded { .. } => ErrorSeverity::Medium,

            // Low severity - temporary issues
            Self::ExecutionTimeout { .. }
            | Self::MarketClosed
            | Self::StalePriceData { .. }
            | Self::RateLimitExceeded { .. } => ErrorSeverity::Low,

            // Info level - normal validation
            Self::OrderNotFound { .. }
            | Self::OrderAlreadyExists { .. }
            | Self::OrderNotModifiable { .. } => ErrorSeverity::Info,
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self, ErrorSeverity::Critical | ErrorSeverity::High)
    }

    /// Check if error should trigger alerts
    pub fn should_alert(&self) -> bool {
        matches!(self, ErrorSeverity::Medium | ErrorSeverity::High | ErrorSeverity::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = TradingError::invalid_order("Invalid price");
        assert!(matches!(error, TradingError::InvalidOrder { .. }));

        let error = TradingError::execution_failed("Network timeout");
        assert!(matches!(error, TradingError::ExecutionFailed { .. }));
    }

    #[test]
    fn test_error_recoverability() {
        assert!(TradingError::ExecutionTimeout { duration_ms: 5000 }.is_recoverable());
        assert!(TradingError::SystemUnavailable.is_recoverable());
        assert!(!TradingError::InvalidOrder { message: "bad".to_string() }.is_recoverable());
        assert!(!TradingError::PrivacyVerificationFailed.is_recoverable());
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(TradingError::Internal { message: "crash".to_string() }.severity(), ErrorSeverity::Critical);
        assert_eq!(TradingError::ExecutionFailed { reason: "fail".to_string() }.severity(), ErrorSeverity::High);
        assert_eq!(TradingError::InvalidOrder { message: "bad".to_string() }.severity(), ErrorSeverity::Medium);
        assert_eq!(TradingError::MarketClosed.severity(), ErrorSeverity::Low);
        assert_eq!(TradingError::OrderNotFound { order_id: 123 }.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn test_severity_levels() {
        assert!(ErrorSeverity::Critical.requires_immediate_attention());
        assert!(ErrorSeverity::High.requires_immediate_attention());
        assert!(!ErrorSeverity::Medium.requires_immediate_attention());

        assert!(ErrorSeverity::Critical.should_alert());
        assert!(ErrorSeverity::High.should_alert());
        assert!(ErrorSeverity::Medium.should_alert());
        assert!(!ErrorSeverity::Low.should_alert());
        assert!(!ErrorSeverity::Info.should_alert());
    }
}