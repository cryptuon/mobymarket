//! # DEX Error Handling
//!
//! Comprehensive error types and handling for the Moby DEX system,
//! covering all aspects of decentralized exchange operations.

use thiserror::Error;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Result type alias for DEX operations
pub type DEXResult<T> = Result<T, DEXError>;

/// Comprehensive error types for DEX operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum DEXError {
    // === AMM Errors ===
    #[error("Insufficient liquidity in pool {pool_id}: available {available}, required {required}")]
    InsufficientLiquidity {
        pool_id: String,
        available: Decimal,
        required: Decimal,
    },

    #[error("Pool {pool_id} not found")]
    PoolNotFound { pool_id: String },

    #[error("Invalid pool configuration: {reason}")]
    InvalidPoolConfig { reason: String },

    #[error("Pool {pool_id} is paused")]
    PoolPaused { pool_id: String },

    #[error("Slippage tolerance exceeded: expected {expected}%, actual {actual}%")]
    SlippageExceeded { expected: f64, actual: f64 },

    #[error("Price impact too high: {impact}% for trade size {trade_size}")]
    PriceImpactTooHigh { impact: f64, trade_size: Decimal },

    #[error("Minimum trade amount not met: {amount} < {minimum}")]
    MinimumTradeAmount { amount: Decimal, minimum: Decimal },

    #[error("Maximum trade amount exceeded: {amount} > {maximum}")]
    MaximumTradeAmount { amount: Decimal, maximum: Decimal },

    // === Order Book Errors ===
    #[error("Order {order_id} not found")]
    OrderNotFound { order_id: String },

    #[error("Invalid order: {reason}")]
    InvalidOrder { reason: String },

    #[error("Order already filled: {order_id}")]
    OrderAlreadyFilled { order_id: String },

    #[error("Order already cancelled: {order_id}")]
    OrderAlreadyCancelled { order_id: String },

    #[error("Insufficient balance for order: required {required}, available {available}")]
    InsufficientBalance { required: Decimal, available: Decimal },

    #[error("Order book {pair} not found")]
    OrderBookNotFound { pair: String },

    #[error("Invalid price: {price}")]
    InvalidPrice { price: Decimal },

    #[error("Invalid quantity: {quantity}")]
    InvalidQuantity { quantity: Decimal },

    // === Trading Errors ===
    #[error("Trading pair {pair} not supported")]
    UnsupportedTradingPair { pair: String },

    #[error("Trading is paused for pair {pair}")]
    TradingPaused { pair: String },

    #[error("Trade size {size} exceeds maximum whale limit {limit}")]
    WhaleTradeLimitExceeded { size: Decimal, limit: Decimal },

    #[error("Trade execution failed: {reason}")]
    TradeExecutionFailed { reason: String },

    #[error("Settlement failed: {reason}")]
    SettlementFailed { reason: String },

    #[error("Invalid trading strategy: {strategy}")]
    InvalidTradingStrategy { strategy: String },

    // === MEV Protection Errors ===
    #[error("MEV attack detected: {attack_type}")]
    MEVAttackDetected { attack_type: String },

    #[error("Sandwich attack prevented for trade {trade_id}")]
    SandwichAttackPrevented { trade_id: String },

    #[error("Front-running detected and blocked")]
    FrontRunningBlocked,

    #[error("MEV protection timeout: trade {trade_id}")]
    MEVProtectionTimeout { trade_id: String },

    #[error("Invalid MEV protection strategy: {strategy}")]
    InvalidMEVStrategy { strategy: String },

    // === Liquidity Errors ===
    #[error("Invalid liquidity position: {reason}")]
    InvalidLiquidityPosition { reason: String },

    #[error("Liquidity position {position_id} not found")]
    LiquidityPositionNotFound { position_id: String },

    #[error("Insufficient liquidity for withdrawal: available {available}, requested {requested}")]
    InsufficientLiquidityForWithdrawal { available: Decimal, requested: Decimal },

    #[error("Liquidity concentration ratio invalid: {ratio}")]
    InvalidConcentrationRatio { ratio: f64 },

    #[error("Price range invalid: lower {lower} >= upper {upper}")]
    InvalidPriceRange { lower: Decimal, upper: Decimal },

    #[error("Impermanent loss exceeds threshold: {loss}% > {threshold}%")]
    ImpermanentLossThreshold { loss: f64, threshold: f64 },

    // === Arbitrage Errors ===
    #[error("No arbitrage opportunity found for pair {pair}")]
    NoArbitrageOpportunity { pair: String },

    #[error("Arbitrage execution failed: {reason}")]
    ArbitrageExecutionFailed { reason: String },

    #[error("Arbitrage opportunity expired: {opportunity_id}")]
    ArbitrageOpportunityExpired { opportunity_id: String },

    #[error("Cross-exchange arbitrage failed: {reason}")]
    CrossExchangeArbitrageFailed { reason: String },

    #[error("Flash loan for arbitrage failed: {reason}")]
    FlashLoanFailed { reason: String },

    // === System Errors ===
    #[error("DEX system not initialized")]
    SystemNotInitialized,

    #[error("Configuration error: parameter {parameter} = {value}")]
    ConfigurationError { parameter: String, value: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Timeout error: operation {operation} timed out after {timeout_ms}ms")]
    TimeoutError { operation: String, timeout_ms: u64 },

    #[error("Rate limit exceeded: {limit} requests per {window_seconds} seconds")]
    RateLimitExceeded { limit: u32, window_seconds: u32 },

    // === Oracle Integration Errors ===
    #[error("Oracle error: {message}")]
    OracleError { message: String },

    #[error("Price feed unavailable for pair {pair}")]
    PriceFeedUnavailable { pair: String },

    #[error("Stale price data: age {age_seconds}s exceeds maximum {max_age_seconds}s")]
    StalePriceData { age_seconds: u64, max_age_seconds: u64 },

    #[error("Price deviation too high: {deviation}% exceeds threshold {threshold}%")]
    PriceDeviationTooHigh { deviation: f64, threshold: f64 },

    // === Bridge Integration Errors ===
    #[error("Cross-chain bridge error: {message}")]
    BridgeError { message: String },

    #[error("Unsupported chain: {chain_id}")]
    UnsupportedChain { chain_id: String },

    #[error("Bridge liquidity insufficient: {available} < {required}")]
    BridgeLiquidityInsufficient { available: Decimal, required: Decimal },

    #[error("Cross-chain settlement failed: {reason}")]
    CrossChainSettlementFailed { reason: String },

    // === Privacy Integration Errors ===
    #[error("Privacy proof generation failed: {reason}")]
    PrivacyProofFailed { reason: String },

    #[error("Zero-knowledge proof verification failed")]
    ZKProofVerificationFailed,

    #[error("Private transaction pool full")]
    PrivateTransactionPoolFull,

    #[error("Nullifier already used: {nullifier}")]
    NullifierAlreadyUsed { nullifier: String },

    // === Mathematical Errors ===
    #[error("Mathematical overflow in calculation: {operation}")]
    MathematicalOverflow { operation: String },

    #[error("Division by zero in calculation: {context}")]
    DivisionByZero { context: String },

    #[error("Square root of negative number: {value}")]
    NegativeSquareRoot { value: Decimal },

    #[error("Invalid mathematical parameters: {details}")]
    InvalidMathematicalParameters { details: String },

    // === Validation Errors ===
    #[error("Invalid token address: {address}")]
    InvalidTokenAddress { address: String },

    #[error("Invalid signature: {reason}")]
    InvalidSignature { reason: String },

    #[error("Invalid timestamp: {timestamp}")]
    InvalidTimestamp { timestamp: i64 },

    #[error("Invalid fee: {fee} must be between 0 and {max_fee}")]
    InvalidFee { fee: f64, max_fee: f64 },

    #[error("Invalid deadline: {deadline} is in the past")]
    InvalidDeadline { deadline: chrono::DateTime<chrono::Utc> },

    // === Gas and Fee Errors ===
    #[error("Gas estimation failed: {reason}")]
    GasEstimationFailed { reason: String },

    #[error("Gas price too high: {gas_price} > {max_gas_price}")]
    GasPriceTooHigh { gas_price: u64, max_gas_price: u64 },

    #[error("Transaction fee too high: {fee} > {max_fee}")]
    TransactionFeeTooHigh { fee: Decimal, max_fee: Decimal },

    #[error("Insufficient gas: required {required}, available {available}")]
    InsufficientGas { required: u64, available: u64 },

    // === Compliance Errors ===
    #[error("Address {address} is blacklisted")]
    BlacklistedAddress { address: String },

    #[error("Trade amount {amount} exceeds compliance limit {limit}")]
    ComplianceLimitExceeded { amount: Decimal, limit: Decimal },

    #[error("KYC verification required for trade size {size}")]
    KYCRequired { size: Decimal },

    #[error("Geographic restriction: trading not allowed from {region}")]
    GeographicRestriction { region: String },

    #[error("Sanctions check failed for address {address}")]
    SanctionsCheckFailed { address: String },

    // === Performance Errors ===
    #[error("System overloaded: {current_load}% > {max_load}%")]
    SystemOverloaded { current_load: f64, max_load: f64 },

    #[error("Memory limit exceeded: {used_mb}MB > {limit_mb}MB")]
    MemoryLimitExceeded { used_mb: u64, limit_mb: u64 },

    #[error("Connection pool exhausted: {active_connections}/{max_connections}")]
    ConnectionPoolExhausted { active_connections: u32, max_connections: u32 },

    #[error("Request queue full: {queue_size}/{max_queue_size}")]
    RequestQueueFull { queue_size: u32, max_queue_size: u32 },

    // === Generic Errors ===
    #[error("Internal error: {message}")]
    InternalError { message: String },

    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Resource locked: {resource}")]
    ResourceLocked { resource: String },

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
}

impl DEXError {
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical errors that require immediate attention
            DEXError::MEVAttackDetected { .. } |
            DEXError::SandwichAttackPrevented { .. } |
            DEXError::FrontRunningBlocked |
            DEXError::SystemOverloaded { .. } |
            DEXError::DatabaseError { .. } => ErrorSeverity::Critical,

            // High severity errors that affect trading
            DEXError::InsufficientLiquidity { .. } |
            DEXError::SlippageExceeded { .. } |
            DEXError::PriceImpactTooHigh { .. } |
            DEXError::TradeExecutionFailed { .. } |
            DEXError::SettlementFailed { .. } |
            DEXError::WhaleTradeLimitExceeded { .. } => ErrorSeverity::High,

            // Medium severity errors
            DEXError::OrderNotFound { .. } |
            DEXError::InvalidOrder { .. } |
            DEXError::TradingPaused { .. } |
            DEXError::PriceFeedUnavailable { .. } |
            DEXError::StalePriceData { .. } => ErrorSeverity::Medium,

            // Low severity errors
            DEXError::MinimumTradeAmount { .. } |
            DEXError::InvalidPrice { .. } |
            DEXError::InvalidQuantity { .. } |
            DEXError::UnsupportedTradingPair { .. } => ErrorSeverity::Low,

            // Info level
            DEXError::NoArbitrageOpportunity { .. } |
            DEXError::ArbitrageOpportunityExpired { .. } => ErrorSeverity::Info,
        }
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            DEXError::InsufficientLiquidity { .. } |
            DEXError::PoolNotFound { .. } |
            DEXError::InvalidPoolConfig { .. } |
            DEXError::PoolPaused { .. } |
            DEXError::SlippageExceeded { .. } |
            DEXError::PriceImpactTooHigh { .. } => ErrorCategory::AMM,

            DEXError::OrderNotFound { .. } |
            DEXError::InvalidOrder { .. } |
            DEXError::OrderAlreadyFilled { .. } |
            DEXError::OrderAlreadyCancelled { .. } |
            DEXError::OrderBookNotFound { .. } => ErrorCategory::OrderBook,

            DEXError::UnsupportedTradingPair { .. } |
            DEXError::TradingPaused { .. } |
            DEXError::WhaleTradeLimitExceeded { .. } |
            DEXError::TradeExecutionFailed { .. } |
            DEXError::SettlementFailed { .. } => ErrorCategory::Trading,

            DEXError::MEVAttackDetected { .. } |
            DEXError::SandwichAttackPrevented { .. } |
            DEXError::FrontRunningBlocked |
            DEXError::MEVProtectionTimeout { .. } => ErrorCategory::MEVProtection,

            DEXError::InvalidLiquidityPosition { .. } |
            DEXError::LiquidityPositionNotFound { .. } |
            DEXError::InsufficientLiquidityForWithdrawal { .. } |
            DEXError::InvalidConcentrationRatio { .. } => ErrorCategory::Liquidity,

            DEXError::NoArbitrageOpportunity { .. } |
            DEXError::ArbitrageExecutionFailed { .. } |
            DEXError::ArbitrageOpportunityExpired { .. } |
            DEXError::FlashLoanFailed { .. } => ErrorCategory::Arbitrage,

            DEXError::OracleError { .. } |
            DEXError::PriceFeedUnavailable { .. } |
            DEXError::StalePriceData { .. } |
            DEXError::PriceDeviationTooHigh { .. } => ErrorCategory::Oracle,

            DEXError::BridgeError { .. } |
            DEXError::UnsupportedChain { .. } |
            DEXError::BridgeLiquidityInsufficient { .. } |
            DEXError::CrossChainSettlementFailed { .. } => ErrorCategory::Bridge,

            DEXError::PrivacyProofFailed { .. } |
            DEXError::ZKProofVerificationFailed |
            DEXError::PrivateTransactionPoolFull |
            DEXError::NullifierAlreadyUsed { .. } => ErrorCategory::Privacy,

            DEXError::MathematicalOverflow { .. } |
            DEXError::DivisionByZero { .. } |
            DEXError::NegativeSquareRoot { .. } |
            DEXError::InvalidMathematicalParameters { .. } => ErrorCategory::Mathematical,

            DEXError::BlacklistedAddress { .. } |
            DEXError::ComplianceLimitExceeded { .. } |
            DEXError::KYCRequired { .. } |
            DEXError::GeographicRestriction { .. } |
            DEXError::SanctionsCheckFailed { .. } => ErrorCategory::Compliance,

            _ => ErrorCategory::System,
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Non-recoverable errors
            DEXError::SystemNotInitialized |
            DEXError::InvalidPoolConfig { .. } |
            DEXError::UnsupportedTradingPair { .. } |
            DEXError::InvalidSignature { .. } |
            DEXError::BlacklistedAddress { .. } => false,

            // Potentially recoverable errors
            DEXError::InsufficientLiquidity { .. } |
            DEXError::SlippageExceeded { .. } |
            DEXError::PriceImpactTooHigh { .. } |
            DEXError::NetworkError { .. } |
            DEXError::TimeoutError { .. } |
            DEXError::RateLimitExceeded { .. } => true,

            // Context-dependent
            _ => true,
        }
    }

    /// Get suggested retry delay in milliseconds
    pub fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            DEXError::RateLimitExceeded { window_seconds, .. } => {
                Some(*window_seconds as u64 * 1000)
            }
            DEXError::NetworkError { .. } => Some(1000),
            DEXError::TimeoutError { .. } => Some(500),
            DEXError::SystemOverloaded { .. } => Some(5000),
            DEXError::InsufficientLiquidity { .. } => Some(2000),
            _ => None,
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            DEXError::InsufficientLiquidity { .. } => "INSUFFICIENT_LIQUIDITY",
            DEXError::PoolNotFound { .. } => "POOL_NOT_FOUND",
            DEXError::SlippageExceeded { .. } => "SLIPPAGE_EXCEEDED",
            DEXError::PriceImpactTooHigh { .. } => "PRICE_IMPACT_TOO_HIGH",
            DEXError::OrderNotFound { .. } => "ORDER_NOT_FOUND",
            DEXError::InvalidOrder { .. } => "INVALID_ORDER",
            DEXError::TradeExecutionFailed { .. } => "TRADE_EXECUTION_FAILED",
            DEXError::MEVAttackDetected { .. } => "MEV_ATTACK_DETECTED",
            DEXError::SandwichAttackPrevented { .. } => "SANDWICH_ATTACK_PREVENTED",
            DEXError::InsufficientBalance { .. } => "INSUFFICIENT_BALANCE",
            DEXError::UnsupportedTradingPair { .. } => "UNSUPPORTED_TRADING_PAIR",
            DEXError::TradingPaused { .. } => "TRADING_PAUSED",
            DEXError::SystemOverloaded { .. } => "SYSTEM_OVERLOADED",
            DEXError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            DEXError::BlacklistedAddress { .. } => "BLACKLISTED_ADDRESS",
            DEXError::ComplianceLimitExceeded { .. } => "COMPLIANCE_LIMIT_EXCEEDED",
            _ => "INTERNAL_ERROR",
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Error categories for organization and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    AMM,
    OrderBook,
    Trading,
    MEVProtection,
    Liquidity,
    Arbitrage,
    Oracle,
    Bridge,
    Privacy,
    Mathematical,
    Compliance,
    System,
}

/// Error context for debugging and logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error: DEXError,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub user_id: Option<String>,
    pub trade_id: Option<String>,
    pub pool_id: Option<String>,
    pub order_id: Option<String>,
    pub additional_data: std::collections::HashMap<String, serde_json::Value>,
}

impl ErrorContext {
    pub fn new(error: DEXError, operation: String) -> Self {
        Self {
            error,
            timestamp: chrono::Utc::now(),
            operation,
            user_id: None,
            trade_id: None,
            pool_id: None,
            order_id: None,
            additional_data: std::collections::HashMap::new(),
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_trade_id(mut self, trade_id: String) -> Self {
        self.trade_id = Some(trade_id);
        self
    }

    pub fn with_pool_id(mut self, pool_id: String) -> Self {
        self.pool_id = Some(pool_id);
        self
    }

    pub fn with_order_id(mut self, order_id: String) -> Self {
        self.order_id = Some(order_id);
        self
    }

    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.additional_data.insert(key, value);
        self
    }
}

// Integration with external error types
impl From<moby_oracle::OracleError> for DEXError {
    fn from(err: moby_oracle::OracleError) -> Self {
        DEXError::OracleError {
            message: err.to_string(),
        }
    }
}

impl From<sqlx::Error> for DEXError {
    fn from(err: sqlx::Error) -> Self {
        DEXError::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for DEXError {
    fn from(err: serde_json::Error) -> Self {
        DEXError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<tokio::time::error::Elapsed> for DEXError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        DEXError::TimeoutError {
            operation: "unknown".to_string(),
            timeout_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let error = DEXError::MEVAttackDetected {
            attack_type: "sandwich".to_string(),
        };
        assert_eq!(error.severity(), ErrorSeverity::Critical);

        let error = DEXError::MinimumTradeAmount {
            amount: Decimal::from(10),
            minimum: Decimal::from(100),
        };
        assert_eq!(error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_category() {
        let error = DEXError::InsufficientLiquidity {
            pool_id: "test".to_string(),
            available: Decimal::from(100),
            required: Decimal::from(1000),
        };
        assert_eq!(error.category(), ErrorCategory::AMM);

        let error = DEXError::OrderNotFound {
            order_id: "test".to_string(),
        };
        assert_eq!(error.category(), ErrorCategory::OrderBook);
    }

    #[test]
    fn test_error_recoverability() {
        let error = DEXError::InsufficientLiquidity {
            pool_id: "test".to_string(),
            available: Decimal::from(100),
            required: Decimal::from(1000),
        };
        assert!(error.is_recoverable());

        let error = DEXError::BlacklistedAddress {
            address: "0x123".to_string(),
        };
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_error_code() {
        let error = DEXError::SlippageExceeded {
            expected: 1.0,
            actual: 5.0,
        };
        assert_eq!(error.error_code(), "SLIPPAGE_EXCEEDED");
    }

    #[test]
    fn test_error_context() {
        let error = DEXError::TradeExecutionFailed {
            reason: "test".to_string(),
        };
        let context = ErrorContext::new(error, "execute_trade".to_string())
            .with_user_id("user123".to_string())
            .with_trade_id("trade456".to_string());

        assert_eq!(context.operation, "execute_trade");
        assert_eq!(context.user_id, Some("user123".to_string()));
        assert_eq!(context.trade_id, Some("trade456".to_string()));
    }
}