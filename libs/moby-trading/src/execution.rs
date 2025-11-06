// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Order execution engine

use std::collections::VecDeque;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{
    TradingConfig, TradingError, TradingResult, OrderRequest,
    strategies::{TwapStrategy, VwapStrategy, SmartStrategy},
    engine::MarketData,
    risk::RiskAssessment,
    fees::FeeEstimate,
};

use moby_types::{ExecutionStrategy, OrderStatus, OrderSide};
use moby_math::{Amount, Price, Percentage};

/// Order execution engine
#[derive(Debug)]
pub struct ExecutionEngine {
    config: TradingConfig,
    twap_strategy: TwapStrategy,
    vwap_strategy: VwapStrategy,
    smart_strategy: SmartStrategy,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new(config: TradingConfig) -> Self {
        Self {
            twap_strategy: TwapStrategy::new(&config),
            vwap_strategy: VwapStrategy::new(&config),
            smart_strategy: SmartStrategy::new(&config),
            config,
        }
    }

    /// Create an execution plan for an order
    pub async fn create_execution_plan(
        &self,
        order_request: &OrderRequest,
        market_data: &MarketData,
        risk_assessment: &RiskAssessment,
        fee_estimate: &FeeEstimate,
    ) -> TradingResult<ExecutionPlan> {
        match &order_request.execution_strategy {
            ExecutionStrategy::Market => {
                self.create_market_execution_plan(order_request, market_data, fee_estimate).await
            }
            ExecutionStrategy::Limit => {
                self.create_limit_execution_plan(order_request, market_data, fee_estimate).await
            }
            ExecutionStrategy::Twap { duration_minutes } => {
                self.twap_strategy.create_execution_plan(
                    order_request,
                    *duration_minutes,
                    market_data,
                    risk_assessment,
                    fee_estimate,
                ).await
            }
            ExecutionStrategy::Vwap => {
                self.vwap_strategy.create_execution_plan(
                    order_request,
                    market_data,
                    risk_assessment,
                    fee_estimate,
                ).await
            }
            ExecutionStrategy::Smart => {
                self.smart_strategy.create_execution_plan(
                    order_request,
                    market_data,
                    risk_assessment,
                    fee_estimate,
                ).await
            }
        }
    }

    /// Execute an order according to the execution plan
    pub async fn execute_order(
        &self,
        order_id: u64,
        order_request: OrderRequest,
        execution_plan: ExecutionPlan,
    ) -> TradingResult<ExecutionResult> {
        let start_time = Utc::now();
        let mut execution_steps = VecDeque::from(execution_plan.steps);
        let mut executed_steps = Vec::new();
        let mut total_filled = Amount::zero();
        let mut total_fees = Amount::zero();
        let mut weighted_average_price = Price::zero();

        while let Some(step) = execution_steps.pop_front() {
            match self.execute_step(order_id, &step).await {
                Ok(step_result) => {
                    total_filled = total_filled.checked_add(&step_result.filled_amount)
                        .ok_or_else(|| TradingError::internal("Amount overflow in execution"))?;

                    total_fees = total_fees.checked_add(&step_result.fees_paid)
                        .ok_or_else(|| TradingError::internal("Fee overflow in execution"))?;

                    // Update weighted average price
                    if !step_result.filled_amount.is_zero() {
                        let step_value = step_result.filled_amount.checked_mul(&step_result.average_price.into())
                            .ok_or_else(|| TradingError::internal("Price calculation overflow"))?;

                        let total_value = if weighted_average_price.is_zero() {
                            step_value
                        } else {
                            let current_value = total_filled.checked_sub(&step_result.filled_amount)
                                .ok_or_else(|| TradingError::internal("Amount underflow"))?
                                .checked_mul(&weighted_average_price.into())
                                .ok_or_else(|| TradingError::internal("Price calculation overflow"))?;

                            current_value.checked_add(&step_value)
                                .ok_or_else(|| TradingError::internal("Value overflow"))?
                        };

                        weighted_average_price = Price::from_ratio(total_value.as_u64(), total_filled.as_u64())
                            .map_err(|e| TradingError::Math(e))?;
                    }

                    executed_steps.push(step_result);

                    // Check if order is completely filled
                    if total_filled >= order_request.size {
                        break;
                    }

                    // Add delay between steps if specified
                    if let Some(delay) = step.delay_before_execution {
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }
                }
                Err(error) => {
                    // Handle partial execution failure
                    let execution_result = ExecutionResult {
                        order_id,
                        status: if total_filled.is_zero() {
                            ExecutionStatus::Failed
                        } else {
                            ExecutionStatus::PartiallyExecuted
                        },
                        total_filled,
                        remaining_amount: order_request.size.checked_sub(&total_filled)
                            .unwrap_or(Amount::zero()),
                        weighted_average_price,
                        total_fees,
                        execution_time: Utc::now().signed_duration_since(start_time),
                        steps: executed_steps,
                        error_message: Some(error.to_string()),
                        slippage: self.calculate_slippage(&order_request, weighted_average_price)?,
                        market_impact: Percentage::zero(), // TODO: Calculate actual market impact
                    };

                    return Ok(execution_result);
                }
            }
        }

        // Determine final status
        let status = if total_filled >= order_request.size {
            ExecutionStatus::FullyExecuted
        } else if total_filled.is_zero() {
            ExecutionStatus::Failed
        } else {
            ExecutionStatus::PartiallyExecuted
        };

        let remaining_amount = order_request.size.checked_sub(&total_filled)
            .unwrap_or(Amount::zero());

        let slippage = self.calculate_slippage(&order_request, weighted_average_price)?;

        Ok(ExecutionResult {
            order_id,
            status,
            total_filled,
            remaining_amount,
            weighted_average_price,
            total_fees,
            execution_time: Utc::now().signed_duration_since(start_time),
            steps: executed_steps,
            error_message: None,
            slippage,
            market_impact: Percentage::zero(), // TODO: Calculate actual market impact
        })
    }

    // Private helper methods

    async fn create_market_execution_plan(
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
            strategy: order_request.execution_strategy.clone(),
            total_amount: order_request.size,
            estimated_duration: Duration::seconds(30),
            estimated_fees: fee_estimate.total_fees,
            steps: vec![step],
            created_at: Utc::now(),
        })
    }

    async fn create_limit_execution_plan(
        &self,
        order_request: &OrderRequest,
        _market_data: &MarketData,
        fee_estimate: &FeeEstimate,
    ) -> TradingResult<ExecutionPlan> {
        let price = order_request.price
            .ok_or_else(|| TradingError::invalid_order("Limit order requires price"))?;

        let step = ExecutionStep {
            step_id: 1,
            execution_type: ExecutionType::Limit,
            amount: order_request.size,
            price: Some(price),
            max_slippage: None, // No slippage for limit orders
            delay_before_execution: None,
            timeout: order_request.expires_at.map(|ts| {
                Duration::seconds(ts - Utc::now().timestamp())
            }),
        };

        Ok(ExecutionPlan {
            plan_id: self.generate_plan_id(),
            strategy: order_request.execution_strategy.clone(),
            total_amount: order_request.size,
            estimated_duration: Duration::hours(24), // Default for limit orders
            estimated_fees: fee_estimate.total_fees,
            steps: vec![step],
            created_at: Utc::now(),
        })
    }

    async fn execute_step(&self, order_id: u64, step: &ExecutionStep) -> TradingResult<ExecutionStepResult> {
        // Mock implementation - in production, this would interact with DEXs/exchanges
        let start_time = Utc::now();

        // Simulate execution delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Mock execution results
        let filled_amount = step.amount; // Assume full fill for simplicity
        let average_price = step.price.unwrap_or(Price::from_u64(100 * Price::PRECISION)?);
        let fees_paid = filled_amount.checked_mul(&Price::from_basis_points(30)?.into()) // 0.3% fee
            .ok_or_else(|| TradingError::internal("Fee calculation overflow"))?;

        Ok(ExecutionStepResult {
            step_id: step.step_id,
            status: StepExecutionStatus::Completed,
            filled_amount,
            average_price,
            fees_paid,
            execution_time: Utc::now().signed_duration_since(start_time),
            transaction_hashes: vec!["mock_tx_hash".to_string()],
            error_message: None,
        })
    }

    fn calculate_slippage(&self, order_request: &OrderRequest, execution_price: Price) -> TradingResult<Percentage> {
        // For market orders, calculate slippage against mid price
        // For limit orders, there should be no slippage
        match order_request.order_type {
            moby_types::OrderType::Market => {
                // TODO: Get actual market mid price
                let mid_price = Price::from_u64(100 * Price::PRECISION)?;
                let slippage_amount = if execution_price > mid_price {
                    execution_price.checked_sub(&mid_price)
                        .ok_or_else(|| TradingError::internal("Price underflow"))?
                } else {
                    mid_price.checked_sub(&execution_price)
                        .ok_or_else(|| TradingError::internal("Price underflow"))?
                };

                Percentage::from_ratio(slippage_amount.as_u64(), mid_price.as_u64())
                    .map_err(|e| TradingError::Math(e))
            }
            _ => Ok(Percentage::zero()),
        }
    }

    fn generate_plan_id(&self) -> u64 {
        // Simple implementation - in production, use proper ID generation
        Utc::now().timestamp_nanos() as u64
    }
}

/// Execution plan for an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub plan_id: u64,
    pub strategy: ExecutionStrategy,
    pub total_amount: Amount,
    pub estimated_duration: Duration,
    pub estimated_fees: Amount,
    pub steps: Vec<ExecutionStep>,
    pub created_at: DateTime<Utc>,
}

/// Individual execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_id: u32,
    pub execution_type: ExecutionType,
    pub amount: Amount,
    pub price: Option<Price>,
    pub max_slippage: Option<Percentage>,
    pub delay_before_execution: Option<u64>, // milliseconds
    pub timeout: Option<Duration>,
}

/// Types of execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionType {
    Market,
    Limit,
    TwapSlice,
    VwapSlice,
    Otc,
    CrossChain,
}

/// Result of order execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub order_id: u64,
    pub status: ExecutionStatus,
    pub total_filled: Amount,
    pub remaining_amount: Amount,
    pub weighted_average_price: Price,
    pub total_fees: Amount,
    pub execution_time: Duration,
    pub steps: Vec<ExecutionStepResult>,
    pub error_message: Option<String>,
    pub slippage: Percentage,
    pub market_impact: Percentage,
}

/// Execution status
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    FullyExecuted,
    PartiallyExecuted,
    Failed,
}

/// Result of a single execution step
#[derive(Debug, Clone)]
pub struct ExecutionStepResult {
    pub step_id: u32,
    pub status: StepExecutionStatus,
    pub filled_amount: Amount,
    pub average_price: Price,
    pub fees_paid: Amount,
    pub execution_time: Duration,
    pub transaction_hashes: Vec<String>,
    pub error_message: Option<String>,
}

/// Status of step execution
#[derive(Debug, Clone, PartialEq)]
pub enum StepExecutionStatus {
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TradingConfig, risk::RiskAssessment, fees::FeeEstimate, engine::MarketData};
    use moby_types::{OrderType, TradingTier, TimeInForce};

    fn create_test_market_data() -> MarketData {
        MarketData {
            base_token: moby_types::AccountKey::new_unique(),
            quote_token: moby_types::AccountKey::new_unique(),
            bid_price: Price::from_u64(99 * Price::PRECISION).unwrap(),
            ask_price: Price::from_u64(101 * Price::PRECISION).unwrap(),
            mid_price: Price::from_u64(100 * Price::PRECISION).unwrap(),
            liquidity: Amount::from_u64(1_000_000 * Price::PRECISION),
            is_active: true,
            last_updated: Utc::now(),
        }
    }

    fn create_test_order_request() -> OrderRequest {
        crate::OrderRequest {
            trader: moby_types::AccountKey::new_unique(),
            base_token: moby_types::AccountKey::new_unique(),
            quote_token: moby_types::AccountKey::new_unique(),
            order_type: OrderType::Market,
            side: OrderSide::Buy,
            size: Amount::from_u64(1000 * Price::PRECISION),
            price: None,
            execution_strategy: ExecutionStrategy::Market,
            slippage_tolerance: 100, // 1%
            time_in_force: TimeInForce::Ioc,
            tier: TradingTier::Retail,
            privacy_enabled: false,
            otc_enabled: false,
            cross_chain_enabled: false,
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_market_execution_plan() {
        let config = TradingConfig::default();
        let engine = ExecutionEngine::new(config);
        let order_request = create_test_order_request();
        let market_data = create_test_market_data();
        let risk_assessment = RiskAssessment::default();
        let fee_estimate = FeeEstimate {
            base_fee: Amount::from_u64(100),
            total_fees: Amount::from_u64(100),
        };

        let plan = engine.create_execution_plan(
            &order_request,
            &market_data,
            &risk_assessment,
            &fee_estimate,
        ).await.unwrap();

        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.total_amount, order_request.size);
        assert!(matches!(plan.steps[0].execution_type, ExecutionType::Market));
    }

    #[tokio::test]
    async fn test_execute_market_order() {
        let config = TradingConfig::default();
        let engine = ExecutionEngine::new(config);
        let order_request = create_test_order_request();
        let market_data = create_test_market_data();
        let risk_assessment = RiskAssessment::default();
        let fee_estimate = FeeEstimate {
            base_fee: Amount::from_u64(100),
            total_fees: Amount::from_u64(100),
        };

        let plan = engine.create_execution_plan(
            &order_request,
            &market_data,
            &risk_assessment,
            &fee_estimate,
        ).await.unwrap();

        let result = engine.execute_order(1, order_request.clone(), plan).await.unwrap();

        assert_eq!(result.status, ExecutionStatus::FullyExecuted);
        assert_eq!(result.total_filled, order_request.size);
        assert_eq!(result.remaining_amount, Amount::zero());
        assert!(!result.steps.is_empty());
    }

    #[tokio::test]
    async fn test_create_limit_execution_plan() {
        let config = TradingConfig::default();
        let engine = ExecutionEngine::new(config);

        let mut order_request = create_test_order_request();
        order_request.order_type = OrderType::Limit;
        order_request.execution_strategy = ExecutionStrategy::Limit;
        order_request.price = Some(Price::from_u64(99 * Price::PRECISION).unwrap());

        let market_data = create_test_market_data();
        let risk_assessment = RiskAssessment::default();
        let fee_estimate = FeeEstimate {
            base_fee: Amount::from_u64(100),
            total_fees: Amount::from_u64(100),
        };

        let plan = engine.create_execution_plan(
            &order_request,
            &market_data,
            &risk_assessment,
            &fee_estimate,
        ).await.unwrap();

        assert_eq!(plan.steps.len(), 1);
        assert!(matches!(plan.steps[0].execution_type, ExecutionType::Limit));
        assert_eq!(plan.steps[0].price, order_request.price);
    }
}