// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Fee calculation and optimization system

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{TradingError, TradingResult, OrderRequest, FeeConfig, engine::MarketData};
use moby_types::{TradingTier, ExecutionStrategy, OrderType, VolumeDiscount};
use moby_math::{Amount, Price, Percentage};

/// Fee calculation system
#[derive(Debug)]
pub struct FeeCalculator {
    config: FeeConfig,
    volume_history: HashMap<moby_types::AccountKey, VolumeHistory>,
}

impl FeeCalculator {
    /// Create a new fee calculator
    pub fn new(config: FeeConfig) -> Self {
        Self {
            config,
            volume_history: HashMap::new(),
        }
    }

    /// Calculate fees for an order
    pub fn calculate_fees(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
    ) -> TradingResult<FeeEstimate> {
        let order_value = order_request.size.checked_mul(&market_data.mid_price.into())
            .ok_or_else(|| TradingError::internal("Order value calculation overflow"))?;

        // Calculate base fee
        let base_fee = self.calculate_base_fee(order_value, &order_request.tier)?;

        // Apply strategy-specific fees
        let strategy_fee = self.calculate_strategy_fee(order_value, &order_request.execution_strategy)?;

        // Apply privacy premium if enabled
        let privacy_fee = if order_request.privacy_enabled {
            order_value.checked_mul(&Percentage::from_basis_points(self.config.privacy_fee_bps)?.into())
                .ok_or_else(|| TradingError::internal("Privacy fee calculation overflow"))?
        } else {
            Amount::zero()
        };

        // Apply OTC fee if enabled
        let otc_fee = if order_request.otc_enabled {
            order_value.checked_mul(&Percentage::from_basis_points(self.config.otc_fee_bps)?.into())
                .ok_or_else(|| TradingError::internal("OTC fee calculation overflow"))?
        } else {
            Amount::zero()
        };

        // Apply cross-chain fee if enabled
        let cross_chain_fee = if order_request.cross_chain_enabled {
            order_value.checked_mul(&Percentage::from_basis_points(self.config.cross_chain_fee_bps)?.into())
                .ok_or_else(|| TradingError::internal("Cross-chain fee calculation overflow"))?
        } else {
            Amount::zero()
        };

        // Apply volume discounts
        let volume_discount = self.calculate_volume_discount(order_request.trader, order_value)?;

        // Calculate total fees
        let gross_fees = base_fee.checked_add(&strategy_fee)?
            .checked_add(&privacy_fee)?
            .checked_add(&otc_fee)?
            .checked_add(&cross_chain_fee)?;

        let total_fees = gross_fees.checked_sub(&volume_discount)
            .unwrap_or(Amount::zero()); // Don't go below zero

        Ok(FeeEstimate {
            base_fee,
            strategy_fee,
            privacy_fee,
            otc_fee,
            cross_chain_fee,
            volume_discount,
            total_fees,
            effective_rate: self.calculate_effective_rate(total_fees, order_value)?,
        })
    }

    /// Calculate fees for a quote (simplified calculation)
    pub fn calculate_quote_fees(
        &self,
        amount: Amount,
        market_data: &MarketData,
    ) -> TradingResult<Amount> {
        let value = amount.checked_mul(&market_data.mid_price.into())
            .ok_or_else(|| TradingError::internal("Quote value calculation overflow"))?;

        // Use base fee for quotes
        let fee_rate = Percentage::from_basis_points(self.config.base_fee_bps)?;
        value.checked_mul(&fee_rate.into())
            .ok_or_else(|| TradingError::internal("Quote fee calculation overflow"))
    }

    /// Update volume history for a trader
    pub fn update_volume_history(
        &mut self,
        trader: moby_types::AccountKey,
        volume: Amount,
    ) -> TradingResult<()> {
        let history = self.volume_history
            .entry(trader)
            .or_insert_with(|| VolumeHistory::new(trader));

        history.add_volume(volume)?;
        Ok(())
    }

    /// Get fee structure for a trader
    pub fn get_fee_structure(&self, trader: moby_types::AccountKey) -> FeeStructure {
        let volume_history = self.volume_history.get(&trader);
        let monthly_volume = volume_history
            .map(|h| h.get_monthly_volume())
            .unwrap_or(Amount::zero());

        let tier_discount = self.get_tier_discount_bps(monthly_volume);
        let volume_discount = self.get_volume_discount_bps(monthly_volume);

        FeeStructure {
            trader,
            base_fee_bps: self.config.base_fee_bps,
            tier_discount_bps: tier_discount,
            volume_discount_bps: volume_discount,
            effective_fee_bps: self.config.base_fee_bps
                .saturating_sub(tier_discount)
                .saturating_sub(volume_discount),
            monthly_volume,
            last_updated: Utc::now(),
        }
    }

    /// Optimize fee strategy for an order
    pub fn optimize_fee_strategy(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
    ) -> TradingResult<FeeOptimization> {
        let current_fees = self.calculate_fees(order_request, market_data)?;

        let mut optimizations = Vec::new();

        // Check if TWAP would reduce fees
        if !matches!(order_request.execution_strategy, ExecutionStrategy::Twap { .. }) {
            let mut twap_request = order_request.clone();
            twap_request.execution_strategy = ExecutionStrategy::Twap { duration_minutes: 60 };

            if let Ok(twap_fees) = self.calculate_fees(&twap_request, market_data) {
                if twap_fees.total_fees < current_fees.total_fees {
                    let savings = current_fees.total_fees.checked_sub(&twap_fees.total_fees)?;
                    optimizations.push(FeeOptimizationOption {
                        strategy: OptimizationStrategy::UseTwap,
                        potential_savings: savings,
                        trade_offs: "Longer execution time".to_string(),
                        confidence: 85,
                    });
                }
            }
        }

        // Check if splitting order would reduce fees
        if order_request.size.as_u64() > 1_000_000 * Price::PRECISION {
            let split_savings = self.calculate_split_order_savings(order_request, market_data)?;
            if split_savings > Amount::zero() {
                optimizations.push(FeeOptimizationOption {
                    strategy: OptimizationStrategy::SplitOrder,
                    potential_savings: split_savings,
                    trade_offs: "Multiple orders to manage".to_string(),
                    confidence: 70,
                });
            }
        }

        // Check if OTC would be more cost-effective for large orders
        if !order_request.otc_enabled && order_request.size.as_u64() > 10_000_000 * Price::PRECISION {
            let otc_savings = self.calculate_otc_savings(order_request, market_data)?;
            if otc_savings > Amount::zero() {
                optimizations.push(FeeOptimizationOption {
                    strategy: OptimizationStrategy::UseOtc,
                    potential_savings: otc_savings,
                    trade_offs: "Limited liquidity, counterparty risk".to_string(),
                    confidence: 60,
                });
            }
        }

        Ok(FeeOptimization {
            current_fees,
            optimization_options: optimizations,
            analysis_timestamp: Utc::now(),
        })
    }

    // Private helper methods

    fn calculate_base_fee(&self, order_value: Amount, tier: &TradingTier) -> TradingResult<Amount> {
        let tier_discount_bps = match tier {
            TradingTier::Retail => 0,
            TradingTier::SmallWhale => 5,   // 0.05% discount
            TradingTier::MediumWhale => 10, // 0.1% discount
            TradingTier::LargeWhale => 15,  // 0.15% discount
            TradingTier::MegaWhale => 20,   // 0.2% discount
        };

        let effective_fee_bps = self.config.base_fee_bps.saturating_sub(tier_discount_bps);
        let fee_rate = Percentage::from_basis_points(effective_fee_bps)?;

        order_value.checked_mul(&fee_rate.into())
            .ok_or_else(|| TradingError::internal("Base fee calculation overflow"))
    }

    fn calculate_strategy_fee(&self, order_value: Amount, strategy: &ExecutionStrategy) -> TradingResult<Amount> {
        let strategy_fee_bps = match strategy {
            ExecutionStrategy::Market => 0,   // No additional fee
            ExecutionStrategy::Limit => 0,   // No additional fee
            ExecutionStrategy::Twap { .. } => 5,  // 0.05% additional for TWAP
            ExecutionStrategy::Vwap => 5,    // 0.05% additional for VWAP
            ExecutionStrategy::Smart => 3,   // 0.03% additional for smart routing
        };

        if strategy_fee_bps == 0 {
            return Ok(Amount::zero());
        }

        let fee_rate = Percentage::from_basis_points(strategy_fee_bps)?;
        order_value.checked_mul(&fee_rate.into())
            .ok_or_else(|| TradingError::internal("Strategy fee calculation overflow"))
    }

    fn calculate_volume_discount(&self, trader: moby_types::AccountKey, _order_value: Amount) -> TradingResult<Amount> {
        let volume_history = self.volume_history.get(&trader);
        if volume_history.is_none() {
            return Ok(Amount::zero());
        }

        // TODO: Implement volume discount calculation based on trading history
        Ok(Amount::zero())
    }

    fn calculate_effective_rate(&self, total_fees: Amount, order_value: Amount) -> TradingResult<Percentage> {
        if order_value.is_zero() {
            return Ok(Percentage::zero());
        }

        Percentage::from_ratio(total_fees.as_u64(), order_value.as_u64())
            .map_err(|e| TradingError::Math(e))
    }

    fn get_tier_discount_bps(&self, monthly_volume: Amount) -> u16 {
        let volume_threshold = monthly_volume.as_u64() / Price::PRECISION;

        if volume_threshold >= 1_000_000_000 {      // $1B+
            20
        } else if volume_threshold >= 500_000_000 { // $500M+
            15
        } else if volume_threshold >= 100_000_000 { // $100M+
            10
        } else if volume_threshold >= 10_000_000 {  // $10M+
            5
        } else {
            0
        }
    }

    fn get_volume_discount_bps(&self, monthly_volume: Amount) -> u16 {
        let volume_threshold = monthly_volume.as_u64() / Price::PRECISION;

        if volume_threshold >= 5_000_000_000 {      // $5B+
            50  // 0.5% additional discount
        } else if volume_threshold >= 1_000_000_000 { // $1B+
            30  // 0.3% additional discount
        } else if volume_threshold >= 100_000_000 { // $100M+
            15  // 0.15% additional discount
        } else if volume_threshold >= 50_000_000 {  // $50M+
            10  // 0.1% additional discount
        } else {
            0
        }
    }

    fn calculate_split_order_savings(&self, _order_request: &OrderRequest, _market_data: &MarketData) -> TradingResult<Amount> {
        // Mock implementation - calculate potential savings from splitting large orders
        Ok(Amount::from_u64(1000)) // $1000 savings
    }

    fn calculate_otc_savings(&self, order_request: &OrderRequest, market_data: &MarketData) -> TradingResult<Amount> {
        let order_value = order_request.size.checked_mul(&market_data.mid_price.into())
            .ok_or_else(|| TradingError::internal("Order value calculation overflow"))?;

        // OTC typically has lower fees for large orders
        let current_fee_rate = Percentage::from_basis_points(self.config.base_fee_bps)?;
        let otc_fee_rate = Percentage::from_basis_points(self.config.otc_fee_bps)?;

        if otc_fee_rate < current_fee_rate {
            let savings_rate = current_fee_rate.checked_sub(&otc_fee_rate)?;
            order_value.checked_mul(&savings_rate.into())
                .ok_or_else(|| TradingError::internal("OTC savings calculation overflow"))
        } else {
            Ok(Amount::zero())
        }
    }
}

/// Fee estimate for an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub base_fee: Amount,
    pub strategy_fee: Amount,
    pub privacy_fee: Amount,
    pub otc_fee: Amount,
    pub cross_chain_fee: Amount,
    pub volume_discount: Amount,
    pub total_fees: Amount,
    pub effective_rate: Percentage,
}

/// Fee structure for a trader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    pub trader: moby_types::AccountKey,
    pub base_fee_bps: u16,
    pub tier_discount_bps: u16,
    pub volume_discount_bps: u16,
    pub effective_fee_bps: u16,
    pub monthly_volume: Amount,
    pub last_updated: DateTime<Utc>,
}

/// Fee optimization analysis
#[derive(Debug, Clone)]
pub struct FeeOptimization {
    pub current_fees: FeeEstimate,
    pub optimization_options: Vec<FeeOptimizationOption>,
    pub analysis_timestamp: DateTime<Utc>,
}

/// Fee optimization option
#[derive(Debug, Clone)]
pub struct FeeOptimizationOption {
    pub strategy: OptimizationStrategy,
    pub potential_savings: Amount,
    pub trade_offs: String,
    pub confidence: u8, // 0-100
}

/// Optimization strategies
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    UseTwap,
    UseVwap,
    SplitOrder,
    UseOtc,
    WaitForBetterRates,
    IncreaseVolume, // To reach higher tier
}

/// Volume history tracking
#[derive(Debug, Clone)]
pub struct VolumeHistory {
    trader: moby_types::AccountKey,
    daily_volumes: Vec<DailyVolume>,
    monthly_total: Amount,
    last_updated: DateTime<Utc>,
}

impl VolumeHistory {
    fn new(trader: moby_types::AccountKey) -> Self {
        Self {
            trader,
            daily_volumes: Vec::new(),
            monthly_total: Amount::zero(),
            last_updated: Utc::now(),
        }
    }

    fn add_volume(&mut self, volume: Amount) -> TradingResult<()> {
        let today = Utc::now().date_naive();

        // Find or create today's entry
        if let Some(daily_volume) = self.daily_volumes.iter_mut().find(|dv| dv.date == today) {
            daily_volume.volume = daily_volume.volume.checked_add(&volume)
                .ok_or_else(|| TradingError::internal("Daily volume overflow"))?;
        } else {
            self.daily_volumes.push(DailyVolume {
                date: today,
                volume,
            });
        }

        // Remove entries older than 30 days
        let cutoff_date = today - Duration::days(30);
        self.daily_volumes.retain(|dv| dv.date >= cutoff_date);

        // Recalculate monthly total
        self.monthly_total = self.daily_volumes.iter()
            .map(|dv| dv.volume)
            .fold(Amount::zero(), |acc, vol| acc.checked_add(&vol).unwrap_or(acc));

        self.last_updated = Utc::now();
        Ok(())
    }

    fn get_monthly_volume(&self) -> Amount {
        self.monthly_total
    }
}

/// Daily volume entry
#[derive(Debug, Clone)]
struct DailyVolume {
    date: chrono::NaiveDate,
    volume: Amount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MarketData;
    use moby_types::{OrderType, OrderSide, TimeInForce};

    fn create_test_order_request() -> OrderRequest {
        crate::OrderRequest {
            trader: moby_types::AccountKey::new_unique(),
            base_token: moby_types::AccountKey::new_unique(),
            quote_token: moby_types::AccountKey::new_unique(),
            order_type: OrderType::Market,
            side: OrderSide::Buy,
            size: Amount::from_u64(1_000_000 * Price::PRECISION), // $1M
            price: None,
            execution_strategy: ExecutionStrategy::Market,
            slippage_tolerance: 100,
            time_in_force: TimeInForce::Ioc,
            tier: TradingTier::SmallWhale,
            privacy_enabled: false,
            otc_enabled: false,
            cross_chain_enabled: false,
            expires_at: None,
        }
    }

    fn create_test_market_data() -> MarketData {
        MarketData {
            base_token: moby_types::AccountKey::new_unique(),
            quote_token: moby_types::AccountKey::new_unique(),
            bid_price: Price::from_u64(99 * Price::PRECISION).unwrap(),
            ask_price: Price::from_u64(101 * Price::PRECISION).unwrap(),
            mid_price: Price::from_u64(100 * Price::PRECISION).unwrap(),
            liquidity: Amount::from_u64(10_000_000 * Price::PRECISION),
            is_active: true,
            last_updated: Utc::now(),
        }
    }

    #[test]
    fn test_fee_calculation() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        let order_request = create_test_order_request();
        let market_data = create_test_market_data();

        let fees = calculator.calculate_fees(&order_request, &market_data).unwrap();

        assert!(fees.base_fee > Amount::zero());
        assert_eq!(fees.strategy_fee, Amount::zero()); // Market orders have no strategy fee
        assert_eq!(fees.privacy_fee, Amount::zero()); // Privacy not enabled
        assert!(fees.total_fees > Amount::zero());
        assert!(fees.effective_rate > Percentage::zero());
    }

    #[test]
    fn test_tier_discount() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        let market_data = create_test_market_data();

        // Test different tiers
        let mut retail_order = create_test_order_request();
        retail_order.tier = TradingTier::Retail;
        let retail_fees = calculator.calculate_fees(&retail_order, &market_data).unwrap();

        let mut whale_order = create_test_order_request();
        whale_order.tier = TradingTier::LargeWhale;
        let whale_fees = calculator.calculate_fees(&whale_order, &market_data).unwrap();

        // Large whale should pay less in base fees
        assert!(whale_fees.base_fee < retail_fees.base_fee);
    }

    #[test]
    fn test_privacy_fee() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        let market_data = create_test_market_data();

        let mut order_request = create_test_order_request();
        order_request.privacy_enabled = true;

        let fees = calculator.calculate_fees(&order_request, &market_data).unwrap();

        assert!(fees.privacy_fee > Amount::zero());
        assert!(fees.total_fees > fees.base_fee);
    }

    #[test]
    fn test_strategy_fee() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        let market_data = create_test_market_data();

        let mut order_request = create_test_order_request();
        order_request.execution_strategy = ExecutionStrategy::Twap { duration_minutes: 60 };

        let fees = calculator.calculate_fees(&order_request, &market_data).unwrap();

        assert!(fees.strategy_fee > Amount::zero());
    }

    #[test]
    fn test_volume_history() {
        let trader = moby_types::AccountKey::new_unique();
        let mut history = VolumeHistory::new(trader);

        let volume1 = Amount::from_u64(1_000_000 * Price::PRECISION);
        let volume2 = Amount::from_u64(500_000 * Price::PRECISION);

        history.add_volume(volume1).unwrap();
        history.add_volume(volume2).unwrap();

        let monthly_volume = history.get_monthly_volume();
        let expected_total = volume1.checked_add(&volume2).unwrap();
        assert_eq!(monthly_volume, expected_total);
    }

    #[test]
    fn test_fee_optimization() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        let order_request = create_test_order_request();
        let market_data = create_test_market_data();

        let optimization = calculator.optimize_fee_strategy(&order_request, &market_data).unwrap();

        assert!(!optimization.optimization_options.is_empty());
        assert!(optimization.current_fees.total_fees > Amount::zero());
    }
}