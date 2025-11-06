use anchor_lang::prelude::*;
use crate::{WhaleAmount, Timestamp, AccountKey, ExecutionStrategy, TradingTier};
use moby_math::Price;

/// Whale order account
#[account]
#[derive(InitSpace)]
pub struct WhaleOrder {
    /// Order unique identifier
    pub order_id: u64,

    /// Trader who created the order
    pub trader: Pubkey,

    /// Order type
    pub order_type: OrderType,

    /// Base token mint
    pub base_token: Pubkey,

    /// Quote token mint (usually USDC/USDT)
    pub quote_token: Pubkey,

    /// Order side (buy/sell)
    pub side: OrderSide,

    /// Order size in base token units
    pub size: u64,

    /// Price in quote token units (for limit orders)
    pub price: Option<u64>,

    /// Filled amount
    pub filled_size: u64,

    /// Average fill price
    pub avg_fill_price: u64,

    /// Order status
    pub status: OrderStatus,

    /// Execution strategy
    pub execution_strategy: ExecutionStrategy,

    /// Slippage tolerance (basis points)
    pub slippage_tolerance: u16,

    /// Time in force
    pub time_in_force: TimeInForce,

    /// Order creation timestamp
    pub created_at: i64,

    /// Order expiration timestamp
    pub expires_at: Option<i64>,

    /// Last update timestamp
    pub updated_at: i64,

    /// Privacy settings
    pub privacy_enabled: bool,

    /// Zero-knowledge proof (if privacy enabled)
    #[max_len(64)]
    pub zk_proof: Vec<u8>,

    /// Order metadata
    pub metadata: OrderMetadata,

    /// Execution parameters for complex strategies
    pub execution_params: ExecutionParams,

    /// Cross-chain execution info
    pub cross_chain_info: Option<CrossChainInfo>,

    /// Reserved space
    pub _reserved: [u8; 128],
}

impl WhaleOrder {
    pub const SEED: &'static str = "whale_order";

    pub fn initialize(
        &mut self,
        order_id: u64,
        trader: Pubkey,
        order_type: OrderType,
        base_token: Pubkey,
        quote_token: Pubkey,
        side: OrderSide,
        size: u64,
        price: Option<u64>,
        execution_strategy: ExecutionStrategy,
        slippage_tolerance: u16,
        time_in_force: TimeInForce,
        expires_at: Option<i64>,
        privacy_enabled: bool,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        self.order_id = order_id;
        self.trader = trader;
        self.order_type = order_type;
        self.base_token = base_token;
        self.quote_token = quote_token;
        self.side = side;
        self.size = size;
        self.price = price;
        self.filled_size = 0;
        self.avg_fill_price = 0;
        self.status = OrderStatus::Pending;
        self.execution_strategy = execution_strategy;
        self.slippage_tolerance = slippage_tolerance;
        self.time_in_force = time_in_force;
        self.created_at = now;
        self.expires_at = expires_at;
        self.updated_at = now;
        self.privacy_enabled = privacy_enabled;
        self.zk_proof = Vec::new();
        self.metadata = OrderMetadata::default();
        self.execution_params = ExecutionParams::default();
        self.cross_chain_info = None;
        self._reserved = [0; 128];

        Ok(())
    }

    pub fn update_fill(&mut self, fill_size: u64, fill_price: u64) -> Result<()> {
        require!(
            self.filled_size + fill_size <= self.size,
            crate::MobyError::InvalidOrderParameters
        );

        // Update average fill price
        let total_value = (self.avg_fill_price as u128)
            .checked_mul(self.filled_size as u128)
            .ok_or(crate::MobyError::MathOverflow)?
            .checked_add(
                (fill_price as u128).checked_mul(fill_size as u128)
                    .ok_or(crate::MobyError::MathOverflow)?
            )
            .ok_or(crate::MobyError::MathOverflow)?;

        let new_filled_size = self.filled_size + fill_size;
        self.avg_fill_price = (total_value / new_filled_size as u128) as u64;
        self.filled_size = new_filled_size;

        // Update status
        if self.filled_size == self.size {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }

        self.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        require!(
            matches!(self.status, OrderStatus::Pending | OrderStatus::PartiallyFilled),
            crate::MobyError::InvalidOrderParameters
        );

        self.status = OrderStatus::Cancelled;
        self.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn expire(&mut self) -> Result<()> {
        self.status = OrderStatus::Expired;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Clock::get().map(|clock| clock.unix_timestamp > expires_at).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn remaining_size(&self) -> u64 {
        self.size.saturating_sub(self.filled_size)
    }

    pub fn fill_percentage(&self) -> u8 {
        if self.size == 0 {
            0
        } else {
            ((self.filled_size as u128 * 100) / self.size as u128) as u8
        }
    }

    pub fn notional_value(&self) -> Result<u64> {
        let price = self.price.unwrap_or(self.avg_fill_price);
        if price == 0 {
            return Ok(0);
        }

        price.checked_mul(self.size)
            .ok_or_else(|| crate::MobyError::MathOverflow.into())
    }
}

/// OTC (Over-the-Counter) trade account for large whale trades
#[account]
#[derive(InitSpace)]
pub struct OtcTrade {
    /// Trade unique identifier
    pub trade_id: u64,

    /// Buyer account
    pub buyer: Pubkey,

    /// Seller account
    pub seller: Pubkey,

    /// Asset being traded
    pub asset_mint: Pubkey,

    /// Payment token mint
    pub payment_mint: Pubkey,

    /// Asset amount
    pub asset_amount: u64,

    /// Payment amount
    pub payment_amount: u64,

    /// Agreed price
    pub agreed_price: u64,

    /// Trade status
    pub status: OtcStatus,

    /// Escrow account for funds
    pub escrow_account: Pubkey,

    /// Settlement parameters
    pub settlement: SettlementParams,

    /// Privacy configuration
    pub privacy_config: PrivacyConfig,

    /// Creation timestamp
    pub created_at: i64,

    /// Settlement deadline
    pub settlement_deadline: i64,

    /// Last update timestamp
    pub updated_at: i64,

    /// Buyer signature
    #[max_len(64)]
    pub buyer_signature: Vec<u8>,

    /// Seller signature
    #[max_len(64)]
    pub seller_signature: Vec<u8>,

    /// Trade metadata
    pub metadata: TradeMetadata,

    /// Reserved space
    pub _reserved: [u8; 64],
}

impl OtcTrade {
    pub const SEED: &'static str = "otc_trade";

    pub fn is_ready_for_settlement(&self) -> bool {
        self.buyer_signature.len() == 64 &&
        self.seller_signature.len() == 64 &&
        matches!(self.status, OtcStatus::Confirmed)
    }

    pub fn is_expired(&self) -> bool {
        Clock::get().map(|clock| clock.unix_timestamp > self.settlement_deadline).unwrap_or(false)
    }
}

/// TWAP (Time-Weighted Average Price) execution state
#[account]
#[derive(InitSpace)]
pub struct TwapExecution {
    /// Parent order ID
    pub order_id: u64,

    /// TWAP configuration
    pub config: TwapConfig,

    /// Execution state
    pub state: TwapState,

    /// Current execution chunk
    pub current_chunk: u32,

    /// Total chunks
    pub total_chunks: u32,

    /// Executed volume so far
    pub executed_volume: u64,

    /// Volume-weighted average price achieved
    pub vwap_achieved: u64,

    /// Next execution timestamp
    pub next_execution_at: i64,

    /// Completion timestamp
    pub completed_at: Option<i64>,

    /// Execution history
    #[max_len(100)]
    pub execution_history: Vec<ExecutionEvent>,

    /// Reserved space
    pub _reserved: [u8; 64],
}

/// Order types available for whale trading
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderType {
    Market,      // Execute immediately at market price
    Limit,       // Execute only at specified price or better
    Stop,        // Convert to market order when price reached
    StopLimit,   // Convert to limit order when price reached
    Iceberg,     // Large order split into smaller visible chunks
    Twap,        // Time-weighted average price execution
    Vwap,        // Volume-weighted average price execution
}

/// Order side (buy or sell)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order status throughout lifecycle
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderStatus {
    Pending,         // Order created, waiting for execution
    PartiallyFilled, // Order partially executed
    Filled,          // Order fully executed
    Cancelled,       // Order cancelled by user
    Rejected,        // Order rejected by system
    Expired,         // Order expired
    Failed,          // Order execution failed
}

/// Time in force options
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeInForce {
    GoodTillCancelled, // Order active until cancelled
    ImmediateOrCancel, // Execute immediately, cancel remainder
    FillOrKill,        // Execute completely or cancel entirely
    GoodTillTime,      // Active until specified time
}

/// OTC trade status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OtcStatus {
    Created,      // Trade proposal created
    Negotiating,  // Parties negotiating terms
    Confirmed,    // Both parties agreed
    Settling,     // Settlement in progress
    Settled,      // Trade completed
    Cancelled,    // Trade cancelled
    Disputed,     // Trade under dispute
}

/// TWAP execution state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TwapState {
    Scheduled,   // TWAP scheduled for execution
    Executing,   // Currently executing chunks
    Paused,      // Execution paused
    Completed,   // All chunks executed
    Cancelled,   // TWAP cancelled
    Failed,      // TWAP execution failed
}

/// Order metadata for additional information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OrderMetadata {
    /// Client order ID (for reference)
    pub client_order_id: Option<u64>,

    /// Order source/origin
    #[max_len(32)]
    pub source: String,

    /// Trading pair symbol
    #[max_len(20)]
    pub symbol: String,

    /// Order priority
    pub priority: OrderPriority,

    /// Risk parameters
    pub risk_params: RiskParams,

    /// Execution venue preferences
    pub venue_preferences: VenuePreferences,
}

impl Default for OrderMetadata {
    fn default() -> Self {
        Self {
            client_order_id: None,
            source: String::new(),
            symbol: String::new(),
            priority: OrderPriority::Normal,
            risk_params: RiskParams::default(),
            venue_preferences: VenuePreferences::default(),
        }
    }
}

/// Execution parameters for complex strategies
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExecutionParams {
    /// Minimum chunk size
    pub min_chunk_size: u64,

    /// Maximum chunk size
    pub max_chunk_size: u64,

    /// Time between executions (seconds)
    pub execution_interval: u32,

    /// Randomization factor (0-100)
    pub randomization: u8,

    /// Market impact threshold
    pub impact_threshold: u16,

    /// Participation rate (0-100)
    pub participation_rate: u8,
}

impl Default for ExecutionParams {
    fn default() -> Self {
        Self {
            min_chunk_size: 1_000_000, // $1M minimum chunk
            max_chunk_size: 10_000_000, // $10M maximum chunk
            execution_interval: 300, // 5 minutes
            randomization: 10, // 10% randomization
            impact_threshold: 200, // 2% impact threshold
            participation_rate: 20, // 20% of volume
        }
    }
}

/// Cross-chain execution information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainInfo {
    /// Target network
    pub target_network: u8,

    /// Target contract address
    pub target_contract: [u8; 32],

    /// Bridge transaction hash
    #[max_len(64)]
    pub bridge_tx_hash: String,

    /// Confirmation status
    pub confirmed: bool,

    /// Required confirmations
    pub required_confirmations: u8,

    /// Current confirmations
    pub current_confirmations: u8,
}

/// Settlement parameters for OTC trades
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SettlementParams {
    /// Settlement method
    pub method: SettlementMethod,

    /// Required confirmations
    pub confirmations_required: u8,

    /// Settlement fee
    pub settlement_fee: u64,

    /// Dispute resolution mechanism
    pub dispute_resolution: DisputeResolution,
}

impl Default for SettlementParams {
    fn default() -> Self {
        Self {
            method: SettlementMethod::Atomic,
            confirmations_required: 2,
            settlement_fee: 0,
            dispute_resolution: DisputeResolution::Arbitration,
        }
    }
}

/// Privacy configuration for trades
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PrivacyConfig {
    /// Hide trade size
    pub hide_size: bool,

    /// Hide trade price
    pub hide_price: bool,

    /// Hide counterparty
    pub hide_counterparty: bool,

    /// Zero-knowledge proof type
    pub zk_proof_type: ZkProofType,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            hide_size: false,
            hide_price: false,
            hide_counterparty: false,
            zk_proof_type: ZkProofType::None,
        }
    }
}

/// Trade metadata for OTC trades
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradeMetadata {
    /// Trade category
    pub category: TradeCategory,

    /// Settlement location
    #[max_len(50)]
    pub settlement_location: String,

    /// Regulatory compliance
    pub compliance: ComplianceInfo,

    /// Additional notes
    #[max_len(200)]
    pub notes: String,
}

impl Default for TradeMetadata {
    fn default() -> Self {
        Self {
            category: TradeCategory::Spot,
            settlement_location: String::new(),
            compliance: ComplianceInfo::default(),
            notes: String::new(),
        }
    }
}

/// TWAP configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TwapConfig {
    /// Total execution time (seconds)
    pub total_time: u32,

    /// Number of chunks
    pub chunk_count: u32,

    /// Time between chunks (seconds)
    pub chunk_interval: u32,

    /// Start time
    pub start_time: i64,

    /// End time
    pub end_time: i64,

    /// Participation rate
    pub participation_rate: u8,
}

/// Execution event for TWAP history
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExecutionEvent {
    /// Execution timestamp
    pub timestamp: i64,

    /// Chunk size executed
    pub size: u64,

    /// Execution price
    pub price: u64,

    /// Market impact observed
    pub market_impact: u16,

    /// Venue used
    pub venue: u8,
}

/// Supporting enums and types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RiskParams {
    pub max_position_size: u64,
    pub max_daily_volume: u64,
    pub stop_loss_threshold: u16,
    pub take_profit_threshold: u16,
}

impl Default for RiskParams {
    fn default() -> Self {
        Self {
            max_position_size: 100_000_000 * moby_math::Price::PRECISION,
            max_daily_volume: 500_000_000 * moby_math::Price::PRECISION,
            stop_loss_threshold: 1000, // 10%
            take_profit_threshold: 500, // 5%
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VenuePreferences {
    pub preferred_venues: Vec<u8>,
    pub avoid_venues: Vec<u8>,
    pub min_liquidity: u64,
}

impl Default for VenuePreferences {
    fn default() -> Self {
        Self {
            preferred_venues: Vec::new(),
            avoid_venues: Vec::new(),
            min_liquidity: 1_000_000 * moby_math::Price::PRECISION,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettlementMethod {
    Atomic,      // Simultaneous exchange
    Escrow,      // Using escrow service
    Delivery,    // Physical delivery
    Cash,        // Cash settlement
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisputeResolution {
    Arbitration,
    Mediation,
    Court,
    Dao,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZkProofType {
    None,
    Groth16,
    Plonk,
    Stark,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TradeCategory {
    Spot,
    Derivative,
    Structured,
    Repo,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ComplianceInfo {
    pub kyc_verified: bool,
    pub aml_checked: bool,
    pub jurisdiction: u8,
    pub regulatory_approval: bool,
}

impl Default for ComplianceInfo {
    fn default() -> Self {
        Self {
            kyc_verified: false,
            aml_checked: false,
            jurisdiction: 0, // Default jurisdiction
            regulatory_approval: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_order_lifecycle() {
        let mut order = WhaleOrder {
            order_id: 1,
            trader: Pubkey::default(),
            order_type: OrderType::Limit,
            base_token: Pubkey::default(),
            quote_token: Pubkey::default(),
            side: OrderSide::Buy,
            size: 1000,
            price: Some(100),
            filled_size: 0,
            avg_fill_price: 0,
            status: OrderStatus::Pending,
            execution_strategy: ExecutionStrategy::Limit,
            slippage_tolerance: 100,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 0,
            expires_at: None,
            updated_at: 0,
            privacy_enabled: false,
            zk_proof: Vec::new(),
            metadata: OrderMetadata::default(),
            execution_params: ExecutionParams::default(),
            cross_chain_info: None,
            _reserved: [0; 128],
        };

        // Test initial state
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.filled_size, 0);
        assert_eq!(order.remaining_size(), 1000);
        assert_eq!(order.fill_percentage(), 0);

        // Test partial fill
        order.update_fill(300, 95).unwrap();
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.filled_size, 300);
        assert_eq!(order.avg_fill_price, 95);
        assert_eq!(order.remaining_size(), 700);
        assert_eq!(order.fill_percentage(), 30);

        // Test another fill
        order.update_fill(200, 105).unwrap();
        assert_eq!(order.filled_size, 500);
        assert_eq!(order.avg_fill_price, 99); // (300*95 + 200*105) / 500 = 99
        assert_eq!(order.fill_percentage(), 50);

        // Test complete fill
        order.update_fill(500, 100).unwrap();
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.filled_size, 1000);
        assert_eq!(order.remaining_size(), 0);
        assert_eq!(order.fill_percentage(), 100);
    }

    #[test]
    fn test_order_cancellation() {
        let mut order = WhaleOrder {
            order_id: 1,
            trader: Pubkey::default(),
            order_type: OrderType::Limit,
            base_token: Pubkey::default(),
            quote_token: Pubkey::default(),
            side: OrderSide::Buy,
            size: 1000,
            price: Some(100),
            filled_size: 0,
            avg_fill_price: 0,
            status: OrderStatus::Pending,
            execution_strategy: ExecutionStrategy::Limit,
            slippage_tolerance: 100,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 0,
            expires_at: None,
            updated_at: 0,
            privacy_enabled: false,
            zk_proof: Vec::new(),
            metadata: OrderMetadata::default(),
            execution_params: ExecutionParams::default(),
            cross_chain_info: None,
            _reserved: [0; 128],
        };

        // Cancel order
        order.cancel().unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[test]
    fn test_notional_value_calculation() {
        let order = WhaleOrder {
            order_id: 1,
            trader: Pubkey::default(),
            order_type: OrderType::Limit,
            base_token: Pubkey::default(),
            quote_token: Pubkey::default(),
            side: OrderSide::Buy,
            size: 1000,
            price: Some(100),
            filled_size: 0,
            avg_fill_price: 0,
            status: OrderStatus::Pending,
            execution_strategy: ExecutionStrategy::Limit,
            slippage_tolerance: 100,
            time_in_force: TimeInForce::GoodTillCancelled,
            created_at: 0,
            expires_at: None,
            updated_at: 0,
            privacy_enabled: false,
            zk_proof: Vec::new(),
            metadata: OrderMetadata::default(),
            execution_params: ExecutionParams::default(),
            cross_chain_info: None,
            _reserved: [0; 128],
        };

        let notional = order.notional_value().unwrap();
        assert_eq!(notional, 100_000); // 1000 * 100
    }
}