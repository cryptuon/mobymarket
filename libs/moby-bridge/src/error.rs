//! Error types for the bridge system

use thiserror::Error;

/// Result type for bridge operations
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Comprehensive error types for bridge operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BridgeError {
    // Chain errors
    #[error("Chain not supported: {chain}")]
    ChainNotSupported { chain: String },

    #[error("Chain not available: {chain}")]
    ChainNotAvailable { chain: String },

    #[error("Chain configuration error: {chain} - {message}")]
    ChainConfigurationError { chain: String, message: String },

    #[error("Chain connection failed: {chain}")]
    ChainConnectionFailed { chain: String },

    #[error("Chain synchronization error: {chain}")]
    ChainSyncError { chain: String },

    // Transfer errors
    #[error("Transfer not found: {id}")]
    TransferNotFound { id: String },

    #[error("Transfer already exists: {id}")]
    TransferAlreadyExists { id: String },

    #[error("Transfer amount too large: {amount} exceeds limit {limit}")]
    TransferAmountTooLarge { amount: u64, limit: u64 },

    #[error("Transfer amount too small: {amount} below minimum {minimum}")]
    TransferAmountTooSmall { amount: u64, minimum: u64 },

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },

    #[error("Transfer timeout: {id}")]
    TransferTimeout { id: String },

    #[error("Transfer failed: {id} - {reason}")]
    TransferFailed { id: String, reason: String },

    // Token errors
    #[error("Token not supported: {token} on chain {chain}")]
    TokenNotSupported { token: String, chain: String },

    #[error("Token mapping not found: {token} from {from_chain} to {to_chain}")]
    TokenMappingNotFound { token: String, from_chain: String, to_chain: String },

    #[error("Token decimals mismatch: {token} - {from_decimals} vs {to_decimals}")]
    TokenDecimalsMismatch { token: String, from_decimals: u8, to_decimals: u8 },

    #[error("Token contract error: {token} - {message}")]
    TokenContractError { token: String, message: String },

    // Liquidity errors
    #[error("Insufficient liquidity: required {required}, available {available}")]
    InsufficientLiquidity { required: u64, available: u64 },

    #[error("Liquidity pool not found: {pool_id}")]
    LiquidityPoolNotFound { pool_id: String },

    #[error("Liquidity provider not found: {provider}")]
    LiquidityProviderNotFound { provider: String },

    #[error("Route not found: from {from_chain} to {to_chain}")]
    RouteNotFound { from_chain: String, to_chain: String },

    #[error("Route optimization failed: {reason}")]
    RouteOptimizationFailed { reason: String },

    // Validation errors
    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Invalid signature: {signer}")]
    InvalidSignature { signer: String },

    #[error("Insufficient validators: required {required}, available {available}")]
    InsufficientValidators { required: usize, available: usize },

    #[error("Consensus not reached: {votes_for}/{total_votes}")]
    ConsensusNotReached { votes_for: usize, total_votes: usize },

    #[error("Validator not found: {validator}")]
    ValidatorNotFound { validator: String },

    #[error("Validator slashed: {validator}")]
    ValidatorSlashed { validator: String },

    // Security errors
    #[error("Security check failed: {check}")]
    SecurityCheckFailed { check: String },

    #[error("Fraud detected: {details}")]
    FraudDetected { details: String },

    #[error("Rate limit exceeded: {limit} per {period}")]
    RateLimitExceeded { limit: u64, period: String },

    #[error("Blacklisted address: {address}")]
    BlacklistedAddress { address: String },

    #[error("Suspicious activity detected: {reason}")]
    SuspiciousActivity { reason: String },

    // Privacy errors
    #[error("Privacy proof generation failed: {reason}")]
    PrivacyProofFailed { reason: String },

    #[error("Privacy proof verification failed")]
    PrivacyProofVerificationFailed,

    #[error("Privacy level not supported: {level}")]
    PrivacyLevelNotSupported { level: String },

    #[error("ZK circuit error: {message}")]
    ZkCircuitError { message: String },

    // Relayer errors
    #[error("Relayer not found: {relayer}")]
    RelayerNotFound { relayer: String },

    #[error("Relayer not available: {relayer}")]
    RelayerNotAvailable { relayer: String },

    #[error("Relayer bond insufficient: {relayer}")]
    RelayerBondInsufficient { relayer: String },

    #[error("Relayer execution failed: {relayer} - {reason}")]
    RelayerExecutionFailed { relayer: String, reason: String },

    #[error("No available relayers for route: {from_chain} to {to_chain}")]
    NoAvailableRelayers { from_chain: String, to_chain: String },

    // Monitoring errors
    #[error("Health check failed: {component}")]
    HealthCheckFailed { component: String },

    #[error("Metrics collection failed: {metric}")]
    MetricsCollectionFailed { metric: String },

    #[error("Alert threshold exceeded: {metric} = {value} > {threshold}")]
    AlertThresholdExceeded { metric: String, value: f64, threshold: f64 },

    #[error("Monitoring system offline")]
    MonitoringSystemOffline,

    // Protocol errors
    #[error("Protocol error: {protocol} - {message}")]
    ProtocolError { protocol: String, message: String },

    #[error("Message parsing failed: {message}")]
    MessageParsingFailed { message: String },

    #[error("Message validation failed: {message}")]
    MessageValidationFailed { message: String },

    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolVersionMismatch { expected: String, actual: String },

    // System errors
    #[error("System configuration error: {message}")]
    SystemConfigurationError { message: String },

    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    // Emergency errors
    #[error("Emergency pause active")]
    EmergencyPauseActive,

    #[error("Circuit breaker triggered: {component}")]
    CircuitBreakerTriggered { component: String },

    #[error("Emergency action required: {action}")]
    EmergencyActionRequired { action: String },

    // Generic errors
    #[error("Operation failed: {reason}")]
    OperationFailed { reason: String },

    #[error("Timeout occurred: {operation}")]
    TimeoutOccurred { operation: String },

    #[error("Resource not available: {resource}")]
    ResourceNotAvailable { resource: String },

    #[error("Concurrent modification detected")]
    ConcurrentModificationDetected,

    #[error("Internal error: {message}")]
    InternalError { message: String },
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - warning
    Medium,
    /// High severity - error
    High,
    /// Critical severity - system failure
    Critical,
}

/// Error recovery strategies
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,
    /// Use alternative route/method
    Alternative,
    /// Fallback to safe mode
    Fallback,
    /// Manual intervention required
    Manual,
    /// Emergency procedures
    Emergency,
    /// No recovery possible
    None,
}

/// Error category for classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Chain-related errors
    Chain,
    /// Transfer-related errors
    Transfer,
    /// Token-related errors
    Token,
    /// Liquidity-related errors
    Liquidity,
    /// Validation-related errors
    Validation,
    /// Security-related errors
    Security,
    /// Privacy-related errors
    Privacy,
    /// Relayer-related errors
    Relayer,
    /// Monitoring-related errors
    Monitoring,
    /// Protocol-related errors
    Protocol,
    /// System-related errors
    System,
    /// Emergency-related errors
    Emergency,
}

impl BridgeError {
    /// Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical severity errors
            Self::EmergencyPauseActive |
            Self::CircuitBreakerTriggered { .. } |
            Self::FraudDetected { .. } |
            Self::SystemConfigurationError { .. } => ErrorSeverity::Critical,

            // High severity errors
            Self::ChainConnectionFailed { .. } |
            Self::TransferFailed { .. } |
            Self::ValidationFailed { .. } |
            Self::InsufficientLiquidity { .. } |
            Self::SecurityCheckFailed { .. } |
            Self::ConsensusNotReached { .. } => ErrorSeverity::High,

            // Medium severity errors
            Self::TransferTimeout { .. } |
            Self::RouteNotFound { .. } |
            Self::RelayerNotAvailable { .. } |
            Self::HealthCheckFailed { .. } |
            Self::RateLimitExceeded { .. } => ErrorSeverity::Medium,

            // Low severity errors (validation, not found, etc.)
            _ => ErrorSeverity::Low,
        }
    }

    /// Get the recovery strategy for the error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            // Retry-able errors
            Self::ChainConnectionFailed { .. } |
            Self::NetworkError { .. } |
            Self::TimeoutOccurred { .. } |
            Self::RelayerNotAvailable { .. } => RecoveryStrategy::Retry,

            // Alternative route errors
            Self::RouteNotFound { .. } |
            Self::InsufficientLiquidity { .. } |
            Self::RelayerExecutionFailed { .. } => RecoveryStrategy::Alternative,

            // Fallback-able errors
            Self::ChainNotAvailable { .. } |
            Self::ProtocolError { .. } |
            Self::MetricsCollectionFailed { .. } => RecoveryStrategy::Fallback,

            // Emergency procedures
            Self::FraudDetected { .. } |
            Self::SecurityCheckFailed { .. } |
            Self::SuspiciousActivity { .. } => RecoveryStrategy::Emergency,

            // Manual intervention required
            Self::ValidationFailed { .. } |
            Self::ConsensusNotReached { .. } |
            Self::SystemConfigurationError { .. } => RecoveryStrategy::Manual,

            // No recovery possible
            Self::TransferAlreadyExists { .. } |
            Self::TokenNotSupported { .. } |
            Self::BlacklistedAddress { .. } |
            Self::EmergencyPauseActive => RecoveryStrategy::None,

            // Most others can retry
            _ => RecoveryStrategy::Retry,
        }
    }

    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ChainNotSupported { .. } |
            Self::ChainNotAvailable { .. } |
            Self::ChainConfigurationError { .. } |
            Self::ChainConnectionFailed { .. } |
            Self::ChainSyncError { .. } => ErrorCategory::Chain,

            Self::TransferNotFound { .. } |
            Self::TransferAlreadyExists { .. } |
            Self::TransferAmountTooLarge { .. } |
            Self::TransferAmountTooSmall { .. } |
            Self::InsufficientBalance { .. } |
            Self::TransferTimeout { .. } |
            Self::TransferFailed { .. } => ErrorCategory::Transfer,

            Self::TokenNotSupported { .. } |
            Self::TokenMappingNotFound { .. } |
            Self::TokenDecimalsMismatch { .. } |
            Self::TokenContractError { .. } => ErrorCategory::Token,

            Self::InsufficientLiquidity { .. } |
            Self::LiquidityPoolNotFound { .. } |
            Self::LiquidityProviderNotFound { .. } |
            Self::RouteNotFound { .. } |
            Self::RouteOptimizationFailed { .. } => ErrorCategory::Liquidity,

            Self::ValidationFailed { .. } |
            Self::InvalidSignature { .. } |
            Self::InsufficientValidators { .. } |
            Self::ConsensusNotReached { .. } |
            Self::ValidatorNotFound { .. } |
            Self::ValidatorSlashed { .. } => ErrorCategory::Validation,

            Self::SecurityCheckFailed { .. } |
            Self::FraudDetected { .. } |
            Self::RateLimitExceeded { .. } |
            Self::BlacklistedAddress { .. } |
            Self::SuspiciousActivity { .. } => ErrorCategory::Security,

            Self::PrivacyProofFailed { .. } |
            Self::PrivacyProofVerificationFailed |
            Self::PrivacyLevelNotSupported { .. } |
            Self::ZkCircuitError { .. } => ErrorCategory::Privacy,

            Self::RelayerNotFound { .. } |
            Self::RelayerNotAvailable { .. } |
            Self::RelayerBondInsufficient { .. } |
            Self::RelayerExecutionFailed { .. } |
            Self::NoAvailableRelayers { .. } => ErrorCategory::Relayer,

            Self::HealthCheckFailed { .. } |
            Self::MetricsCollectionFailed { .. } |
            Self::AlertThresholdExceeded { .. } |
            Self::MonitoringSystemOffline => ErrorCategory::Monitoring,

            Self::ProtocolError { .. } |
            Self::MessageParsingFailed { .. } |
            Self::MessageValidationFailed { .. } |
            Self::ProtocolVersionMismatch { .. } => ErrorCategory::Protocol,

            Self::SystemConfigurationError { .. } |
            Self::StorageError { .. } |
            Self::NetworkError { .. } |
            Self::SerializationError { .. } |
            Self::DeserializationError { .. } |
            Self::OperationFailed { .. } |
            Self::TimeoutOccurred { .. } |
            Self::ResourceNotAvailable { .. } |
            Self::ConcurrentModificationDetected |
            Self::InternalError { .. } => ErrorCategory::System,

            Self::EmergencyPauseActive |
            Self::CircuitBreakerTriggered { .. } |
            Self::EmergencyActionRequired { .. } => ErrorCategory::Emergency,
        }
    }

    /// Check if the error indicates a security issue
    pub fn is_security_related(&self) -> bool {
        matches!(self.category(), ErrorCategory::Security | ErrorCategory::Emergency)
    }

    /// Check if the error affects bridge operations
    pub fn affects_bridge_operations(&self) -> bool {
        matches!(self.category(),
            ErrorCategory::Chain |
            ErrorCategory::Transfer |
            ErrorCategory::Liquidity |
            ErrorCategory::Validation |
            ErrorCategory::Emergency
        )
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !matches!(self.recovery_strategy(),
            RecoveryStrategy::Manual |
            RecoveryStrategy::Emergency |
            RecoveryStrategy::None
        )
    }

    /// Check if immediate action is required
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Critical) ||
        matches!(self.recovery_strategy(), RecoveryStrategy::Emergency)
    }
}

impl From<serde_json::Error> for BridgeError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<bincode::Error> for BridgeError {
    fn from(error: bincode::Error) -> Self {
        Self::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for BridgeError {
    fn from(error: std::io::Error) -> Self {
        Self::StorageError {
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for BridgeError {
    fn from(error: reqwest::Error) -> Self {
        Self::NetworkError {
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        assert_eq!(
            BridgeError::TransferNotFound { id: "test".to_string() }.severity(),
            ErrorSeverity::Low
        );

        assert_eq!(
            BridgeError::TransferFailed { id: "test".to_string(), reason: "test".to_string() }.severity(),
            ErrorSeverity::High
        );

        assert_eq!(
            BridgeError::FraudDetected { details: "test".to_string() }.severity(),
            ErrorSeverity::Critical
        );
    }

    #[test]
    fn test_recovery_strategy() {
        assert_eq!(
            BridgeError::NetworkError { message: "timeout".to_string() }.recovery_strategy(),
            RecoveryStrategy::Retry
        );

        assert_eq!(
            BridgeError::RouteNotFound { from_chain: "eth".to_string(), to_chain: "sol".to_string() }.recovery_strategy(),
            RecoveryStrategy::Alternative
        );

        assert_eq!(
            BridgeError::FraudDetected { details: "test".to_string() }.recovery_strategy(),
            RecoveryStrategy::Emergency
        );
    }

    #[test]
    fn test_error_category() {
        assert_eq!(
            BridgeError::ChainNotSupported { chain: "test".to_string() }.category(),
            ErrorCategory::Chain
        );

        assert_eq!(
            BridgeError::TransferFailed { id: "test".to_string(), reason: "test".to_string() }.category(),
            ErrorCategory::Transfer
        );

        assert_eq!(
            BridgeError::SecurityCheckFailed { check: "test".to_string() }.category(),
            ErrorCategory::Security
        );
    }

    #[test]
    fn test_security_related() {
        assert!(BridgeError::FraudDetected { details: "test".to_string() }.is_security_related());
        assert!(!BridgeError::TransferNotFound { id: "test".to_string() }.is_security_related());
    }

    #[test]
    fn test_affects_bridge_operations() {
        assert!(BridgeError::ChainConnectionFailed { chain: "test".to_string() }.affects_bridge_operations());
        assert!(!BridgeError::MetricsCollectionFailed { metric: "test".to_string() }.affects_bridge_operations());
    }

    #[test]
    fn test_is_recoverable() {
        assert!(BridgeError::NetworkError { message: "test".to_string() }.is_recoverable());
        assert!(!BridgeError::EmergencyPauseActive.is_recoverable());
    }

    #[test]
    fn test_requires_immediate_action() {
        assert!(BridgeError::FraudDetected { details: "test".to_string() }.requires_immediate_action());
        assert!(!BridgeError::TransferNotFound { id: "test".to_string() }.requires_immediate_action());
    }
}