// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Core trading engine implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    TradingConfig, TradingError, TradingResult,
    ExecutionEngine, ExecutionPlan, ExecutionResult,
    OrderManager, OrderRequest, OrderUpdate,
    RiskManager, RiskAssessment,
    FeeCalculator,
    TradingAnalytics,
};

use moby_types::{WhaleOrder, OrderStatus, TradingTier, AccountKey};
use moby_oracle::{PriceOracle, OraclePrice};
use moby_math::{Price, Amount};

/// Core trading engine that coordinates all trading operations
#[derive(Debug)]
pub struct TradingEngine {
    /// Configuration
    config: TradingConfig,

    /// Order management
    order_manager: Arc<RwLock<OrderManager>>,

    /// Execution engine
    execution_engine: Arc<ExecutionEngine>,

    /// Risk management
    risk_manager: Arc<RiskManager>,

    /// Fee calculator
    fee_calculator: Arc<FeeCalculator>,

    /// Price oracle
    price_oracle: Arc<dyn PriceOracle + Send + Sync>,

    /// Trading analytics
    analytics: Arc<RwLock<TradingAnalytics>>,

    /// Active traders
    active_traders: Arc<RwLock<HashMap<AccountKey, TraderSession>>>,
}

impl TradingEngine {
    /// Create a new trading engine
    pub fn new() -> Self {
        Self::with_config(TradingConfig::default())
    }

    /// Create a new trading engine with custom configuration
    pub fn with_config(config: TradingConfig) -> Self {
        Self {
            order_manager: Arc::new(RwLock::new(OrderManager::new())),
            execution_engine: Arc::new(ExecutionEngine::new(config.clone())),
            risk_manager: Arc::new(RiskManager::new(config.risk_config.clone())),
            fee_calculator: Arc::new(FeeCalculator::new(config.fee_config.clone())),
            price_oracle: Arc::new(MockPriceOracle::new()), // Replace with real oracle
            analytics: Arc::new(RwLock::new(TradingAnalytics::new())),
            active_traders: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Set price oracle
    pub fn set_price_oracle(&mut self, oracle: Arc<dyn PriceOracle + Send + Sync>) {
        self.price_oracle = oracle;
    }

    /// Plan execution for an order
    pub async fn plan_execution(&self, order_request: &OrderRequest) -> TradingResult<ExecutionPlan> {
        // Validate order
        self.validate_order(order_request).await?;

        // Get current market data
        let market_data = self.get_market_data(&order_request.base_token, &order_request.quote_token).await?;

        // Assess risk
        let risk_assessment = self.risk_manager.assess_order_risk(order_request, &market_data).await?;

        // Calculate fees
        let fee_estimate = self.fee_calculator.calculate_fees(order_request, &market_data)?;

        // Create execution plan
        let execution_plan = self.execution_engine.create_execution_plan(
            order_request,
            &market_data,
            &risk_assessment,
            &fee_estimate,
        ).await?;

        Ok(execution_plan)
    }

    /// Execute an order
    pub async fn execute_order(
        &self,
        order_request: OrderRequest,
        execution_plan: ExecutionPlan,
    ) -> TradingResult<ExecutionResult> {
        // Create order
        let order_id = self.create_order(order_request.clone()).await?;

        // Start execution
        let execution_result = self.execution_engine.execute_order(
            order_id,
            order_request,
            execution_plan,
        ).await;

        // Update analytics
        let mut analytics = self.analytics.write().await;
        analytics.record_execution(&order_request, &execution_result);

        execution_result
    }

    /// Create a new order
    pub async fn create_order(&self, order_request: OrderRequest) -> TradingResult<u64> {
        // Validate order
        self.validate_order(&order_request).await?;

        // Check trader session
        self.ensure_trader_session(&order_request.trader).await?;

        // Create order
        let mut order_manager = self.order_manager.write().await;
        let order_id = order_manager.create_order(order_request).await?;

        Ok(order_id)
    }

    /// Update an existing order
    pub async fn update_order(&self, order_id: u64, update: OrderUpdate) -> TradingResult<()> {
        let mut order_manager = self.order_manager.write().await;
        order_manager.update_order(order_id, update).await
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: u64, trader: AccountKey) -> TradingResult<()> {
        // Verify ownership
        let order = self.get_order(order_id).await?;
        if order.trader != trader {
            return Err(TradingError::invalid_order("Order does not belong to trader"));
        }

        // Cancel order
        let mut order_manager = self.order_manager.write().await;
        order_manager.cancel_order(order_id).await
    }

    /// Get order details
    pub async fn get_order(&self, order_id: u64) -> TradingResult<WhaleOrder> {
        let order_manager = self.order_manager.read().await;
        order_manager.get_order(order_id).await
    }

    /// Get orders for a trader
    pub async fn get_trader_orders(&self, trader: AccountKey) -> TradingResult<Vec<WhaleOrder>> {
        let order_manager = self.order_manager.read().await;
        order_manager.get_trader_orders(trader).await
    }

    /// Get market quote
    pub async fn get_quote(
        &self,
        base_token: AccountKey,
        quote_token: AccountKey,
        amount: Amount,
        side: OrderSide,
    ) -> TradingResult<Quote> {
        let market_data = self.get_market_data(&base_token, &quote_token).await?;

        let price = match side {
            OrderSide::Buy => market_data.ask_price,
            OrderSide::Sell => market_data.bid_price,
        };

        let total_amount = amount.checked_mul(&price.into())
            .ok_or_else(|| TradingError::internal("Quote calculation overflow"))?;

        let fees = self.fee_calculator.calculate_quote_fees(amount, &market_data)?;

        Ok(Quote {
            price,
            amount,
            total_amount,
            fees,
            valid_until: chrono::Utc::now() + chrono::Duration::seconds(30),
        })
    }

    /// Get trading statistics
    pub async fn get_trading_stats(&self, trader: Option<AccountKey>) -> TradingResult<TradingStats> {
        let analytics = self.analytics.read().await;
        analytics.get_trading_stats(trader).await
    }

    // Private helper methods

    async fn validate_order(&self, order_request: &OrderRequest) -> TradingResult<()> {
        // Basic validation
        if order_request.size.is_zero() {
            return Err(TradingError::invalid_order("Order size must be greater than zero"));
        }

        // Risk validation
        self.risk_manager.validate_order(order_request).await?;

        // Market validation
        let market_data = self.get_market_data(&order_request.base_token, &order_request.quote_token).await?;
        if !market_data.is_active {
            return Err(TradingError::MarketClosed);
        }

        Ok(())
    }

    async fn ensure_trader_session(&self, trader: &AccountKey) -> TradingResult<()> {
        let mut active_traders = self.active_traders.write().await;

        if !active_traders.contains_key(trader) {
            let session = TraderSession::new(*trader);
            active_traders.insert(*trader, session);
        }

        Ok(())
    }

    async fn get_market_data(
        &self,
        base_token: &AccountKey,
        quote_token: &AccountKey,
    ) -> TradingResult<MarketData> {
        let base_price = self.price_oracle.get_price(base_token).await
            .map_err(|e| TradingError::PriceFeedUnavailable {
                token: base_token.to_string()
            })?;

        let quote_price = self.price_oracle.get_price(quote_token).await
            .map_err(|e| TradingError::PriceFeedUnavailable {
                token: quote_token.to_string()
            })?;

        // Calculate relative price
        let relative_price = Price::from_ratio(base_price.price, quote_price.price)
            .map_err(|e| TradingError::Math(e))?;

        Ok(MarketData {
            base_token: *base_token,
            quote_token: *quote_token,
            bid_price: relative_price * Price::from_percentage(99.95)?, // 0.05% spread
            ask_price: relative_price * Price::from_percentage(100.05)?, // 0.05% spread
            mid_price: relative_price,
            liquidity: Amount::from_u64(1_000_000 * Price::PRECISION), // Mock liquidity
            is_active: true,
            last_updated: chrono::Utc::now(),
        })
    }
}

/// Trader session information
#[derive(Debug, Clone)]
pub struct TraderSession {
    pub trader: AccountKey,
    pub tier: TradingTier,
    pub session_start: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub daily_volume: Amount,
    pub open_orders: u32,
}

impl TraderSession {
    pub fn new(trader: AccountKey) -> Self {
        let now = chrono::Utc::now();
        Self {
            trader,
            tier: TradingTier::Retail, // Default tier
            session_start: now,
            last_activity: now,
            daily_volume: Amount::zero(),
            open_orders: 0,
        }
    }
}

/// Market data structure
#[derive(Debug, Clone)]
pub struct MarketData {
    pub base_token: AccountKey,
    pub quote_token: AccountKey,
    pub bid_price: Price,
    pub ask_price: Price,
    pub mid_price: Price,
    pub liquidity: Amount,
    pub is_active: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Order side enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Price quote structure
#[derive(Debug, Clone)]
pub struct Quote {
    pub price: Price,
    pub amount: Amount,
    pub total_amount: Amount,
    pub fees: Amount,
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

/// Trading statistics
#[derive(Debug, Clone)]
pub struct TradingStats {
    pub total_volume: Amount,
    pub total_trades: u64,
    pub average_trade_size: Amount,
    pub total_fees_paid: Amount,
    pub success_rate: f64,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}

/// Mock price oracle for testing
struct MockPriceOracle;

impl MockPriceOracle {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PriceOracle for MockPriceOracle {
    async fn get_price(&self, token: &AccountKey) -> Result<OraclePrice, moby_oracle::OracleError> {
        // Mock prices for testing
        let price = Price::from_u64(100 * Price::PRECISION)?; // $100
        Ok(OraclePrice {
            price,
            confidence: 95,
            timestamp: chrono::Utc::now().timestamp(),
            source: "mock".to_string(),
        })
    }

    async fn get_prices(&self, tokens: &[AccountKey]) -> Result<Vec<OraclePrice>, moby_oracle::OracleError> {
        let mut prices = Vec::new();
        for token in tokens {
            prices.push(self.get_price(token).await?);
        }
        Ok(prices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moby_types::{TradingTier, ExecutionStrategy};

    #[tokio::test]
    async fn test_trading_engine_creation() {
        let engine = TradingEngine::new();
        assert_eq!(engine.config.max_slippage_bps, 100);
    }

    #[tokio::test]
    async fn test_order_validation() {
        let engine = TradingEngine::new();

        let order_request = OrderRequest {
            trader: AccountKey::new_unique(),
            base_token: AccountKey::new_unique(),
            quote_token: AccountKey::new_unique(),
            size: Amount::zero(), // Invalid: zero size
            order_type: moby_types::OrderType::Market,
            execution_strategy: ExecutionStrategy::Market,
            tier: TradingTier::Retail,
            privacy_enabled: false,
        };

        let result = engine.validate_order(&order_request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TradingError::InvalidOrder { .. }));
    }

    #[tokio::test]
    async fn test_trader_session() {
        let trader = AccountKey::new_unique();
        let session = TraderSession::new(trader);

        assert_eq!(session.trader, trader);
        assert_eq!(session.tier, TradingTier::Retail);
        assert_eq!(session.open_orders, 0);
        assert_eq!(session.daily_volume, Amount::zero());
    }

    #[tokio::test]
    async fn test_quote_generation() {
        let engine = TradingEngine::new();

        let base_token = AccountKey::new_unique();
        let quote_token = AccountKey::new_unique();
        let amount = Amount::from_u64(1000 * Price::PRECISION);

        let quote = engine.get_quote(base_token, quote_token, amount, OrderSide::Buy).await;
        assert!(quote.is_ok());

        let quote = quote.unwrap();
        assert_eq!(quote.amount, amount);
        assert!(quote.price > Price::zero());
        assert!(quote.total_amount > Amount::zero());
    }
}