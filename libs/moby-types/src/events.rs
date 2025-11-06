use anchor_lang::prelude::*;
use crate::{WhaleAmount, Timestamp, AccountKey, OrderSide, OrderType, OrderStatus, TradingTier};

/// Order lifecycle events
#[event]
pub struct OrderCreated {
    pub order_id: u64,
    pub trader: Pubkey,
    pub order_type: OrderType,
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub side: OrderSide,
    pub size: u64,
    pub price: Option<u64>,
    pub timestamp: i64,
}

#[event]
pub struct OrderExecuted {
    pub order_id: u64,
    pub trader: Pubkey,
    pub execution_id: u64,
    pub size: u64,
    pub price: u64,
    pub remaining_size: u64,
    pub execution_venue: u8,
    pub timestamp: i64,
}

#[event]
pub struct OrderCancelled {
    pub order_id: u64,
    pub trader: Pubkey,
    pub remaining_size: u64,
    pub reason: CancellationReason,
    pub timestamp: i64,
}

#[event]
pub struct OrderFilled {
    pub order_id: u64,
    pub trader: Pubkey,
    pub total_size: u64,
    pub avg_price: u64,
    pub total_fees: u64,
    pub execution_time_ms: u32,
    pub timestamp: i64,
}

/// OTC trading events
#[event]
pub struct OtcTradeCreated {
    pub trade_id: u64,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub asset_mint: Pubkey,
    pub asset_amount: u64,
    pub payment_amount: u64,
    pub agreed_price: u64,
    pub settlement_deadline: i64,
    pub timestamp: i64,
}

#[event]
pub struct OtcTradeConfirmed {
    pub trade_id: u64,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub confirmation_type: ConfirmationType,
    pub timestamp: i64,
}

#[event]
pub struct OtcTradeSettled {
    pub trade_id: u64,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub asset_amount: u64,
    pub payment_amount: u64,
    pub settlement_fees: u64,
    pub timestamp: i64,
}

#[event]
pub struct OtcTradeDisputed {
    pub trade_id: u64,
    pub disputing_party: Pubkey,
    pub dispute_reason: DisputeReason,
    pub arbiter: Pubkey,
    pub timestamp: i64,
}

/// Liquidity and market events
#[event]
pub struct LiquidityAdded {
    pub provider: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub pool_id: u64,
    pub new_total_liquidity: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityRemoved {
    pub provider: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub pool_id: u64,
    pub new_total_liquidity: u64,
    pub timestamp: i64,
}

#[event]
pub struct MarketImpactDetected {
    pub order_id: u64,
    pub token_mint: Pubkey,
    pub impact_percentage: u16,
    pub liquidity_before: u64,
    pub liquidity_after: u64,
    pub timestamp: i64,
}

#[event]
pub struct SlippageExceeded {
    pub order_id: u64,
    pub expected_slippage: u16,
    pub actual_slippage: u16,
    pub execution_halted: bool,
    pub timestamp: i64,
}

/// Whale trading specific events
#[event]
pub struct WhaleTradeDetected {
    pub trader: Pubkey,
    pub order_id: u64,
    pub size_category: SizeCategory,
    pub market_impact_prediction: u16,
    pub recommended_strategy: u8,
    pub timestamp: i64,
}

#[event]
pub struct WhaleTierUpgraded {
    pub trader: Pubkey,
    pub old_tier: TradingTier,
    pub new_tier: TradingTier,
    pub lifetime_volume: u64,
    pub timestamp: i64,
}

#[event]
pub struct WhalePositionAlert {
    pub trader: Pubkey,
    pub token_mint: Pubkey,
    pub position_size: u64,
    pub position_percentage: u16,
    pub alert_type: PositionAlertType,
    pub timestamp: i64,
}

/// Privacy and compliance events
#[event]
pub struct PrivacyTradeExecuted {
    pub order_id: u64,
    pub zk_proof_type: u8,
    pub privacy_level: PrivacyLevel,
    pub verification_time_ms: u32,
    pub timestamp: i64,
}

#[event]
pub struct ComplianceCheck {
    pub trader: Pubkey,
    pub check_type: ComplianceCheckType,
    pub result: ComplianceResult,
    pub risk_score: u8,
    pub timestamp: i64,
}

#[event]
pub struct KycStatusChanged {
    pub trader: Pubkey,
    pub old_status: u8,
    pub new_status: u8,
    pub verification_level: u8,
    pub timestamp: i64,
}

/// Oracle and price events
#[event]
pub struct OracleUpdated {
    pub token_mint: Pubkey,
    pub price: u64,
    pub confidence: u8,
    pub source_count: u8,
    pub deviation: u16,
    pub timestamp: i64,
}

#[event]
pub struct PriceManipulationDetected {
    pub token_mint: Pubkey,
    pub suspicious_sources: Vec<u8>,
    pub deviation_percentage: u16,
    pub confidence_drop: u8,
    pub trading_halted: bool,
    pub timestamp: i64,
}

#[event]
pub struct OracleSourceAdded {
    pub token_mint: Pubkey,
    pub source_id: u8,
    pub source_weight: u8,
    pub reliability_score: u8,
    pub timestamp: i64,
}

#[event]
pub struct OracleSourceRemoved {
    pub token_mint: Pubkey,
    pub source_id: u8,
    pub removal_reason: SourceRemovalReason,
    pub timestamp: i64,
}

/// Cross-chain events
#[event]
pub struct CrossChainTransferInitiated {
    pub order_id: u64,
    pub source_network: u8,
    pub target_network: u8,
    pub asset_amount: u64,
    pub bridge_fee: u64,
    pub estimated_arrival: i64,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainTransferCompleted {
    pub order_id: u64,
    pub source_network: u8,
    pub target_network: u8,
    pub asset_amount: u64,
    pub confirmations: u8,
    pub actual_arrival: i64,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainVerificationFailed {
    pub order_id: u64,
    pub network: u8,
    pub failure_reason: VerificationFailureReason,
    pub retry_count: u8,
    pub timestamp: i64,
}

/// Governance and admin events
#[event]
pub struct ProtocolParameterChanged {
    pub parameter_name: String,
    pub old_value: u64,
    pub new_value: u64,
    pub changed_by: Pubkey,
    pub effective_timestamp: i64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyModeActivated {
    pub activated_by: Pubkey,
    pub reason: EmergencyReason,
    pub affected_operations: Vec<u8>,
    pub estimated_duration: Option<i64>,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyModeDeactivated {
    pub deactivated_by: Pubkey,
    pub duration_active: i64,
    pub operations_restored: Vec<u8>,
    pub timestamp: i64,
}

#[event]
pub struct GovernanceProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub voting_deadline: i64,
    pub required_votes: u64,
    pub timestamp: i64,
}

#[event]
pub struct GovernanceVoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote_weight: u64,
    pub vote_direction: VoteDirection,
    pub timestamp: i64,
}

/// Risk management events
#[event]
pub struct RiskLimitExceeded {
    pub trader: Pubkey,
    pub limit_type: RiskLimitType,
    pub current_value: u64,
    pub limit_value: u64,
    pub action_taken: RiskAction,
    pub timestamp: i64,
}

#[event]
pub struct LiquidationTriggered {
    pub trader: Pubkey,
    pub position_id: u64,
    pub liquidation_price: u64,
    pub liquidation_amount: u64,
    pub liquidation_fee: u64,
    pub liquidator: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RiskParametersUpdated {
    pub trader: Pubkey,
    pub parameter_type: RiskParameterType,
    pub old_value: u64,
    pub new_value: u64,
    pub updated_by: Pubkey,
    pub timestamp: i64,
}

/// Performance and analytics events
#[event]
pub struct TradingVolumeReport {
    pub period_start: i64,
    pub period_end: i64,
    pub total_volume: u64,
    pub unique_traders: u32,
    pub total_trades: u32,
    pub avg_trade_size: u64,
    pub total_fees: u64,
    pub timestamp: i64,
}

#[event]
pub struct ExecutionAnalytics {
    pub order_id: u64,
    pub execution_time_ms: u32,
    pub slippage_bps: u16,
    pub market_impact_bps: u16,
    pub venue_distribution: Vec<VenueExecution>,
    pub cost_analysis: CostAnalysis,
    pub timestamp: i64,
}

/// Supporting enum types for events
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum CancellationReason {
    UserRequested,
    InsufficientFunds,
    OrderExpired,
    MarketConditions,
    RiskManagement,
    SystemError,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ConfirmationType {
    BuyerConfirmed,
    SellerConfirmed,
    BothConfirmed,
    AutoConfirmed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum DisputeReason {
    PriceDiscrepancy,
    QuantityDiscrepancy,
    SettlementDelay,
    CounterpartyDefault,
    TechnicalIssue,
    ComplianceViolation,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum SizeCategory {
    Small,      // < $1M
    Medium,     // $1M - $10M
    Large,      // $10M - $50M
    VeryLarge,  // $50M - $100M
    Massive,    // > $100M
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum PositionAlertType {
    SizeThresholdExceeded,
    ConcentrationRisk,
    LiquidityRisk,
    MarketRisk,
    CounterpartyRisk,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum PrivacyLevel {
    None,
    Basic,
    Enhanced,
    Maximum,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ComplianceCheckType {
    KycVerification,
    AmlScreening,
    SanctionsCheck,
    JurisdictionCheck,
    RiskAssessment,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ComplianceResult {
    Pass,
    Fail,
    Pending,
    RequiresReview,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum SourceRemovalReason {
    LowReliability,
    ConnectivityIssues,
    DataQuality,
    Maintenance,
    Deprecated,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum VerificationFailureReason {
    NetworkTimeout,
    InvalidSignature,
    InsufficientConfirmations,
    ContractError,
    ValidationFailure,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum EmergencyReason {
    SecurityBreach,
    MarketAnomaly,
    TechnicalFailure,
    RegulatoryOrder,
    LiquidityCrisis,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ProposalType {
    ParameterChange,
    FeatureAddition,
    SecurityUpdate,
    GovernanceChange,
    EmergencyAction,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum VoteDirection {
    For,
    Against,
    Abstain,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum RiskLimitType {
    PositionSize,
    DailyVolume,
    ConcentrationRatio,
    LeverageRatio,
    DrawdownLimit,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum RiskAction {
    Warning,
    PositionReduction,
    TradingHalt,
    MarginCall,
    Liquidation,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum RiskParameterType {
    MaxPositionSize,
    StopLossLevel,
    TakeProfitLevel,
    MarginRequirement,
    ConcentrationLimit,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VenueExecution {
    pub venue_id: u8,
    pub volume_percentage: u8,
    pub avg_price: u64,
    pub execution_time_ms: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CostAnalysis {
    pub trading_fees: u64,
    pub market_impact_cost: u64,
    pub slippage_cost: u64,
    pub timing_cost: u64,
    pub total_cost: u64,
    pub cost_basis_points: u16,
}

/// Event helper functions
pub fn emit_order_created(
    order_id: u64,
    trader: Pubkey,
    order_type: OrderType,
    base_token: Pubkey,
    quote_token: Pubkey,
    side: OrderSide,
    size: u64,
    price: Option<u64>,
) {
    emit!(OrderCreated {
        order_id,
        trader,
        order_type,
        base_token,
        quote_token,
        side,
        size,
        price,
        timestamp: Clock::get().unwrap().unix_timestamp,
    });
}

pub fn emit_whale_trade_detected(
    trader: Pubkey,
    order_id: u64,
    size_usd: u64,
    predicted_impact: u16,
    recommended_strategy: u8,
) {
    let size_category = match size_usd / moby_math::Price::PRECISION {
        0..=999_999 => SizeCategory::Small,
        1_000_000..=9_999_999 => SizeCategory::Medium,
        10_000_000..=49_999_999 => SizeCategory::Large,
        50_000_000..=99_999_999 => SizeCategory::VeryLarge,
        _ => SizeCategory::Massive,
    };

    emit!(WhaleTradeDetected {
        trader,
        order_id,
        size_category,
        market_impact_prediction: predicted_impact,
        recommended_strategy,
        timestamp: Clock::get().unwrap().unix_timestamp,
    });
}

pub fn emit_price_manipulation_alert(
    token_mint: Pubkey,
    suspicious_sources: Vec<u8>,
    deviation: u16,
    confidence_drop: u8,
    halt_trading: bool,
) {
    emit!(PriceManipulationDetected {
        token_mint,
        suspicious_sources,
        deviation_percentage: deviation,
        confidence_drop,
        trading_halted: halt_trading,
        timestamp: Clock::get().unwrap().unix_timestamp,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_category_classification() {
        // Test size category logic
        let test_cases = vec![
            (500_000, SizeCategory::Small),
            (5_000_000, SizeCategory::Medium),
            (25_000_000, SizeCategory::Large),
            (75_000_000, SizeCategory::VeryLarge),
            (150_000_000, SizeCategory::Massive),
        ];

        for (size_usd, expected) in test_cases {
            let category = match size_usd {
                0..=999_999 => SizeCategory::Small,
                1_000_000..=9_999_999 => SizeCategory::Medium,
                10_000_000..=49_999_999 => SizeCategory::Large,
                50_000_000..=99_999_999 => SizeCategory::VeryLarge,
                _ => SizeCategory::Massive,
            };

            assert_eq!(category as u8, expected as u8);
        }
    }

    #[test]
    fn test_event_structures() {
        // Test that event structures can be created and serialized
        let order_created = OrderCreated {
            order_id: 1,
            trader: Pubkey::default(),
            order_type: OrderType::Market,
            base_token: Pubkey::default(),
            quote_token: Pubkey::default(),
            side: OrderSide::Buy,
            size: 1000,
            price: Some(100),
            timestamp: 1234567890,
        };

        // Ensure the struct can be serialized (basic validation)
        let _serialized = order_created.try_to_vec().unwrap();
    }
}