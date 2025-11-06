// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Trading analytics and performance metrics

use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{TradingError, TradingResult, OrderRequest, execution::ExecutionResult};
use moby_types::{AccountKey, OrderSide, TradingTier};
use moby_math::{Amount, Price, Percentage};

/// Trading analytics system
#[derive(Debug)]
pub struct TradingAnalytics {
    /// Trading metrics by trader
    trader_metrics: HashMap<AccountKey, TraderMetrics>,

    /// Market metrics by trading pair
    market_metrics: HashMap<(AccountKey, AccountKey), MarketMetrics>,

    /// Global platform metrics
    platform_metrics: PlatformMetrics,

    /// Recent execution history for analysis
    execution_history: VecDeque<ExecutionRecord>,

    /// Performance benchmarks
    benchmarks: HashMap<String, Benchmark>,
}

impl TradingAnalytics {
    /// Create a new analytics system
    pub fn new() -> Self {
        Self {
            trader_metrics: HashMap::new(),
            market_metrics: HashMap::new(),
            platform_metrics: PlatformMetrics::new(),
            execution_history: VecDeque::new(),
            benchmarks: HashMap::new(),
        }
    }

    /// Record an execution for analytics
    pub fn record_execution(
        &mut self,
        order_request: &OrderRequest,
        execution_result: &Result<ExecutionResult, TradingError>,
    ) {
        let execution_record = ExecutionRecord {
            trader: order_request.trader,
            base_token: order_request.base_token,
            quote_token: order_request.quote_token,
            side: order_request.side,
            order_size: order_request.size,
            execution_strategy: order_request.execution_strategy.clone(),
            tier: order_request.tier,
            timestamp: Utc::now(),
            success: execution_result.is_ok(),
            execution_result: execution_result.as_ref().ok().cloned(),
            error_type: execution_result.as_ref().err().map(|e| format!("{:?}", e)),
        };

        // Update trader metrics
        self.update_trader_metrics(&execution_record);

        // Update market metrics
        self.update_market_metrics(&execution_record);

        // Update platform metrics
        self.update_platform_metrics(&execution_record);

        // Store execution record
        self.execution_history.push_back(execution_record);
        if self.execution_history.len() > 10000 {
            self.execution_history.pop_front();
        }
    }

    /// Get trading statistics for a trader
    pub async fn get_trading_stats(&self, trader: Option<AccountKey>) -> TradingResult<TradingStats> {
        match trader {
            Some(trader_key) => {
                let metrics = self.trader_metrics.get(&trader_key)
                    .ok_or_else(|| TradingError::invalid_order("Trader not found"))?;

                Ok(TradingStats {
                    trader: Some(trader_key),
                    total_volume: metrics.total_volume,
                    total_trades: metrics.total_trades,
                    successful_trades: metrics.successful_trades,
                    failed_trades: metrics.failed_trades,
                    average_trade_size: metrics.calculate_average_trade_size(),
                    total_fees_paid: metrics.total_fees_paid,
                    success_rate: metrics.calculate_success_rate(),
                    average_slippage: metrics.calculate_average_slippage(),
                    best_execution_rate: metrics.calculate_best_execution_rate(),
                    period_start: metrics.first_trade_time,
                    period_end: metrics.last_trade_time,
                    tier_distribution: self.calculate_tier_distribution(Some(trader_key)),
                    strategy_performance: metrics.strategy_performance.clone(),
                })
            }
            None => {
                // Platform-wide statistics
                let total_volume = self.platform_metrics.total_volume;
                let total_trades = self.platform_metrics.total_trades;

                Ok(TradingStats {
                    trader: None,
                    total_volume,
                    total_trades,
                    successful_trades: self.platform_metrics.successful_trades,
                    failed_trades: self.platform_metrics.failed_trades,
                    average_trade_size: if total_trades > 0 {
                        total_volume.checked_div_u64(total_trades).unwrap_or(Amount::zero())
                    } else {
                        Amount::zero()
                    },
                    total_fees_paid: self.platform_metrics.total_fees_collected,
                    success_rate: self.platform_metrics.calculate_success_rate(),
                    average_slippage: self.platform_metrics.average_slippage,
                    best_execution_rate: self.platform_metrics.best_execution_rate,
                    period_start: self.platform_metrics.start_time,
                    period_end: Utc::now(),
                    tier_distribution: self.calculate_tier_distribution(None),
                    strategy_performance: HashMap::new(),
                })
            }
        }
    }

    /// Get performance metrics for a trader
    pub fn get_performance_metrics(&self, trader: AccountKey) -> TradingResult<PerformanceMetrics> {
        let metrics = self.trader_metrics.get(&trader)
            .ok_or_else(|| TradingError::invalid_order("Trader not found"))?;

        Ok(PerformanceMetrics {
            trader,
            total_pnl: metrics.total_pnl,
            realized_pnl: metrics.realized_pnl,
            unrealized_pnl: metrics.unrealized_pnl,
            roi: metrics.calculate_roi(),
            sharpe_ratio: metrics.calculate_sharpe_ratio(),
            max_drawdown: metrics.max_drawdown,
            win_rate: metrics.calculate_win_rate(),
            profit_factor: metrics.calculate_profit_factor(),
            average_win: metrics.calculate_average_win(),
            average_loss: metrics.calculate_average_loss(),
            largest_win: metrics.largest_win,
            largest_loss: metrics.largest_loss,
            consecutive_wins: metrics.consecutive_wins,
            consecutive_losses: metrics.consecutive_losses,
            calmar_ratio: metrics.calculate_calmar_ratio(),
            sortino_ratio: metrics.calculate_sortino_ratio(),
            last_updated: metrics.last_updated,
        })
    }

    /// Get market impact analysis
    pub fn get_market_impact_analysis(
        &self,
        base_token: AccountKey,
        quote_token: AccountKey,
        time_window: Duration,
    ) -> TradingResult<MarketImpactAnalysis> {
        let market_pair = (base_token, quote_token);
        let cutoff_time = Utc::now() - time_window;

        // Filter executions for this market and time window
        let relevant_executions: Vec<_> = self.execution_history
            .iter()
            .filter(|record| {
                record.base_token == base_token &&
                record.quote_token == quote_token &&
                record.timestamp >= cutoff_time &&
                record.success
            })
            .collect();

        if relevant_executions.is_empty() {
            return Ok(MarketImpactAnalysis {
                base_token,
                quote_token,
                analysis_period: time_window,
                total_trades: 0,
                total_volume: Amount::zero(),
                average_impact: Percentage::zero(),
                median_impact: Percentage::zero(),
                impact_by_size: HashMap::new(),
                impact_by_strategy: HashMap::new(),
                impact_by_tier: HashMap::new(),
                temporary_impact: Percentage::zero(),
                permanent_impact: Percentage::zero(),
                last_updated: Utc::now(),
            });
        }

        let total_trades = relevant_executions.len() as u64;
        let total_volume = relevant_executions.iter()
            .map(|r| r.order_size)
            .fold(Amount::zero(), |acc, size| acc.checked_add(&size).unwrap_or(acc));

        // Calculate impact metrics
        let impacts: Vec<f64> = relevant_executions.iter()
            .filter_map(|r| r.execution_result.as_ref())
            .map(|result| result.slippage.as_percentage())
            .collect();

        let average_impact = if !impacts.is_empty() {
            Percentage::from_percentage(impacts.iter().sum::<f64>() / impacts.len() as f64)
                .unwrap_or(Percentage::zero())
        } else {
            Percentage::zero()
        };

        let median_impact = if !impacts.is_empty() {
            let mut sorted_impacts = impacts.clone();
            sorted_impacts.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median_value = sorted_impacts[sorted_impacts.len() / 2];
            Percentage::from_percentage(median_value).unwrap_or(Percentage::zero())
        } else {
            Percentage::zero()
        };

        // Calculate impact by size buckets
        let impact_by_size = self.calculate_impact_by_size(&relevant_executions);

        // Calculate impact by strategy
        let impact_by_strategy = self.calculate_impact_by_strategy(&relevant_executions);

        // Calculate impact by tier
        let impact_by_tier = self.calculate_impact_by_tier(&relevant_executions);

        Ok(MarketImpactAnalysis {
            base_token,
            quote_token,
            analysis_period: time_window,
            total_trades,
            total_volume,
            average_impact,
            median_impact,
            impact_by_size,
            impact_by_strategy,
            impact_by_tier,
            temporary_impact: Percentage::zero(), // TODO: Calculate temporary impact
            permanent_impact: Percentage::zero(), // TODO: Calculate permanent impact
            last_updated: Utc::now(),
        })
    }

    /// Get execution quality metrics
    pub fn get_execution_quality_metrics(
        &self,
        trader: Option<AccountKey>,
        time_window: Duration,
    ) -> TradingResult<ExecutionQualityMetrics> {
        let cutoff_time = Utc::now() - time_window;

        let relevant_executions: Vec<_> = self.execution_history
            .iter()
            .filter(|record| {
                record.timestamp >= cutoff_time &&
                record.success &&
                trader.map_or(true, |t| record.trader == t)
            })
            .collect();

        if relevant_executions.is_empty() {
            return Ok(ExecutionQualityMetrics::default());
        }

        let total_trades = relevant_executions.len() as u64;

        // Calculate slippage metrics
        let slippages: Vec<f64> = relevant_executions.iter()
            .filter_map(|r| r.execution_result.as_ref())
            .map(|result| result.slippage.as_percentage())
            .collect();

        let average_slippage = if !slippages.is_empty() {
            Percentage::from_percentage(slippages.iter().sum::<f64>() / slippages.len() as f64)
                .unwrap_or(Percentage::zero())
        } else {
            Percentage::zero()
        };

        // Calculate execution time metrics
        let execution_times: Vec<i64> = relevant_executions.iter()
            .filter_map(|r| r.execution_result.as_ref())
            .map(|result| result.execution_time.num_milliseconds())
            .collect();

        let average_execution_time = if !execution_times.is_empty() {
            Duration::milliseconds(execution_times.iter().sum::<i64>() / execution_times.len() as i64)
        } else {
            Duration::zero()
        };

        // Calculate fill rate
        let successful_fills = relevant_executions.iter()
            .filter_map(|r| r.execution_result.as_ref())
            .filter(|result| result.total_filled > Amount::zero())
            .count() as u64;

        let fill_rate = if total_trades > 0 {
            (successful_fills as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        // Calculate price improvement instances
        let price_improvements = relevant_executions.iter()
            .filter_map(|r| r.execution_result.as_ref())
            .filter(|result| {
                // TODO: Determine if execution had price improvement
                false
            })
            .count() as u64;

        let price_improvement_rate = if total_trades > 0 {
            (price_improvements as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        Ok(ExecutionQualityMetrics {
            analysis_period: time_window,
            total_trades,
            average_slippage,
            median_slippage: Percentage::zero(), // TODO: Calculate median
            slippage_std_dev: Percentage::zero(), // TODO: Calculate standard deviation
            average_execution_time,
            median_execution_time: Duration::zero(), // TODO: Calculate median
            fill_rate,
            partial_fill_rate: 0.0, // TODO: Calculate partial fill rate
            price_improvement_rate,
            benchmark_performance: 0.0, // TODO: Compare against benchmark
            last_updated: Utc::now(),
        })
    }

    // Private helper methods

    fn update_trader_metrics(&mut self, record: &ExecutionRecord) {
        let metrics = self.trader_metrics
            .entry(record.trader)
            .or_insert_with(|| TraderMetrics::new(record.trader));

        metrics.total_trades += 1;
        metrics.total_volume = metrics.total_volume.checked_add(&record.order_size)
            .unwrap_or(metrics.total_volume);

        if record.success {
            metrics.successful_trades += 1;

            if let Some(result) = &record.execution_result {
                metrics.total_fees_paid = metrics.total_fees_paid.checked_add(&result.total_fees)
                    .unwrap_or(metrics.total_fees_paid);

                // Update slippage tracking
                metrics.slippage_history.push(result.slippage.as_percentage());
                if metrics.slippage_history.len() > 1000 {
                    metrics.slippage_history.remove(0);
                }

                // Update strategy performance
                let strategy_key = format!("{:?}", record.execution_strategy);
                let strategy_metrics = metrics.strategy_performance
                    .entry(strategy_key)
                    .or_insert_with(StrategyMetrics::new);
                strategy_metrics.record_execution(result);
            }
        } else {
            metrics.failed_trades += 1;
        }

        if metrics.first_trade_time.is_none() {
            metrics.first_trade_time = Some(record.timestamp);
        }
        metrics.last_trade_time = Some(record.timestamp);
        metrics.last_updated = Utc::now();
    }

    fn update_market_metrics(&mut self, record: &ExecutionRecord) {
        let market_pair = (record.base_token, record.quote_token);
        let metrics = self.market_metrics
            .entry(market_pair)
            .or_insert_with(|| MarketMetrics::new(record.base_token, record.quote_token));

        metrics.total_trades += 1;
        metrics.total_volume = metrics.total_volume.checked_add(&record.order_size)
            .unwrap_or(metrics.total_volume);

        if record.success {
            metrics.successful_trades += 1;
        } else {
            metrics.failed_trades += 1;
        }

        metrics.last_updated = Utc::now();
    }

    fn update_platform_metrics(&mut self, record: &ExecutionRecord) {
        self.platform_metrics.total_trades += 1;
        self.platform_metrics.total_volume = self.platform_metrics.total_volume
            .checked_add(&record.order_size)
            .unwrap_or(self.platform_metrics.total_volume);

        if record.success {
            self.platform_metrics.successful_trades += 1;

            if let Some(result) = &record.execution_result {
                self.platform_metrics.total_fees_collected = self.platform_metrics.total_fees_collected
                    .checked_add(&result.total_fees)
                    .unwrap_or(self.platform_metrics.total_fees_collected);
            }
        } else {
            self.platform_metrics.failed_trades += 1;
        }

        // Update tier distribution
        *self.platform_metrics.tier_distribution.entry(record.tier).or_insert(0) += 1;

        self.platform_metrics.last_updated = Utc::now();
    }

    fn calculate_tier_distribution(&self, trader: Option<AccountKey>) -> HashMap<TradingTier, u64> {
        match trader {
            Some(trader_key) => {
                // Get distribution for specific trader (simplified - just their current tier)
                if let Some(metrics) = self.trader_metrics.get(&trader_key) {
                    let mut distribution = HashMap::new();
                    distribution.insert(TradingTier::Retail, 1); // Simplified
                    distribution
                } else {
                    HashMap::new()
                }
            }
            None => self.platform_metrics.tier_distribution.clone(),
        }
    }

    fn calculate_impact_by_size(&self, executions: &[&ExecutionRecord]) -> HashMap<String, Percentage> {
        let mut buckets: HashMap<String, Vec<f64>> = HashMap::new();

        for execution in executions {
            if let Some(result) = &execution.execution_result {
                let size_usd = execution.order_size.as_u64() / Price::PRECISION;
                let bucket = match size_usd {
                    0..=10_000 => "0-10K",
                    10_001..=100_000 => "10K-100K",
                    100_001..=1_000_000 => "100K-1M",
                    1_000_001..=10_000_000 => "1M-10M",
                    _ => "10M+",
                };

                buckets.entry(bucket.to_string())
                    .or_insert_with(Vec::new)
                    .push(result.slippage.as_percentage());
            }
        }

        buckets.into_iter()
            .map(|(bucket, impacts)| {
                let avg_impact = impacts.iter().sum::<f64>() / impacts.len() as f64;
                (bucket, Percentage::from_percentage(avg_impact).unwrap_or(Percentage::zero()))
            })
            .collect()
    }

    fn calculate_impact_by_strategy(&self, executions: &[&ExecutionRecord]) -> HashMap<String, Percentage> {
        let mut strategy_impacts: HashMap<String, Vec<f64>> = HashMap::new();

        for execution in executions {
            if let Some(result) = &execution.execution_result {
                let strategy_key = format!("{:?}", execution.execution_strategy);
                strategy_impacts.entry(strategy_key)
                    .or_insert_with(Vec::new)
                    .push(result.slippage.as_percentage());
            }
        }

        strategy_impacts.into_iter()
            .map(|(strategy, impacts)| {
                let avg_impact = impacts.iter().sum::<f64>() / impacts.len() as f64;
                (strategy, Percentage::from_percentage(avg_impact).unwrap_or(Percentage::zero()))
            })
            .collect()
    }

    fn calculate_impact_by_tier(&self, executions: &[&ExecutionRecord]) -> HashMap<TradingTier, Percentage> {
        let mut tier_impacts: HashMap<TradingTier, Vec<f64>> = HashMap::new();

        for execution in executions {
            if let Some(result) = &execution.execution_result {
                tier_impacts.entry(execution.tier)
                    .or_insert_with(Vec::new)
                    .push(result.slippage.as_percentage());
            }
        }

        tier_impacts.into_iter()
            .map(|(tier, impacts)| {
                let avg_impact = impacts.iter().sum::<f64>() / impacts.len() as f64;
                (tier, Percentage::from_percentage(avg_impact).unwrap_or(Percentage::zero()))
            })
            .collect()
    }
}

// Supporting data structures

#[derive(Debug, Clone)]
struct ExecutionRecord {
    trader: AccountKey,
    base_token: AccountKey,
    quote_token: AccountKey,
    side: OrderSide,
    order_size: Amount,
    execution_strategy: moby_types::ExecutionStrategy,
    tier: TradingTier,
    timestamp: DateTime<Utc>,
    success: bool,
    execution_result: Option<ExecutionResult>,
    error_type: Option<String>,
}

#[derive(Debug)]
struct TraderMetrics {
    trader: AccountKey,
    total_trades: u64,
    successful_trades: u64,
    failed_trades: u64,
    total_volume: Amount,
    total_fees_paid: Amount,
    total_pnl: Amount,
    realized_pnl: Amount,
    unrealized_pnl: Amount,
    max_drawdown: Percentage,
    largest_win: Amount,
    largest_loss: Amount,
    consecutive_wins: u32,
    consecutive_losses: u32,
    slippage_history: Vec<f64>,
    first_trade_time: Option<DateTime<Utc>>,
    last_trade_time: Option<DateTime<Utc>>,
    last_updated: DateTime<Utc>,
    strategy_performance: HashMap<String, StrategyMetrics>,
}

impl TraderMetrics {
    fn new(trader: AccountKey) -> Self {
        Self {
            trader,
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_volume: Amount::zero(),
            total_fees_paid: Amount::zero(),
            total_pnl: Amount::zero(),
            realized_pnl: Amount::zero(),
            unrealized_pnl: Amount::zero(),
            max_drawdown: Percentage::zero(),
            largest_win: Amount::zero(),
            largest_loss: Amount::zero(),
            consecutive_wins: 0,
            consecutive_losses: 0,
            slippage_history: Vec::new(),
            first_trade_time: None,
            last_trade_time: None,
            last_updated: Utc::now(),
            strategy_performance: HashMap::new(),
        }
    }

    fn calculate_average_trade_size(&self) -> Amount {
        if self.total_trades > 0 {
            self.total_volume.checked_div_u64(self.total_trades).unwrap_or(Amount::zero())
        } else {
            Amount::zero()
        }
    }

    fn calculate_success_rate(&self) -> f64 {
        if self.total_trades > 0 {
            (self.successful_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            0.0
        }
    }

    fn calculate_average_slippage(&self) -> Percentage {
        if !self.slippage_history.is_empty() {
            let avg = self.slippage_history.iter().sum::<f64>() / self.slippage_history.len() as f64;
            Percentage::from_percentage(avg).unwrap_or(Percentage::zero())
        } else {
            Percentage::zero()
        }
    }

    fn calculate_best_execution_rate(&self) -> f64 {
        // TODO: Implement best execution rate calculation
        0.0
    }

    fn calculate_roi(&self) -> Percentage {
        // TODO: Implement ROI calculation
        Percentage::zero()
    }

    fn calculate_sharpe_ratio(&self) -> f64 {
        // TODO: Implement Sharpe ratio calculation
        0.0
    }

    fn calculate_win_rate(&self) -> f64 {
        // TODO: Implement win rate calculation
        0.0
    }

    fn calculate_profit_factor(&self) -> f64 {
        // TODO: Implement profit factor calculation
        0.0
    }

    fn calculate_average_win(&self) -> Amount {
        // TODO: Implement average win calculation
        Amount::zero()
    }

    fn calculate_average_loss(&self) -> Amount {
        // TODO: Implement average loss calculation
        Amount::zero()
    }

    fn calculate_calmar_ratio(&self) -> f64 {
        // TODO: Implement Calmar ratio calculation
        0.0
    }

    fn calculate_sortino_ratio(&self) -> f64 {
        // TODO: Implement Sortino ratio calculation
        0.0
    }
}

#[derive(Debug)]
struct MarketMetrics {
    base_token: AccountKey,
    quote_token: AccountKey,
    total_trades: u64,
    successful_trades: u64,
    failed_trades: u64,
    total_volume: Amount,
    last_updated: DateTime<Utc>,
}

impl MarketMetrics {
    fn new(base_token: AccountKey, quote_token: AccountKey) -> Self {
        Self {
            base_token,
            quote_token,
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_volume: Amount::zero(),
            last_updated: Utc::now(),
        }
    }
}

#[derive(Debug)]
struct PlatformMetrics {
    total_trades: u64,
    successful_trades: u64,
    failed_trades: u64,
    total_volume: Amount,
    total_fees_collected: Amount,
    average_slippage: Percentage,
    best_execution_rate: f64,
    tier_distribution: HashMap<TradingTier, u64>,
    start_time: DateTime<Utc>,
    last_updated: DateTime<Utc>,
}

impl PlatformMetrics {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_volume: Amount::zero(),
            total_fees_collected: Amount::zero(),
            average_slippage: Percentage::zero(),
            best_execution_rate: 0.0,
            tier_distribution: HashMap::new(),
            start_time: now,
            last_updated: now,
        }
    }

    fn calculate_success_rate(&self) -> f64 {
        if self.total_trades > 0 {
            (self.successful_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
struct StrategyMetrics {
    total_executions: u64,
    successful_executions: u64,
    total_slippage: Percentage,
    average_execution_time: Duration,
    total_fees: Amount,
}

impl StrategyMetrics {
    fn new() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            total_slippage: Percentage::zero(),
            average_execution_time: Duration::zero(),
            total_fees: Amount::zero(),
        }
    }

    fn record_execution(&mut self, result: &ExecutionResult) {
        self.total_executions += 1;
        self.successful_executions += 1; // Only successful executions reach here

        // Update slippage (rolling average)
        let current_avg = self.total_slippage.as_percentage();
        let new_slippage = result.slippage.as_percentage();
        let updated_avg = (current_avg * (self.total_executions - 1) as f64 + new_slippage) / self.total_executions as f64;
        self.total_slippage = Percentage::from_percentage(updated_avg).unwrap_or(Percentage::zero());

        // Update execution time (rolling average)
        let current_time_ms = self.average_execution_time.num_milliseconds();
        let new_time_ms = result.execution_time.num_milliseconds();
        let updated_time_ms = (current_time_ms * (self.total_executions - 1) as i64 + new_time_ms) / self.total_executions as i64;
        self.average_execution_time = Duration::milliseconds(updated_time_ms);

        self.total_fees = self.total_fees.checked_add(&result.total_fees).unwrap_or(self.total_fees);
    }
}

#[derive(Debug, Clone)]
struct Benchmark {
    name: String,
    value: f64,
    timestamp: DateTime<Utc>,
}

// Public API structures

/// Trading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStats {
    pub trader: Option<AccountKey>,
    pub total_volume: Amount,
    pub total_trades: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub average_trade_size: Amount,
    pub total_fees_paid: Amount,
    pub success_rate: f64,
    pub average_slippage: Percentage,
    pub best_execution_rate: f64,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: DateTime<Utc>,
    pub tier_distribution: HashMap<TradingTier, u64>,
    pub strategy_performance: HashMap<String, StrategyMetrics>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub trader: AccountKey,
    pub total_pnl: Amount,
    pub realized_pnl: Amount,
    pub unrealized_pnl: Amount,
    pub roi: Percentage,
    pub sharpe_ratio: f64,
    pub max_drawdown: Percentage,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub average_win: Amount,
    pub average_loss: Amount,
    pub largest_win: Amount,
    pub largest_loss: Amount,
    pub consecutive_wins: u32,
    pub consecutive_losses: u32,
    pub calmar_ratio: f64,
    pub sortino_ratio: f64,
    pub last_updated: DateTime<Utc>,
}

/// Market impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpactAnalysis {
    pub base_token: AccountKey,
    pub quote_token: AccountKey,
    pub analysis_period: Duration,
    pub total_trades: u64,
    pub total_volume: Amount,
    pub average_impact: Percentage,
    pub median_impact: Percentage,
    pub impact_by_size: HashMap<String, Percentage>,
    pub impact_by_strategy: HashMap<String, Percentage>,
    pub impact_by_tier: HashMap<TradingTier, Percentage>,
    pub temporary_impact: Percentage,
    pub permanent_impact: Percentage,
    pub last_updated: DateTime<Utc>,
}

/// Execution quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionQualityMetrics {
    pub analysis_period: Duration,
    pub total_trades: u64,
    pub average_slippage: Percentage,
    pub median_slippage: Percentage,
    pub slippage_std_dev: Percentage,
    pub average_execution_time: Duration,
    pub median_execution_time: Duration,
    pub fill_rate: f64,
    pub partial_fill_rate: f64,
    pub price_improvement_rate: f64,
    pub benchmark_performance: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for ExecutionQualityMetrics {
    fn default() -> Self {
        Self {
            analysis_period: Duration::zero(),
            total_trades: 0,
            average_slippage: Percentage::zero(),
            median_slippage: Percentage::zero(),
            slippage_std_dev: Percentage::zero(),
            average_execution_time: Duration::zero(),
            median_execution_time: Duration::zero(),
            fill_rate: 0.0,
            partial_fill_rate: 0.0,
            price_improvement_rate: 0.0,
            benchmark_performance: 0.0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for TradingAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{ExecutionStatus, ExecutionStepResult, StepExecutionStatus};
    use moby_types::{OrderType, TimeInForce, ExecutionStrategy};

    fn create_test_order_request() -> OrderRequest {
        crate::OrderRequest {
            trader: AccountKey::new_unique(),
            base_token: AccountKey::new_unique(),
            quote_token: AccountKey::new_unique(),
            order_type: OrderType::Market,
            side: OrderSide::Buy,
            size: Amount::from_u64(100_000 * Price::PRECISION),
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

    fn create_test_execution_result() -> ExecutionResult {
        ExecutionResult {
            order_id: 1,
            status: ExecutionStatus::FullyExecuted,
            total_filled: Amount::from_u64(100_000 * Price::PRECISION),
            remaining_amount: Amount::zero(),
            weighted_average_price: Price::from_u64(100 * Price::PRECISION).unwrap(),
            total_fees: Amount::from_u64(300 * Price::PRECISION),
            execution_time: Duration::seconds(5),
            steps: vec![ExecutionStepResult {
                step_id: 1,
                status: StepExecutionStatus::Completed,
                filled_amount: Amount::from_u64(100_000 * Price::PRECISION),
                average_price: Price::from_u64(100 * Price::PRECISION).unwrap(),
                fees_paid: Amount::from_u64(300 * Price::PRECISION),
                execution_time: Duration::seconds(5),
                transaction_hashes: vec!["test_hash".to_string()],
                error_message: None,
            }],
            error_message: None,
            slippage: Percentage::from_basis_points(50).unwrap(),
            market_impact: Percentage::from_basis_points(25).unwrap(),
        }
    }

    #[tokio::test]
    async fn test_analytics_creation() {
        let analytics = TradingAnalytics::new();
        assert_eq!(analytics.trader_metrics.len(), 0);
        assert_eq!(analytics.market_metrics.len(), 0);
        assert_eq!(analytics.execution_history.len(), 0);
    }

    #[test]
    fn test_record_successful_execution() {
        let mut analytics = TradingAnalytics::new();
        let order_request = create_test_order_request();
        let execution_result = Ok(create_test_execution_result());

        analytics.record_execution(&order_request, &execution_result);

        // Check trader metrics were updated
        assert_eq!(analytics.trader_metrics.len(), 1);
        let trader_metrics = analytics.trader_metrics.get(&order_request.trader).unwrap();
        assert_eq!(trader_metrics.total_trades, 1);
        assert_eq!(trader_metrics.successful_trades, 1);
        assert_eq!(trader_metrics.failed_trades, 0);

        // Check platform metrics were updated
        assert_eq!(analytics.platform_metrics.total_trades, 1);
        assert_eq!(analytics.platform_metrics.successful_trades, 1);

        // Check execution history
        assert_eq!(analytics.execution_history.len(), 1);
    }

    #[test]
    fn test_record_failed_execution() {
        let mut analytics = TradingAnalytics::new();
        let order_request = create_test_order_request();
        let execution_result = Err(TradingError::execution_failed("Test error"));

        analytics.record_execution(&order_request, &execution_result);

        let trader_metrics = analytics.trader_metrics.get(&order_request.trader).unwrap();
        assert_eq!(trader_metrics.total_trades, 1);
        assert_eq!(trader_metrics.successful_trades, 0);
        assert_eq!(trader_metrics.failed_trades, 1);
    }

    #[tokio::test]
    async fn test_get_trading_stats() {
        let mut analytics = TradingAnalytics::new();
        let order_request = create_test_order_request();
        let execution_result = Ok(create_test_execution_result());

        analytics.record_execution(&order_request, &execution_result);

        let stats = analytics.get_trading_stats(Some(order_request.trader)).await.unwrap();
        assert_eq!(stats.total_trades, 1);
        assert_eq!(stats.successful_trades, 1);
        assert_eq!(stats.success_rate, 100.0);
    }

    #[test]
    fn test_market_impact_analysis() {
        let mut analytics = TradingAnalytics::new();
        let order_request = create_test_order_request();
        let execution_result = Ok(create_test_execution_result());

        analytics.record_execution(&order_request, &execution_result);

        let analysis = analytics.get_market_impact_analysis(
            order_request.base_token,
            order_request.quote_token,
            Duration::hours(1),
        ).unwrap();

        assert_eq!(analysis.total_trades, 1);
        assert!(analysis.average_impact > Percentage::zero());
    }

    #[test]
    fn test_strategy_metrics_recording() {
        let mut strategy_metrics = StrategyMetrics::new();
        let execution_result = create_test_execution_result();

        strategy_metrics.record_execution(&execution_result);

        assert_eq!(strategy_metrics.total_executions, 1);
        assert_eq!(strategy_metrics.successful_executions, 1);
        assert!(strategy_metrics.total_fees > Amount::zero());
    }
}