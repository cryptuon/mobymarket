use anchor_lang::prelude::*;

#[error_code]
pub enum MobyError {
    #[msg("Insufficient funds for whale trade")]
    InsufficientFunds,

    #[msg("Trade amount exceeds whale limits")]
    ExceedsWhaleLimit,

    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,

    #[msg("Oracle confidence too low for whale trade")]
    OracleConfidenceTooLow,

    #[msg("Price manipulation detected")]
    PriceManipulation,

    #[msg("Order size below minimum whale threshold")]
    BelowWhaleThreshold,

    #[msg("Unauthorized whale trader")]
    UnauthorizedWhaleTrader,

    #[msg("Trade execution window expired")]
    ExecutionWindowExpired,

    #[msg("Insufficient liquidity for execution")]
    InsufficientLiquidity,

    #[msg("Invalid order parameters")]
    InvalidOrderParameters,

    #[msg("Account data deserialization failed")]
    DeserializationFailed,

    #[msg("Account not properly initialized")]
    AccountNotInitialized,

    #[msg("Invalid account owner")]
    InvalidAccountOwner,

    #[msg("Privacy verification failed")]
    PrivacyVerificationFailed,

    #[msg("Zero knowledge proof invalid")]
    InvalidProof,

    #[msg("Emergency mode active - trading suspended")]
    EmergencyMode,

    #[msg("Governance approval required")]
    GovernanceRequired,

    #[msg("Rate limit exceeded")]
    RateLimitExceeded,

    #[msg("Invalid signature")]
    InvalidSignature,

    #[msg("Account size limit exceeded")]
    AccountSizeExceeded,

    #[msg("Math overflow in calculation")]
    MathOverflow,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Token account not found")]
    TokenAccountNotFound,

    #[msg("Cross-chain verification failed")]
    CrossChainVerificationFailed,

    #[msg("Invalid network state")]
    InvalidNetworkState,

    #[msg("Consensus threshold not met")]
    ConsensusThresholdNotMet,

    #[msg("Invalid time window")]
    InvalidTimeWindow,

    #[msg("Duplicate transaction")]
    DuplicateTransaction,

    #[msg("Invalid nonce")]
    InvalidNonce,

    #[msg("Account frozen")]
    AccountFrozen,

    #[msg("Feature not available")]
    FeatureNotAvailable,
}

impl From<moby_math::MathError> for MobyError {
    fn from(_: moby_math::MathError) -> Self {
        MobyError::MathOverflow
    }
}

impl From<moby_math::PriceError> for MobyError {
    fn from(_: moby_math::PriceError) -> Self {
        MobyError::InvalidOrderParameters
    }
}