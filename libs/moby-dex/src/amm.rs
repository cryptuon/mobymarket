//! # Automated Market Maker (AMM) Implementation
//!
//! This module provides sophisticated AMM algorithms optimized for whale trading,
//! including constant product, constant sum, and concentrated liquidity models.

use crate::error::{DEXError, DEXResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Types of AMM algorithms supported
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AMMType {
    /// Constant product formula (x * y = k) - Uniswap V2 style
    ConstantProduct,
    /// Constant sum formula (x + y = k) - for stable pairs
    ConstantSum,
    /// Concentrated liquidity - Uniswap V3 style
    ConcentratedLiquidity {
        tick_spacing: u32,
        fee_tier: u32,
    },
    /// Curve StableSwap - for correlated assets
    StableSwap {
        amplification: u32,
    },
    /// Balancer weighted pools
    WeightedPool {
        weights: Vec<Decimal>,
    },
    /// Custom AMM with configurable parameters
    Custom {
        name: String,
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// AMM pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Pool identifier
    pub id: String,
    /// Trading pair (e.g., "ETH/USDC")
    pub pair: String,
    /// Token A address
    pub token_a: String,
    /// Token B address
    pub token_b: String,
    /// AMM algorithm type
    pub amm_type: AMMType,
    /// Trading fee (as decimal, e.g., 0.003 for 0.3%)
    pub fee: Decimal,
    /// Whale trading fee (for large trades)
    pub whale_fee: Decimal,
    /// Protocol fee (portion of trading fee)
    pub protocol_fee: Decimal,
    /// Minimum liquidity requirement
    pub min_liquidity: Decimal,
    /// Maximum position size (as percentage of pool)
    pub max_position_percentage: Decimal,
    /// Slippage protection threshold
    pub slippage_threshold: Decimal,
    /// Price impact threshold for whale trades
    pub whale_price_impact_threshold: Decimal,
    /// Pool is active
    pub is_active: bool,
    /// Pool creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Current state of an AMM pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    /// Pool configuration
    pub config: PoolConfig,
    /// Current reserve of token A
    pub reserve_a: Decimal,
    /// Current reserve of token B
    pub reserve_b: Decimal,
    /// Total liquidity tokens outstanding
    pub total_liquidity: Decimal,
    /// Current price (token B per token A)
    pub current_price: Decimal,
    /// 24h volume in USD
    pub volume_24h: Decimal,
    /// 24h fees collected
    pub fees_24h: Decimal,
    /// Number of liquidity providers
    pub lp_count: u32,
    /// Last trade timestamp
    pub last_trade_at: Option<DateTime<Utc>>,
    /// Pool health score (0.0 to 1.0)
    pub health_score: f64,
    /// Concentrated liquidity specific state
    pub concentrated_liquidity: Option<ConcentratedLiquidityState>,
}

/// State for concentrated liquidity pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentratedLiquidityState {
    /// Current tick
    pub current_tick: i32,
    /// Liquidity at current tick
    pub current_liquidity: Decimal,
    /// Active liquidity positions
    pub active_positions: HashMap<String, LiquidityPosition>,
    /// Tick spacing
    pub tick_spacing: u32,
    /// Fee tier
    pub fee_tier: u32,
}

/// Liquidity position in concentrated liquidity pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    /// Position ID
    pub id: String,
    /// Lower tick boundary
    pub tick_lower: i32,
    /// Upper tick boundary
    pub tick_upper: i32,
    /// Liquidity amount
    pub liquidity: Decimal,
    /// Fees earned token A
    pub fees_a: Decimal,
    /// Fees earned token B
    pub fees_b: Decimal,
    /// Position creation time
    pub created_at: DateTime<Utc>,
}

/// Result of a swap operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    /// Trade ID
    pub trade_id: String,
    /// Input token amount
    pub amount_in: Decimal,
    /// Output token amount
    pub amount_out: Decimal,
    /// Fee paid
    pub fee_paid: Decimal,
    /// Price impact (as percentage)
    pub price_impact: Decimal,
    /// Slippage (as percentage)
    pub slippage: Decimal,
    /// New pool state after swap
    pub new_pool_state: PoolState,
    /// Execution timestamp
    pub executed_at: DateTime<Utc>,
    /// Gas used (if applicable)
    pub gas_used: Option<u64>,
    /// Whale trade indicators
    pub whale_metrics: WhaleTradeMetrics,
}

/// Metrics specific to whale trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleTradeMetrics {
    /// Whether this qualifies as a whale trade
    pub is_whale_trade: bool,
    /// Estimated liquidity after trade
    pub remaining_liquidity: Decimal,
    /// Recommended max trade size
    pub max_recommended_size: Decimal,
    /// Price recovery estimate (in blocks)
    pub price_recovery_blocks: u32,
    /// Arbitrage opportunity created
    pub arbitrage_opportunity: Option<Decimal>,
}

/// Parameters for calculating optimal trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeParameters {
    /// Input token amount
    pub amount_in: Decimal,
    /// Minimum acceptable output amount
    pub min_amount_out: Decimal,
    /// Maximum acceptable slippage
    pub max_slippage: Decimal,
    /// Trade deadline
    pub deadline: DateTime<Utc>,
    /// Whether to use whale optimization
    pub whale_optimization: bool,
    /// Split trade into multiple parts
    pub split_trade: bool,
    /// Maximum parts to split into
    pub max_parts: u32,
}

/// Main AMM pool implementation
pub struct AMMPool {
    state: PoolState,
}

impl AMMPool {
    /// Create a new constant product AMM pool
    pub fn constant_product(
        pair: &str,
        token_a: &str,
        token_b: &str,
        fee: Decimal,
    ) -> DEXResult<Self> {
        let config = PoolConfig {
            id: Uuid::new_v4().to_string(),
            pair: pair.to_string(),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            amm_type: AMMType::ConstantProduct,
            fee,
            whale_fee: fee * Decimal::from_f64_retain(1.5).unwrap(), // 1.5x fee for whale trades
            protocol_fee: fee * Decimal::from_f64_retain(0.1).unwrap(), // 10% of fee
            min_liquidity: Decimal::from(10_000), // $10k minimum
            max_position_percentage: Decimal::from_f64_retain(0.1).unwrap(), // 10% max
            slippage_threshold: Decimal::from_f64_retain(0.05).unwrap(), // 5%
            whale_price_impact_threshold: Decimal::from_f64_retain(0.03).unwrap(), // 3%
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let state = PoolState {
            config,
            reserve_a: Decimal::ZERO,
            reserve_b: Decimal::ZERO,
            total_liquidity: Decimal::ZERO,
            current_price: Decimal::ZERO,
            volume_24h: Decimal::ZERO,
            fees_24h: Decimal::ZERO,
            lp_count: 0,
            last_trade_at: None,
            health_score: 1.0,
            concentrated_liquidity: None,
        };

        Ok(Self { state })
    }

    /// Create a new concentrated liquidity pool
    pub fn concentrated_liquidity(
        pair: &str,
        token_a: &str,
        token_b: &str,
        fee: Decimal,
        tick_spacing: u32,
    ) -> DEXResult<Self> {
        let config = PoolConfig {
            id: Uuid::new_v4().to_string(),
            pair: pair.to_string(),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            amm_type: AMMType::ConcentratedLiquidity {
                tick_spacing,
                fee_tier: (fee * Decimal::from(1_000_000)).to_string().parse().unwrap_or(3000),
            },
            fee,
            whale_fee: fee * Decimal::from_f64_retain(1.2).unwrap(),
            protocol_fee: fee * Decimal::from_f64_retain(0.05).unwrap(),
            min_liquidity: Decimal::from(50_000), // Higher minimum for concentrated liquidity
            max_position_percentage: Decimal::from_f64_retain(0.15).unwrap(),
            slippage_threshold: Decimal::from_f64_retain(0.03).unwrap(),
            whale_price_impact_threshold: Decimal::from_f64_retain(0.02).unwrap(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let concentrated_liquidity = ConcentratedLiquidityState {
            current_tick: 0,
            current_liquidity: Decimal::ZERO,
            active_positions: HashMap::new(),
            tick_spacing,
            fee_tier: (fee * Decimal::from(1_000_000)).to_string().parse().unwrap_or(3000),
        };

        let state = PoolState {
            config,
            reserve_a: Decimal::ZERO,
            reserve_b: Decimal::ZERO,
            total_liquidity: Decimal::ZERO,
            current_price: Decimal::ZERO,
            volume_24h: Decimal::ZERO,
            fees_24h: Decimal::ZERO,
            lp_count: 0,
            last_trade_at: None,
            health_score: 1.0,
            concentrated_liquidity: Some(concentrated_liquidity),
        };

        Ok(Self { state })
    }

    /// Add initial liquidity to the pool
    pub fn add_initial_liquidity(
        &mut self,
        amount_a: Decimal,
        amount_b: Decimal,
    ) -> DEXResult<Decimal> {
        if self.state.total_liquidity > Decimal::ZERO {
            return Err(DEXError::InvalidPoolConfig {
                reason: "Pool already has liquidity".to_string(),
            });
        }

        if amount_a <= Decimal::ZERO || amount_b <= Decimal::ZERO {
            return Err(DEXError::InvalidLiquidityPosition {
                reason: "Initial liquidity amounts must be positive".to_string(),
            });
        }

        // Calculate initial liquidity tokens using geometric mean
        let liquidity = (amount_a * amount_b).sqrt()
            .ok_or_else(|| DEXError::MathematicalOverflow {
                operation: "initial liquidity calculation".to_string(),
            })?;

        self.state.reserve_a = amount_a;
        self.state.reserve_b = amount_b;
        self.state.total_liquidity = liquidity;
        self.state.current_price = amount_b / amount_a;
        self.state.lp_count = 1;
        self.state.updated_at = Utc::now();

        Ok(liquidity)
    }

    /// Add liquidity to existing pool
    pub fn add_liquidity(
        &mut self,
        amount_a: Decimal,
        amount_b: Decimal,
    ) -> DEXResult<Decimal> {
        if self.state.total_liquidity == Decimal::ZERO {
            return self.add_initial_liquidity(amount_a, amount_b);
        }

        // Check if amounts maintain current ratio
        let current_ratio = self.state.reserve_b / self.state.reserve_a;
        let provided_ratio = amount_b / amount_a;

        let ratio_difference = (current_ratio - provided_ratio).abs() / current_ratio;
        if ratio_difference > Decimal::from_f64_retain(0.02).unwrap() { // 2% tolerance
            return Err(DEXError::InvalidLiquidityPosition {
                reason: format!("Liquidity ratio mismatch: expected {:.4}, got {:.4}",
                    current_ratio, provided_ratio),
            });
        }

        // Calculate liquidity tokens to mint
        let liquidity_a = amount_a * self.state.total_liquidity / self.state.reserve_a;
        let liquidity_b = amount_b * self.state.total_liquidity / self.state.reserve_b;
        let liquidity = liquidity_a.min(liquidity_b);

        // Update reserves
        self.state.reserve_a += amount_a;
        self.state.reserve_b += amount_b;
        self.state.total_liquidity += liquidity;
        self.state.lp_count += 1;
        self.state.updated_at = Utc::now();

        Ok(liquidity)
    }

    /// Remove liquidity from pool
    pub fn remove_liquidity(&mut self, liquidity_tokens: Decimal) -> DEXResult<(Decimal, Decimal)> {
        if liquidity_tokens <= Decimal::ZERO {
            return Err(DEXError::InvalidQuantity { quantity: liquidity_tokens });
        }

        if liquidity_tokens > self.state.total_liquidity {
            return Err(DEXError::InsufficientLiquidityForWithdrawal {
                available: self.state.total_liquidity,
                requested: liquidity_tokens,
            });
        }

        // Calculate amounts to return
        let amount_a = liquidity_tokens * self.state.reserve_a / self.state.total_liquidity;
        let amount_b = liquidity_tokens * self.state.reserve_b / self.state.total_liquidity;

        // Update reserves
        self.state.reserve_a -= amount_a;
        self.state.reserve_b -= amount_b;
        self.state.total_liquidity -= liquidity_tokens;
        self.state.lp_count = self.state.lp_count.saturating_sub(1);
        self.state.updated_at = Utc::now();

        // Update current price
        if self.state.reserve_a > Decimal::ZERO {
            self.state.current_price = self.state.reserve_b / self.state.reserve_a;
        }

        Ok((amount_a, amount_b))
    }

    /// Execute a swap with whale optimization
    pub async fn swap(
        &mut self,
        amount_in: Decimal,
        token_in: &str,
        params: TradeParameters,
    ) -> DEXResult<SwapResult> {
        // Validate inputs
        self.validate_swap_inputs(amount_in, token_in, &params)?;

        // Check if this is a whale trade
        let is_whale_trade = self.is_whale_trade(amount_in, token_in)?;

        // Calculate swap based on AMM type
        let swap_result = match &self.state.config.amm_type {
            AMMType::ConstantProduct => {
                self.swap_constant_product(amount_in, token_in, &params, is_whale_trade).await?
            }
            AMMType::ConstantSum => {
                self.swap_constant_sum(amount_in, token_in, &params, is_whale_trade).await?
            }
            AMMType::ConcentratedLiquidity { .. } => {
                self.swap_concentrated_liquidity(amount_in, token_in, &params, is_whale_trade).await?
            }
            AMMType::StableSwap { amplification } => {
                self.swap_stable_swap(amount_in, token_in, &params, is_whale_trade, *amplification).await?
            }
            _ => {
                return Err(DEXError::NotImplemented {
                    feature: format!("AMM type: {:?}", self.state.config.amm_type),
                });
            }
        };

        // Update pool state
        self.update_pool_state_after_swap(&swap_result);

        Ok(swap_result)
    }

    /// Constant product AMM swap (x * y = k)
    async fn swap_constant_product(
        &mut self,
        amount_in: Decimal,
        token_in: &str,
        params: &TradeParameters,
        is_whale_trade: bool,
    ) -> DEXResult<SwapResult> {
        let (reserve_in, reserve_out) = if token_in == self.state.config.token_a {
            (self.state.reserve_a, self.state.reserve_b)
        } else {
            (self.state.reserve_b, self.state.reserve_a)
        };

        // Apply fee
        let fee = if is_whale_trade {
            self.state.config.whale_fee
        } else {
            self.state.config.fee
        };

        let amount_in_after_fee = amount_in * (Decimal::ONE - fee);

        // Calculate output using constant product formula
        let amount_out = (reserve_out * amount_in_after_fee) / (reserve_in + amount_in_after_fee);

        // Check minimum output
        if amount_out < params.min_amount_out {
            return Err(DEXError::SlippageExceeded {
                expected: ((params.min_amount_out - amount_out) / amount_out * Decimal::from(100))
                    .to_string().parse().unwrap_or(0.0),
                actual: 0.0,
            });
        }

        // Calculate price impact
        let old_price = reserve_out / reserve_in;
        let new_reserve_in = reserve_in + amount_in;
        let new_reserve_out = reserve_out - amount_out;
        let new_price = new_reserve_out / new_reserve_in;
        let price_impact = ((old_price - new_price).abs() / old_price * Decimal::from(100))
            .to_string().parse().unwrap_or(0.0);

        // Check price impact for whale trades
        if is_whale_trade && Decimal::from_f64_retain(price_impact).unwrap() > self.state.config.whale_price_impact_threshold {
            return Err(DEXError::PriceImpactTooHigh {
                impact: price_impact,
                trade_size: amount_in,
            });
        }

        // Calculate slippage
        let expected_price = self.state.current_price;
        let actual_price = amount_out / amount_in;
        let slippage = ((expected_price - actual_price).abs() / expected_price * Decimal::from(100))
            .to_string().parse().unwrap_or(0.0);

        // Update reserves
        if token_in == self.state.config.token_a {
            self.state.reserve_a = new_reserve_in;
            self.state.reserve_b = new_reserve_out;
        } else {
            self.state.reserve_a = new_reserve_out;
            self.state.reserve_b = new_reserve_in;
        }

        // Calculate whale metrics
        let whale_metrics = self.calculate_whale_metrics(amount_in, amount_out, is_whale_trade);

        let swap_result = SwapResult {
            trade_id: Uuid::new_v4().to_string(),
            amount_in,
            amount_out,
            fee_paid: amount_in * fee,
            price_impact: Decimal::from_f64_retain(price_impact).unwrap(),
            slippage: Decimal::from_f64_retain(slippage).unwrap(),
            new_pool_state: self.state.clone(),
            executed_at: Utc::now(),
            gas_used: Some(150_000), // Estimated gas usage
            whale_metrics,
        };

        Ok(swap_result)
    }

    /// Constant sum AMM swap (x + y = k) - for stable pairs
    async fn swap_constant_sum(
        &mut self,
        amount_in: Decimal,
        token_in: &str,
        params: &TradeParameters,
        is_whale_trade: bool,
    ) -> DEXResult<SwapResult> {
        // For constant sum, price is always 1:1 (minus fees)
        let fee = if is_whale_trade {
            self.state.config.whale_fee
        } else {
            self.state.config.fee
        };

        let amount_out = amount_in * (Decimal::ONE - fee);

        if amount_out < params.min_amount_out {
            return Err(DEXError::SlippageExceeded {
                expected: ((params.min_amount_out - amount_out) / amount_out * Decimal::from(100))
                    .to_string().parse().unwrap_or(0.0),
                actual: 0.0,
            });
        }

        // Update reserves
        if token_in == self.state.config.token_a {
            self.state.reserve_a += amount_in;
            self.state.reserve_b -= amount_out;
        } else {
            self.state.reserve_a -= amount_out;
            self.state.reserve_b += amount_in;
        }

        let whale_metrics = self.calculate_whale_metrics(amount_in, amount_out, is_whale_trade);

        let swap_result = SwapResult {
            trade_id: Uuid::new_v4().to_string(),
            amount_in,
            amount_out,
            fee_paid: amount_in * fee,
            price_impact: Decimal::ZERO, // Minimal price impact for stable pairs
            slippage: Decimal::ZERO,
            new_pool_state: self.state.clone(),
            executed_at: Utc::now(),
            gas_used: Some(120_000),
            whale_metrics,
        };

        Ok(swap_result)
    }

    /// Concentrated liquidity swap
    async fn swap_concentrated_liquidity(
        &mut self,
        amount_in: Decimal,
        token_in: &str,
        params: &TradeParameters,
        is_whale_trade: bool,
    ) -> DEXResult<SwapResult> {
        // Simplified concentrated liquidity implementation
        // In practice, this would involve tick calculations and multiple price ranges

        let fee = if is_whale_trade {
            self.state.config.whale_fee
        } else {
            self.state.config.fee
        };

        // Use current active liquidity for calculation
        let cl_state = self.state.concentrated_liquidity.as_ref()
            .ok_or_else(|| DEXError::InvalidPoolConfig {
                reason: "Concentrated liquidity state not found".to_string(),
            })?;

        if cl_state.current_liquidity == Decimal::ZERO {
            return Err(DEXError::InsufficientLiquidity {
                pool_id: self.state.config.id.clone(),
                available: Decimal::ZERO,
                required: amount_in,
            });
        }

        // Simplified calculation - would need proper tick math in production
        let amount_in_after_fee = amount_in * (Decimal::ONE - fee);
        let amount_out = amount_in_after_fee * self.state.current_price;

        if amount_out < params.min_amount_out {
            return Err(DEXError::SlippageExceeded {
                expected: ((params.min_amount_out - amount_out) / amount_out * Decimal::from(100))
                    .to_string().parse().unwrap_or(0.0),
                actual: 0.0,
            });
        }

        let whale_metrics = self.calculate_whale_metrics(amount_in, amount_out, is_whale_trade);

        let swap_result = SwapResult {
            trade_id: Uuid::new_v4().to_string(),
            amount_in,
            amount_out,
            fee_paid: amount_in * fee,
            price_impact: Decimal::from_f64_retain(0.5).unwrap(), // Lower impact with concentrated liquidity
            slippage: Decimal::from_f64_retain(0.1).unwrap(),
            new_pool_state: self.state.clone(),
            executed_at: Utc::now(),
            gas_used: Some(200_000), // Higher gas for concentrated liquidity
            whale_metrics,
        };

        Ok(swap_result)
    }

    /// StableSwap AMM for correlated assets
    async fn swap_stable_swap(
        &mut self,
        amount_in: Decimal,
        token_in: &str,
        params: &TradeParameters,
        is_whale_trade: bool,
        amplification: u32,
    ) -> DEXResult<SwapResult> {
        // Simplified StableSwap implementation
        // Real implementation would use the full StableSwap invariant

        let fee = if is_whale_trade {
            self.state.config.whale_fee
        } else {
            self.state.config.fee
        };

        let amp_factor = Decimal::from(amplification);
        let amount_in_after_fee = amount_in * (Decimal::ONE - fee);

        // Simplified calculation with amplification factor
        let amount_out = amount_in_after_fee * (Decimal::ONE + Decimal::ONE / amp_factor);

        if amount_out < params.min_amount_out {
            return Err(DEXError::SlippageExceeded {
                expected: ((params.min_amount_out - amount_out) / amount_out * Decimal::from(100))
                    .to_string().parse().unwrap_or(0.0),
                actual: 0.0,
            });
        }

        let whale_metrics = self.calculate_whale_metrics(amount_in, amount_out, is_whale_trade);

        let swap_result = SwapResult {
            trade_id: Uuid::new_v4().to_string(),
            amount_in,
            amount_out,
            fee_paid: amount_in * fee,
            price_impact: Decimal::from_f64_retain(0.1).unwrap(), // Very low impact for stable pairs
            slippage: Decimal::from_f64_retain(0.05).unwrap(),
            new_pool_state: self.state.clone(),
            executed_at: Utc::now(),
            gas_used: Some(180_000),
            whale_metrics,
        };

        Ok(swap_result)
    }

    /// Validate swap inputs
    fn validate_swap_inputs(
        &self,
        amount_in: Decimal,
        token_in: &str,
        params: &TradeParameters,
    ) -> DEXResult<()> {
        if amount_in <= Decimal::ZERO {
            return Err(DEXError::InvalidQuantity { quantity: amount_in });
        }

        if token_in != self.state.config.token_a && token_in != self.state.config.token_b {
            return Err(DEXError::UnsupportedTradingPair {
                pair: format!("{}/{}", token_in, "unknown"),
            });
        }

        if params.deadline < Utc::now() {
            return Err(DEXError::InvalidDeadline { deadline: params.deadline });
        }

        if !self.state.config.is_active {
            return Err(DEXError::TradingPaused {
                pair: self.state.config.pair.clone(),
            });
        }

        // Check if pool has sufficient liquidity
        let (reserve_in, _) = if token_in == self.state.config.token_a {
            (self.state.reserve_a, self.state.reserve_b)
        } else {
            (self.state.reserve_b, self.state.reserve_a)
        };

        if amount_in > reserve_in * self.state.config.max_position_percentage {
            return Err(DEXError::MaximumTradeAmount {
                amount: amount_in,
                maximum: reserve_in * self.state.config.max_position_percentage,
            });
        }

        Ok(())
    }

    /// Check if trade qualifies as whale trade
    fn is_whale_trade(&self, amount_in: Decimal, token_in: &str) -> DEXResult<bool> {
        let threshold = Decimal::from_f64_retain(crate::WHALE_TRADE_THRESHOLD).unwrap();

        // Estimate USD value (simplified - would use oracle in practice)
        let estimated_usd_value = if token_in == self.state.config.token_a {
            amount_in * self.state.current_price
        } else {
            amount_in
        };

        Ok(estimated_usd_value >= threshold)
    }

    /// Calculate whale-specific metrics
    fn calculate_whale_metrics(
        &self,
        amount_in: Decimal,
        amount_out: Decimal,
        is_whale_trade: bool,
    ) -> WhaleTradeMetrics {
        let remaining_liquidity = self.state.reserve_a + self.state.reserve_b;
        let max_recommended_size = remaining_liquidity * self.state.config.max_position_percentage;

        // Estimate price recovery (simplified)
        let price_recovery_blocks = if is_whale_trade { 5 } else { 1 };

        // Check for arbitrage opportunity
        let arbitrage_opportunity = if is_whale_trade {
            Some(amount_out * Decimal::from_f64_retain(0.002).unwrap()) // 0.2% arb opportunity
        } else {
            None
        };

        WhaleTradeMetrics {
            is_whale_trade,
            remaining_liquidity,
            max_recommended_size,
            price_recovery_blocks,
            arbitrage_opportunity,
        }
    }

    /// Update pool state after successful swap
    fn update_pool_state_after_swap(&mut self, swap_result: &SwapResult) {
        self.state.volume_24h += swap_result.amount_in;
        self.state.fees_24h += swap_result.fee_paid;
        self.state.last_trade_at = Some(swap_result.executed_at);
        self.state.updated_at = swap_result.executed_at;

        // Update current price
        if self.state.reserve_a > Decimal::ZERO {
            self.state.current_price = self.state.reserve_b / self.state.reserve_a;
        }

        // Update health score based on recent activity
        self.update_health_score();
    }

    /// Update pool health score
    fn update_health_score(&mut self) {
        let mut score = 1.0;

        // Reduce score if reserves are imbalanced
        if self.state.reserve_a > Decimal::ZERO && self.state.reserve_b > Decimal::ZERO {
            let ratio = self.state.reserve_a / self.state.reserve_b;
            let ideal_ratio = Decimal::ONE;
            let imbalance = (ratio - ideal_ratio).abs() / ideal_ratio;
            score -= imbalance.to_string().parse::<f64>().unwrap_or(0.0) * 0.5;
        }

        // Reduce score if no recent activity
        if let Some(last_trade) = self.state.last_trade_at {
            let hours_since_trade = Utc::now().signed_duration_since(last_trade).num_hours();
            if hours_since_trade > 24 {
                score -= 0.2;
            }
        }

        // Reduce score if low liquidity
        let total_liquidity_usd = (self.state.reserve_a + self.state.reserve_b * self.state.current_price)
            .to_string().parse::<f64>().unwrap_or(0.0);
        if total_liquidity_usd < self.state.config.min_liquidity.to_string().parse().unwrap_or(0.0) {
            score -= 0.3;
        }

        self.state.health_score = score.max(0.0).min(1.0);
    }

    /// Get current pool state
    pub fn get_state(&self) -> &PoolState {
        &self.state
    }

    /// Get pool ID
    pub fn id(&self) -> &str {
        &self.state.config.id
    }

    /// Get trading pair
    pub fn pair(&self) -> &str {
        &self.state.config.pair
    }

    /// Check if pool is active
    pub fn is_active(&self) -> bool {
        self.state.config.is_active
    }

    /// Pause/unpause pool
    pub fn set_active(&mut self, active: bool) {
        self.state.config.is_active = active;
        self.state.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_constant_product_pool_creation() {
        let pool = AMMPool::constant_product(
            "ETH/USDC",
            "0x123",
            "0x456",
            Decimal::from_f64_retain(0.003).unwrap(),
        );

        assert!(pool.is_ok());
        let pool = pool.unwrap();
        assert_eq!(pool.pair(), "ETH/USDC");
        assert!(pool.is_active());
    }

    #[tokio::test]
    async fn test_add_initial_liquidity() {
        let mut pool = AMMPool::constant_product(
            "ETH/USDC",
            "0x123",
            "0x456",
            Decimal::from_f64_retain(0.003).unwrap(),
        ).unwrap();

        let liquidity = pool.add_initial_liquidity(
            Decimal::from(100),
            Decimal::from(200_000),
        );

        assert!(liquidity.is_ok());
        let liquidity = liquidity.unwrap();
        assert!(liquidity > Decimal::ZERO);
        assert_eq!(pool.get_state().reserve_a, Decimal::from(100));
        assert_eq!(pool.get_state().reserve_b, Decimal::from(200_000));
    }

    #[tokio::test]
    async fn test_constant_product_swap() {
        let mut pool = AMMPool::constant_product(
            "ETH/USDC",
            "0x123",
            "0x456",
            Decimal::from_f64_retain(0.003).unwrap(),
        ).unwrap();

        // Add initial liquidity
        pool.add_initial_liquidity(
            Decimal::from(100),
            Decimal::from(200_000),
        ).unwrap();

        let params = TradeParameters {
            amount_in: Decimal::from(1),
            min_amount_out: Decimal::from(1900),
            max_slippage: Decimal::from_f64_retain(0.05).unwrap(),
            deadline: Utc::now() + chrono::Duration::hours(1),
            whale_optimization: false,
            split_trade: false,
            max_parts: 1,
        };

        let result = pool.swap(Decimal::from(1), "0x123", params).await;
        assert!(result.is_ok());

        let swap_result = result.unwrap();
        assert!(swap_result.amount_out > Decimal::ZERO);
        assert!(swap_result.fee_paid > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_concentrated_liquidity_pool() {
        let pool = AMMPool::concentrated_liquidity(
            "ETH/USDC",
            "0x123",
            "0x456",
            Decimal::from_f64_retain(0.003).unwrap(),
            60,
        );

        assert!(pool.is_ok());
        let pool = pool.unwrap();

        match &pool.get_state().config.amm_type {
            AMMType::ConcentratedLiquidity { tick_spacing, .. } => {
                assert_eq!(*tick_spacing, 60);
            }
            _ => panic!("Expected concentrated liquidity pool"),
        }
    }

    #[tokio::test]
    async fn test_whale_trade_detection() {
        let mut pool = AMMPool::constant_product(
            "ETH/USDC",
            "0x123",
            "0x456",
            Decimal::from_f64_retain(0.003).unwrap(),
        ).unwrap();

        pool.add_initial_liquidity(
            Decimal::from(1000),
            Decimal::from(2_000_000),
        ).unwrap();

        // Large trade should be detected as whale trade
        let is_whale = pool.is_whale_trade(Decimal::from(500), "0x123").unwrap();
        assert!(is_whale);

        // Small trade should not be whale trade
        let is_whale = pool.is_whale_trade(Decimal::from(1), "0x123").unwrap();
        assert!(!is_whale);
    }

    #[tokio::test]
    async fn test_liquidity_operations() {
        let mut pool = AMMPool::constant_product(
            "ETH/USDC",
            "0x123",
            "0x456",
            Decimal::from_f64_retain(0.003).unwrap(),
        ).unwrap();

        // Add initial liquidity
        let initial_liquidity = pool.add_initial_liquidity(
            Decimal::from(100),
            Decimal::from(200_000),
        ).unwrap();

        // Add more liquidity
        let additional_liquidity = pool.add_liquidity(
            Decimal::from(10),
            Decimal::from(20_000),
        ).unwrap();

        assert!(additional_liquidity > Decimal::ZERO);
        assert_eq!(pool.get_state().lp_count, 2);

        // Remove liquidity
        let (amount_a, amount_b) = pool.remove_liquidity(initial_liquidity / Decimal::from(2)).unwrap();
        assert!(amount_a > Decimal::ZERO);
        assert!(amount_b > Decimal::ZERO);
    }
}