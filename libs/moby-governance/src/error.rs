//! Error types for the governance system

use thiserror::Error;

/// Result type for governance operations
pub type GovernanceResult<T> = Result<T, GovernanceError>;

/// Comprehensive error types for governance operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum GovernanceError {
    // Proposal errors
    #[error("Proposal not found: {id}")]
    ProposalNotFound { id: String },

    #[error("Proposal already exists: {id}")]
    ProposalAlreadyExists { id: String },

    #[error("Invalid proposal state: expected {expected}, found {actual}")]
    InvalidProposalState { expected: String, actual: String },

    #[error("Proposal voting period has ended")]
    VotingPeriodEnded,

    #[error("Proposal voting period has not started")]
    VotingPeriodNotStarted,

    #[error("Insufficient voting power: required {required}, available {available}")]
    InsufficientVotingPower { required: u64, available: u64 },

    // Voting errors
    #[error("Vote not found for voter: {voter}")]
    VoteNotFound { voter: String },

    #[error("Voter has already voted: {voter}")]
    AlreadyVoted { voter: String },

    #[error("Invalid vote type: {vote_type}")]
    InvalidVoteType { vote_type: String },

    #[error("Quorum not reached: required {required}%, actual {actual}%")]
    QuorumNotReached { required: u8, actual: u8 },

    // Delegation errors
    #[error("Delegation not found: {delegator} -> {delegate}")]
    DelegationNotFound { delegator: String, delegate: String },

    #[error("Self-delegation not allowed")]
    SelfDelegationNotAllowed,

    #[error("Circular delegation detected")]
    CircularDelegationDetected,

    #[error("Delegation already exists: {delegator} -> {delegate}")]
    DelegationAlreadyExists { delegator: String, delegate: String },

    // Token errors
    #[error("Insufficient token balance: required {required}, available {available}")]
    InsufficientTokenBalance { required: u64, available: u64 },

    #[error("Invalid token amount: {amount}")]
    InvalidTokenAmount { amount: String },

    #[error("Token transfer failed: {reason}")]
    TokenTransferFailed { reason: String },

    #[error("Staking period not ended")]
    StakingPeriodNotEnded,

    #[error("Invalid staking duration: {duration}")]
    InvalidStakingDuration { duration: String },

    // Treasury errors
    #[error("Treasury action not authorized")]
    TreasuryActionNotAuthorized,

    #[error("Insufficient treasury funds: required {required}, available {available}")]
    InsufficientTreasuryFunds { required: u64, available: u64 },

    #[error("Invalid treasury proposal: {reason}")]
    InvalidTreasuryProposal { reason: String },

    // Parameter errors
    #[error("Parameter not found: {name}")]
    ParameterNotFound { name: String },

    #[error("Invalid parameter value: {name} = {value}")]
    InvalidParameterValue { name: String, value: String },

    #[error("Parameter update timelock not expired")]
    ParameterTimelockNotExpired,

    #[error("Parameter is immutable: {name}")]
    ParameterIsImmutable { name: String },

    // Upgrade errors
    #[error("Upgrade not found: {id}")]
    UpgradeNotFound { id: String },

    #[error("Upgrade already applied: {id}")]
    UpgradeAlreadyApplied { id: String },

    #[error("Invalid upgrade type: {upgrade_type}")]
    InvalidUpgradeType { upgrade_type: String },

    #[error("Upgrade execution failed: {reason}")]
    UpgradeExecutionFailed { reason: String },

    // Emergency errors
    #[error("Emergency action not authorized: {action}")]
    EmergencyActionNotAuthorized { action: String },

    #[error("System not in emergency state")]
    SystemNotInEmergencyState,

    #[error("Invalid emergency role: {role}")]
    InvalidEmergencyRole { role: String },

    // Authorization errors
    #[error("Unauthorized access: {action}")]
    UnauthorizedAccess { action: String },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Signer not found: {signer}")]
    SignerNotFound { signer: String },

    #[error("Multi-signature threshold not met: required {required}, provided {provided}")]
    MultiSigThresholdNotMet { required: u8, provided: u8 },

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

    // Validation errors
    #[error("Invalid timestamp: {timestamp}")]
    InvalidTimestamp { timestamp: String },

    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    #[error("Invalid hash: {hash}")]
    InvalidHash { hash: String },

    #[error("Invalid range: {min} to {max}")]
    InvalidRange { min: String, max: String },

    // Generic errors
    #[error("Operation failed: {reason}")]
    OperationFailed { reason: String },

    #[error("Timeout occurred: {operation}")]
    TimeoutOccurred { operation: String },

    #[error("Resource not available: {resource}")]
    ResourceNotAvailable { resource: String },

    #[error("Concurrent modification detected")]
    ConcurrentModificationDetected,
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
    /// Use fallback mechanism
    Fallback,
    /// Manual intervention required
    Manual,
    /// System reset required
    Reset,
    /// No recovery possible
    None,
}

impl GovernanceError {
    /// Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // High severity errors
            Self::SystemConfigurationError { .. } |
            Self::StorageError { .. } |
            Self::NetworkError { .. } |
            Self::UpgradeExecutionFailed { .. } |
            Self::EmergencyActionNotAuthorized { .. } => ErrorSeverity::High,

            // Critical severity errors
            Self::ConcurrentModificationDetected |
            Self::CircularDelegationDetected => ErrorSeverity::Critical,

            // Medium severity errors
            Self::UnauthorizedAccess { .. } |
            Self::InvalidSignature |
            Self::MultiSigThresholdNotMet { .. } |
            Self::QuorumNotReached { .. } => ErrorSeverity::Medium,

            // Low severity errors (validation, not found, etc.)
            _ => ErrorSeverity::Low,
        }
    }

    /// Get the recovery strategy for the error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            // Retry-able errors
            Self::NetworkError { .. } |
            Self::TimeoutOccurred { .. } => RecoveryStrategy::Retry,

            // Fallback-able errors
            Self::ResourceNotAvailable { .. } |
            Self::InsufficientTreasuryFunds { .. } => RecoveryStrategy::Fallback,

            // Manual intervention required
            Self::EmergencyActionNotAuthorized { .. } |
            Self::UpgradeExecutionFailed { .. } |
            Self::ConcurrentModificationDetected => RecoveryStrategy::Manual,

            // System reset required
            Self::SystemConfigurationError { .. } |
            Self::CircularDelegationDetected => RecoveryStrategy::Reset,

            // No recovery possible
            Self::InvalidSignature |
            Self::ParameterIsImmutable { .. } |
            Self::UpgradeAlreadyApplied { .. } => RecoveryStrategy::None,

            // Most others can retry
            _ => RecoveryStrategy::Retry,
        }
    }

    /// Check if the error indicates a security issue
    pub fn is_security_related(&self) -> bool {
        matches!(self,
            Self::UnauthorizedAccess { .. } |
            Self::InvalidSignature |
            Self::EmergencyActionNotAuthorized { .. } |
            Self::MultiSigThresholdNotMet { .. } |
            Self::CircularDelegationDetected
        )
    }

    /// Check if the error affects governance operations
    pub fn affects_governance(&self) -> bool {
        matches!(self,
            Self::ProposalNotFound { .. } |
            Self::InvalidProposalState { .. } |
            Self::VotingPeriodEnded |
            Self::QuorumNotReached { .. } |
            Self::InsufficientVotingPower { .. }
        )
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !matches!(self.recovery_strategy(),
            RecoveryStrategy::Manual |
            RecoveryStrategy::Reset |
            RecoveryStrategy::None
        )
    }
}

impl From<serde_json::Error> for GovernanceError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<bincode::Error> for GovernanceError {
    fn from(error: bincode::Error) -> Self {
        Self::SerializationError {
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for GovernanceError {
    fn from(error: std::io::Error) -> Self {
        Self::StorageError {
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
            GovernanceError::ProposalNotFound { id: "test".to_string() }.severity(),
            ErrorSeverity::Low
        );

        assert_eq!(
            GovernanceError::UnauthorizedAccess { action: "vote".to_string() }.severity(),
            ErrorSeverity::Medium
        );

        assert_eq!(
            GovernanceError::SystemConfigurationError { message: "test".to_string() }.severity(),
            ErrorSeverity::High
        );

        assert_eq!(
            GovernanceError::ConcurrentModificationDetected.severity(),
            ErrorSeverity::Critical
        );
    }

    #[test]
    fn test_recovery_strategy() {
        assert_eq!(
            GovernanceError::NetworkError { message: "timeout".to_string() }.recovery_strategy(),
            RecoveryStrategy::Retry
        );

        assert_eq!(
            GovernanceError::InvalidSignature.recovery_strategy(),
            RecoveryStrategy::None
        );
    }

    #[test]
    fn test_security_related() {
        assert!(GovernanceError::UnauthorizedAccess { action: "test".to_string() }.is_security_related());
        assert!(!GovernanceError::ProposalNotFound { id: "test".to_string() }.is_security_related());
    }

    #[test]
    fn test_affects_governance() {
        assert!(GovernanceError::VotingPeriodEnded.affects_governance());
        assert!(!GovernanceError::NetworkError { message: "test".to_string() }.affects_governance());
    }

    #[test]
    fn test_is_recoverable() {
        assert!(GovernanceError::NetworkError { message: "test".to_string() }.is_recoverable());
        assert!(!GovernanceError::InvalidSignature.is_recoverable());
    }
}