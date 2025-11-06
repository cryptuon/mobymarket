// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Error types for the privacy system

use thiserror::Error;

/// Result type for privacy operations
pub type PrivacyResult<T> = Result<T, PrivacyError>;

/// Comprehensive error types for privacy operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum PrivacyError {
    // Proof generation errors
    #[error("Proof generation failed: {reason}")]
    ProofGenerationFailed { reason: String },

    #[error("Invalid circuit parameters: {parameter}")]
    InvalidCircuitParameters { parameter: String },

    #[error("Circuit setup failed: {reason}")]
    CircuitSetupFailed { reason: String },

    #[error("Trusted setup required but not available")]
    TrustedSetupRequired,

    #[error("Proving key not found: {circuit}")]
    ProvingKeyNotFound { circuit: String },

    // Proof verification errors
    #[error("Proof verification failed")]
    ProofVerificationFailed,

    #[error("Invalid proof format")]
    InvalidProofFormat,

    #[error("Proof expired at {expiry_time}")]
    ProofExpired { expiry_time: String },

    #[error("Verification key not found: {circuit}")]
    VerificationKeyNotFound { circuit: String },

    #[error("Public inputs mismatch: expected {expected}, got {actual}")]
    PublicInputsMismatch { expected: usize, actual: usize },

    // Commitment errors
    #[error("Invalid commitment: {reason}")]
    InvalidCommitment { reason: String },

    #[error("Commitment verification failed")]
    CommitmentVerificationFailed,

    #[error("Commitment opening failed")]
    CommitmentOpeningFailed,

    #[error("Invalid randomness for commitment")]
    InvalidCommitmentRandomness,

    // Privacy pool errors
    #[error("Privacy pool not found: {pool_id}")]
    PrivacyPoolNotFound { pool_id: String },

    #[error("Insufficient anonymity set size: {current} < {required}")]
    InsufficientAnonymitySet { current: u32, required: u32 },

    #[error("Pool capacity exceeded: {current} >= {max}")]
    PoolCapacityExceeded { current: u32, max: u32 },

    #[error("Pool membership verification failed")]
    PoolMembershipFailed,

    #[error("Invalid pool state: {state}")]
    InvalidPoolState { state: String },

    // Mixer errors
    #[error("Mixing protocol error: {reason}")]
    MixingProtocolError { reason: String },

    #[error("Invalid mix transaction: {reason}")]
    InvalidMixTransaction { reason: String },

    #[error("Mixer queue full")]
    MixerQueueFull,

    #[error("Mix delay not satisfied: {remaining_ms}ms remaining")]
    MixDelayNotSatisfied { remaining_ms: u64 },

    #[error("Double mixing attempt detected")]
    DoubleMixingAttempt,

    // Stealth address errors
    #[error("Invalid stealth address: {reason}")]
    InvalidStealthAddress { reason: String },

    #[error("Stealth key derivation failed")]
    StealthKeyDerivationFailed,

    #[error("Stealth address generation failed")]
    StealthAddressGenerationFailed,

    #[error("Stealth payment detection failed")]
    StealthPaymentDetectionFailed,

    // Range proof errors
    #[error("Range proof generation failed: {reason}")]
    RangeProofGenerationFailed { reason: String },

    #[error("Range proof verification failed")]
    RangeProofVerificationFailed,

    #[error("Value out of range: {value} not in [0, 2^{bits})")]
    ValueOutOfRange { value: u64, bits: usize },

    #[error("Invalid range proof parameters")]
    InvalidRangeProofParameters,

    // Nullifier errors
    #[error("Nullifier already spent: {nullifier}")]
    NullifierAlreadySpent { nullifier: String },

    #[error("Invalid nullifier: {reason}")]
    InvalidNullifier { reason: String },

    #[error("Nullifier generation failed")]
    NullifierGenerationFailed,

    #[error("Double spend detected: {nullifier}")]
    DoubleSpendDetected { nullifier: String },

    // Compliance errors
    #[error("Compliance check failed: {reason}")]
    ComplianceCheckFailed { reason: String },

    #[error("Unauthorized compliance access")]
    UnauthorizedComplianceAccess,

    #[error("Selective disclosure failed: {reason}")]
    SelectiveDisclosureFailed { reason: String },

    #[error("Regulatory proof required but not provided")]
    RegulatoryProofRequired,

    #[error("Compliance officer not authorized: {officer}")]
    ComplianceOfficerNotAuthorized { officer: String },

    // Cryptographic errors
    #[error("Invalid cryptographic parameters")]
    InvalidCryptographicParameters,

    #[error("Key generation failed: {reason}")]
    KeyGenerationFailed { reason: String },

    #[error("Encryption failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error("Decryption failed: {reason}")]
    DecryptionFailed { reason: String },

    #[error("Hash computation failed")]
    HashComputationFailed,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    // Serialization errors
    #[error("Serialization failed: {reason}")]
    SerializationFailed { reason: String },

    #[error("Deserialization failed: {reason}")]
    DeserializationFailed { reason: String },

    #[error("Invalid data format: {format}")]
    InvalidDataFormat { format: String },

    // Configuration errors
    #[error("Invalid privacy configuration: {field}")]
    InvalidConfiguration { field: String },

    #[error("Missing configuration: {field}")]
    MissingConfiguration { field: String },

    #[error("Unsupported proof system: {system}")]
    UnsupportedProofSystem { system: String },

    #[error("Unsupported hash function: {function}")]
    UnsupportedHashFunction { function: String },

    // Performance errors
    #[error("Operation timeout after {timeout_ms}ms")]
    OperationTimeout { timeout_ms: u64 },

    #[error("Insufficient memory for operation")]
    InsufficientMemory,

    #[error("Hardware acceleration not available")]
    HardwareAccelerationUnavailable,

    #[error("Performance optimization failed: {reason}")]
    PerformanceOptimizationFailed { reason: String },

    // System errors
    #[error("Privacy system not initialized")]
    SystemNotInitialized,

    #[error("Privacy feature not enabled: {feature}")]
    FeatureNotEnabled { feature: String },

    #[error("Resource limit exceeded: {resource}")]
    ResourceLimitExceeded { resource: String },

    #[error("Internal privacy error: {message}")]
    Internal { message: String },

    // Integration errors
    #[error("Math error: {0}")]
    Math(#[from] moby_math::MathError),

    #[error("Core type error: {0}")]
    CoreType(#[from] moby_types::MobyError),
}

impl PrivacyError {
    /// Create a new proof generation error
    pub fn proof_generation_failed(reason: impl Into<String>) -> Self {
        Self::ProofGenerationFailed { reason: reason.into() }
    }

    /// Create a new mixing protocol error
    pub fn mixing_protocol_error(reason: impl Into<String>) -> Self {
        Self::MixingProtocolError { reason: reason.into() }
    }

    /// Create a new compliance check error
    pub fn compliance_check_failed(reason: impl Into<String>) -> Self {
        Self::ComplianceCheckFailed { reason: reason.into() }
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into() }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Temporary system issues
            Self::OperationTimeout { .. }
            | Self::InsufficientMemory
            | Self::MixerQueueFull
            | Self::MixDelayNotSatisfied { .. }
            | Self::PoolCapacityExceeded { .. } => true,

            // Configuration issues that might be fixed
            Self::InvalidConfiguration { .. }
            | Self::MissingConfiguration { .. }
            | Self::FeatureNotEnabled { .. } => true,

            // Cryptographic failures that might succeed on retry
            Self::ProofGenerationFailed { .. }
            | Self::KeyGenerationFailed { .. }
            | Self::EncryptionFailed { .. } => true,

            // Non-recoverable errors
            Self::ProofVerificationFailed
            | Self::InvalidProofFormat
            | Self::CommitmentVerificationFailed
            | Self::DoubleSpendDetected { .. }
            | Self::NullifierAlreadySpent { .. }
            | Self::InvalidStealthAddress { .. }
            | Self::RangeProofVerificationFailed => false,

            // Security violations
            Self::UnauthorizedComplianceAccess
            | Self::ComplianceOfficerNotAuthorized { .. }
            | Self::DoubleMixingAttempt => false,

            // Context dependent
            _ => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical security issues
            Self::DoubleSpendDetected { .. }
            | Self::DoubleMixingAttempt
            | Self::UnauthorizedComplianceAccess => ErrorSeverity::Critical,

            // High severity - privacy breaches or verification failures
            Self::ProofVerificationFailed
            | Self::CommitmentVerificationFailed
            | Self::RangeProofVerificationFailed
            | Self::PoolMembershipFailed
            | Self::ComplianceCheckFailed { .. } => ErrorSeverity::High,

            // Medium severity - generation failures and configuration issues
            Self::ProofGenerationFailed { .. }
            | Self::InvalidConfiguration { .. }
            | Self::InvalidCommitment { .. }
            | Self::InvalidMixTransaction { .. } => ErrorSeverity::Medium,

            // Low severity - temporary issues
            Self::OperationTimeout { .. }
            | Self::MixDelayNotSatisfied { .. }
            | Self::InsufficientMemory
            | Self::MixerQueueFull => ErrorSeverity::Low,

            // Info level - expected conditions
            Self::PrivacyPoolNotFound { .. }
            | Self::ProvingKeyNotFound { .. }
            | Self::VerificationKeyNotFound { .. } => ErrorSeverity::Info,
        }
    }

    /// Check if error should trigger security alerts
    pub fn is_security_critical(&self) -> bool {
        matches!(
            self,
            Self::DoubleSpendDetected { .. }
                | Self::DoubleMixingAttempt
                | Self::UnauthorizedComplianceAccess
                | Self::ComplianceOfficerNotAuthorized { .. }
                | Self::NullifierAlreadySpent { .. }
        )
    }

    /// Check if error affects privacy guarantees
    pub fn affects_privacy(&self) -> bool {
        matches!(
            self,
            Self::ProofVerificationFailed
                | Self::CommitmentVerificationFailed
                | Self::PoolMembershipFailed
                | Self::MixingProtocolError { .. }
                | Self::StealthAddressGenerationFailed
                | Self::RangeProofVerificationFailed
        )
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

    /// Check if error should stop system operation
    pub fn should_halt_system(&self) -> bool {
        matches!(self, ErrorSeverity::Critical)
    }
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            component: component.into(),
            timestamp: chrono::Utc::now(),
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn with_info(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }
}

/// Enhanced result type with context
pub type ContextualResult<T> = Result<T, (PrivacyError, ErrorContext)>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = PrivacyError::proof_generation_failed("Invalid witness");
        assert!(matches!(error, PrivacyError::ProofGenerationFailed { .. }));

        let error = PrivacyError::mixing_protocol_error("Network timeout");
        assert!(matches!(error, PrivacyError::MixingProtocolError { .. }));
    }

    #[test]
    fn test_error_recoverability() {
        assert!(PrivacyError::OperationTimeout { timeout_ms: 5000 }.is_recoverable());
        assert!(PrivacyError::MixerQueueFull.is_recoverable());
        assert!(!PrivacyError::ProofVerificationFailed.is_recoverable());
        assert!(!PrivacyError::DoubleSpendDetected { nullifier: "test".to_string() }.is_recoverable());
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(
            PrivacyError::DoubleSpendDetected { nullifier: "test".to_string() }.severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(PrivacyError::ProofVerificationFailed.severity(), ErrorSeverity::High);
        assert_eq!(
            PrivacyError::ProofGenerationFailed { reason: "test".to_string() }.severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(PrivacyError::MixerQueueFull.severity(), ErrorSeverity::Low);
        assert_eq!(
            PrivacyError::PrivacyPoolNotFound { pool_id: "test".to_string() }.severity(),
            ErrorSeverity::Info
        );
    }

    #[test]
    fn test_security_criticality() {
        assert!(PrivacyError::DoubleSpendDetected { nullifier: "test".to_string() }.is_security_critical());
        assert!(PrivacyError::DoubleMixingAttempt.is_security_critical());
        assert!(!PrivacyError::ProofGenerationFailed { reason: "test".to_string() }.is_security_critical());
    }

    #[test]
    fn test_privacy_impact() {
        assert!(PrivacyError::ProofVerificationFailed.affects_privacy());
        assert!(PrivacyError::MixingProtocolError { reason: "test".to_string() }.affects_privacy());
        assert!(!PrivacyError::InvalidConfiguration { field: "test".to_string() }.affects_privacy());
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

        assert!(ErrorSeverity::Critical.should_halt_system());
        assert!(!ErrorSeverity::High.should_halt_system());
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("generate_proof", "groth16_prover")
            .with_info("circuit", "private_trade")
            .with_info("inputs", "4");

        assert_eq!(context.operation, "generate_proof");
        assert_eq!(context.component, "groth16_prover");
        assert_eq!(context.additional_info.get("circuit").unwrap(), "private_trade");
    }
}