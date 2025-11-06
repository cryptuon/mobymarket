// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Execution strategies for whale trading

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{
    TradingConfig, TradingError, TradingResult, OrderRequest,
    execution::{ExecutionPlan, ExecutionStep, ExecutionType},
    engine::MarketData,
    risk::RiskAssessment,
    fees::FeeEstimate,
};

use moby_types::{ExecutionStrategy, OrderSide, TradingTier};
use moby_math::{Amount, Price, Percentage};

/// TWAP (Time-Weighted Average Price) execution strategy
#[derive(Debug)]
pub struct TwapStrategy {
    config: TradingConfig,
}

impl TwapStrategy {
    pub fn new(config: &TradingConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn create_execution_plan(
        &self,
        order_request: &OrderRequest,
        duration_minutes: u32,
        market_data: &MarketData,
        risk_assessment: &RiskAssessment,
        fee_estimate: &FeeEstimate,
    ) -> TradingResult<ExecutionPlan> {
        // Calculate optimal slice size and intervals
        let slice_config = self.calculate_twap_slices(
            order_request.size,
            duration_minutes,
            &order_request.tier,
            market_data,
        )?;

        let mut steps = Vec::new();
        let slice_interval_ms = (duration_minutes * 60 * 1000) / slice_config.num_slices;

        for i in 0..slice_config.num_slices {
            let slice_amount = if i == slice_config.num_slices - 1 {
                // Last slice gets remaining amount to handle rounding
                order_request.size.checked_sub(&(slice_config.slice_size.checked_mul(&Amount::from_u64((slice_config.num_slices - 1) as u64))?))
                    .ok_or_else(|| TradingError::internal("Amount calculation error"))?
            } else {
                slice_config.slice_size
            };

            let step = ExecutionStep {
                step_id: i + 1,
                execution_type: ExecutionType::TwapSlice,
                amount: slice_amount,
                price: None, // Market price for each slice
                max_slippage: Some(Percentage::from_basis_points(order_request.slippage_tolerance)),
                delay_before_execution: if i == 0 { None } else { Some(slice_interval_ms as u64) },
                timeout: Some(Duration::minutes(5)), // 5 minute timeout per slice
            };

            steps.push(step);
        }

        Ok(ExecutionPlan {
            plan_id: self.generate_plan_id(),
            strategy: ExecutionStrategy::Twap { duration_minutes },
            total_amount: order_request.size,
            estimated_duration: Duration::minutes(duration_minutes as i64),
            estimated_fees: fee_estimate.total_fees,
            steps,
            created_at: Utc::now(),
        })
    }

    fn calculate_twap_slices(
        &self,
        total_size: Amount,
        duration_minutes: u32,
        tier: &TradingTier,
        market_data: &MarketData,
    ) -> TradingResult<TwapSliceConfig> {
        // Base configuration by tier
        let (min_slices, max_slices, target_slice_pct) = match tier {
            TradingTier::Retail => (2, 10, 10.0),      // 10% of liquidity per slice
            TradingTier::SmallWhale => (5, 20, 5.0),   // 5% of liquidity per slice
            TradingTier::MediumWhale => (10, 50, 2.0), // 2% of liquidity per slice
            TradingTier::LargeWhale => (20, 100, 1.0), // 1% of liquidity per slice
            TradingTier::MegaWhale => (50, 200, 0.5),  // 0.5% of liquidity per slice
        };

        // Calculate optimal slice size based on liquidity
        let target_slice_amount = market_data.liquidity.checked_mul(&Percentage::from_percentage(target_slice_pct)?.into())
            .ok_or_else(|| TradingError::internal("Slice calculation overflow"))?;

        // Calculate number of slices needed
        let calculated_slices = (total_size.as_f64() / target_slice_amount.as_f64()).ceil() as u32;
        let num_slices = calculated_slices.max(min_slices).min(max_slices);

        // Ensure minimum time between slices (at least 30 seconds)
        let min_interval_minutes = 0.5; // 30 seconds
        let required_duration = num_slices as f64 * min_interval_minutes;

        if duration_minutes as f64 < required_duration {
            return Err(TradingError::invalid_order(
                format!("TWAP duration too short: need at least {:.1} minutes for {} slices",
                    required_duration, num_slices)
            ));
        }

        let slice_size = total_size.checked_div_u64(num_slices as u64)
            .ok_or_else(|| TradingError::internal("Slice size calculation error"))?;

        Ok(TwapSliceConfig {
            num_slices,
            slice_size,
            interval_minutes: duration_minutes as f64 / num_slices as f64,
        })
    }

    fn generate_plan_id(&self) -> u64 {
        Utc::now().timestamp_nanos() as u64
    }
}

/// VWAP (Volume-Weighted Average Price) execution strategy
#[derive(Debug)]
pub struct VwapStrategy {
    config: TradingConfig,
}

impl VwapStrategy {
    pub fn new(config: &TradingConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn create_execution_plan(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
        risk_assessment: &RiskAssessment,
        fee_estimate: &FeeEstimate,
    ) -> TradingResult<ExecutionPlan> {
        // Get historical volume patterns (mock implementation)
        let volume_profile = self.get_volume_profile(market_data).await?;

        // Calculate VWAP slices based on historical volume distribution
        let slice_config = self.calculate_vwap_slices(
            order_request.size,
            &volume_profile,
            &order_request.tier,
        )?;

        let mut steps = Vec::new();

        for (i, slice) in slice_config.slices.iter().enumerate() {
            let step = ExecutionStep {
                step_id: i as u32 + 1,
                execution_type: ExecutionType::VwapSlice,
                amount: slice.amount,
                price: None, // Market price, weighted by volume
                max_slippage: Some(Percentage::from_basis_points(order_request.slippage_tolerance)),
                delay_before_execution: if i == 0 { None } else { Some(slice.delay_ms) },
                timeout: Some(Duration::minutes(10)), // 10 minute timeout per slice
            };

            steps.push(step);
        }

        Ok(ExecutionPlan {
            plan_id: self.generate_plan_id(),
            strategy: ExecutionStrategy::Vwap,
            total_amount: order_request.size,
            estimated_duration: Duration::minutes(slice_config.total_duration_minutes as i64),
            estimated_fees: fee_estimate.total_fees,
            steps,
            created_at: Utc::now(),
        })
    }

    async fn get_volume_profile(&self, _market_data: &MarketData) -> TradingResult<VolumeProfile> {
        // Mock implementation - in production, fetch from market data provider
        Ok(VolumeProfile {
            hourly_volumes: vec![
                100, 80, 60, 40, 30, 25, 20, 30,   // 00:00 - 07:00
                50, 80, 120, 150, 180, 200, 220,   // 08:00 - 14:00
                200, 180, 150, 120, 100, 80, 60,   // 15:00 - 21:00
                40                                  // 22:00 - 23:00
            ],
            peak_hours: vec![12, 13, 14, 15], // UTC hours
            low_volume_hours: vec![2, 3, 4, 5, 6],
        })
    }

    fn calculate_vwap_slices(
        &self,
        total_size: Amount,
        volume_profile: &VolumeProfile,
        tier: &TradingTier,
    ) -> TradingResult<VwapSliceConfig> {
        let total_volume: u64 = volume_profile.hourly_volumes.iter().sum();
        let mut slices = Vec::new();
        let mut cumulative_delay = 0u64;

        // Adjust slice timing based on tier
        let participation_rate = match tier {
            TradingTier::Retail => 0.05,       // 5% of market volume
            TradingTier::SmallWhale => 0.03,   // 3% of market volume
            TradingTier::MediumWhale => 0.02,  // 2% of market volume
            TradingTier::LargeWhale => 0.01,   // 1% of market volume
            TradingTier::MegaWhale => 0.005,   // 0.5% of market volume
        };

        for (hour, &volume) in volume_profile.hourly_volumes.iter().enumerate() {
            if volume == 0 { continue; }

            let volume_weight = volume as f64 / total_volume as f64;
            let slice_amount_f64 = total_size.as_f64() * volume_weight;
            let slice_amount = Amount::from_f64(slice_amount_f64)?;

            // Skip very small slices
            if slice_amount.as_u64() < 1000 { continue; }

            let slice = VwapSlice {
                amount: slice_amount,
                target_hour: hour as u8,
                delay_ms: cumulative_delay,
                participation_rate,
            };

            slices.push(slice);
            cumulative_delay += 3600000; // 1 hour in milliseconds
        }

        Ok(VwapSliceConfig {
            slices,
            total_duration_minutes: 24 * 60, // 24 hours
        })
    }

    fn generate_plan_id(&self) -> u64 {
        Utc::now().timestamp_nanos() as u64
    }
}

/// Smart execution strategy that chooses optimal approach
#[derive(Debug)]
pub struct SmartStrategy {
    config: TradingConfig,
    twap_strategy: TwapStrategy,
    vwap_strategy: VwapStrategy,
}

impl SmartStrategy {
    pub fn new(config: &TradingConfig) -> Self {
        Self {
            twap_strategy: TwapStrategy::new(config),
            vwap_strategy: VwapStrategy::new(config),
            config: config.clone(),
        }
    }

    pub async fn create_execution_plan(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
        risk_assessment: &RiskAssessment,
        fee_estimate: &FeeEstimate,
    ) -> TradingResult<ExecutionPlan> {
        // Analyze market conditions to choose optimal strategy
        let strategy_recommendation = self.analyze_optimal_strategy(
            order_request,
            market_data,
            risk_assessment,
        ).await?;

        match strategy_recommendation.strategy_type {
            SmartStrategyType::Market => {
                // Small orders or urgent execution
                self.create_market_plan(order_request, market_data, fee_estimate).await
            }
            SmartStrategyType::Twap { duration_minutes } => {
                self.twap_strategy.create_execution_plan(
                    order_request,
                    duration_minutes,
                    market_data,
                    risk_assessment,
                    fee_estimate,
                ).await
            }
            SmartStrategyType::Vwap => {
                self.vwap_strategy.create_execution_plan(
                    order_request,
                    market_data,
                    risk_assessment,
                    fee_estimate,
                ).await
            }
            SmartStrategyType::Hybrid { twap_portion, vwap_portion } => {
                self.create_hybrid_plan(
                    order_request,
                    market_data,
                    risk_assessment,
                    fee_estimate,
                    twap_portion,
                    vwap_portion,
                ).await
            }
        }
    }

    async fn analyze_optimal_strategy(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
        risk_assessment: &RiskAssessment,
    ) -> TradingResult<StrategyRecommendation> {
        let order_value = order_request.size.checked_mul(&market_data.mid_price.into())
            .ok_or_else(|| TradingError::internal("Order value calculation overflow"))?;

        let liquidity_ratio = order_value.as_f64() / market_data.liquidity.as_f64();

        // Decision tree for strategy selection
        let strategy_type = if liquidity_ratio < 0.01 {
            // Less than 1% of liquidity - can execute as market order
            SmartStrategyType::Market
        } else if liquidity_ratio < 0.05 {
            // 1-5% of liquidity - use short TWAP
            SmartStrategyType::Twap { duration_minutes: 30 }
        } else if liquidity_ratio < 0.1 {
            // 5-10% of liquidity - use medium TWAP
            SmartStrategyType::Twap { duration_minutes: 60 }
        } else if liquidity_ratio < 0.2 {
            // 10-20% of liquidity - use VWAP or long TWAP based on volatility
            if risk_assessment.volatility_score > 70 {
                SmartStrategyType::Vwap // High volatility - use VWAP
            } else {
                SmartStrategyType::Twap { duration_minutes: 120 }
            }
        } else {
            // >20% of liquidity - use hybrid approach
            SmartStrategyType::Hybrid {
                twap_portion: 0.6,  // 60% TWAP
                vwap_portion: 0.4,  // 40% VWAP
            }
        };

        Ok(StrategyRecommendation {
            strategy_type,
            confidence: self.calculate_confidence(&strategy_type, liquidity_ratio),
            expected_slippage: self.estimate_slippage(liquidity_ratio),
            expected_market_impact: self.estimate_market_impact(liquidity_ratio),
        })
    }

    async fn create_market_plan(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
        fee_estimate: &FeeEstimate,
    ) -> TradingResult<ExecutionPlan> {
        let execution_price = match order_request.side {
            OrderSide::Buy => market_data.ask_price,
            OrderSide::Sell => market_data.bid_price,
        };

        let step = ExecutionStep {
            step_id: 1,
            execution_type: ExecutionType::Market,
            amount: order_request.size,
            price: Some(execution_price),
            max_slippage: Some(Percentage::from_basis_points(order_request.slippage_tolerance)),
            delay_before_execution: None,
            timeout: Some(Duration::seconds(30)),
        };

        Ok(ExecutionPlan {
            plan_id: self.generate_plan_id(),
            strategy: ExecutionStrategy::Smart,
            total_amount: order_request.size,
            estimated_duration: Duration::seconds(30),
            estimated_fees: fee_estimate.total_fees,
            steps: vec![step],
            created_at: Utc::now(),
        })
    }

    async fn create_hybrid_plan(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
        risk_assessment: &RiskAssessment,
        fee_estimate: &FeeEstimate,
        twap_portion: f64,
        vwap_portion: f64,
    ) -> TradingResult<ExecutionPlan> {
        let twap_amount = order_request.size.checked_mul(&Amount::from_f64(twap_portion)?)
            .ok_or_else(|| TradingError::internal("TWAP amount calculation overflow"))?;

        let vwap_amount = order_request.size.checked_sub(&twap_amount)
            .ok_or_else(|| TradingError::internal("VWAP amount calculation underflow"))?;

        // Create TWAP plan for first portion
        let mut twap_request = order_request.clone();
        twap_request.size = twap_amount;

        let twap_plan = self.twap_strategy.create_execution_plan(
            &twap_request,
            120, // 2 hours
            market_data,
            risk_assessment,
            fee_estimate,
        ).await?;

        // Create VWAP plan for second portion
        let mut vwap_request = order_request.clone();
        vwap_request.size = vwap_amount;

        let vwap_plan = self.vwap_strategy.create_execution_plan(
            &vwap_request,
            market_data,
            risk_assessment,
            fee_estimate,
        ).await?;

        // Combine plans
        let mut combined_steps = twap_plan.steps;
        let mut vwap_steps = vwap_plan.steps;

        // Adjust step IDs for VWAP steps
        let twap_step_count = combined_steps.len() as u32;
        for step in &mut vwap_steps {
            step.step_id += twap_step_count;
        }

        combined_steps.extend(vwap_steps);

        Ok(ExecutionPlan {
            plan_id: self.generate_plan_id(),
            strategy: ExecutionStrategy::Smart,
            total_amount: order_request.size,
            estimated_duration: Duration::hours(24), // Maximum of TWAP and VWAP
            estimated_fees: fee_estimate.total_fees,
            steps: combined_steps,
            created_at: Utc::now(),
        })
    }

    fn calculate_confidence(&self, strategy_type: &SmartStrategyType, liquidity_ratio: f64) -> u8 {
        match strategy_type {
            SmartStrategyType::Market if liquidity_ratio < 0.005 => 95,
            SmartStrategyType::Market => 80,
            SmartStrategyType::Twap { .. } if liquidity_ratio < 0.1 => 90,
            SmartStrategyType::Vwap if liquidity_ratio < 0.15 => 85,
            SmartStrategyType::Hybrid { .. } => 75,
            _ => 65,
        }
    }

    fn estimate_slippage(&self, liquidity_ratio: f64) -> Percentage {
        let slippage_bps = if liquidity_ratio < 0.01 {
            5 // 0.05%
        } else if liquidity_ratio < 0.05 {
            20 // 0.2%
        } else if liquidity_ratio < 0.1 {
            50 // 0.5%
        } else {
            100 // 1%
        };

        Percentage::from_basis_points(slippage_bps).unwrap_or(Percentage::zero())
    }

    fn estimate_market_impact(&self, liquidity_ratio: f64) -> Percentage {
        let impact_bps = (liquidity_ratio * 10000.0) as u16; // Square root relationship
        Percentage::from_basis_points(impact_bps.min(500)).unwrap_or(Percentage::zero())
    }

    fn generate_plan_id(&self) -> u64 {
        Utc::now().timestamp_nanos() as u64
    }
}

// Supporting data structures

#[derive(Debug, Clone)]
pub struct TwapSliceConfig {
    pub num_slices: u32,
    pub slice_size: Amount,
    pub interval_minutes: f64,
}

#[derive(Debug, Clone)]
pub struct VwapSliceConfig {
    pub slices: Vec<VwapSlice>,
    pub total_duration_minutes: u32,
}

#[derive(Debug, Clone)]
pub struct VwapSlice {
    pub amount: Amount,
    pub target_hour: u8,
    pub delay_ms: u64,
    pub participation_rate: f64,
}

#[derive(Debug, Clone)]
pub struct VolumeProfile {
    pub hourly_volumes: Vec<u64>,
    pub peak_hours: Vec<u8>,
    pub low_volume_hours: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StrategyRecommendation {
    pub strategy_type: SmartStrategyType,
    pub confidence: u8, // 0-100
    pub expected_slippage: Percentage,
    pub expected_market_impact: Percentage,
}

#[derive(Debug, Clone)]
pub enum SmartStrategyType {
    Market,
    Twap { duration_minutes: u32 },
    Vwap,
    Hybrid { twap_portion: f64, vwap_portion: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TradingConfig, engine::MarketData, risk::RiskAssessment, fees::FeeEstimate};
    use moby_types::{OrderType, TradingTier, TimeInForce};

    fn create_test_order_request() -> OrderRequest {
        crate::OrderRequest {
            trader: moby_types::AccountKey::new_unique(),
            base_token: moby_types::AccountKey::new_unique(),
            quote_token: moby_types::AccountKey::new_unique(),
            order_type: OrderType::Twap,
            side: OrderSide::Buy,
            size: Amount::from_u64(1_000_000 * moby_math::Price::PRECISION), // $1M order
            price: None,
            execution_strategy: ExecutionStrategy::Twap { duration_minutes: 60 },
            slippage_tolerance: 50, // 0.5%
            time_in_force: TimeInForce::Gtc,
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
            bid_price: Price::from_u64(99 * moby_math::Price::PRECISION).unwrap(),
            ask_price: Price::from_u64(101 * moby_math::Price::PRECISION).unwrap(),
            mid_price: Price::from_u64(100 * moby_math::Price::PRECISION).unwrap(),
            liquidity: Amount::from_u64(10_000_000 * moby_math::Price::PRECISION), // $10M liquidity
            is_active: true,
            last_updated: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_twap_strategy() {
        let config = TradingConfig::default();
        let strategy = TwapStrategy::new(&config);
        let order_request = create_test_order_request();
        let market_data = create_test_market_data();
        let risk_assessment = RiskAssessment::default();
        let fee_estimate = FeeEstimate {
            base_fee: Amount::from_u64(1000),
            total_fees: Amount::from_u64(1000),
        };

        let plan = strategy.create_execution_plan(
            &order_request,
            60, // 1 hour
            &market_data,
            &risk_assessment,
            &fee_estimate,
        ).await.unwrap();

        assert!(!plan.steps.is_empty());
        assert_eq!(plan.total_amount, order_request.size);
        assert!(plan.estimated_duration <= Duration::hours(1));

        // Verify all steps are TWAP slices
        for step in &plan.steps {
            assert!(matches!(step.execution_type, ExecutionType::TwapSlice));
        }
    }

    #[tokio::test]
    async fn test_smart_strategy_small_order() {
        let config = TradingConfig::default();
        let strategy = SmartStrategy::new(&config);

        let mut order_request = create_test_order_request();
        order_request.size = Amount::from_u64(50_000 * moby_math::Price::PRECISION); // $50K order
        order_request.tier = TradingTier::Retail;

        let market_data = create_test_market_data();
        let risk_assessment = RiskAssessment::default();
        let fee_estimate = FeeEstimate {
            base_fee: Amount::from_u64(100),
            total_fees: Amount::from_u64(100),
        };

        let plan = strategy.create_execution_plan(
            &order_request,
            &market_data,
            &risk_assessment,
            &fee_estimate,
        ).await.unwrap();

        // Small order should use market execution
        assert_eq!(plan.steps.len(), 1);
        assert!(matches!(plan.steps[0].execution_type, ExecutionType::Market));
    }

    #[tokio::test]
    async fn test_twap_slice_calculation() {
        let config = TradingConfig::default();
        let strategy = TwapStrategy::new(&config);
        let market_data = create_test_market_data();

        let slice_config = strategy.calculate_twap_slices(
            Amount::from_u64(1_000_000 * moby_math::Price::PRECISION),
            60, // 1 hour
            &TradingTier::SmallWhale,
            &market_data,
        ).unwrap();

        assert!(slice_config.num_slices >= 5);
        assert!(slice_config.num_slices <= 20);
        assert!(slice_config.interval_minutes >= 0.5); // At least 30 seconds between slices
    }
}