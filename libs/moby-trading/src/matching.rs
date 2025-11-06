// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Order matching engine for whale trading

use std::collections::{BTreeMap, VecDeque};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{TradingError, TradingResult};
use moby_types::{WhaleOrder, OrderSide, OrderStatus, OrderType, AccountKey, LiquidityType};
use moby_math::{Amount, Price};

/// Order matching engine
#[derive(Debug)]
pub struct MatchingEngine {
    /// Buy orders (price descending)
    buy_orders: BTreeMap<PriceLevel, VecDeque<WhaleOrder>>,

    /// Sell orders (price ascending)
    sell_orders: BTreeMap<PriceLevel, VecDeque<WhaleOrder>>,

    /// Order lookup by ID
    order_lookup: std::collections::HashMap<u64, (OrderSide, PriceLevel)>,

    /// Last trade price
    last_trade_price: Option<Price>,

    /// Total volume traded
    total_volume: Amount,

    /// Match history (last 1000 matches)
    match_history: VecDeque<TradeMatch>,
}

impl MatchingEngine {
    /// Create a new matching engine
    pub fn new() -> Self {
        Self {
            buy_orders: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
            order_lookup: std::collections::HashMap::new(),
            last_trade_price: None,
            total_volume: Amount::zero(),
            match_history: VecDeque::new(),
        }
    }

    /// Add an order to the order book
    pub fn add_order(&mut self, order: WhaleOrder) -> TradingResult<Vec<TradeMatch>> {
        // Validate order
        if !matches!(order.status, OrderStatus::Pending) {
            return Err(TradingError::invalid_order("Only pending orders can be added to order book"));
        }

        if order.remaining_size == 0 {
            return Err(TradingError::invalid_order("Order has no remaining size"));
        }

        let mut matches = Vec::new();
        let mut order = order;

        // Try to match the order immediately
        if order.order_type == OrderType::Market || self.can_match_immediately(&order) {
            matches = self.match_order(&mut order)?;
        }

        // If order has remaining size, add to order book
        if order.remaining_size > 0 && matches!(order.order_type, OrderType::Limit | OrderType::Stop | OrderType::StopLimit) {
            self.add_to_order_book(order)?;
        }

        Ok(matches)
    }

    /// Remove an order from the order book
    pub fn remove_order(&mut self, order_id: u64) -> TradingResult<()> {
        if let Some((side, price_level)) = self.order_lookup.remove(&order_id) {
            let order_queue = match side {
                OrderSide::Buy => self.buy_orders.get_mut(&price_level),
                OrderSide::Sell => self.sell_orders.get_mut(&price_level),
            };

            if let Some(queue) = order_queue {
                queue.retain(|order| order.order_id != order_id);

                // Remove empty price level
                if queue.is_empty() {
                    match side {
                        OrderSide::Buy => { self.buy_orders.remove(&price_level); }
                        OrderSide::Sell => { self.sell_orders.remove(&price_level); }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get current order book snapshot
    pub fn get_order_book(&self, depth: usize) -> OrderBookSnapshot {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        // Collect bids (highest price first)
        for (price_level, orders) in self.buy_orders.iter().rev().take(depth) {
            let total_size: u64 = orders.iter().map(|o| o.remaining_size).sum();
            if total_size > 0 {
                bids.push(OrderBookLevel {
                    price: price_level.price,
                    size: total_size,
                    order_count: orders.len() as u32,
                });
            }
        }

        // Collect asks (lowest price first)
        for (price_level, orders) in self.sell_orders.iter().take(depth) {
            let total_size: u64 = orders.iter().map(|o| o.remaining_size).sum();
            if total_size > 0 {
                asks.push(OrderBookLevel {
                    price: price_level.price,
                    size: total_size,
                    order_count: orders.len() as u32,
                });
            }
        }

        OrderBookSnapshot {
            bids,
            asks,
            last_trade_price: self.last_trade_price.map(|p| p.as_u64()).unwrap_or(0),
            total_volume: self.total_volume.as_u64(),
            timestamp: Utc::now(),
        }
    }

    /// Get market statistics
    pub fn get_market_stats(&self) -> MarketStats {
        let best_bid = self.buy_orders.keys().next_back().map(|pl| pl.price);
        let best_ask = self.sell_orders.keys().next().map(|pl| pl.price);

        let spread = if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            ask.checked_sub(&bid).unwrap_or(Price::zero())
        } else {
            Price::zero()
        };

        let mid_price = if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            Price::from_ratio(bid.as_u64() + ask.as_u64(), 2).unwrap_or(Price::zero())
        } else {
            self.last_trade_price.unwrap_or(Price::zero())
        };

        MarketStats {
            best_bid: best_bid.map(|p| p.as_u64()),
            best_ask: best_ask.map(|p| p.as_u64()),
            mid_price: mid_price.as_u64(),
            spread: spread.as_u64(),
            last_trade_price: self.last_trade_price.map(|p| p.as_u64()),
            total_volume: self.total_volume.as_u64(),
            total_orders: self.order_lookup.len() as u32,
            last_updated: Utc::now(),
        }
    }

    /// Get recent trade history
    pub fn get_trade_history(&self, limit: usize) -> Vec<TradeMatch> {
        self.match_history
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    // Private helper methods

    fn can_match_immediately(&self, order: &WhaleOrder) -> bool {
        match order.side {
            OrderSide::Buy => {
                // Buy order can match if there are sells at or below our price
                self.sell_orders.keys().next()
                    .map(|best_ask| order.price >= best_ask.price)
                    .unwrap_or(false)
            }
            OrderSide::Sell => {
                // Sell order can match if there are buys at or above our price
                self.buy_orders.keys().next_back()
                    .map(|best_bid| order.price <= best_bid.price)
                    .unwrap_or(false)
            }
        }
    }

    fn match_order(&mut self, order: &mut WhaleOrder) -> TradingResult<Vec<TradeMatch>> {
        let mut matches = Vec::new();

        match order.side {
            OrderSide::Buy => {
                // Match against sell orders (lowest price first)
                while order.remaining_size > 0 {
                    let best_ask_price = match self.sell_orders.keys().next() {
                        Some(price_level) => *price_level,
                        None => break, // No more sell orders
                    };

                    // Check if we can match at this price
                    if order.order_type == OrderType::Limit && order.price < best_ask_price.price {
                        break; // Price doesn't match for limit order
                    }

                    let trade_match = self.execute_match(order, &best_ask_price, OrderSide::Sell)?;
                    matches.push(trade_match);
                }
            }
            OrderSide::Sell => {
                // Match against buy orders (highest price first)
                while order.remaining_size > 0 {
                    let best_bid_price = match self.buy_orders.keys().next_back() {
                        Some(price_level) => *price_level,
                        None => break, // No more buy orders
                    };

                    // Check if we can match at this price
                    if order.order_type == OrderType::Limit && order.price > best_bid_price.price {
                        break; // Price doesn't match for limit order
                    }

                    let trade_match = self.execute_match(order, &best_bid_price, OrderSide::Buy)?;
                    matches.push(trade_match);
                }
            }
        }

        Ok(matches)
    }

    fn execute_match(
        &mut self,
        taker_order: &mut WhaleOrder,
        maker_price_level: &PriceLevel,
        maker_side: OrderSide,
    ) -> TradingResult<TradeMatch> {
        let maker_orders = match maker_side {
            OrderSide::Buy => self.buy_orders.get_mut(maker_price_level),
            OrderSide::Sell => self.sell_orders.get_mut(maker_price_level),
        }.ok_or_else(|| TradingError::internal("Price level not found"))?;

        let mut maker_order = maker_orders.pop_front()
            .ok_or_else(|| TradingError::internal("No orders at price level"))?;

        // Calculate trade size (minimum of both remaining sizes)
        let trade_size = taker_order.remaining_size.min(maker_order.remaining_size);
        let trade_price = maker_price_level.price; // Price improvement goes to taker

        // Update order sizes
        taker_order.remaining_size -= trade_size;
        taker_order.filled_size += trade_size;
        maker_order.remaining_size -= trade_size;
        maker_order.filled_size += trade_size;

        // Update order statuses
        if taker_order.remaining_size == 0 {
            taker_order.status = OrderStatus::Filled;
        } else {
            taker_order.status = OrderStatus::PartiallyFilled;
        }

        if maker_order.remaining_size == 0 {
            maker_order.status = OrderStatus::Filled;
            // Remove from lookup since it's fully filled
            self.order_lookup.remove(&maker_order.order_id);
        } else {
            maker_order.status = OrderStatus::PartiallyFilled;
            // Put back in queue if partially filled
            maker_orders.push_front(maker_order.clone());
        }

        // Clean up empty price level
        if maker_orders.is_empty() {
            match maker_side {
                OrderSide::Buy => { self.buy_orders.remove(maker_price_level); }
                OrderSide::Sell => { self.sell_orders.remove(maker_price_level); }
            }
        }

        // Update market data
        self.last_trade_price = Some(trade_price);
        self.total_volume = self.total_volume.checked_add(&Amount::from_u64(trade_size))
            .ok_or_else(|| TradingError::internal("Volume overflow"))?;

        // Create trade match record
        let trade_match = TradeMatch {
            match_id: self.generate_match_id(),
            taker_order_id: taker_order.order_id,
            maker_order_id: maker_order.order_id,
            price: trade_price.as_u64(),
            size: trade_size,
            taker_side: taker_order.side,
            timestamp: Utc::now(),
            taker_liquidity_type: LiquidityType::Taker,
            maker_liquidity_type: LiquidityType::Maker,
        };

        // Add to history
        self.match_history.push_back(trade_match.clone());
        if self.match_history.len() > 1000 {
            self.match_history.pop_front();
        }

        Ok(trade_match)
    }

    fn add_to_order_book(&mut self, order: WhaleOrder) -> TradingResult<()> {
        let price_level = PriceLevel {
            price: Price::from_u64(order.price)?,
            timestamp: Utc::now(),
        };

        // Add to lookup
        self.order_lookup.insert(order.order_id, (order.side, price_level));

        // Add to appropriate side of order book
        match order.side {
            OrderSide::Buy => {
                self.buy_orders
                    .entry(price_level)
                    .or_insert_with(VecDeque::new)
                    .push_back(order);
            }
            OrderSide::Sell => {
                self.sell_orders
                    .entry(price_level)
                    .or_insert_with(VecDeque::new)
                    .push_back(order);
            }
        }

        Ok(())
    }

    fn generate_match_id(&self) -> u64 {
        Utc::now().timestamp_nanos() as u64
    }
}

/// Price level for order book organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PriceLevel {
    price: Price,
    timestamp: DateTime<Utc>,
}

/// Order book level for external API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: u64,
    pub size: u64,
    pub order_count: u32,
}

/// Order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub last_trade_price: u64,
    pub total_volume: u64,
    pub timestamp: DateTime<Utc>,
}

/// Market statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
    pub mid_price: u64,
    pub spread: u64,
    pub last_trade_price: Option<u64>,
    pub total_volume: u64,
    pub total_orders: u32,
    pub last_updated: DateTime<Utc>,
}

/// Trade match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeMatch {
    pub match_id: u64,
    pub taker_order_id: u64,
    pub maker_order_id: u64,
    pub price: u64,
    pub size: u64,
    pub taker_side: OrderSide,
    pub timestamp: DateTime<Utc>,
    pub taker_liquidity_type: LiquidityType,
    pub maker_liquidity_type: LiquidityType,
}

/// Order matcher interface for external systems
pub trait OrderMatcher {
    fn match_orders(&mut self, orders: Vec<WhaleOrder>) -> TradingResult<Vec<TradeMatch>>;
    fn get_best_prices(&self) -> (Option<Price>, Option<Price>); // (bid, ask)
    fn get_market_depth(&self, depth: usize) -> OrderBookSnapshot;
}

impl OrderMatcher for MatchingEngine {
    fn match_orders(&mut self, orders: Vec<WhaleOrder>) -> TradingResult<Vec<TradeMatch>> {
        let mut all_matches = Vec::new();

        for order in orders {
            let matches = self.add_order(order)?;
            all_matches.extend(matches);
        }

        Ok(all_matches)
    }

    fn get_best_prices(&self) -> (Option<Price>, Option<Price>) {
        let best_bid = self.buy_orders.keys().next_back().map(|pl| pl.price);
        let best_ask = self.sell_orders.keys().next().map(|pl| pl.price);
        (best_bid, best_ask)
    }

    fn get_market_depth(&self, depth: usize) -> OrderBookSnapshot {
        self.get_order_book(depth)
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moby_types::{OrderStatus, OrderType, TimeInForce, TradingTier, ExecutionStrategy};

    fn create_test_order(
        order_id: u64,
        side: OrderSide,
        size: u64,
        price: u64,
    ) -> WhaleOrder {
        WhaleOrder {
            order_id,
            trader: AccountKey::new_unique(),
            base_token: AccountKey::new_unique(),
            quote_token: AccountKey::new_unique(),
            order_type: OrderType::Limit,
            side,
            size,
            price,
            filled_size: 0,
            remaining_size: size,
            status: OrderStatus::Pending,
            execution_strategy: ExecutionStrategy::Limit,
            slippage_tolerance: 100,
            time_in_force: TimeInForce::Gtc,
            trader_tier: TradingTier::Retail,
            privacy_enabled: false,
            otc_enabled: false,
            cross_chain_enabled: false,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            expires_at: None,
            fee_tier: 0,
            estimated_fees: 0,
            actual_fees: 0,
            execution_metadata: String::new(),
            settlement_status: moby_types::SettlementStatus::Pending,
            fills: Vec::new(),
            _reserved: [0; 128],
        }
    }

    #[test]
    fn test_add_order_to_empty_book() {
        let mut engine = MatchingEngine::new();

        let order = create_test_order(1, OrderSide::Buy, 1000, 100);
        let matches = engine.add_order(order).unwrap();

        assert!(matches.is_empty()); // No matches in empty book
        assert_eq!(engine.buy_orders.len(), 1);
    }

    #[test]
    fn test_simple_match() {
        let mut engine = MatchingEngine::new();

        // Add buy order first
        let buy_order = create_test_order(1, OrderSide::Buy, 1000, 100);
        let matches1 = engine.add_order(buy_order).unwrap();
        assert!(matches1.is_empty());

        // Add matching sell order
        let sell_order = create_test_order(2, OrderSide::Sell, 500, 100);
        let matches2 = engine.add_order(sell_order).unwrap();

        assert_eq!(matches2.len(), 1);
        let trade_match = &matches2[0];
        assert_eq!(trade_match.size, 500);
        assert_eq!(trade_match.price, 100);
        assert_eq!(trade_match.taker_order_id, 2);
        assert_eq!(trade_match.maker_order_id, 1);
    }

    #[test]
    fn test_partial_fill() {
        let mut engine = MatchingEngine::new();

        // Large buy order
        let buy_order = create_test_order(1, OrderSide::Buy, 1000, 100);
        engine.add_order(buy_order).unwrap();

        // Smaller sell order
        let sell_order = create_test_order(2, OrderSide::Sell, 300, 100);
        let matches = engine.add_order(sell_order).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].size, 300);

        // Buy order should still be in the book with reduced size
        let order_book = engine.get_order_book(10);
        assert_eq!(order_book.bids.len(), 1);
        assert_eq!(order_book.bids[0].size, 700); // 1000 - 300
    }

    #[test]
    fn test_price_priority() {
        let mut engine = MatchingEngine::new();

        // Add multiple buy orders at different prices
        engine.add_order(create_test_order(1, OrderSide::Buy, 100, 98)).unwrap();
        engine.add_order(create_test_order(2, OrderSide::Buy, 100, 100)).unwrap();
        engine.add_order(create_test_order(3, OrderSide::Buy, 100, 99)).unwrap();

        // Add sell order that should match with highest bid
        let sell_order = create_test_order(4, OrderSide::Sell, 50, 98);
        let matches = engine.add_order(sell_order).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].maker_order_id, 2); // Order with price 100
        assert_eq!(matches[0].price, 100); // Maker price
    }

    #[test]
    fn test_time_priority() {
        let mut engine = MatchingEngine::new();

        // Add multiple buy orders at same price
        engine.add_order(create_test_order(1, OrderSide::Buy, 100, 100)).unwrap();
        engine.add_order(create_test_order(2, OrderSide::Buy, 100, 100)).unwrap();

        // Add sell order
        let sell_order = create_test_order(3, OrderSide::Sell, 50, 100);
        let matches = engine.add_order(sell_order).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].maker_order_id, 1); // First order at this price
    }

    #[test]
    fn test_market_order_matching() {
        let mut engine = MatchingEngine::new();

        // Add limit orders
        engine.add_order(create_test_order(1, OrderSide::Sell, 100, 102)).unwrap();
        engine.add_order(create_test_order(2, OrderSide::Sell, 100, 101)).unwrap();

        // Add market buy order
        let mut market_order = create_test_order(3, OrderSide::Buy, 150, 0);
        market_order.order_type = OrderType::Market;

        let matches = engine.add_order(market_order).unwrap();

        // Should match with both sell orders, starting with best price
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].maker_order_id, 2); // Better price (101)
        assert_eq!(matches[0].size, 100);
        assert_eq!(matches[1].maker_order_id, 1); // Next best price (102)
        assert_eq!(matches[1].size, 50);
    }

    #[test]
    fn test_order_book_snapshot() {
        let mut engine = MatchingEngine::new();

        // Add some orders
        engine.add_order(create_test_order(1, OrderSide::Buy, 100, 99)).unwrap();
        engine.add_order(create_test_order(2, OrderSide::Buy, 200, 98)).unwrap();
        engine.add_order(create_test_order(3, OrderSide::Sell, 150, 101)).unwrap();
        engine.add_order(create_test_order(4, OrderSide::Sell, 100, 102)).unwrap();

        let snapshot = engine.get_order_book(10);

        // Check bids (should be sorted by price descending)
        assert_eq!(snapshot.bids.len(), 2);
        assert_eq!(snapshot.bids[0].price, 99); // Higher price first
        assert_eq!(snapshot.bids[0].size, 100);
        assert_eq!(snapshot.bids[1].price, 98);
        assert_eq!(snapshot.bids[1].size, 200);

        // Check asks (should be sorted by price ascending)
        assert_eq!(snapshot.asks.len(), 2);
        assert_eq!(snapshot.asks[0].price, 101); // Lower price first
        assert_eq!(snapshot.asks[0].size, 150);
        assert_eq!(snapshot.asks[1].price, 102);
        assert_eq!(snapshot.asks[1].size, 100);
    }

    #[test]
    fn test_remove_order() {
        let mut engine = MatchingEngine::new();

        let order = create_test_order(1, OrderSide::Buy, 1000, 100);
        engine.add_order(order).unwrap();

        assert_eq!(engine.buy_orders.len(), 1);
        assert_eq!(engine.order_lookup.len(), 1);

        engine.remove_order(1).unwrap();

        assert_eq!(engine.buy_orders.len(), 0);
        assert_eq!(engine.order_lookup.len(), 0);
    }
}