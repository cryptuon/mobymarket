use anchor_lang::prelude::*;
use crate::{WhaleAmount, Timestamp, AccountKey};

/// Global state account for the whale trading protocol
#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    /// Protocol authority
    pub authority: Pubkey,

    /// Emergency pause state
    pub paused: bool,

    /// Emergency mode (more restrictive than pause)
    pub emergency_mode: bool,

    /// Total volume traded through the protocol
    pub total_volume_traded: u64,

    /// Total fees collected
    pub total_fees_collected: u64,

    /// Supported token mints
    #[max_len(50)]
    pub supported_tokens: Vec<Pubkey>,

    /// Fee collection account
    pub fee_collector: Pubkey,

    /// Program upgrade authority
    pub upgrade_authority: Pubkey,

    /// Protocol creation timestamp
    pub created_at: i64,

    /// Last update timestamp
    pub last_updated: i64,

    /// Minimum whale trade threshold (in USD with precision)
    pub min_whale_threshold: u64,

    /// Maximum single trade amount
    pub max_trade_amount: u64,

    /// Protocol version
    pub version: u8,

    /// Reserved space for future upgrades
    pub _reserved: [u8; 64],
}

impl GlobalState {
    pub const SEED: &'static str = "global_state";

    pub fn initialize(
        &mut self,
        authority: Pubkey,
        fee_collector: Pubkey,
        upgrade_authority: Pubkey,
    ) -> Result<()> {
        self.authority = authority;
        self.paused = false;
        self.emergency_mode = false;
        self.total_volume_traded = 0;
        self.total_fees_collected = 0;
        self.supported_tokens = Vec::new();
        self.fee_collector = fee_collector;
        self.upgrade_authority = upgrade_authority;
        self.created_at = Clock::get()?.unix_timestamp;
        self.last_updated = self.created_at;
        self.min_whale_threshold = 1_000_000 * moby_math::Price::PRECISION; // $1M minimum
        self.max_trade_amount = 1_000_000_000 * moby_math::Price::PRECISION; // $1B maximum
        self.version = 1;
        self._reserved = [0; 64];

        Ok(())
    }

    pub fn add_supported_token(&mut self, mint: Pubkey) -> Result<()> {
        if !self.supported_tokens.contains(&mint) {
            self.supported_tokens.push(mint);
        }
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn is_supported_token(&self, mint: &Pubkey) -> bool {
        self.supported_tokens.contains(mint)
    }

    pub fn update_volume(&mut self, amount: u64) -> Result<()> {
        self.total_volume_traded = self.total_volume_traded
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

/// Whale trader profile account
#[account]
#[derive(InitSpace)]
pub struct WhaleTrader {
    /// Trader's wallet address
    pub authority: Pubkey,

    /// Trader tier (determines limits and fees)
    pub tier: TradingTier,

    /// KYC verification status
    pub kyc_status: KycStatus,

    /// Total trading volume (lifetime)
    pub lifetime_volume: u64,

    /// Volume in current month
    pub monthly_volume: u64,

    /// Current month timestamp
    pub current_month: i64,

    /// Number of trades executed
    pub trade_count: u64,

    /// Average trade size
    pub avg_trade_size: u64,

    /// Risk score (0-100)
    pub risk_score: u8,

    /// Account creation timestamp
    pub created_at: i64,

    /// Last activity timestamp
    pub last_activity: i64,

    /// Account status
    pub status: AccountStatus,

    /// Referral code (if any)
    #[max_len(32)]
    pub referral_code: String,

    /// Whitelisted addresses for cross-chain
    #[max_len(10)]
    pub whitelisted_addresses: Vec<[u8; 32]>,

    /// Trading preferences
    pub preferences: TradingPreferences,

    /// Reserved space
    pub _reserved: [u8; 128],
}

impl WhaleTrader {
    pub const SEED: &'static str = "whale_trader";

    pub fn initialize(
        &mut self,
        authority: Pubkey,
        tier: TradingTier,
    ) -> Result<()> {
        self.authority = authority;
        self.tier = tier;
        self.kyc_status = KycStatus::Pending;
        self.lifetime_volume = 0;
        self.monthly_volume = 0;
        self.current_month = Clock::get()?.unix_timestamp;
        self.trade_count = 0;
        self.avg_trade_size = 0;
        self.risk_score = 50; // Start with neutral risk
        self.created_at = Clock::get()?.unix_timestamp;
        self.last_activity = self.created_at;
        self.status = AccountStatus::Active;
        self.referral_code = String::new();
        self.whitelisted_addresses = Vec::new();
        self.preferences = TradingPreferences::default();
        self._reserved = [0; 128];

        Ok(())
    }

    pub fn update_volume(&mut self, amount: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let current_month_start = (now / (30 * 24 * 3600)) * (30 * 24 * 3600);

        // Reset monthly volume if new month
        if current_month_start > self.current_month {
            self.monthly_volume = 0;
            self.current_month = current_month_start;
        }

        self.lifetime_volume = self.lifetime_volume
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        self.monthly_volume = self.monthly_volume
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        self.trade_count += 1;
        self.avg_trade_size = self.lifetime_volume / self.trade_count;
        self.last_activity = now;

        // Auto-upgrade tier based on volume
        self.tier = self.calculate_tier();

        Ok(())
    }

    fn calculate_tier(&self) -> TradingTier {
        match self.lifetime_volume / moby_math::Price::PRECISION {
            0..=9_999_999 => TradingTier::Retail,
            10_000_000..=99_999_999 => TradingTier::SmallWhale,
            100_000_000..=499_999_999 => TradingTier::MediumWhale,
            500_000_000..=999_999_999 => TradingTier::LargeWhale,
            _ => TradingTier::MegaWhale,
        }
    }
}

/// Token configuration for supported assets
#[account]
#[derive(InitSpace)]
pub struct TokenConfig {
    /// Token mint address
    pub mint: Pubkey,

    /// Token symbol
    #[max_len(10)]
    pub symbol: String,

    /// Token name
    #[max_len(50)]
    pub name: String,

    /// Decimal places
    pub decimals: u8,

    /// Whether token is active for trading
    pub active: bool,

    /// Minimum trade amount in token units
    pub min_trade_amount: u64,

    /// Maximum trade amount in token units
    pub max_trade_amount: u64,

    /// Oracle configuration
    pub oracle_config: OracleConfig,

    /// Fee configuration
    pub fee_config: FeeConfig,

    /// Liquidity requirements
    pub liquidity_requirements: LiquidityRequirements,

    /// Cross-chain configuration
    pub cross_chain_config: CrossChainConfig,

    /// Creation timestamp
    pub created_at: i64,

    /// Last update timestamp
    pub last_updated: i64,

    /// Reserved space
    pub _reserved: [u8; 64],
}

/// Trading tiers for whale classification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TradingTier {
    Retail,      // < $10M lifetime
    SmallWhale,  // $10M - $100M
    MediumWhale, // $100M - $500M
    LargeWhale,  // $500M - $1B
    MegaWhale,   // > $1B
}

impl Default for TradingTier {
    fn default() -> Self {
        TradingTier::Retail
    }
}

/// KYC verification status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum KycStatus {
    Pending,
    Verified,
    Rejected,
    Expired,
}

/// Account status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccountStatus {
    Active,
    Suspended,
    Frozen,
    Closed,
}

/// Trading preferences
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradingPreferences {
    /// Default slippage tolerance (basis points)
    pub default_slippage_tolerance: u16,

    /// Preferred execution strategy
    pub execution_strategy: ExecutionStrategy,

    /// Privacy preferences
    pub privacy_enabled: bool,

    /// Auto-approval for small trades
    pub auto_approve_threshold: u64,

    /// Notification preferences
    pub notifications_enabled: bool,
}

impl Default for TradingPreferences {
    fn default() -> Self {
        Self {
            default_slippage_tolerance: 100, // 1%
            execution_strategy: ExecutionStrategy::Smart,
            privacy_enabled: false,
            auto_approve_threshold: 1_000_000 * moby_math::Price::PRECISION, // $1M
            notifications_enabled: true,
        }
    }
}

/// Execution strategy options
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExecutionStrategy {
    Market,      // Immediate execution
    Limit,       // Limit order
    Twap,        // Time-weighted average price
    Vwap,        // Volume-weighted average price
    Smart,       // Algorithm chooses best strategy
}

/// Oracle configuration for tokens
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OracleConfig {
    /// Primary oracle source
    pub primary_oracle: Pubkey,

    /// Secondary oracle sources
    pub secondary_oracles: Vec<Pubkey>,

    /// Minimum confidence threshold
    pub min_confidence: u8,

    /// Maximum price deviation allowed
    pub max_price_deviation: u16,

    /// Price staleness threshold (seconds)
    pub staleness_threshold: u32,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            primary_oracle: Pubkey::default(),
            secondary_oracles: Vec::new(),
            min_confidence: 85, // 85%
            max_price_deviation: 300, // 3%
            staleness_threshold: 300, // 5 minutes
        }
    }
}

/// Fee configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FeeConfig {
    /// Base fee (basis points)
    pub base_fee: u16,

    /// Tier-based fee discounts
    pub tier_discounts: [u16; 5], // One for each tier

    /// Volume-based fee discounts
    pub volume_discounts: Vec<VolumeDiscount>,

    /// Privacy fee premium
    pub privacy_premium: u16,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            base_fee: 30, // 0.3%
            tier_discounts: [0, 5, 10, 15, 20], // Discounts by tier
            volume_discounts: Vec::new(),
            privacy_premium: 10, // 0.1% additional for privacy
        }
    }
}

/// Volume-based discount structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VolumeDiscount {
    pub min_volume: u64,
    pub discount_bps: u16,
}

/// Liquidity requirements
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityRequirements {
    /// Minimum liquidity ratio required
    pub min_liquidity_ratio: u16,

    /// Maximum impact per trade
    pub max_impact_per_trade: u16,

    /// Liquidity buffer requirement
    pub liquidity_buffer: u16,
}

impl Default for LiquidityRequirements {
    fn default() -> Self {
        Self {
            min_liquidity_ratio: 200, // 2x trade size
            max_impact_per_trade: 500, // 5% max impact
            liquidity_buffer: 1000, // 10% buffer
        }
    }
}

/// Cross-chain configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrossChainConfig {
    /// Supported networks
    pub supported_networks: Vec<u8>,

    /// Bridge contracts
    pub bridge_contracts: Vec<[u8; 32]>,

    /// Verification requirements
    pub verification_threshold: u8,

    /// Cross-chain fees
    pub cross_chain_fee: u16,
}

impl Default for CrossChainConfig {
    fn default() -> Self {
        Self {
            supported_networks: Vec::new(),
            bridge_contracts: Vec::new(),
            verification_threshold: 2, // 2/3 consensus
            cross_chain_fee: 50, // 0.5%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_tier_classification() {
        let mut trader = WhaleTrader {
            authority: Pubkey::default(),
            tier: TradingTier::Retail,
            kyc_status: KycStatus::Pending,
            lifetime_volume: 0,
            monthly_volume: 0,
            current_month: 0,
            trade_count: 0,
            avg_trade_size: 0,
            risk_score: 50,
            created_at: 0,
            last_activity: 0,
            status: AccountStatus::Active,
            referral_code: String::new(),
            whitelisted_addresses: Vec::new(),
            preferences: TradingPreferences::default(),
            _reserved: [0; 128],
        };

        // Test tier calculation
        trader.lifetime_volume = 5_000_000 * moby_math::Price::PRECISION;
        assert_eq!(trader.calculate_tier(), TradingTier::Retail);

        trader.lifetime_volume = 50_000_000 * moby_math::Price::PRECISION;
        assert_eq!(trader.calculate_tier(), TradingTier::SmallWhale);

        trader.lifetime_volume = 200_000_000 * moby_math::Price::PRECISION;
        assert_eq!(trader.calculate_tier(), TradingTier::MediumWhale);

        trader.lifetime_volume = 750_000_000 * moby_math::Price::PRECISION;
        assert_eq!(trader.calculate_tier(), TradingTier::LargeWhale);

        trader.lifetime_volume = 1_500_000_000 * moby_math::Price::PRECISION;
        assert_eq!(trader.calculate_tier(), TradingTier::MegaWhale);
    }

    #[test]
    fn test_global_state_initialization() {
        let mut global_state = GlobalState {
            authority: Pubkey::default(),
            paused: true,
            emergency_mode: true,
            total_volume_traded: 100,
            total_fees_collected: 10,
            supported_tokens: vec![Pubkey::new_unique()],
            fee_collector: Pubkey::default(),
            upgrade_authority: Pubkey::default(),
            created_at: 0,
            last_updated: 0,
            min_whale_threshold: 0,
            max_trade_amount: 0,
            version: 0,
            _reserved: [0; 64],
        };

        let authority = Pubkey::new_unique();
        let fee_collector = Pubkey::new_unique();
        let upgrade_authority = Pubkey::new_unique();

        global_state.initialize(authority, fee_collector, upgrade_authority).unwrap();

        assert_eq!(global_state.authority, authority);
        assert!(!global_state.paused);
        assert!(!global_state.emergency_mode);
        assert_eq!(global_state.total_volume_traded, 0);
        assert_eq!(global_state.total_fees_collected, 0);
        assert_eq!(global_state.supported_tokens.len(), 0);
        assert_eq!(global_state.min_whale_threshold, 1_000_000 * moby_math::Price::PRECISION);
        assert_eq!(global_state.version, 1);
    }

    #[test]
    fn test_supported_token_management() {
        let mut global_state = GlobalState {
            authority: Pubkey::default(),
            paused: false,
            emergency_mode: false,
            total_volume_traded: 0,
            total_fees_collected: 0,
            supported_tokens: Vec::new(),
            fee_collector: Pubkey::default(),
            upgrade_authority: Pubkey::default(),
            created_at: 0,
            last_updated: 0,
            min_whale_threshold: 0,
            max_trade_amount: 0,
            version: 1,
            _reserved: [0; 64],
        };

        let token_mint = Pubkey::new_unique();

        // Add token
        global_state.add_supported_token(token_mint).unwrap();
        assert!(global_state.is_supported_token(&token_mint));
        assert_eq!(global_state.supported_tokens.len(), 1);

        // Adding same token again should not duplicate
        global_state.add_supported_token(token_mint).unwrap();
        assert_eq!(global_state.supported_tokens.len(), 1);

        // Check non-existent token
        let other_token = Pubkey::new_unique();
        assert!(!global_state.is_supported_token(&other_token));
    }
}