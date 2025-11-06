use anchor_lang::prelude::*;
use crate::{WhaleAmount, Timestamp, AccountKey, TradingTier};
use std::collections::HashMap;

/// Market state for tracking liquidity and trading activity
#[account]
#[derive(InitSpace)]
pub struct MarketState {
    /// Token mint this market represents
    pub token_mint: Pubkey,

    /// Current spot price (oracle aggregated)
    pub current_price: u64,

    /// 24-hour trading volume
    pub volume_24h: u64,

    /// Total liquidity available
    pub total_liquidity: u64,

    /// Number of active orders
    pub active_orders: u32,

    /// Number of whale traders
    pub whale_traders: u32,

    /// Average trade size (24h)
    pub avg_trade_size: u64,

    /// Market maker count
    pub market_makers: u16,

    /// Price volatility (basis points)
    pub volatility: u16,

    /// Market impact factors
    pub impact_factors: ImpactFactors,

    /// Liquidity distribution
    pub liquidity_tiers: [u64; 5], // Amount in each price tier

    /// Recent price history (last 24 points)
    #[max_len(24)]
    pub price_history: Vec<PricePoint>,

    /// Market status
    pub status: MarketStatus,

    /// Last update timestamp
    pub last_updated: i64,

    /// Reserved space
    pub _reserved: [u8; 64],
}

impl MarketState {
    pub const SEED: &'static str = "market_state";

    pub fn update_price(&mut self, new_price: u64, volume: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        // Add to price history
        let price_point = PricePoint {
            price: new_price,
            volume,
            timestamp: now,
        };

        self.price_history.push(price_point);

        // Keep only last 24 points
        if self.price_history.len() > 24 {
            self.price_history.remove(0);
        }

        self.current_price = new_price;
        self.last_updated = now;

        // Update volatility calculation
        self.update_volatility();

        Ok(())
    }

    fn update_volatility(&mut self) {
        if self.price_history.len() < 2 {
            return;
        }

        let prices: Vec<f64> = self.price_history
            .iter()
            .map(|p| p.price as f64)
            .collect();

        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        let std_dev = variance.sqrt();
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

        // Convert to basis points (volatility as percentage * 100)
        self.volatility = (cv * 10000.0) as u16;
    }

    pub fn calculate_market_impact(&self, trade_size: u64) -> u16 {
        if self.total_liquidity == 0 {
            return 10000; // 100% impact if no liquidity
        }

        let impact_ratio = trade_size as f64 / self.total_liquidity as f64;

        // Apply impact factors
        let base_impact = impact_ratio * self.impact_factors.depth_factor as f64;
        let quadratic_impact = impact_ratio.powi(2) * self.impact_factors.concentration_factor as f64;

        let total_impact = (base_impact + quadratic_impact) * 100.0; // Convert to basis points
        total_impact.min(10000.0) as u16 // Cap at 100%
    }
}

/// Portfolio state for whale traders
#[account]
#[derive(InitSpace)]
pub struct Portfolio {
    /// Portfolio owner
    pub owner: Pubkey,

    /// Portfolio value in USD
    pub total_value_usd: u64,

    /// Portfolio performance metrics
    pub performance: PerformanceMetrics,

    /// Risk metrics
    pub risk_metrics: PortfolioRiskMetrics,

    /// Asset allocations
    #[max_len(20)]
    pub allocations: Vec<AssetAllocation>,

    /// Active positions
    #[max_len(50)]
    pub positions: Vec<Position>,

    /// Portfolio creation timestamp
    pub created_at: i64,

    /// Last rebalancing timestamp
    pub last_rebalanced: i64,

    /// Last update timestamp
    pub last_updated: i64,

    /// Portfolio status
    pub status: PortfolioStatus,

    /// Reserved space
    pub _reserved: [u8; 128],
}

impl Portfolio {
    pub const SEED: &'static str = "portfolio";

    pub fn add_position(&mut self, position: Position) -> Result<()> {
        // Check if position for this token already exists
        if let Some(existing) = self.positions.iter_mut().find(|p| p.token_mint == position.token_mint) {
            // Update existing position
            existing.amount = existing.amount
                .checked_add(position.amount)
                .ok_or(crate::MobyError::MathOverflow)?;
            existing.avg_price = self.calculate_weighted_avg_price(
                existing.amount,
                existing.avg_price,
                position.amount,
                position.avg_price,
            )?;
        } else {
            // Add new position
            self.positions.push(position);
        }

        self.update_portfolio_value()?;
        Ok(())
    }

    fn calculate_weighted_avg_price(
        &self,
        amount1: u64,
        price1: u64,
        amount2: u64,
        price2: u64,
    ) -> Result<u64> {
        let total_amount = amount1.checked_add(amount2)
            .ok_or(crate::MobyError::MathOverflow)?;

        if total_amount == 0 {
            return Ok(0);
        }

        let value1 = (amount1 as u128).checked_mul(price1 as u128)
            .ok_or(crate::MobyError::MathOverflow)?;
        let value2 = (amount2 as u128).checked_mul(price2 as u128)
            .ok_or(crate::MobyError::MathOverflow)?;

        let total_value = value1.checked_add(value2)
            .ok_or(crate::MobyError::MathOverflow)?;

        Ok((total_value / total_amount as u128) as u64)
    }

    fn update_portfolio_value(&mut self) -> Result<()> {
        let mut total_value = 0u64;

        for position in &self.positions {
            let position_value = (position.amount as u128)
                .checked_mul(position.current_price as u128)
                .ok_or(crate::MobyError::MathOverflow)?;

            total_value = total_value.checked_add(position_value as u64)
                .ok_or(crate::MobyError::MathOverflow)?;
        }

        self.total_value_usd = total_value;
        self.last_updated = Clock::get()?.unix_timestamp;

        Ok(())
    }
}

/// Liquidity pool state
#[account]
#[derive(InitSpace)]
pub struct LiquidityPool {
    /// Pool identifier
    pub pool_id: u64,

    /// Token A mint
    pub token_a: Pubkey,

    /// Token B mint
    pub token_b: Pubkey,

    /// Token A reserve
    pub reserve_a: u64,

    /// Token B reserve
    pub reserve_b: u64,

    /// Pool token mint (LP tokens)
    pub pool_token: Pubkey,

    /// Total LP token supply
    pub total_supply: u64,

    /// Pool fee (basis points)
    pub fee_bps: u16,

    /// Pool status
    pub status: PoolStatus,

    /// Whale-friendly parameters
    pub whale_params: WhalePoolParams,

    /// Liquidity providers
    #[max_len(100)]
    pub providers: Vec<LiquidityProvider>,

    /// Pool creation timestamp
    pub created_at: i64,

    /// Last update timestamp
    pub last_updated: i64,

    /// Reserved space
    pub _reserved: [u8; 64],
}

/// Supporting structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct ImpactFactors {
    pub depth_factor: u16,
    pub concentration_factor: u16,
    pub volatility_multiplier: u16,
    pub liquidity_adjustment: u16,
}

impl Default for ImpactFactors {
    fn default() -> Self {
        Self {
            depth_factor: 100,    // 1%
            concentration_factor: 50,  // 0.5%
            volatility_multiplier: 200, // 2x volatility impact
            liquidity_adjustment: 100,  // 1% liquidity adjustment
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct PricePoint {
    pub price: u64,
    pub volume: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum MarketStatus {
    Active,
    Paused,
    Halted,
    Maintenance,
    Emergency,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceMetrics {
    /// Total return (basis points)
    pub total_return_bps: i32,

    /// Annualized return (basis points)
    pub annualized_return_bps: i32,

    /// Maximum drawdown (basis points)
    pub max_drawdown_bps: u16,

    /// Sharpe ratio (scaled by 100)
    pub sharpe_ratio: i16,

    /// Number of profitable trades
    pub profitable_trades: u32,

    /// Total number of trades
    pub total_trades: u32,

    /// Average trade size
    pub avg_trade_size: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return_bps: 0,
            annualized_return_bps: 0,
            max_drawdown_bps: 0,
            sharpe_ratio: 0,
            profitable_trades: 0,
            total_trades: 0,
            avg_trade_size: 0,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PortfolioRiskMetrics {
    /// Value at Risk (1-day, 95% confidence)
    pub var_1d_95: u64,

    /// Expected Shortfall
    pub expected_shortfall: u64,

    /// Beta to market
    pub beta: i16, // Scaled by 100

    /// Portfolio concentration (Herfindahl index)
    pub concentration_index: u16,

    /// Correlation with major indices
    pub correlations: [i16; 5], // BTC, ETH, SPY, etc.
}

impl Default for PortfolioRiskMetrics {
    fn default() -> Self {
        Self {
            var_1d_95: 0,
            expected_shortfall: 0,
            beta: 100, // Beta of 1.0
            concentration_index: 0,
            correlations: [0; 5],
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AssetAllocation {
    pub token_mint: Pubkey,
    pub target_percentage: u8,
    pub current_percentage: u8,
    pub rebalance_threshold: u8,
    pub last_rebalanced: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Position {
    pub token_mint: Pubkey,
    pub amount: u64,
    pub avg_price: u64,
    pub current_price: u64,
    pub unrealized_pnl: i64,
    pub realized_pnl: i64,
    pub last_updated: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum PortfolioStatus {
    Active,
    Rebalancing,
    Liquidating,
    Frozen,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum PoolStatus {
    Active,
    Paused,
    Draining,
    Closed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WhalePoolParams {
    /// Minimum whale trade size
    pub min_whale_size: u64,

    /// Maximum impact per whale trade
    pub max_whale_impact: u16,

    /// Whale fee discount
    pub whale_fee_discount: u16,

    /// Priority execution for whales
    pub whale_priority: bool,
}

impl Default for WhalePoolParams {
    fn default() -> Self {
        Self {
            min_whale_size: 1_000_000 * moby_math::Price::PRECISION,
            max_whale_impact: 500, // 5%
            whale_fee_discount: 25, // 0.25% discount
            whale_priority: true,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityProvider {
    pub provider: Pubkey,
    pub lp_tokens: u64,
    pub share_percentage: u16,
    pub last_provided: i64,
}

/// Network state for cross-chain operations
#[account]
#[derive(InitSpace)]
pub struct NetworkState {
    /// Network identifier
    pub network_id: u8,

    /// Network name
    #[max_len(32)]
    pub name: String,

    /// Chain ID
    pub chain_id: u64,

    /// Bridge contract address
    pub bridge_contract: [u8; 32],

    /// Current block height
    pub block_height: u64,

    /// Network status
    pub status: NetworkStatus,

    /// Finality requirements
    pub finality_blocks: u16,

    /// Average block time (seconds)
    pub avg_block_time: u16,

    /// Gas price estimates
    pub gas_estimates: GasEstimates,

    /// Network performance metrics
    pub performance: NetworkPerformance,

    /// Supported tokens on this network
    #[max_len(50)]
    pub supported_tokens: Vec<Pubkey>,

    /// Last sync timestamp
    pub last_synced: i64,

    /// Reserved space
    pub _reserved: [u8; 64],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum NetworkStatus {
    Active,
    Congested,
    Degraded,
    Offline,
    Maintenance,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GasEstimates {
    /// Fast execution gas price
    pub fast: u64,

    /// Standard execution gas price
    pub standard: u64,

    /// Safe execution gas price
    pub safe: u64,

    /// Estimated confirmation times (seconds)
    pub confirmation_times: [u16; 3], // fast, standard, safe
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NetworkPerformance {
    /// Average transaction time (seconds)
    pub avg_tx_time: u16,

    /// Success rate (basis points)
    pub success_rate: u16,

    /// Network utilization (basis points)
    pub utilization: u16,

    /// Congestion level (0-100)
    pub congestion_level: u8,
}

/// Privacy pool state for confidential trading
#[account]
#[derive(InitSpace)]
pub struct PrivacyPool {
    /// Pool identifier
    pub pool_id: u64,

    /// Supported token mint
    pub token_mint: Pubkey,

    /// Anonymity set size
    pub anonymity_set_size: u32,

    /// Total deposited amount (encrypted)
    pub total_deposited: [u8; 32], // Encrypted value

    /// Number of active participants
    pub participant_count: u32,

    /// Minimum deposit amount
    pub min_deposit: u64,

    /// Maximum deposit amount
    pub max_deposit: u64,

    /// Zero-knowledge proof parameters
    pub zk_params: ZkParams,

    /// Pool status
    pub status: PrivacyPoolStatus,

    /// Withdrawal queue
    #[max_len(100)]
    pub withdrawal_queue: Vec<WithdrawalRequest>,

    /// Pool creation timestamp
    pub created_at: i64,

    /// Last activity timestamp
    pub last_activity: i64,

    /// Reserved space
    pub _reserved: [u8; 64],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ZkParams {
    /// Proof system type
    pub proof_system: u8,

    /// Circuit parameters hash
    pub circuit_hash: [u8; 32],

    /// Trusted setup parameters
    pub setup_params: [u8; 64],

    /// Verification key
    pub verification_key: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum PrivacyPoolStatus {
    Active,
    Draining,
    Closed,
    Upgrading,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WithdrawalRequest {
    pub requester: Pubkey,
    pub amount: u64,
    pub proof: [u8; 32],
    pub requested_at: i64,
    pub status: WithdrawalStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum WithdrawalStatus {
    Pending,
    Processing,
    Completed,
    Rejected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_state_price_update() {
        let mut market = MarketState {
            token_mint: Pubkey::default(),
            current_price: 0,
            volume_24h: 0,
            total_liquidity: 1_000_000 * moby_math::Price::PRECISION,
            active_orders: 0,
            whale_traders: 0,
            avg_trade_size: 0,
            market_makers: 0,
            volatility: 0,
            impact_factors: ImpactFactors::default(),
            liquidity_tiers: [0; 5],
            price_history: Vec::new(),
            status: MarketStatus::Active,
            last_updated: 0,
            _reserved: [0; 64],
        };

        // Update price
        market.update_price(1000, 500).unwrap();
        assert_eq!(market.current_price, 1000);
        assert_eq!(market.price_history.len(), 1);

        // Add more price points
        for i in 1..30 {
            market.update_price(1000 + i, 500).unwrap();
        }

        // Should keep only last 24 points
        assert_eq!(market.price_history.len(), 24);
    }

    #[test]
    fn test_market_impact_calculation() {
        let market = MarketState {
            token_mint: Pubkey::default(),
            current_price: 1000,
            volume_24h: 0,
            total_liquidity: 100_000_000 * moby_math::Price::PRECISION, // $100M liquidity
            active_orders: 0,
            whale_traders: 0,
            avg_trade_size: 0,
            market_makers: 0,
            volatility: 0,
            impact_factors: ImpactFactors::default(),
            liquidity_tiers: [0; 5],
            price_history: Vec::new(),
            status: MarketStatus::Active,
            last_updated: 0,
            _reserved: [0; 64],
        };

        // Test various trade sizes
        let small_trade = 1_000_000 * moby_math::Price::PRECISION; // $1M
        let impact = market.calculate_market_impact(small_trade);
        assert!(impact < 200); // Should be less than 2%

        let whale_trade = 20_000_000 * moby_math::Price::PRECISION; // $20M
        let whale_impact = market.calculate_market_impact(whale_trade);
        assert!(whale_impact > impact); // Larger trade should have more impact
    }

    #[test]
    fn test_portfolio_position_management() {
        let mut portfolio = Portfolio {
            owner: Pubkey::default(),
            total_value_usd: 0,
            performance: PerformanceMetrics::default(),
            risk_metrics: PortfolioRiskMetrics::default(),
            allocations: Vec::new(),
            positions: Vec::new(),
            created_at: 0,
            last_rebalanced: 0,
            last_updated: 0,
            status: PortfolioStatus::Active,
            _reserved: [0; 128],
        };

        let token_mint = Pubkey::new_unique();

        // Add position
        let position = Position {
            token_mint,
            amount: 1000,
            avg_price: 100,
            current_price: 105,
            unrealized_pnl: 5000, // 1000 * (105 - 100)
            realized_pnl: 0,
            last_updated: 0,
        };

        portfolio.add_position(position).unwrap();
        assert_eq!(portfolio.positions.len(), 1);

        // Add to existing position
        let additional_position = Position {
            token_mint,
            amount: 500,
            avg_price: 110,
            current_price: 105,
            unrealized_pnl: 0,
            realized_pnl: 0,
            last_updated: 0,
        };

        portfolio.add_position(additional_position).unwrap();
        assert_eq!(portfolio.positions.len(), 1); // Should merge
        assert_eq!(portfolio.positions[0].amount, 1500);

        // Check weighted average price calculation
        // (1000 * 100 + 500 * 110) / 1500 = 103.33
        assert_eq!(portfolio.positions[0].avg_price, 103);
    }

    #[test]
    fn test_weighted_avg_price_calculation() {
        let portfolio = Portfolio {
            owner: Pubkey::default(),
            total_value_usd: 0,
            performance: PerformanceMetrics::default(),
            risk_metrics: PortfolioRiskMetrics::default(),
            allocations: Vec::new(),
            positions: Vec::new(),
            created_at: 0,
            last_rebalanced: 0,
            last_updated: 0,
            status: PortfolioStatus::Active,
            _reserved: [0; 128],
        };

        let avg_price = portfolio.calculate_weighted_avg_price(1000, 100, 500, 110).unwrap();
        assert_eq!(avg_price, 103); // (1000*100 + 500*110) / 1500 = 103.33 -> 103
    }
}