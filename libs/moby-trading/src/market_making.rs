// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Market making and liquidity provision system

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{
    TradingError, TradingResult, OrderRequest,
    engine::MarketData,
    matching::{OrderBookSnapshot, OrderBookLevel},
};

use moby_types::{OrderSide, OrderType, AccountKey, TradingTier, ExecutionStrategy, TimeInForce};
use moby_math::{Amount, Price, Percentage};

/// Market making system for providing liquidity
#[derive(Debug)]
pub struct MarketMaker {
    config: MarketMakingConfig,
    active_strategies: HashMap<(AccountKey, AccountKey), MarketMakingStrategy>,
    performance_metrics: HashMap<(AccountKey, AccountKey), PerformanceMetrics>,
}

impl MarketMaker {
    /// Create a new market maker
    pub fn new(config: MarketMakingConfig) -> Self {
        Self {
            config,
            active_strategies: HashMap::new(),
            performance_metrics: HashMap::new(),
        }
    }

    /// Start market making for a trading pair
    pub async fn start_market_making(
        &mut self,
        base_token: AccountKey,
        quote_token: AccountKey,
        strategy_config: StrategyConfig,
    ) -> TradingResult<()> {
        let market_pair = (base_token, quote_token);

        if self.active_strategies.contains_key(&market_pair) {
            return Err(TradingError::invalid_order("Market making already active for this pair"));
        }

        let strategy = MarketMakingStrategy::new(
            base_token,
            quote_token,
            strategy_config,
            self.config.clone(),
        );

        self.active_strategies.insert(market_pair, strategy);
        self.performance_metrics.insert(market_pair, PerformanceMetrics::new());

        Ok(())
    }

    /// Stop market making for a trading pair
    pub async fn stop_market_making(
        &mut self,
        base_token: AccountKey,
        quote_token: AccountKey,
    ) -> TradingResult<()> {
        let market_pair = (base_token, quote_token);
        self.active_strategies.remove(&market_pair);
        Ok(())
    }

    /// Generate quotes for market making
    pub async fn generate_quotes(
        &self,
        base_token: AccountKey,
        quote_token: AccountKey,
        market_data: &MarketData,
        order_book: &OrderBookSnapshot,
    ) -> TradingResult<Vec<QuoteRequest>> {
        let market_pair = (base_token, quote_token);

        let strategy = self.active_strategies.get(&market_pair)
            .ok_or_else(|| TradingError::invalid_order("No active market making strategy for this pair"))?;

        strategy.generate_quotes(market_data, order_book).await
    }

    /// Update quotes based on market conditions
    pub async fn update_quotes(
        &mut self,
        base_token: AccountKey,
        quote_token: AccountKey,
        market_data: &MarketData,
        order_book: &OrderBookSnapshot,
    ) -> TradingResult<Vec<QuoteUpdate>> {
        let market_pair = (base_token, quote_token);

        let strategy = self.active_strategies.get_mut(&market_pair)
            .ok_or_else(|| TradingError::invalid_order("No active market making strategy for this pair"))?;

        strategy.update_quotes(market_data, order_book).await
    }

    /// Record a trade execution for performance tracking
    pub fn record_trade(
        &mut self,
        base_token: AccountKey,
        quote_token: AccountKey,
        side: OrderSide,
        size: Amount,
        price: Price,
        fees: Amount,
    ) -> TradingResult<()> {
        let market_pair = (base_token, quote_token);

        if let Some(metrics) = self.performance_metrics.get_mut(&market_pair) {
            metrics.record_trade(side, size, price, fees)?;
        }

        Ok(())
    }

    /// Get performance metrics for a trading pair
    pub fn get_performance_metrics(
        &self,
        base_token: AccountKey,
        quote_token: AccountKey,
    ) -> Option<&PerformanceMetrics> {
        let market_pair = (base_token, quote_token);
        self.performance_metrics.get(&market_pair)
    }

    /// Get risk exposure for all active strategies
    pub fn get_risk_exposure(&self) -> HashMap<(AccountKey, AccountKey), RiskExposure> {
        self.active_strategies
            .iter()
            .map(|(pair, strategy)| {
                let exposure = strategy.calculate_risk_exposure();
                (*pair, exposure)
            })
            .collect()
    }
}

/// Market making strategy for a specific trading pair
#[derive(Debug)]
pub struct MarketMakingStrategy {
    base_token: AccountKey,
    quote_token: AccountKey,
    config: StrategyConfig,
    global_config: MarketMakingConfig,
    inventory: Inventory,
    active_quotes: HashMap<u64, ActiveQuote>,
    last_update: DateTime<Utc>,
}

impl MarketMakingStrategy {
    fn new(
        base_token: AccountKey,
        quote_token: AccountKey,
        config: StrategyConfig,
        global_config: MarketMakingConfig,
    ) -> Self {
        Self {
            base_token,
            quote_token,
            config,
            global_config,
            inventory: Inventory::new(),
            active_quotes: HashMap::new(),
            last_update: Utc::now(),
        }
    }

    async fn generate_quotes(
        &self,
        market_data: &MarketData,
        order_book: &OrderBookSnapshot,
    ) -> TradingResult<Vec<QuoteRequest>> {
        let mut quotes = Vec::new();

        // Calculate fair value and spread
        let fair_value = self.calculate_fair_value(market_data, order_book)?;
        let spread = self.calculate_spread(market_data, order_book)?;

        // Calculate inventory adjustment
        let inventory_adjustment = self.calculate_inventory_adjustment()?;

        // Generate bid quotes
        let bid_price = fair_value
            .checked_sub(&(spread / 2.0))?
            .checked_sub(&inventory_adjustment)?;

        let bid_size = self.calculate_quote_size(OrderSide::Buy, market_data)?;

        if bid_size > Amount::zero() {
            quotes.push(QuoteRequest {
                side: OrderSide::Buy,
                price: bid_price,
                size: bid_size,
                strategy: QuoteStrategy::MarketMaking,
                valid_until: Utc::now() + Duration::seconds(self.config.quote_ttl_seconds as i64),
            });
        }

        // Generate ask quotes
        let ask_price = fair_value
            .checked_add(&(spread / 2.0))?
            .checked_add(&inventory_adjustment)?;

        let ask_size = self.calculate_quote_size(OrderSide::Sell, market_data)?;

        if ask_size > Amount::zero() {
            quotes.push(QuoteRequest {
                side: OrderSide::Sell,
                price: ask_price,
                size: ask_size,
                strategy: QuoteStrategy::MarketMaking,
                valid_until: Utc::now() + Duration::seconds(self.config.quote_ttl_seconds as i64),
            });
        }

        Ok(quotes)
    }

    async fn update_quotes(
        &mut self,
        market_data: &MarketData,
        order_book: &OrderBookSnapshot,
    ) -> TradingResult<Vec<QuoteUpdate>> {
        let mut updates = Vec::new();

        // Check if market conditions have changed significantly
        if !self.should_update_quotes(market_data)? {
            return Ok(updates);
        }

        // Cancel existing quotes that are no longer competitive
        let expired_quotes = self.find_expired_quotes(market_data, order_book)?;
        for quote_id in expired_quotes {
            updates.push(QuoteUpdate {
                quote_id,
                action: QuoteAction::Cancel,
                new_price: None,
                new_size: None,
            });
            self.active_quotes.remove(&quote_id);
        }

        // Generate new quotes
        let new_quotes = self.generate_quotes(market_data, order_book).await?;
        for quote_request in new_quotes {
            let quote_id = self.generate_quote_id();
            updates.push(QuoteUpdate {
                quote_id,
                action: QuoteAction::Create,
                new_price: Some(quote_request.price),
                new_size: Some(quote_request.size),
            });

            self.active_quotes.insert(quote_id, ActiveQuote {
                id: quote_id,
                side: quote_request.side,
                price: quote_request.price,
                size: quote_request.size,
                created_at: Utc::now(),
                valid_until: quote_request.valid_until,
            });
        }

        self.last_update = Utc::now();
        Ok(updates)
    }

    fn calculate_fair_value(
        &self,
        market_data: &MarketData,
        order_book: &OrderBookSnapshot,
    ) -> TradingResult<Price> {
        // Use multiple signals to determine fair value
        let mid_price = market_data.mid_price;

        // Weight order book mid-price
        let order_book_mid = if !order_book.bids.is_empty() && !order_book.asks.is_empty() {
            let best_bid = Price::from_u64(order_book.bids[0].price)?;
            let best_ask = Price::from_u64(order_book.asks[0].price)?;
            Price::from_ratio(best_bid.as_u64() + best_ask.as_u64(), 2)?
        } else {
            mid_price
        };

        // Simple weighted average (can be enhanced with more sophisticated models)
        let oracle_weight = 0.7;
        let orderbook_weight = 0.3;

        let weighted_price = Price::from_f64(
            mid_price.as_f64() * oracle_weight + order_book_mid.as_f64() * orderbook_weight
        )?;

        Ok(weighted_price)
    }

    fn calculate_spread(
        &self,
        market_data: &MarketData,
        order_book: &OrderBookSnapshot,
    ) -> TradingResult<Price> {
        // Base spread from configuration
        let base_spread = market_data.mid_price
            .checked_mul(&self.config.min_spread_bps.into())?
            .checked_div(&Price::from_u64(10000)?)?;

        // Adjust spread based on market conditions
        let volatility_adjustment = self.calculate_volatility_adjustment(market_data)?;
        let liquidity_adjustment = self.calculate_liquidity_adjustment(order_book)?;

        let adjusted_spread = base_spread
            .checked_add(&volatility_adjustment)?
            .checked_add(&liquidity_adjustment)?;

        // Ensure spread doesn't exceed maximum
        let max_spread = market_data.mid_price
            .checked_mul(&self.config.max_spread_bps.into())?
            .checked_div(&Price::from_u64(10000)?)?;

        Ok(adjusted_spread.min(max_spread))
    }

    fn calculate_inventory_adjustment(&self) -> TradingResult<Price> {
        // Adjust quotes based on current inventory imbalance
        let inventory_imbalance = self.inventory.calculate_imbalance()?;
        let adjustment_factor = inventory_imbalance * self.config.inventory_adjustment_factor;

        Price::from_f64(adjustment_factor)
    }

    fn calculate_quote_size(&self, side: OrderSide, market_data: &MarketData) -> TradingResult<Amount> {
        let base_size = self.config.base_quote_size;

        // Adjust size based on inventory limits
        let inventory_limit = match side {
            OrderSide::Buy => {
                let max_additional = self.config.max_inventory_base.checked_sub(&self.inventory.base_amount)?;
                max_additional.min(base_size)
            }
            OrderSide::Sell => {
                let available_base = self.inventory.base_amount;
                available_base.min(base_size)
            }
        };

        // Adjust size based on market conditions
        let liquidity_factor = self.calculate_liquidity_factor(market_data)?;
        let adjusted_size = inventory_limit.checked_mul(&Amount::from_f64(liquidity_factor)?)?;

        // Ensure minimum size
        if adjusted_size < self.config.min_quote_size {
            Ok(Amount::zero()) // Don't quote if size is too small
        } else {
            Ok(adjusted_size)
        }
    }

    fn should_update_quotes(&self, market_data: &MarketData) -> TradingResult<bool> {
        // Check if enough time has passed
        let time_threshold = Duration::seconds(self.config.update_frequency_seconds as i64);
        if Utc::now().signed_duration_since(self.last_update) < time_threshold {
            return Ok(false);
        }

        // Check if price has moved significantly
        // TODO: Implement price movement threshold check

        Ok(true)
    }

    fn find_expired_quotes(
        &self,
        _market_data: &MarketData,
        _order_book: &OrderBookSnapshot,
    ) -> TradingResult<Vec<u64>> {
        let now = Utc::now();
        let expired_quotes = self.active_quotes
            .iter()
            .filter(|(_, quote)| quote.valid_until < now)
            .map(|(id, _)| *id)
            .collect();

        Ok(expired_quotes)
    }

    fn calculate_volatility_adjustment(&self, _market_data: &MarketData) -> TradingResult<Price> {
        // TODO: Implement volatility-based spread adjustment
        Ok(Price::zero())
    }

    fn calculate_liquidity_adjustment(&self, _order_book: &OrderBookSnapshot) -> TradingResult<Price> {
        // TODO: Implement liquidity-based spread adjustment
        Ok(Price::zero())
    }

    fn calculate_liquidity_factor(&self, _market_data: &MarketData) -> TradingResult<f64> {
        // TODO: Implement liquidity factor calculation
        Ok(1.0)
    }

    fn calculate_risk_exposure(&self) -> RiskExposure {
        RiskExposure {
            base_exposure: self.inventory.base_amount,
            quote_exposure: self.inventory.quote_amount,
            total_value: self.inventory.total_value(),
            inventory_ratio: self.inventory.calculate_imbalance().unwrap_or(0.0),
            last_updated: Utc::now(),
        }
    }

    fn generate_quote_id(&self) -> u64 {
        Utc::now().timestamp_nanos() as u64
    }
}

/// Inventory tracking for market making
#[derive(Debug, Clone)]
pub struct Inventory {
    base_amount: Amount,
    quote_amount: Amount,
    target_ratio: f64, // Target ratio of base to total value
}

impl Inventory {
    fn new() -> Self {
        Self {
            base_amount: Amount::zero(),
            quote_amount: Amount::zero(),
            target_ratio: 0.5, // 50% base, 50% quote
        }
    }

    fn calculate_imbalance(&self) -> TradingResult<f64> {
        let total_value = self.total_value();
        if total_value.is_zero() {
            return Ok(0.0);
        }

        let base_value_ratio = self.base_amount.as_f64() / total_value.as_f64();
        Ok(base_value_ratio - self.target_ratio)
    }

    fn total_value(&self) -> Amount {
        self.base_amount.checked_add(&self.quote_amount).unwrap_or(Amount::zero())
    }
}

/// Performance metrics for market making
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    total_volume: Amount,
    total_trades: u64,
    total_fees_earned: Amount,
    pnl: Amount, // Profit and Loss
    sharpe_ratio: f64,
    max_drawdown: f64,
    uptime_percentage: f64,
    start_time: DateTime<Utc>,
    last_updated: DateTime<Utc>,
}

impl PerformanceMetrics {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            total_volume: Amount::zero(),
            total_trades: 0,
            total_fees_earned: Amount::zero(),
            pnl: Amount::zero(),
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            uptime_percentage: 100.0,
            start_time: now,
            last_updated: now,
        }
    }

    fn record_trade(
        &mut self,
        _side: OrderSide,
        size: Amount,
        _price: Price,
        fees: Amount,
    ) -> TradingResult<()> {
        self.total_volume = self.total_volume.checked_add(&size)?;
        self.total_trades += 1;
        self.total_fees_earned = self.total_fees_earned.checked_add(&fees)?;
        self.last_updated = Utc::now();

        // TODO: Calculate PnL, Sharpe ratio, drawdown
        Ok(())
    }
}

/// Configuration for market making
#[derive(Debug, Clone)]
pub struct MarketMakingConfig {
    pub enabled: bool,
    pub max_active_pairs: u32,
    pub risk_limits: RiskLimits,
    pub default_strategy_config: StrategyConfig,
}

impl Default for MarketMakingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active_pairs: 10,
            risk_limits: RiskLimits::default(),
            default_strategy_config: StrategyConfig::default(),
        }
    }
}

/// Strategy configuration for a trading pair
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub min_spread_bps: u16,
    pub max_spread_bps: u16,
    pub base_quote_size: Amount,
    pub min_quote_size: Amount,
    pub max_inventory_base: Amount,
    pub max_inventory_quote: Amount,
    pub inventory_adjustment_factor: f64,
    pub quote_ttl_seconds: u32,
    pub update_frequency_seconds: u32,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            min_spread_bps: 10,  // 0.1%
            max_spread_bps: 100, // 1%
            base_quote_size: Amount::from_u64(10_000 * Price::PRECISION), // $10K
            min_quote_size: Amount::from_u64(1_000 * Price::PRECISION),   // $1K
            max_inventory_base: Amount::from_u64(100_000 * Price::PRECISION), // $100K
            max_inventory_quote: Amount::from_u64(100_000 * Price::PRECISION), // $100K
            inventory_adjustment_factor: 0.1,
            quote_ttl_seconds: 30,
            update_frequency_seconds: 5,
        }
    }
}

/// Risk limits for market making
#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_total_exposure: Amount,
    pub max_drawdown_pct: f64,
    pub max_daily_loss: Amount,
    pub position_concentration_limit: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_total_exposure: Amount::from_u64(1_000_000 * Price::PRECISION), // $1M
            max_drawdown_pct: 5.0, // 5%
            max_daily_loss: Amount::from_u64(10_000 * Price::PRECISION), // $10K
            position_concentration_limit: 0.2, // 20% max in any single position
        }
    }
}

// Supporting structures

#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub side: OrderSide,
    pub price: Price,
    pub size: Amount,
    pub strategy: QuoteStrategy,
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum QuoteStrategy {
    MarketMaking,
    Arbitrage,
    Inventory,
}

#[derive(Debug, Clone)]
pub struct QuoteUpdate {
    pub quote_id: u64,
    pub action: QuoteAction,
    pub new_price: Option<Price>,
    pub new_size: Option<Amount>,
}

#[derive(Debug, Clone)]
pub enum QuoteAction {
    Create,
    Update,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct ActiveQuote {
    pub id: u64,
    pub side: OrderSide,
    pub price: Price,
    pub size: Amount,
    pub created_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RiskExposure {
    pub base_exposure: Amount,
    pub quote_exposure: Amount,
    pub total_value: Amount,
    pub inventory_ratio: f64,
    pub last_updated: DateTime<Utc>,
}

/// Liquidity provider interface
pub trait LiquidityProvider {
    fn provide_liquidity(
        &mut self,
        base_token: AccountKey,
        quote_token: AccountKey,
        amount: Amount,
    ) -> TradingResult<()>;

    fn withdraw_liquidity(
        &mut self,
        base_token: AccountKey,
        quote_token: AccountKey,
        amount: Amount,
    ) -> TradingResult<()>;

    fn get_liquidity_stats(&self) -> HashMap<(AccountKey, AccountKey), LiquidityStats>;
}

#[derive(Debug, Clone)]
pub struct LiquidityStats {
    pub total_provided: Amount,
    pub current_amount: Amount,
    pub fees_earned: Amount,
    pub utilization_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// Spread manager for dynamic spread adjustment
pub struct SpreadManager {
    volatility_model: VolatilityModel,
    liquidity_model: LiquidityModel,
}

impl SpreadManager {
    pub fn new() -> Self {
        Self {
            volatility_model: VolatilityModel::new(),
            liquidity_model: LiquidityModel::new(),
        }
    }

    pub fn calculate_optimal_spread(
        &self,
        market_data: &MarketData,
        order_book: &OrderBookSnapshot,
        inventory_imbalance: f64,
    ) -> TradingResult<Price> {
        let volatility_spread = self.volatility_model.calculate_spread(market_data)?;
        let liquidity_spread = self.liquidity_model.calculate_spread(order_book)?;
        let inventory_spread = Price::from_f64(inventory_imbalance.abs() * 0.001)?; // 0.1% per 10% imbalance

        // Combine spreads
        volatility_spread
            .checked_add(&liquidity_spread)?
            .checked_add(&inventory_spread)
    }
}

struct VolatilityModel;
impl VolatilityModel {
    fn new() -> Self { Self }
    fn calculate_spread(&self, _market_data: &MarketData) -> TradingResult<Price> {
        // TODO: Implement volatility-based spread calculation
        Price::from_basis_points(20) // 0.2% base spread
    }
}

struct LiquidityModel;
impl LiquidityModel {
    fn new() -> Self { Self }
    fn calculate_spread(&self, _order_book: &OrderBookSnapshot) -> TradingResult<Price> {
        // TODO: Implement liquidity-based spread calculation
        Price::from_basis_points(10) // 0.1% liquidity spread
    }
}

impl Default for SpreadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MarketData;

    fn create_test_market_data() -> MarketData {
        MarketData {
            base_token: AccountKey::new_unique(),
            quote_token: AccountKey::new_unique(),
            bid_price: Price::from_u64(99 * Price::PRECISION).unwrap(),
            ask_price: Price::from_u64(101 * Price::PRECISION).unwrap(),
            mid_price: Price::from_u64(100 * Price::PRECISION).unwrap(),
            liquidity: Amount::from_u64(1_000_000 * Price::PRECISION),
            is_active: true,
            last_updated: Utc::now(),
        }
    }

    fn create_test_order_book() -> OrderBookSnapshot {
        OrderBookSnapshot {
            bids: vec![
                OrderBookLevel { price: 99 * Price::PRECISION, size: 1000, order_count: 1 },
                OrderBookLevel { price: 98 * Price::PRECISION, size: 2000, order_count: 2 },
            ],
            asks: vec![
                OrderBookLevel { price: 101 * Price::PRECISION, size: 1500, order_count: 1 },
                OrderBookLevel { price: 102 * Price::PRECISION, size: 1000, order_count: 1 },
            ],
            last_trade_price: 100 * Price::PRECISION,
            total_volume: 50000,
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_market_maker_creation() {
        let config = MarketMakingConfig::default();
        let market_maker = MarketMaker::new(config);

        assert_eq!(market_maker.active_strategies.len(), 0);
        assert_eq!(market_maker.performance_metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_start_market_making() {
        let config = MarketMakingConfig::default();
        let mut market_maker = MarketMaker::new(config);

        let base_token = AccountKey::new_unique();
        let quote_token = AccountKey::new_unique();
        let strategy_config = StrategyConfig::default();

        let result = market_maker.start_market_making(base_token, quote_token, strategy_config).await;
        assert!(result.is_ok());

        assert_eq!(market_maker.active_strategies.len(), 1);
        assert_eq!(market_maker.performance_metrics.len(), 1);
    }

    #[tokio::test]
    async fn test_generate_quotes() {
        let config = MarketMakingConfig::default();
        let mut market_maker = MarketMaker::new(config);

        let base_token = AccountKey::new_unique();
        let quote_token = AccountKey::new_unique();
        let strategy_config = StrategyConfig::default();

        market_maker.start_market_making(base_token, quote_token, strategy_config).await.unwrap();

        let market_data = create_test_market_data();
        let order_book = create_test_order_book();

        let quotes = market_maker.generate_quotes(base_token, quote_token, &market_data, &order_book).await.unwrap();

        // Should generate both bid and ask quotes
        assert_eq!(quotes.len(), 2);

        // Find bid and ask quotes
        let bid_quote = quotes.iter().find(|q| q.side == OrderSide::Buy).unwrap();
        let ask_quote = quotes.iter().find(|q| q.side == OrderSide::Sell).unwrap();

        // Bid should be below mid price, ask should be above
        assert!(bid_quote.price < market_data.mid_price);
        assert!(ask_quote.price > market_data.mid_price);
    }

    #[test]
    fn test_inventory_imbalance_calculation() {
        let mut inventory = Inventory::new();
        inventory.base_amount = Amount::from_u64(100_000 * Price::PRECISION);  // $100K
        inventory.quote_amount = Amount::from_u64(50_000 * Price::PRECISION);  // $50K

        let imbalance = inventory.calculate_imbalance().unwrap();

        // Should be positive since base > target (67% vs 50%)
        assert!(imbalance > 0.0);
        assert!((imbalance - 0.1667).abs() < 0.001); // Approximately 16.67%
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        let size = Amount::from_u64(10_000 * Price::PRECISION);
        let price = Price::from_u64(100 * Price::PRECISION).unwrap();
        let fees = Amount::from_u64(30 * Price::PRECISION); // $30 fee

        metrics.record_trade(OrderSide::Buy, size, price, fees).unwrap();

        assert_eq!(metrics.total_volume, size);
        assert_eq!(metrics.total_trades, 1);
        assert_eq!(metrics.total_fees_earned, fees);
    }

    #[test]
    fn test_spread_manager() {
        let spread_manager = SpreadManager::new();
        let market_data = create_test_market_data();
        let order_book = create_test_order_book();
        let inventory_imbalance = 0.1; // 10% imbalance

        let spread = spread_manager.calculate_optimal_spread(&market_data, &order_book, inventory_imbalance).unwrap();

        assert!(spread > Price::zero());
        assert!(spread < market_data.mid_price); // Spread should be reasonable
    }
}