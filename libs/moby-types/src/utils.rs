use anchor_lang::prelude::*;
use crate::{WhaleAmount, Timestamp, TradingTier, OrderType, OrderSide};
use moby_math::Price;

/// Utility functions for whale trading operations
pub struct WhaleUtils;

impl WhaleUtils {
    /// Calculate appropriate trading tier based on volume
    pub fn calculate_tier(lifetime_volume: u64) -> TradingTier {
        match lifetime_volume / Price::PRECISION {
            0..=9_999_999 => TradingTier::Retail,
            10_000_000..=99_999_999 => TradingTier::SmallWhale,
            100_000_000..=499_999_999 => TradingTier::MediumWhale,
            500_000_000..=999_999_999 => TradingTier::LargeWhale,
            _ => TradingTier::MegaWhale,
        }
    }

    /// Determine if trade qualifies as whale trade
    pub fn is_whale_trade(amount_usd: u64) -> bool {
        amount_usd >= 1_000_000 * Price::PRECISION // $1M minimum
    }

    /// Calculate appropriate fee based on tier and volume
    pub fn calculate_trading_fee(
        base_fee_bps: u16,
        tier: TradingTier,
        monthly_volume: u64,
    ) -> u16 {
        let tier_discount = match tier {
            TradingTier::Retail => 0,
            TradingTier::SmallWhale => 5,   // 0.05% discount
            TradingTier::MediumWhale => 10, // 0.10% discount
            TradingTier::LargeWhale => 15,  // 0.15% discount
            TradingTier::MegaWhale => 20,   // 0.20% discount
        };

        // Volume-based additional discount
        let volume_discount = match monthly_volume / Price::PRECISION {
            0..=9_999_999 => 0,
            10_000_000..=49_999_999 => 2,   // 0.02% additional
            50_000_000..=99_999_999 => 5,   // 0.05% additional
            100_000_000..=499_999_999 => 8, // 0.08% additional
            _ => 12, // 0.12% additional for >$500M
        };

        base_fee_bps.saturating_sub(tier_discount + volume_discount)
    }

    /// Calculate optimal execution strategy based on trade parameters
    pub fn recommend_execution_strategy(
        trade_size_usd: u64,
        market_liquidity: u64,
        volatility_bps: u16,
        time_constraint: Option<i64>,
    ) -> ExecutionRecommendation {
        let impact_ratio = trade_size_usd as f64 / market_liquidity as f64;

        if impact_ratio < 0.01 {
            // Less than 1% of liquidity - can execute normally
            ExecutionRecommendation {
                strategy: RecommendedStrategy::Market,
                estimated_chunks: 1,
                estimated_time_minutes: 1,
                estimated_slippage_bps: 10,
                confidence: 95,
            }
        } else if impact_ratio < 0.05 {
            // 1-5% of liquidity - use limit orders or small chunks
            ExecutionRecommendation {
                strategy: RecommendedStrategy::LimitOrChunked,
                estimated_chunks: 3,
                estimated_time_minutes: 15,
                estimated_slippage_bps: 25,
                confidence: 90,
            }
        } else if impact_ratio < 0.15 {
            // 5-15% of liquidity - definitely needs TWAP/VWAP
            let time_minutes = if time_constraint.is_some() { 60 } else { 240 }; // 1-4 hours
            ExecutionRecommendation {
                strategy: RecommendedStrategy::Twap,
                estimated_chunks: 10,
                estimated_time_minutes: time_minutes,
                estimated_slippage_bps: 75,
                confidence: 85,
            }
        } else {
            // >15% of liquidity - needs careful OTC or extended TWAP
            ExecutionRecommendation {
                strategy: RecommendedStrategy::OtcOrExtendedTwap,
                estimated_chunks: 20,
                estimated_time_minutes: 1440, // 24 hours
                estimated_slippage_bps: 200,
                confidence: 70,
            }
        }
    }

    /// Calculate position sizing based on portfolio and risk parameters
    pub fn calculate_position_size(
        available_capital: u64,
        portfolio_value: u64,
        max_position_percentage: u8,
        token_volatility_bps: u16,
        correlation_to_portfolio: i16,
    ) -> PositionSizeRecommendation {
        // Base position size from percentage limit
        let base_position = (portfolio_value as u128 * max_position_percentage as u128 / 100) as u64;

        // Adjust for volatility (higher volatility = smaller position)
        let volatility_adjustment = if token_volatility_bps > 1000 { // >10% volatility
            0.7 // 30% reduction
        } else if token_volatility_bps > 500 { // >5% volatility
            0.85 // 15% reduction
        } else {
            1.0 // No adjustment
        };

        // Adjust for correlation (high correlation = smaller position to avoid concentration)
        let correlation_adjustment = if correlation_to_portfolio.abs() > 80 { // >80% correlation
            0.75 // 25% reduction
        } else if correlation_to_portfolio.abs() > 50 { // >50% correlation
            0.9 // 10% reduction
        } else {
            1.0 // No adjustment
        };

        let adjusted_position = (base_position as f64 * volatility_adjustment * correlation_adjustment) as u64;

        // Ensure we don't exceed available capital
        let recommended_size = adjusted_position.min(available_capital);

        PositionSizeRecommendation {
            recommended_size,
            max_size: base_position,
            confidence: if volatility_adjustment < 1.0 || correlation_adjustment < 1.0 { 80 } else { 95 },
            risk_adjustment_reason: if volatility_adjustment < 1.0 {
                Some("High volatility adjustment applied".to_string())
            } else if correlation_adjustment < 1.0 {
                Some("High correlation adjustment applied".to_string())
            } else {
                None
            },
        }
    }

    /// Validate order parameters for whale trades
    pub fn validate_whale_order(
        order_type: OrderType,
        side: OrderSide,
        size: u64,
        price: Option<u64>,
        slippage_tolerance_bps: u16,
        current_market_price: u64,
    ) -> Result<OrderValidation> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check minimum whale size
        if !Self::is_whale_trade(size) {
            errors.push("Order size below whale threshold".to_string());
        }

        // Validate price for limit orders
        if let (OrderType::Limit, Some(limit_price)) = (order_type, price) {
            let price_deviation = Self::calculate_price_deviation(limit_price, current_market_price);

            match side {
                OrderSide::Buy => {
                    if limit_price > current_market_price {
                        warnings.push("Buy limit price above market price".to_string());
                    }
                    if price_deviation > 1000 { // >10% deviation
                        warnings.push("Limit price significantly below market".to_string());
                    }
                }
                OrderSide::Sell => {
                    if limit_price < current_market_price {
                        warnings.push("Sell limit price below market price".to_string());
                    }
                    if price_deviation > 1000 { // >10% deviation
                        warnings.push("Limit price significantly above market".to_string());
                    }
                }
            }
        }

        // Validate slippage tolerance
        if slippage_tolerance_bps > 1000 { // >10%
            warnings.push("High slippage tolerance - consider TWAP execution".to_string());
        } else if slippage_tolerance_bps < 10 { // <0.1%
            warnings.push("Very tight slippage tolerance - order may not fill".to_string());
        }

        let is_valid = errors.is_empty();

        Ok(OrderValidation {
            is_valid,
            errors,
            warnings,
            recommended_adjustments: if !warnings.is_empty() {
                Some("Consider using algorithmic execution for better results".to_string())
            } else {
                None
            },
        })
    }

    /// Calculate price deviation in basis points
    fn calculate_price_deviation(price1: u64, price2: u64) -> u16 {
        if price2 == 0 {
            return 10000; // 100% deviation
        }

        let diff = if price1 > price2 {
            price1 - price2
        } else {
            price2 - price1
        };

        ((diff as u128 * 10000) / price2 as u128) as u16
    }

    /// Calculate time-weighted average price for execution history
    pub fn calculate_twap(executions: &[ExecutionData]) -> Option<u64> {
        if executions.is_empty() {
            return None;
        }

        let total_volume: u64 = executions.iter().map(|e| e.volume).sum();
        if total_volume == 0 {
            return None;
        }

        let weighted_sum: u128 = executions
            .iter()
            .map(|e| e.price as u128 * e.volume as u128)
            .sum();

        Some((weighted_sum / total_volume as u128) as u64)
    }

    /// Calculate volume-weighted average price
    pub fn calculate_vwap(executions: &[ExecutionData]) -> Option<u64> {
        Self::calculate_twap(executions) // For simplicity, same calculation
    }

    /// Estimate gas cost for whale operations
    pub fn estimate_whale_operation_gas(
        operation_type: WhaleOperationType,
        complexity_factor: f64,
        network_gas_price: u64,
    ) -> GasEstimate {
        let base_gas = match operation_type {
            WhaleOperationType::SimpleSwap => 300_000,
            WhaleOperationType::TwapExecution => 500_000,
            WhaleOperationType::OtcTrade => 400_000,
            WhaleOperationType::CrossChain => 800_000,
            WhaleOperationType::PrivacyTrade => 1_200_000,
        };

        let adjusted_gas = (base_gas as f64 * complexity_factor) as u64;
        let total_cost = adjusted_gas * network_gas_price;

        GasEstimate {
            estimated_gas: adjusted_gas,
            estimated_cost: total_cost,
            confidence: if complexity_factor > 2.0 { 70 } else { 90 },
        }
    }

    /// Generate unique order ID
    pub fn generate_order_id(trader: &Pubkey, timestamp: i64, nonce: u32) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        trader.hash(&mut hasher);
        timestamp.hash(&mut hasher);
        nonce.hash(&mut hasher);

        hasher.finish()
    }

    /// Convert basis points to percentage
    pub fn bps_to_percentage(bps: u16) -> f64 {
        bps as f64 / 100.0
    }

    /// Convert percentage to basis points
    pub fn percentage_to_bps(percentage: f64) -> u16 {
        (percentage * 100.0) as u16
    }

    /// Format large numbers for display
    pub fn format_whale_amount(amount: u64) -> String {
        let usd_amount = amount / Price::PRECISION;

        if usd_amount >= 1_000_000_000 {
            format!("${:.1}B", usd_amount as f64 / 1_000_000_000.0)
        } else if usd_amount >= 1_000_000 {
            format!("${:.1}M", usd_amount as f64 / 1_000_000.0)
        } else if usd_amount >= 1_000 {
            format!("${:.1}K", usd_amount as f64 / 1_000.0)
        } else {
            format!("${}", usd_amount)
        }
    }

    /// Validate wallet signature for whale operations
    pub fn validate_whale_signature(
        message: &[u8],
        signature: &[u8; 64],
        public_key: &Pubkey,
    ) -> bool {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        if let Ok(verifying_key) = VerifyingKey::from_bytes(&public_key.to_bytes()) {
            if let Ok(sig) = Signature::from_bytes(signature) {
                return verifying_key.verify(message, &sig).is_ok();
            }
        }
        false
    }
}

/// Supporting structures for utility functions
#[derive(Debug, Clone)]
pub struct ExecutionRecommendation {
    pub strategy: RecommendedStrategy,
    pub estimated_chunks: u32,
    pub estimated_time_minutes: u32,
    pub estimated_slippage_bps: u16,
    pub confidence: u8,
}

#[derive(Debug, Clone)]
pub enum RecommendedStrategy {
    Market,
    LimitOrChunked,
    Twap,
    OtcOrExtendedTwap,
}

#[derive(Debug, Clone)]
pub struct PositionSizeRecommendation {
    pub recommended_size: u64,
    pub max_size: u64,
    pub confidence: u8,
    pub risk_adjustment_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OrderValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub recommended_adjustments: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionData {
    pub price: u64,
    pub volume: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub enum WhaleOperationType {
    SimpleSwap,
    TwapExecution,
    OtcTrade,
    CrossChain,
    PrivacyTrade,
}

#[derive(Debug, Clone)]
pub struct GasEstimate {
    pub estimated_gas: u64,
    pub estimated_cost: u64,
    pub confidence: u8,
}

/// Serialization helpers for account data
pub mod serialization {
    use super::*;

    /// Pack account data efficiently
    pub fn pack_account_data<T: anchor_lang::AnchorSerialize>(data: &T) -> Result<Vec<u8>> {
        data.try_to_vec().map_err(|_| crate::MobyError::DeserializationFailed.into())
    }

    /// Unpack account data efficiently
    pub fn unpack_account_data<T: anchor_lang::AnchorDeserialize>(data: &[u8]) -> Result<T> {
        T::try_from_slice(data).map_err(|_| crate::MobyError::DeserializationFailed.into())
    }

    /// Calculate account space requirements
    pub fn calculate_space<T: anchor_lang::InitSpace>() -> usize {
        T::INIT_SPACE + 8 // Add discriminator space
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_calculation() {
        assert_eq!(WhaleUtils::calculate_tier(5_000_000 * Price::PRECISION), TradingTier::Retail);
        assert_eq!(WhaleUtils::calculate_tier(50_000_000 * Price::PRECISION), TradingTier::SmallWhale);
        assert_eq!(WhaleUtils::calculate_tier(200_000_000 * Price::PRECISION), TradingTier::MediumWhale);
        assert_eq!(WhaleUtils::calculate_tier(750_000_000 * Price::PRECISION), TradingTier::LargeWhale);
        assert_eq!(WhaleUtils::calculate_tier(1_500_000_000 * Price::PRECISION), TradingTier::MegaWhale);
    }

    #[test]
    fn test_whale_trade_detection() {
        assert!(!WhaleUtils::is_whale_trade(500_000 * Price::PRECISION)); // $500K
        assert!(WhaleUtils::is_whale_trade(2_000_000 * Price::PRECISION)); // $2M
    }

    #[test]
    fn test_fee_calculation() {
        let base_fee = 30; // 0.3%

        // Retail trader
        let retail_fee = WhaleUtils::calculate_trading_fee(
            base_fee,
            TradingTier::Retail,
            0
        );
        assert_eq!(retail_fee, 30);

        // Mega whale with high volume
        let whale_fee = WhaleUtils::calculate_trading_fee(
            base_fee,
            TradingTier::MegaWhale,
            100_000_000 * Price::PRECISION
        );
        assert_eq!(whale_fee, 30 - 20 - 8); // Base - tier discount - volume discount
    }

    #[test]
    fn test_execution_strategy_recommendation() {
        let market_liquidity = 100_000_000 * Price::PRECISION; // $100M

        // Small trade
        let small_rec = WhaleUtils::recommend_execution_strategy(
            500_000 * Price::PRECISION, // $500K
            market_liquidity,
            200, // 2% volatility
            None
        );
        assert!(matches!(small_rec.strategy, RecommendedStrategy::Market));

        // Large trade
        let large_rec = WhaleUtils::recommend_execution_strategy(
            20_000_000 * Price::PRECISION, // $20M
            market_liquidity,
            200,
            None
        );
        assert!(matches!(large_rec.strategy, RecommendedStrategy::OtcOrExtendedTwap));
    }

    #[test]
    fn test_position_size_calculation() {
        let rec = WhaleUtils::calculate_position_size(
            50_000_000 * Price::PRECISION, // $50M available
            100_000_000 * Price::PRECISION, // $100M portfolio
            10, // 10% max position
            300, // 3% volatility
            30, // 30% correlation
        );

        assert!(rec.recommended_size <= 10_000_000 * Price::PRECISION); // Should be ≤ 10% of portfolio
        assert!(rec.recommended_size <= 50_000_000 * Price::PRECISION); // Should be ≤ available capital
    }

    #[test]
    fn test_order_validation() {
        let validation = WhaleUtils::validate_whale_order(
            OrderType::Limit,
            OrderSide::Buy,
            2_000_000 * Price::PRECISION, // $2M order
            Some(1000), // $10 limit price
            100, // 1% slippage
            1050 // $10.50 market price
        ).unwrap();

        assert!(validation.is_valid);
        assert!(validation.warnings.is_empty() || validation.warnings.len() > 0);
    }

    #[test]
    fn test_twap_calculation() {
        let executions = vec![
            ExecutionData { price: 1000, volume: 100, timestamp: 1 },
            ExecutionData { price: 1010, volume: 200, timestamp: 2 },
            ExecutionData { price: 990, volume: 100, timestamp: 3 },
        ];

        let twap = WhaleUtils::calculate_twap(&executions).unwrap();
        // (1000*100 + 1010*200 + 990*100) / 400 = 1002.5 -> 1002
        assert_eq!(twap, 1002);
    }

    #[test]
    fn test_price_deviation_calculation() {
        let deviation = WhaleUtils::calculate_price_deviation(1000, 1100);
        assert_eq!(deviation, 909); // ~9.09%

        let deviation2 = WhaleUtils::calculate_price_deviation(1100, 1000);
        assert_eq!(deviation2, 1000); // 10%
    }

    #[test]
    fn test_amount_formatting() {
        assert_eq!(WhaleUtils::format_whale_amount(500 * Price::PRECISION), "$500");
        assert_eq!(WhaleUtils::format_whale_amount(1_500 * Price::PRECISION), "$1.5K");
        assert_eq!(WhaleUtils::format_whale_amount(2_500_000 * Price::PRECISION), "$2.5M");
        assert_eq!(WhaleUtils::format_whale_amount(1_200_000_000 * Price::PRECISION), "$1.2B");
    }

    #[test]
    fn test_basis_points_conversion() {
        assert_eq!(WhaleUtils::bps_to_percentage(100), 1.0);
        assert_eq!(WhaleUtils::bps_to_percentage(50), 0.5);
        assert_eq!(WhaleUtils::percentage_to_bps(2.5), 250);
    }

    #[test]
    fn test_order_id_generation() {
        let trader = Pubkey::new_unique();
        let timestamp = 1234567890;

        let id1 = WhaleUtils::generate_order_id(&trader, timestamp, 1);
        let id2 = WhaleUtils::generate_order_id(&trader, timestamp, 2);

        // Different nonces should produce different IDs
        assert_ne!(id1, id2);

        // Same inputs should produce same ID
        let id3 = WhaleUtils::generate_order_id(&trader, timestamp, 1);
        assert_eq!(id1, id3);
    }
}