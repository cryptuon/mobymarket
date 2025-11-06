//! Error types and handling for the Moby Oracle system.
//!
//! This module provides comprehensive error handling for oracle operations,
//! data source failures, aggregation issues, and security violations.

use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Result type alias for oracle operations
pub type OracleResult<T> = Result<T, OracleError>;

/// Comprehensive error types for oracle operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum OracleError {
    // Data source errors
    #[error("Data source unavailable: {source} - {reason}")]
    DataSourceUnavailable {
        source: String,
        reason: String,
        retry_after: Option<DateTime<Utc>>,
    },

    #[error("Data source timeout: {source} after {timeout_ms}ms")]
    DataSourceTimeout {
        source: String,
        timeout_ms: u64,
        last_successful: Option<DateTime<Utc>>,
    },

    #[error("Invalid data from source: {source} - {details}")]
    InvalidData {
        source: String,
        details: String,
        raw_data: Option<String>,
    },

    #[error("Authentication failed for source: {source}")]
    AuthenticationFailed {
        source: String,
        error_code: Option<String>,
    },

    #[error("Rate limit exceeded for source: {source}")]
    RateLimitExceeded {
        source: String,
        reset_time: Option<DateTime<Utc>>,
        retry_after: u64,
    },

    // Price feed errors
    #[error("Price feed not found: {feed_id}")]
    PriceFeedNotFound {
        feed_id: String,
        available_feeds: Vec<String>,
    },

    #[error("Price feed configuration invalid: {reason}")]
    InvalidFeedConfig {
        reason: String,
        field: Option<String>,
    },

    #[error("Price data stale: {symbol} - last update {last_update}")]
    StaleData {
        symbol: String,
        last_update: DateTime<Utc>,
        max_age_seconds: u64,
    },

    #[error("Price deviation too high: {symbol} - {deviation}% (max: {max_deviation}%)")]
    ExcessiveDeviation {
        symbol: String,
        current_price: f64,
        expected_price: f64,
        deviation: f64,
        max_deviation: f64,
    },

    #[error("Insufficient data sources: {symbol} - {available}/{required} sources")]
    InsufficientSources {
        symbol: String,
        available: usize,
        required: usize,
        failed_sources: Vec<String>,
    },

    // Aggregation errors
    #[error("Aggregation failed: {strategy} - {reason}")]
    AggregationFailed {
        strategy: String,
        reason: String,
        data_points: usize,
    },

    #[error("Aggregation timeout: {symbol} after {timeout_ms}ms")]
    AggregationTimeout {
        symbol: String,
        timeout_ms: u64,
        partial_results: usize,
    },

    #[error("Consensus not reached: {symbol} - {agreement}% agreement (min: {min_consensus}%)")]
    ConsensusNotReached {
        symbol: String,
        agreement: f64,
        min_consensus: f64,
        conflicting_sources: Vec<String>,
    },

    // Security errors
    #[error("Data integrity violation: {details}")]
    DataIntegrityViolation {
        details: String,
        source: Option<String>,
        hash_mismatch: bool,
    },

    #[error("Signature verification failed: {source}")]
    SignatureVerificationFailed {
        source: String,
        expected_signer: Option<String>,
    },

    #[error("Anomaly detected: {symbol} - {anomaly_type}")]
    AnomalyDetected {
        symbol: String,
        anomaly_type: String,
        severity: AnomalySeverity,
        details: HashMap<String, String>,
    },

    #[error("Manipulation attempt detected: {symbol} - {pattern}")]
    ManipulationDetected {
        symbol: String,
        pattern: String,
        confidence: f64,
        evidence: Vec<String>,
    },

    // Storage errors
    #[error("Storage operation failed: {operation} - {reason}")]
    StorageFailed {
        operation: String,
        reason: String,
        recoverable: bool,
    },

    #[error("Database connection failed: {database} - {error}")]
    DatabaseConnectionFailed {
        database: String,
        error: String,
        retry_count: u32,
    },

    #[error("Data query failed: {query} - {error}")]
    QueryFailed {
        query: String,
        error: String,
        parameters: HashMap<String, String>,
    },

    #[error("Storage capacity exceeded: {used}/{capacity} bytes")]
    StorageCapacityExceeded {
        used: u64,
        capacity: u64,
        oldest_data: Option<DateTime<Utc>>,
    },

    // Governance errors
    #[error("Governance proposal invalid: {proposal_id} - {reason}")]
    InvalidGovernanceProposal {
        proposal_id: String,
        reason: String,
        violations: Vec<String>,
    },

    #[error("Voting period expired: {proposal_id}")]
    VotingPeriodExpired {
        proposal_id: String,
        expired_at: DateTime<Utc>,
    },

    #[error("Insufficient voting power: {voter} - {power}/{required}")]
    InsufficientVotingPower {
        voter: String,
        power: u64,
        required: u64,
    },

    #[error("Governance unauthorized: {action} - {reason}")]
    GovernanceUnauthorized {
        action: String,
        reason: String,
        required_role: Option<String>,
    },

    // System errors
    #[error("Oracle system not initialized")]
    SystemNotInitialized,

    #[error("Oracle system shutting down")]
    SystemShuttingDown {
        shutdown_reason: String,
        estimated_downtime: Option<u64>,
    },

    #[error("Resource exhausted: {resource} - {current}/{limit}")]
    ResourceExhausted {
        resource: String,
        current: u64,
        limit: u64,
        recovery_suggestion: Option<String>,
    },

    #[error("Configuration error: {parameter} - {reason}")]
    ConfigurationError {
        parameter: String,
        reason: String,
        valid_range: Option<String>,
    },

    #[error("Service dependency unavailable: {service}")]
    ServiceUnavailable {
        service: String,
        last_ping: Option<DateTime<Utc>>,
        estimated_recovery: Option<DateTime<Utc>>,
    },

    // Network errors
    #[error("Network error: {operation} - {error}")]
    NetworkError {
        operation: String,
        error: String,
        endpoint: Option<String>,
    },

    #[error("WebSocket connection failed: {endpoint} - {reason}")]
    WebSocketConnectionFailed {
        endpoint: String,
        reason: String,
        reconnect_attempts: u32,
    },

    #[error("API request failed: {url} - HTTP {status}")]
    ApiRequestFailed {
        url: String,
        status: u16,
        response_body: Option<String>,
        retry_after: Option<u64>,
    },

    // Validation errors
    #[error("Symbol validation failed: {symbol} - {reason}")]
    InvalidSymbol {
        symbol: String,
        reason: String,
        suggestions: Vec<String>,
    },

    #[error("Price validation failed: {price} - {reason}")]
    InvalidPrice {
        price: f64,
        reason: String,
        valid_range: Option<(f64, f64)>,
    },

    #[error("Timestamp validation failed: {timestamp} - {reason}")]
    InvalidTimestamp {
        timestamp: DateTime<Utc>,
        reason: String,
        acceptable_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    },

    // Whale trading specific errors
    #[error("Whale trading threshold not met: {volume} < {threshold}")]
    WhaleThresholdNotMet {
        volume: f64,
        threshold: f64,
        symbol: String,
    },

    #[error("Large order impact detected: {symbol} - {impact}% price impact")]
    LargeOrderImpact {
        symbol: String,
        order_size: f64,
        impact: f64,
        max_impact: f64,
    },

    #[error("Liquidity insufficient for whale trade: {symbol} - {available}/{required}")]
    InsufficientLiquidity {
        symbol: String,
        available: f64,
        required: f64,
        exchanges: Vec<String>,
    },

    // Generic errors
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        error_code: Option<String>,
        context: HashMap<String, String>,
    },

    #[error("Temporary failure: {operation} - retry recommended")]
    TemporaryFailure {
        operation: String,
        retry_after_seconds: u64,
        max_retries: Option<u32>,
    },

    #[error("Feature not implemented: {feature}")]
    NotImplemented {
        feature: String,
        planned_version: Option<String>,
    },

    #[error("Operation cancelled: {operation}")]
    Cancelled {
        operation: String,
        reason: Option<String>,
    },
}

/// Severity levels for anomalies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl OracleError {
    /// Create a data source unavailable error
    pub fn data_source_unavailable(source: &str, reason: &str) -> Self {
        Self::DataSourceUnavailable {
            source: source.to_string(),
            reason: reason.to_string(),
            retry_after: None,
        }
    }

    /// Create a data source timeout error
    pub fn data_source_timeout(source: &str, timeout_ms: u64) -> Self {
        Self::DataSourceTimeout {
            source: source.to_string(),
            timeout_ms,
            last_successful: None,
        }
    }

    /// Create an invalid data error
    pub fn invalid_data(source: &str, details: &str) -> Self {
        Self::InvalidData {
            source: source.to_string(),
            details: details.to_string(),
            raw_data: None,
        }
    }

    /// Create a price feed not found error
    pub fn price_feed_not_found(feed_id: &str) -> Self {
        Self::PriceFeedNotFound {
            feed_id: feed_id.to_string(),
            available_feeds: Vec::new(),
        }
    }

    /// Create an aggregation failed error
    pub fn aggregation_failed(strategy: &str, reason: &str) -> Self {
        Self::AggregationFailed {
            strategy: strategy.to_string(),
            reason: reason.to_string(),
            data_points: 0,
        }
    }

    /// Create a configuration error
    pub fn configuration_error(parameter: &str, reason: &str) -> Self {
        Self::ConfigurationError {
            parameter: parameter.to_string(),
            reason: reason.to_string(),
            valid_range: None,
        }
    }

    /// Create an internal error
    pub fn internal(message: &str) -> Self {
        Self::Internal {
            message: message.to_string(),
            error_code: None,
            context: HashMap::new(),
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::DataSourceTimeout { .. } => true,
            Self::DataSourceUnavailable { .. } => true,
            Self::RateLimitExceeded { .. } => true,
            Self::AggregationTimeout { .. } => true,
            Self::NetworkError { .. } => true,
            Self::WebSocketConnectionFailed { .. } => true,
            Self::ApiRequestFailed { status, .. } => *status >= 500,
            Self::DatabaseConnectionFailed { .. } => true,
            Self::ServiceUnavailable { .. } => true,
            Self::TemporaryFailure { .. } => true,
            _ => false,
        }
    }

    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::DataSourceUnavailable { .. } |
            Self::DataSourceTimeout { .. } |
            Self::InvalidData { .. } |
            Self::AuthenticationFailed { .. } |
            Self::RateLimitExceeded { .. } => ErrorCategory::DataSource,

            Self::PriceFeedNotFound { .. } |
            Self::InvalidFeedConfig { .. } |
            Self::StaleData { .. } |
            Self::ExcessiveDeviation { .. } |
            Self::InsufficientSources { .. } => ErrorCategory::PriceFeed,

            Self::AggregationFailed { .. } |
            Self::AggregationTimeout { .. } |
            Self::ConsensusNotReached { .. } => ErrorCategory::Aggregation,

            Self::DataIntegrityViolation { .. } |
            Self::SignatureVerificationFailed { .. } |
            Self::AnomalyDetected { .. } |
            Self::ManipulationDetected { .. } => ErrorCategory::Security,

            Self::StorageFailed { .. } |
            Self::DatabaseConnectionFailed { .. } |
            Self::QueryFailed { .. } |
            Self::StorageCapacityExceeded { .. } => ErrorCategory::Storage,

            Self::InvalidGovernanceProposal { .. } |
            Self::VotingPeriodExpired { .. } |
            Self::InsufficientVotingPower { .. } |
            Self::GovernanceUnauthorized { .. } => ErrorCategory::Governance,

            Self::NetworkError { .. } |
            Self::WebSocketConnectionFailed { .. } |
            Self::ApiRequestFailed { .. } => ErrorCategory::Network,

            Self::InvalidSymbol { .. } |
            Self::InvalidPrice { .. } |
            Self::InvalidTimestamp { .. } => ErrorCategory::Validation,

            Self::WhaleThresholdNotMet { .. } |
            Self::LargeOrderImpact { .. } |
            Self::InsufficientLiquidity { .. } => ErrorCategory::WhaleTrading,

            Self::SystemNotInitialized |
            Self::SystemShuttingDown { .. } |
            Self::ResourceExhausted { .. } |
            Self::ConfigurationError { .. } |
            Self::ServiceUnavailable { .. } => ErrorCategory::System,

            Self::Internal { .. } |
            Self::TemporaryFailure { .. } |
            Self::NotImplemented { .. } |
            Self::Cancelled { .. } => ErrorCategory::Generic,
        }
    }

    /// Get the error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::DataIntegrityViolation { .. } |
            Self::ManipulationDetected { .. } |
            Self::SystemShuttingDown { .. } => ErrorSeverity::Critical,

            Self::ExcessiveDeviation { .. } |
            Self::AnomalyDetected { anomaly_type: _, severity, .. } => {
                match severity {
                    AnomalySeverity::Critical => ErrorSeverity::Critical,
                    AnomalySeverity::High => ErrorSeverity::High,
                    AnomalySeverity::Medium => ErrorSeverity::Medium,
                    AnomalySeverity::Low => ErrorSeverity::Low,
                }
            },

            Self::ConsensusNotReached { .. } |
            Self::InsufficientSources { .. } |
            Self::SignatureVerificationFailed { .. } => ErrorSeverity::High,

            Self::StaleData { .. } |
            Self::AggregationFailed { .. } |
            Self::PriceFeedNotFound { .. } => ErrorSeverity::Medium,

            Self::DataSourceTimeout { .. } |
            Self::RateLimitExceeded { .. } |
            Self::TemporaryFailure { .. } => ErrorSeverity::Low,

            _ => ErrorSeverity::Medium,
        }
    }

    /// Get recovery suggestions
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::DataSourceUnavailable { .. } => vec![
                "Check data source connectivity".to_string(),
                "Try alternative data sources".to_string(),
                "Wait for service recovery".to_string(),
            ],
            Self::RateLimitExceeded { .. } => vec![
                "Reduce request frequency".to_string(),
                "Wait for rate limit reset".to_string(),
                "Use cached data if available".to_string(),
            ],
            Self::InsufficientSources { .. } => vec![
                "Add more data sources".to_string(),
                "Lower minimum source requirements".to_string(),
                "Check source configurations".to_string(),
            ],
            Self::ExcessiveDeviation { .. } => vec![
                "Verify data source accuracy".to_string(),
                "Check for market volatility".to_string(),
                "Adjust deviation thresholds".to_string(),
            ],
            Self::AggregationTimeout { .. } => vec![
                "Increase aggregation timeout".to_string(),
                "Optimize data source performance".to_string(),
                "Use cached aggregated data".to_string(),
            ],
            _ => vec!["Contact system administrator".to_string()],
        }
    }

    /// Get additional context information
    pub fn context(&self) -> HashMap<String, String> {
        match self {
            Self::Internal { context, .. } => context.clone(),
            _ => HashMap::new(),
        }
    }
}

/// Error categories for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    DataSource,
    PriceFeed,
    Aggregation,
    Security,
    Storage,
    Governance,
    Network,
    Validation,
    WhaleTrading,
    System,
    Generic,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error context for debugging and recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,
    /// Operation that failed
    pub operation: String,
    /// Additional context data
    pub context: HashMap<String, String>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(operation: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: operation.to_string(),
            context: HashMap::new(),
            stack_trace: None,
        }
    }

    /// Add context information
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
}

/// Error categories for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    DataSource,
    PriceFeed,
    Aggregation,
    Security,
    Storage,
    Governance,
    Network,
    Validation,
    WhaleTrading,
    System,
    Generic,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = OracleError::data_source_unavailable("chainlink", "network timeout");
        assert!(matches!(error, OracleError::DataSourceUnavailable { .. }));
        assert_eq!(error.category(), ErrorCategory::DataSource);
    }

    #[test]
    fn test_error_retryable() {
        let timeout_error = OracleError::data_source_timeout("pyth", 5000);
        assert!(timeout_error.is_retryable());

        let config_error = OracleError::configuration_error("invalid_param", "out of range");
        assert!(!config_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let manipulation_error = OracleError::ManipulationDetected {
            symbol: "ETH/USD".to_string(),
            pattern: "price_spike".to_string(),
            confidence: 0.95,
            evidence: vec!["unusual_volume".to_string()],
        };
        assert_eq!(manipulation_error.severity(), ErrorSeverity::Critical);

        let timeout_error = OracleError::data_source_timeout("source", 1000);
        assert_eq!(timeout_error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("price_aggregation")
            .with_context("symbol", "BTC/USD")
            .with_context("sources", "3");

        assert_eq!(context.operation, "price_aggregation");
        assert_eq!(context.context.get("symbol").unwrap(), "BTC/USD");
    }

    #[test]
    fn test_recovery_suggestions() {
        let rate_limit_error = OracleError::RateLimitExceeded {
            source: "api".to_string(),
            reset_time: None,
            retry_after: 60,
        };

        let suggestions = rate_limit_error.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("rate limit")));
    }

    #[test]
    fn test_anomaly_severity() {
        let high_anomaly = OracleError::AnomalyDetected {
            symbol: "ETH/USD".to_string(),
            anomaly_type: "price_spike".to_string(),
            severity: AnomalySeverity::High,
            details: HashMap::new(),
        };

        assert_eq!(high_anomaly.severity(), ErrorSeverity::High);
    }
}