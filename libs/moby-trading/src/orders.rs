// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Order management system

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{TradingError, TradingResult};
use moby_types::{
    WhaleOrder, OrderType, OrderStatus, ExecutionStrategy, TradingTier, AccountKey,
    WhaleAmount, Timestamp,
};
use moby_math::{Amount, Price};

/// Order management system
#[derive(Debug)]
pub struct OrderManager {
    /// Active orders by ID
    orders: HashMap<u64, WhaleOrder>,

    /// Orders by trader
    trader_orders: HashMap<AccountKey, Vec<u64>>,

    /// Next order ID
    next_order_id: u64,

    /// Order history (last 1000 orders)
    order_history: Vec<OrderHistoryEntry>,
}

impl OrderManager {
    /// Create a new order manager
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            trader_orders: HashMap::new(),
            next_order_id: 1,
            order_history: Vec::new(),
        }
    }

    /// Create a new order
    pub async fn create_order(&mut self, request: OrderRequest) -> TradingResult<u64> {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let now = Utc::now().timestamp();

        let order = WhaleOrder {
            order_id,
            trader: request.trader,
            base_token: request.base_token,
            quote_token: request.quote_token,
            order_type: request.order_type,
            side: request.side,
            size: request.size.into(),
            price: request.price.unwrap_or(Price::zero()).into(),
            filled_size: 0,
            remaining_size: request.size.into(),
            status: OrderStatus::Pending,
            execution_strategy: request.execution_strategy,
            slippage_tolerance: request.slippage_tolerance,
            time_in_force: request.time_in_force,
            trader_tier: request.tier,
            privacy_enabled: request.privacy_enabled,
            otc_enabled: request.otc_enabled,
            cross_chain_enabled: request.cross_chain_enabled,
            created_at: now,
            updated_at: now,
            expires_at: request.expires_at,
            fee_tier: 0, // Will be calculated
            estimated_fees: 0,
            actual_fees: 0,
            execution_metadata: String::new(),
            settlement_status: moby_types::SettlementStatus::Pending,
            fills: Vec::new(),
            _reserved: [0; 128],
        };

        // Store order
        self.orders.insert(order_id, order);

        // Track by trader
        self.trader_orders
            .entry(request.trader)
            .or_insert_with(Vec::new)
            .push(order_id);

        // Add to history
        self.add_to_history(order_id, OrderHistoryAction::Created);

        Ok(order_id)
    }

    /// Update an order
    pub async fn update_order(&mut self, order_id: u64, update: OrderUpdate) -> TradingResult<()> {
        let order = self.orders.get_mut(&order_id)
            .ok_or_else(|| TradingError::OrderNotFound { order_id })?;

        // Check if order can be modified
        if !self.can_modify_order(order) {
            return Err(TradingError::OrderNotModifiable {
                current_state: format!("{:?}", order.status),
            });
        }

        // Apply updates
        if let Some(new_size) = update.size {
            order.size = new_size.into();
            order.remaining_size = order.size.saturating_sub(order.filled_size);
        }

        if let Some(new_price) = update.price {
            order.price = new_price.into();
        }

        if let Some(new_slippage) = update.slippage_tolerance {
            order.slippage_tolerance = new_slippage;
        }

        if let Some(new_expires_at) = update.expires_at {
            order.expires_at = new_expires_at;
        }

        order.updated_at = Utc::now().timestamp();

        // Add to history
        self.add_to_history(order_id, OrderHistoryAction::Updated);

        Ok(())
    }

    /// Cancel an order
    pub async fn cancel_order(&mut self, order_id: u64) -> TradingResult<()> {
        let order = self.orders.get_mut(&order_id)
            .ok_or_else(|| TradingError::OrderNotFound { order_id })?;

        // Check if order can be cancelled
        if !self.can_cancel_order(order) {
            return Err(TradingError::OrderNotModifiable {
                current_state: format!("{:?}", order.status),
            });
        }

        order.status = OrderStatus::Cancelled;
        order.updated_at = Utc::now().timestamp();

        // Add to history
        self.add_to_history(order_id, OrderHistoryAction::Cancelled);

        Ok(())
    }

    /// Get order by ID
    pub async fn get_order(&self, order_id: u64) -> TradingResult<WhaleOrder> {
        self.orders.get(&order_id)
            .cloned()
            .ok_or_else(|| TradingError::OrderNotFound { order_id })
    }

    /// Get orders for a trader
    pub async fn get_trader_orders(&self, trader: AccountKey) -> TradingResult<Vec<WhaleOrder>> {
        let order_ids = self.trader_orders.get(&trader).unwrap_or(&Vec::new());
        let mut orders = Vec::new();

        for &order_id in order_ids {
            if let Some(order) = self.orders.get(&order_id) {
                orders.push(order.clone());
            }
        }

        Ok(orders)
    }

    /// Get active orders for a trader
    pub async fn get_active_orders(&self, trader: AccountKey) -> TradingResult<Vec<WhaleOrder>> {
        let all_orders = self.get_trader_orders(trader).await?;
        Ok(all_orders.into_iter()
            .filter(|order| matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled))
            .collect())
    }

    /// Fill an order (partially or completely)
    pub async fn fill_order(
        &mut self,
        order_id: u64,
        fill_size: u64,
        fill_price: u64,
        fill_timestamp: i64,
    ) -> TradingResult<()> {
        let order = self.orders.get_mut(&order_id)
            .ok_or_else(|| TradingError::OrderNotFound { order_id })?;

        // Validate fill
        if order.filled_size + fill_size > order.size {
            return Err(TradingError::execution_failed("Fill size exceeds remaining order size"));
        }

        // Update order
        order.filled_size += fill_size;
        order.remaining_size = order.size - order.filled_size;
        order.updated_at = Utc::now().timestamp();

        // Create fill record
        let fill = moby_types::OrderFill {
            fill_id: self.generate_fill_id(),
            order_id,
            size: fill_size,
            price: fill_price,
            timestamp: fill_timestamp,
            fee_paid: 0, // Will be calculated separately
            liquidity_type: moby_types::LiquidityType::Taker, // Default
        };

        order.fills.push(fill);

        // Update order status
        if order.remaining_size == 0 {
            order.status = OrderStatus::Filled;
            self.add_to_history(order_id, OrderHistoryAction::Filled);
        } else {
            order.status = OrderStatus::PartiallyFilled;
            self.add_to_history(order_id, OrderHistoryAction::PartiallyFilled);
        }

        Ok(())
    }

    /// Get order book for a trading pair
    pub async fn get_order_book(
        &self,
        base_token: AccountKey,
        quote_token: AccountKey,
    ) -> TradingResult<OrderBook> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for order in self.orders.values() {
            if order.base_token != base_token || order.quote_token != quote_token {
                continue;
            }

            if !matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
                continue;
            }

            if order.order_type != OrderType::Limit {
                continue;
            }

            let level = OrderBookLevel {
                price: order.price,
                size: order.remaining_size,
                order_count: 1,
            };

            match order.side {
                moby_types::OrderSide::Buy => bids.push(level),
                moby_types::OrderSide::Sell => asks.push(level),
            }
        }

        // Sort bids (highest price first) and asks (lowest price first)
        bids.sort_by(|a, b| b.price.cmp(&a.price));
        asks.sort_by(|a, b| a.price.cmp(&b.price));

        Ok(OrderBook {
            base_token,
            quote_token,
            bids,
            asks,
            last_updated: Utc::now(),
        })
    }

    /// Get order history
    pub fn get_order_history(&self, limit: Option<usize>) -> Vec<OrderHistoryEntry> {
        let limit = limit.unwrap_or(100).min(self.order_history.len());
        self.order_history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    // Private helper methods

    fn can_modify_order(&self, order: &WhaleOrder) -> bool {
        matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled)
    }

    fn can_cancel_order(&self, order: &WhaleOrder) -> bool {
        !matches!(order.status, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Expired)
    }

    fn add_to_history(&mut self, order_id: u64, action: OrderHistoryAction) {
        let entry = OrderHistoryEntry {
            order_id,
            action,
            timestamp: Utc::now(),
        };

        self.order_history.push(entry);

        // Keep only last 1000 entries
        if self.order_history.len() > 1000 {
            self.order_history.remove(0);
        }
    }

    fn generate_fill_id(&self) -> u64 {
        // Simple implementation - in production, use proper ID generation
        Utc::now().timestamp_nanos() as u64
    }
}

/// Order request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub trader: AccountKey,
    pub base_token: AccountKey,
    pub quote_token: AccountKey,
    pub order_type: OrderType,
    pub side: moby_types::OrderSide,
    pub size: Amount,
    pub price: Option<Price>, // None for market orders
    pub execution_strategy: ExecutionStrategy,
    pub slippage_tolerance: u16,
    pub time_in_force: moby_types::TimeInForce,
    pub tier: TradingTier,
    pub privacy_enabled: bool,
    pub otc_enabled: bool,
    pub cross_chain_enabled: bool,
    pub expires_at: Option<i64>,
}

impl OrderRequest {
    /// Create a new market order
    pub fn market_order(
        trader: AccountKey,
        base_token: AccountKey,
        quote_token: AccountKey,
        side: moby_types::OrderSide,
        size: Amount,
        tier: TradingTier,
    ) -> Self {
        Self {
            trader,
            base_token,
            quote_token,
            order_type: OrderType::Market,
            side,
            size,
            price: None,
            execution_strategy: ExecutionStrategy::Market,
            slippage_tolerance: 100, // 1%
            time_in_force: moby_types::TimeInForce::Ioc,
            tier,
            privacy_enabled: false,
            otc_enabled: false,
            cross_chain_enabled: false,
            expires_at: None,
        }
    }

    /// Create a new limit order
    pub fn limit_order(
        trader: AccountKey,
        base_token: AccountKey,
        quote_token: AccountKey,
        side: moby_types::OrderSide,
        size: Amount,
        price: Price,
        tier: TradingTier,
    ) -> Self {
        Self {
            trader,
            base_token,
            quote_token,
            order_type: OrderType::Limit,
            side,
            size,
            price: Some(price),
            execution_strategy: ExecutionStrategy::Limit,
            slippage_tolerance: 0, // No slippage for limit orders
            time_in_force: moby_types::TimeInForce::Gtc,
            tier,
            privacy_enabled: false,
            otc_enabled: false,
            cross_chain_enabled: false,
            expires_at: None,
        }
    }

    /// Create a TWAP order
    pub fn twap_order(
        trader: AccountKey,
        base_token: AccountKey,
        quote_token: AccountKey,
        side: moby_types::OrderSide,
        size: Amount,
        duration_minutes: u32,
        tier: TradingTier,
    ) -> Self {
        Self {
            trader,
            base_token,
            quote_token,
            order_type: OrderType::Twap,
            side,
            size,
            price: None,
            execution_strategy: ExecutionStrategy::Twap { duration_minutes },
            slippage_tolerance: 50, // 0.5%
            time_in_force: moby_types::TimeInForce::Gtc,
            tier,
            privacy_enabled: false,
            otc_enabled: false,
            cross_chain_enabled: false,
            expires_at: Some(Utc::now().timestamp() + (duration_minutes as i64 * 60)),
        }
    }
}

/// Order update structure
#[derive(Debug, Clone)]
pub struct OrderUpdate {
    pub size: Option<Amount>,
    pub price: Option<Price>,
    pub slippage_tolerance: Option<u16>,
    pub expires_at: Option<Option<i64>>,
}

/// Order book structure
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub base_token: AccountKey,
    pub quote_token: AccountKey,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub last_updated: DateTime<Utc>,
}

/// Order book level
#[derive(Debug, Clone)]
pub struct OrderBookLevel {
    pub price: u64,
    pub size: u64,
    pub order_count: u32,
}

/// Order history entry
#[derive(Debug, Clone)]
pub struct OrderHistoryEntry {
    pub order_id: u64,
    pub action: OrderHistoryAction,
    pub timestamp: DateTime<Utc>,
}

/// Order history actions
#[derive(Debug, Clone)]
pub enum OrderHistoryAction {
    Created,
    Updated,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
}

impl Default for OrderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moby_types::OrderSide;

    #[tokio::test]
    async fn test_create_order() {
        let mut manager = OrderManager::new();

        let request = OrderRequest::market_order(
            AccountKey::new_unique(),
            AccountKey::new_unique(),
            AccountKey::new_unique(),
            OrderSide::Buy,
            Amount::from_u64(1000),
            TradingTier::Retail,
        );

        let order_id = manager.create_order(request).await.unwrap();
        assert_eq!(order_id, 1);

        let order = manager.get_order(order_id).await.unwrap();
        assert_eq!(order.order_id, order_id);
        assert_eq!(order.status, OrderStatus::Pending);
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let mut manager = OrderManager::new();

        let request = OrderRequest::limit_order(
            AccountKey::new_unique(),
            AccountKey::new_unique(),
            AccountKey::new_unique(),
            OrderSide::Buy,
            Amount::from_u64(1000),
            Price::from_u64(100),
            TradingTier::Retail,
        );

        let order_id = manager.create_order(request).await.unwrap();
        manager.cancel_order(order_id).await.unwrap();

        let order = manager.get_order(order_id).await.unwrap();
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_fill_order() {
        let mut manager = OrderManager::new();

        let request = OrderRequest::market_order(
            AccountKey::new_unique(),
            AccountKey::new_unique(),
            AccountKey::new_unique(),
            OrderSide::Buy,
            Amount::from_u64(1000),
            TradingTier::Retail,
        );

        let order_id = manager.create_order(request).await.unwrap();

        // Partial fill
        manager.fill_order(order_id, 300, 100, Utc::now().timestamp()).await.unwrap();

        let order = manager.get_order(order_id).await.unwrap();
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.filled_size, 300);
        assert_eq!(order.remaining_size, 700);

        // Complete fill
        manager.fill_order(order_id, 700, 100, Utc::now().timestamp()).await.unwrap();

        let order = manager.get_order(order_id).await.unwrap();
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.filled_size, 1000);
        assert_eq!(order.remaining_size, 0);
    }

    #[tokio::test]
    async fn test_order_book() {
        let mut manager = OrderManager::new();
        let base_token = AccountKey::new_unique();
        let quote_token = AccountKey::new_unique();

        // Add buy order
        let buy_request = OrderRequest::limit_order(
            AccountKey::new_unique(),
            base_token,
            quote_token,
            OrderSide::Buy,
            Amount::from_u64(1000),
            Price::from_u64(99),
            TradingTier::Retail,
        );
        manager.create_order(buy_request).await.unwrap();

        // Add sell order
        let sell_request = OrderRequest::limit_order(
            AccountKey::new_unique(),
            base_token,
            quote_token,
            OrderSide::Sell,
            Amount::from_u64(500),
            Price::from_u64(101),
            TradingTier::Retail,
        );
        manager.create_order(sell_request).await.unwrap();

        let order_book = manager.get_order_book(base_token, quote_token).await.unwrap();
        assert_eq!(order_book.bids.len(), 1);
        assert_eq!(order_book.asks.len(), 1);
        assert_eq!(order_book.bids[0].price, 99);
        assert_eq!(order_book.asks[0].price, 101);
    }
}