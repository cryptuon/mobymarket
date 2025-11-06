// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Risk management system for whale trading

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{
    TradingError, TradingResult, OrderRequest, RiskConfig,
    engine::MarketData,
};

use moby_types::{TradingTier, AccountKey, OrderSide};
use moby_math::{Amount, Price, Percentage};

/// Risk management system
#[derive(Debug)]
pub struct RiskManager {
    config: RiskConfig,
    trader_exposures: HashMap<AccountKey, TraderExposure>,
    market_exposures: HashMap<(AccountKey, AccountKey), MarketExposure>,
}

impl RiskManager {
    /// Create a new risk manager
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            trader_exposures: HashMap::new(),
            market_exposures: HashMap::new(),
        }
    }

    /// Validate an order against risk limits
    pub async fn validate_order(&self, order_request: &OrderRequest) -> TradingResult<()> {
        // Check trader-level limits
        self.check_trader_limits(order_request).await?;

        // Check position limits
        self.check_position_limits(order_request).await?;

        // Check concentration limits
        self.check_concentration_limits(order_request).await?;

        // Check tier authorization
        self.check_tier_authorization(order_request).await?;

        Ok(())
    }

    /// Assess risk for an order
    pub async fn assess_order_risk(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
    ) -> TradingResult<RiskAssessment> {
        let order_value = order_request.size.checked_mul(&market_data.mid_price.into())
            .ok_or_else(|| TradingError::internal("Order value calculation overflow"))?;

        let liquidity_ratio = order_value.as_f64() / market_data.liquidity.as_f64();

        // Calculate various risk metrics
        let volatility_score = self.calculate_volatility_score(market_data).await?;
        let liquidity_score = self.calculate_liquidity_score(liquidity_ratio);
        let concentration_score = self.calculate_concentration_score(order_request).await?;
        let market_impact_score = self.calculate_market_impact_score(liquidity_ratio);

        // Overall risk score (weighted average)
        let overall_risk_score = (
            volatility_score as f64 * 0.3 +
            liquidity_score as f64 * 0.25 +
            concentration_score as f64 * 0.25 +
            market_impact_score as f64 * 0.2
        ) as u8;

        let risk_level = match overall_risk_score {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        let recommended_actions = self.generate_risk_recommendations(
            overall_risk_score,
            liquidity_ratio,
            order_request,
        );

        Ok(RiskAssessment {
            overall_risk_score,
            risk_level,
            volatility_score,
            liquidity_score,
            concentration_score,
            market_impact_score,
            liquidity_ratio,
            estimated_slippage: self.estimate_slippage(liquidity_ratio),
            estimated_market_impact: self.estimate_market_impact(liquidity_ratio),
            recommended_actions,
            assessment_timestamp: Utc::now(),
        })
    }

    /// Update trader exposure after order execution
    pub async fn update_trader_exposure(
        &mut self,
        trader: AccountKey,
        base_token: AccountKey,
        quote_token: AccountKey,
        side: OrderSide,
        size: Amount,
        price: Price,
    ) -> TradingResult<()> {
        let exposure = self.trader_exposures
            .entry(trader)
            .or_insert_with(|| TraderExposure::new(trader));

        let value = size.checked_mul(&price.into())
            .ok_or_else(|| TradingError::internal("Value calculation overflow"))?;

        // Update daily volume
        exposure.daily_volume = exposure.daily_volume.checked_add(&value)
            .ok_or_else(|| TradingError::internal("Daily volume overflow"))?;

        // Update position
        let position_key = (base_token, quote_token);
        let position = exposure.positions.entry(position_key).or_insert_with(|| Position {
            base_token,
            quote_token,
            net_amount: Amount::zero(),
            total_value: Amount::zero(),
            last_updated: Utc::now(),
        });

        match side {
            OrderSide::Buy => {
                position.net_amount = position.net_amount.checked_add(&size)
                    .ok_or_else(|| TradingError::internal("Position amount overflow"))?;
            }
            OrderSide::Sell => {
                position.net_amount = position.net_amount.checked_sub(&size)
                    .unwrap_or(Amount::zero());
            }
        }

        position.total_value = position.net_amount.checked_mul(&price.into())
            .ok_or_else(|| TradingError::internal("Position value overflow"))?;
        position.last_updated = Utc::now();

        // Update market exposure
        let market_key = (base_token, quote_token);
        let market_exposure = self.market_exposures
            .entry(market_key)
            .or_insert_with(|| MarketExposure::new(base_token, quote_token));

        market_exposure.total_volume = market_exposure.total_volume.checked_add(&value)
            .ok_or_else(|| TradingError::internal("Market volume overflow"))?;
        market_exposure.active_traders.insert(trader);
        market_exposure.last_updated = Utc::now();

        Ok(())
    }

    /// Get trader's current risk metrics
    pub async fn get_trader_risk_metrics(&self, trader: AccountKey) -> TradingResult<TraderRiskMetrics> {
        let exposure = self.trader_exposures.get(&trader)
            .unwrap_or(&TraderExposure::new(trader));

        let total_position_value = exposure.positions.values()
            .map(|p| p.total_value)
            .fold(Amount::zero(), |acc, val| acc.checked_add(&val).unwrap_or(acc));

        let daily_volume_utilization = if self.config.max_daily_volume > 0 {
            (exposure.daily_volume.as_f64() / self.config.max_daily_volume as f64 * 100.0) as u8
        } else {
            0
        };

        let position_utilization = if self.config.max_position_size > 0 {
            (total_position_value.as_f64() / self.config.max_position_size as f64 * 100.0) as u8
        } else {
            0
        };

        let open_orders_utilization = if self.config.max_open_orders > 0 {
            (exposure.open_orders as f64 / self.config.max_open_orders as f64 * 100.0) as u8
        } else {
            0
        };

        Ok(TraderRiskMetrics {
            trader,
            daily_volume: exposure.daily_volume,
            daily_volume_utilization,
            total_position_value,
            position_utilization,
            open_orders: exposure.open_orders,
            open_orders_utilization,
            risk_score: self.calculate_trader_risk_score(exposure),
            last_updated: Utc::now(),
        })
    }

    // Private helper methods

    async fn check_trader_limits(&self, order_request: &OrderRequest) -> TradingResult<()> {
        let exposure = self.trader_exposures.get(&order_request.trader)
            .unwrap_or(&TraderExposure::new(order_request.trader));

        // Check daily volume limit
        let order_value = order_request.size.checked_mul(&Price::from_u64(100 * Price::PRECISION)?.into()) // Mock price
            .ok_or_else(|| TradingError::internal("Order value calculation overflow"))?;

        let new_daily_volume = exposure.daily_volume.checked_add(&order_value)
            .ok_or_else(|| TradingError::internal("Daily volume calculation overflow"))?;

        if new_daily_volume.as_u64() > self.config.max_daily_volume {
            return Err(TradingError::DailyVolumeLimitExceeded {
                current: new_daily_volume.as_u64(),
                limit: self.config.max_daily_volume,
            });
        }

        // Check open orders limit
        if exposure.open_orders >= self.config.max_open_orders {
            return Err(TradingError::MaxOpenOrdersReached {
                limit: self.config.max_open_orders,
            });
        }

        Ok(())
    }

    async fn check_position_limits(&self, order_request: &OrderRequest) -> TradingResult<()> {
        let exposure = self.trader_exposures.get(&order_request.trader)
            .unwrap_or(&TraderExposure::new(order_request.trader));

        let position_key = (order_request.base_token, order_request.quote_token);
        let current_position = exposure.positions.get(&position_key);

        let order_value = order_request.size.checked_mul(&Price::from_u64(100 * Price::PRECISION)?.into()) // Mock price
            .ok_or_else(|| TradingError::internal("Order value calculation overflow"))?;

        let new_position_value = if let Some(position) = current_position {
            match order_request.side {
                OrderSide::Buy => position.total_value.checked_add(&order_value),
                OrderSide::Sell => position.total_value.checked_sub(&order_value),
            }.unwrap_or(order_value)
        } else {
            order_value
        };

        if new_position_value.as_u64() > self.config.max_position_size {
            return Err(TradingError::PositionSizeLimitExceeded {
                current: new_position_value.as_u64(),
                limit: self.config.max_position_size,
            });
        }

        Ok(())
    }

    async fn check_concentration_limits(&self, _order_request: &OrderRequest) -> TradingResult<()> {
        // TODO: Implement concentration limit checks
        // - Maximum exposure per token
        // - Maximum exposure per market
        // - Correlation limits
        Ok(())
    }

    async fn check_tier_authorization(&self, order_request: &OrderRequest) -> TradingResult<()> {
        let order_value = order_request.size.checked_mul(&Price::from_u64(100 * Price::PRECISION)?.into()) // Mock price
            .ok_or_else(|| TradingError::internal("Order value calculation overflow"))?;

        let max_allowed = match order_request.tier {
            TradingTier::Retail => 1_000_000 * Price::PRECISION,      // $1M
            TradingTier::SmallWhale => 10_000_000 * Price::PRECISION, // $10M
            TradingTier::MediumWhale => 100_000_000 * Price::PRECISION, // $100M
            TradingTier::LargeWhale => 500_000_000 * Price::PRECISION,  // $500M
            TradingTier::MegaWhale => u64::MAX, // No limit
        };

        if order_value.as_u64() > max_allowed {
            return Err(TradingError::TierNotAuthorized {
                tier: format!("{:?}", order_request.tier),
                size: order_value.as_u64(),
            });
        }

        Ok(())
    }

    async fn calculate_volatility_score(&self, _market_data: &MarketData) -> TradingResult<u8> {
        // Mock implementation - in production, calculate from historical price data
        Ok(30) // Medium volatility
    }

    fn calculate_liquidity_score(&self, liquidity_ratio: f64) -> u8 {
        if liquidity_ratio < 0.01 {
            10 // Low risk
        } else if liquidity_ratio < 0.05 {
            30 // Medium-low risk
        } else if liquidity_ratio < 0.1 {
            50 // Medium risk
        } else if liquidity_ratio < 0.2 {
            70 // High risk
        } else {
            90 // Very high risk
        }
    }

    async fn calculate_concentration_score(&self, _order_request: &OrderRequest) -> TradingResult<u8> {
        // Mock implementation - calculate based on portfolio concentration
        Ok(25) // Low concentration
    }

    fn calculate_market_impact_score(&self, liquidity_ratio: f64) -> u8 {
        ((liquidity_ratio * 100.0) as u8).min(100)
    }

    fn estimate_slippage(&self, liquidity_ratio: f64) -> Percentage {
        let slippage_bps = if liquidity_ratio < 0.01 {
            5 // 0.05%
        } else if liquidity_ratio < 0.05 {
            25 // 0.25%
        } else if liquidity_ratio < 0.1 {
            50 // 0.5%
        } else {
            100 // 1%+
        };

        Percentage::from_basis_points(slippage_bps).unwrap_or(Percentage::zero())
    }

    fn estimate_market_impact(&self, liquidity_ratio: f64) -> Percentage {
        let impact_bps = (liquidity_ratio.sqrt() * 100.0) as u16;
        Percentage::from_basis_points(impact_bps.min(500)).unwrap_or(Percentage::zero())
    }

    fn generate_risk_recommendations(
        &self,
        risk_score: u8,
        liquidity_ratio: f64,
        order_request: &OrderRequest,
    ) -> Vec<RiskRecommendation> {
        let mut recommendations = Vec::new();

        if risk_score > 70 {
            recommendations.push(RiskRecommendation {
                recommendation_type: RecommendationType::ReduceOrderSize,
                message: "Consider reducing order size to minimize market impact".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        if liquidity_ratio > 0.1 {
            recommendations.push(RiskRecommendation {
                recommendation_type: RecommendationType::UseTwapExecution,
                message: "Use TWAP execution to reduce market impact".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        if matches!(order_request.execution_strategy, moby_types::ExecutionStrategy::Market) && risk_score > 50 {
            recommendations.push(RiskRecommendation {
                recommendation_type: RecommendationType::AvoidMarketOrder,
                message: "Avoid market order execution due to high risk".to_string(),
                priority: RecommendationPriority::Medium,
            });
        }

        if order_request.privacy_enabled && risk_score > 60 {
            recommendations.push(RiskRecommendation {
                recommendation_type: RecommendationType::ConsiderOtc,
                message: "Consider OTC execution for better privacy and lower impact".to_string(),
                priority: RecommendationPriority::Medium,
            });
        }

        recommendations
    }

    fn calculate_trader_risk_score(&self, exposure: &TraderExposure) -> u8 {
        let volume_score = if self.config.max_daily_volume > 0 {
            ((exposure.daily_volume.as_f64() / self.config.max_daily_volume as f64) * 100.0) as u8
        } else {
            0
        };

        let position_score = if self.config.max_position_size > 0 {
            let total_value: Amount = exposure.positions.values()
                .map(|p| p.total_value)
                .fold(Amount::zero(), |acc, val| acc.checked_add(&val).unwrap_or(acc));
            ((total_value.as_f64() / self.config.max_position_size as f64) * 100.0) as u8
        } else {
            0
        };

        let orders_score = if self.config.max_open_orders > 0 {
            ((exposure.open_orders as f64 / self.config.max_open_orders as f64) * 100.0) as u8
        } else {
            0
        };

        // Weighted average
        ((volume_score as f64 * 0.4 + position_score as f64 * 0.4 + orders_score as f64 * 0.2) as u8).min(100)
    }
}

/// Risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_score: u8, // 0-100
    pub risk_level: RiskLevel,
    pub volatility_score: u8,
    pub liquidity_score: u8,
    pub concentration_score: u8,
    pub market_impact_score: u8,
    pub liquidity_ratio: f64,
    pub estimated_slippage: Percentage,
    pub estimated_market_impact: Percentage,
    pub recommended_actions: Vec<RiskRecommendation>,
    pub assessment_timestamp: DateTime<Utc>,
}

impl Default for RiskAssessment {
    fn default() -> Self {
        Self {
            overall_risk_score: 0,
            risk_level: RiskLevel::Low,
            volatility_score: 0,
            liquidity_score: 0,
            concentration_score: 0,
            market_impact_score: 0,
            liquidity_ratio: 0.0,
            estimated_slippage: Percentage::zero(),
            estimated_market_impact: Percentage::zero(),
            recommended_actions: Vec::new(),
            assessment_timestamp: Utc::now(),
        }
    }
}

/// Risk levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRecommendation {
    pub recommendation_type: RecommendationType,
    pub message: String,
    pub priority: RecommendationPriority,
}

/// Types of risk recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    ReduceOrderSize,
    UseTwapExecution,
    UseVwapExecution,
    AvoidMarketOrder,
    ConsiderOtc,
    AddLiquidity,
    WaitForBetterConditions,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Trader exposure tracking
#[derive(Debug, Clone)]
pub struct TraderExposure {
    pub trader: AccountKey,
    pub daily_volume: Amount,
    pub positions: HashMap<(AccountKey, AccountKey), Position>,
    pub open_orders: u32,
    pub last_reset: DateTime<Utc>,
}

impl TraderExposure {
    pub fn new(trader: AccountKey) -> Self {
        Self {
            trader,
            daily_volume: Amount::zero(),
            positions: HashMap::new(),
            open_orders: 0,
            last_reset: Utc::now(),
        }
    }
}

/// Position information
#[derive(Debug, Clone)]
pub struct Position {
    pub base_token: AccountKey,
    pub quote_token: AccountKey,
    pub net_amount: Amount,
    pub total_value: Amount,
    pub last_updated: DateTime<Utc>,
}

/// Market exposure tracking
#[derive(Debug, Clone)]
pub struct MarketExposure {
    pub base_token: AccountKey,
    pub quote_token: AccountKey,
    pub total_volume: Amount,
    pub active_traders: std::collections::HashSet<AccountKey>,
    pub last_updated: DateTime<Utc>,
}

impl MarketExposure {
    pub fn new(base_token: AccountKey, quote_token: AccountKey) -> Self {
        Self {
            base_token,
            quote_token,
            total_volume: Amount::zero(),
            active_traders: std::collections::HashSet::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Trader risk metrics
#[derive(Debug, Clone)]
pub struct TraderRiskMetrics {
    pub trader: AccountKey,
    pub daily_volume: Amount,
    pub daily_volume_utilization: u8, // Percentage of limit used
    pub total_position_value: Amount,
    pub position_utilization: u8, // Percentage of limit used
    pub open_orders: u32,
    pub open_orders_utilization: u8, // Percentage of limit used
    pub risk_score: u8, // 0-100
    pub last_updated: DateTime<Utc>,
}

/// Position limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLimits {
    pub max_position_size: Amount,
    pub max_concentration_pct: u8, // Max % of portfolio in single position
    pub max_correlation: f64, // Max correlation between positions
}

impl Default for PositionLimits {
    fn default() -> Self {
        Self {
            max_position_size: Amount::from_u64(100_000_000 * Price::PRECISION), // $100M
            max_concentration_pct: 25, // 25%
            max_correlation: 0.7, // 70%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moby_types::{OrderType, TimeInForce};
    use crate::engine::MarketData;

    fn create_test_order_request() -> OrderRequest {
        crate::OrderRequest {
            trader: AccountKey::new_unique(),
            base_token: AccountKey::new_unique(),
            quote_token: AccountKey::new_unique(),
            order_type: OrderType::Market,
            side: OrderSide::Buy,
            size: Amount::from_u64(500_000 * Price::PRECISION), // $500K
            price: None,
            execution_strategy: moby_types::ExecutionStrategy::Market,
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
            base_token: AccountKey::new_unique(),
            quote_token: AccountKey::new_unique(),
            bid_price: Price::from_u64(99 * Price::PRECISION).unwrap(),
            ask_price: Price::from_u64(101 * Price::PRECISION).unwrap(),
            mid_price: Price::from_u64(100 * Price::PRECISION).unwrap(),
            liquidity: Amount::from_u64(10_000_000 * Price::PRECISION),
            is_active: true,
            last_updated: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_validate_order_success() {
        let config = RiskConfig::default();
        let risk_manager = RiskManager::new(config);
        let order_request = create_test_order_request();

        let result = risk_manager.validate_order(&order_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_assess_order_risk() {
        let config = RiskConfig::default();
        let risk_manager = RiskManager::new(config);
        let order_request = create_test_order_request();
        let market_data = create_test_market_data();

        let assessment = risk_manager.assess_order_risk(&order_request, &market_data).await.unwrap();

        assert!(assessment.overall_risk_score <= 100);
        assert!(assessment.liquidity_ratio > 0.0);
        assert!(!assessment.recommended_actions.is_empty() || assessment.overall_risk_score < 30);
    }

    #[tokio::test]
    async fn test_tier_authorization() {
        let config = RiskConfig::default();
        let risk_manager = RiskManager::new(config);

        // Test oversized order for retail tier
        let mut order_request = create_test_order_request();
        order_request.tier = TradingTier::Retail;
        order_request.size = Amount::from_u64(5_000_000 * Price::PRECISION); // $5M

        let result = risk_manager.check_tier_authorization(&order_request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TradingError::TierNotAuthorized { .. }));
    }

    #[tokio::test]
    async fn test_liquidity_score_calculation() {
        let config = RiskConfig::default();
        let risk_manager = RiskManager::new(config);

        // Low liquidity ratio should have low score
        let low_score = risk_manager.calculate_liquidity_score(0.005);
        assert!(low_score < 20);

        // High liquidity ratio should have high score
        let high_score = risk_manager.calculate_liquidity_score(0.15);
        assert!(high_score > 60);
    }

    #[tokio::test]
    async fn test_update_trader_exposure() {
        let config = RiskConfig::default();
        let mut risk_manager = RiskManager::new(config);

        let trader = AccountKey::new_unique();
        let base_token = AccountKey::new_unique();
        let quote_token = AccountKey::new_unique();
        let size = Amount::from_u64(1000 * Price::PRECISION);
        let price = Price::from_u64(100 * Price::PRECISION).unwrap();

        risk_manager.update_trader_exposure(
            trader,
            base_token,
            quote_token,
            OrderSide::Buy,
            size,
            price,
        ).await.unwrap();

        let metrics = risk_manager.get_trader_risk_metrics(trader).await.unwrap();
        assert!(metrics.daily_volume > Amount::zero());
        assert!(metrics.total_position_value > Amount::zero());
    }
}