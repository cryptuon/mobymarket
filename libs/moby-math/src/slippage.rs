use crate::price::{Price, PriceError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageParams {
    pub depth_coefficient: f64,
    pub impact_exponent: f64,
    pub base_slippage: f64,
    pub max_slippage: f64,
}

impl Default for SlippageParams {
    fn default() -> Self {
        Self {
            depth_coefficient: 0.1,
            impact_exponent: 1.5,
            base_slippage: 0.0005, // 0.05%
            max_slippage: 0.1,     // 10%
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageResult {
    pub linear_slippage: f64,
    pub quadratic_slippage: f64,
    pub total_slippage: f64,
    pub effective_price_impact: f64,
    pub estimated_execution_price: Price,
    pub volume_buckets: Vec<VolumeBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeBucket {
    pub percentage: f64,
    pub amount: u64,
    pub slippage: f64,
    pub effective_price: Price,
}

#[derive(Debug, Clone)]
pub struct SlippageCalculator {
    params: SlippageParams,
    market_depth_cache: HashMap<String, MarketDepth>,
}

#[derive(Debug, Clone)]
struct MarketDepth {
    total_liquidity: u64,
    depth_levels: Vec<(u64, f64)>, // (amount, slippage_rate)
    last_updated: std::time::SystemTime,
}

impl SlippageCalculator {
    pub fn new(params: SlippageParams) -> Self {
        Self {
            params,
            market_depth_cache: HashMap::new(),
        }
    }

    pub fn calculate_slippage(
        &self,
        trade_amount: u64,
        market_liquidity: u64,
    ) -> Result<SlippageResult, PriceError> {
        if trade_amount == 0 {
            return Err(PriceError::InvalidPrice("Trade amount cannot be zero".to_string()));
        }

        if market_liquidity == 0 {
            return Err(PriceError::InvalidPrice("Market liquidity cannot be zero".to_string()));
        }

        let impact_ratio = trade_amount as f64 / market_liquidity as f64;

        // Linear component (market maker spread widening)
        let linear_slippage = self.params.base_slippage +
            (self.params.depth_coefficient * impact_ratio);

        // Quadratic component (depth exhaustion)
        let quadratic_slippage = self.params.depth_coefficient *
            impact_ratio.powf(self.params.impact_exponent);

        let total_slippage = (linear_slippage + quadratic_slippage)
            .min(self.params.max_slippage);

        let effective_price_impact = total_slippage / 2.0; // Average impact

        // Create volume buckets for progressive execution
        let volume_buckets = self.create_volume_buckets(
            trade_amount,
            market_liquidity,
            total_slippage,
        )?;

        // Calculate weighted average execution price
        let weighted_slippage = volume_buckets.iter()
            .map(|bucket| bucket.slippage * bucket.percentage / 100.0)
            .sum::<f64>();

        let base_price = Price::from_float(1.0, 6)?; // Normalized price
        let execution_price_value = 1.0 + weighted_slippage;
        let estimated_execution_price = Price::from_float(execution_price_value, 6)?;

        Ok(SlippageResult {
            linear_slippage,
            quadratic_slippage,
            total_slippage,
            effective_price_impact,
            estimated_execution_price,
            volume_buckets,
        })
    }

    pub fn calculate_whale_optimal_chunks(
        &self,
        total_amount: u64,
        market_liquidity: u64,
        max_chunk_impact: f64,
    ) -> Result<Vec<u64>, PriceError> {
        let mut chunks = Vec::new();
        let mut remaining = total_amount;

        while remaining > 0 {
            let max_chunk = (market_liquidity as f64 * max_chunk_impact) as u64;
            let chunk_size = remaining.min(max_chunk);

            chunks.push(chunk_size);
            remaining = remaining.saturating_sub(chunk_size);

            // Avoid infinite loops with very small remaining amounts
            if chunk_size == 0 || chunks.len() > 1000 {
                break;
            }
        }

        Ok(chunks)
    }

    pub fn estimate_twap_execution_cost(
        &self,
        total_amount: u64,
        market_liquidity: u64,
        time_buckets: u32,
    ) -> Result<f64, PriceError> {
        let chunk_size = total_amount / time_buckets as u64;
        let mut total_cost = 0.0;

        for i in 0..time_buckets {
            let current_amount = if i == time_buckets - 1 {
                // Last bucket gets remainder
                total_amount - (chunk_size * i as u64)
            } else {
                chunk_size
            };

            let slippage = self.calculate_slippage(current_amount, market_liquidity)?;
            total_cost += slippage.effective_price_impact * (current_amount as f64);
        }

        Ok(total_cost / total_amount as f64)
    }

    fn create_volume_buckets(
        &self,
        trade_amount: u64,
        market_liquidity: u64,
        max_slippage: f64,
    ) -> Result<Vec<VolumeBucket>, PriceError> {
        let bucket_count = 10;
        let chunk_size = trade_amount / bucket_count;
        let mut buckets = Vec::new();
        let mut cumulative_amount = 0u64;

        for i in 0..bucket_count {
            let current_chunk = if i == bucket_count - 1 {
                trade_amount - cumulative_amount
            } else {
                chunk_size
            };

            cumulative_amount += current_chunk;
            let cumulative_impact = cumulative_amount as f64 / market_liquidity as f64;

            // Progressive slippage based on cumulative impact
            let bucket_slippage = self.params.base_slippage +
                (max_slippage - self.params.base_slippage) *
                (cumulative_impact / (trade_amount as f64 / market_liquidity as f64));

            let percentage = (current_chunk as f64 / trade_amount as f64) * 100.0;
            let effective_price = Price::from_float(1.0 + bucket_slippage, 6)?;

            buckets.push(VolumeBucket {
                percentage,
                amount: current_chunk,
                slippage: bucket_slippage,
                effective_price,
            });
        }

        Ok(buckets)
    }

    pub fn update_market_depth(&mut self, symbol: String, depth: MarketDepth) {
        self.market_depth_cache.insert(symbol, depth);
    }

    pub fn get_whale_tier_slippage(&self, amount_usd: u64) -> f64 {
        match amount_usd / Price::PRECISION {
            0..=999_999 => 0.001,          // <$1M: 0.1%
            1_000_000..=9_999_999 => 0.005, // $1M-$10M: 0.5%
            10_000_000..=49_999_999 => 0.015, // $10M-$50M: 1.5%
            50_000_000..=99_999_999 => 0.025, // $50M-$100M: 2.5%
            _ => 0.05,                     // >$100M: 5%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_slippage_calculation() {
        let calc = SlippageCalculator::new(SlippageParams::default());

        // $10M trade in $100M liquidity pool
        let trade_amount = 10_000_000 * Price::PRECISION;
        let liquidity = 100_000_000 * Price::PRECISION;

        let result = calc.calculate_slippage(trade_amount, liquidity).unwrap();

        assert!(result.total_slippage > 0.0);
        assert!(result.total_slippage < 0.1);
        assert!(result.linear_slippage > 0.0);
        assert!(result.quadratic_slippage > 0.0);
        assert_eq!(result.volume_buckets.len(), 10);
    }

    #[test]
    fn test_extreme_whale_trade() {
        let calc = SlippageCalculator::new(SlippageParams {
            depth_coefficient: 0.2,
            impact_exponent: 1.8,
            base_slippage: 0.001,
            max_slippage: 0.05,
        });

        // $500M trade (sovereign wealth fund size)
        let trade_amount = 500_000_000 * Price::PRECISION;
        let liquidity = 1_000_000_000 * Price::PRECISION; // $1B pool

        let result = calc.calculate_slippage(trade_amount, liquidity).unwrap();

        // Should hit max slippage cap
        assert_eq!(result.total_slippage, 0.05);

        // Progressive buckets should show increasing slippage
        for i in 1..result.volume_buckets.len() {
            assert!(result.volume_buckets[i].slippage >= result.volume_buckets[i-1].slippage);
        }
    }

    #[test]
    fn test_optimal_chunking() {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let total_amount = 100_000_000 * Price::PRECISION; // $100M
        let liquidity = 500_000_000 * Price::PRECISION;    // $500M
        let max_impact = 0.02; // 2% max impact per chunk

        let chunks = calc.calculate_whale_optimal_chunks(
            total_amount,
            liquidity,
            max_impact
        ).unwrap();

        assert!(!chunks.is_empty());

        // Each chunk should respect max impact
        for chunk in &chunks {
            let impact = *chunk as f64 / liquidity as f64;
            assert!(impact <= max_impact + f64::EPSILON);
        }

        // Total should equal original amount
        let total_chunked: u64 = chunks.iter().sum();
        assert_eq!(total_chunked, total_amount);
    }

    #[test]
    fn test_twap_cost_estimation() {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let total_amount = 50_000_000 * Price::PRECISION; // $50M
        let liquidity = 200_000_000 * Price::PRECISION;   // $200M
        let time_buckets = 24; // 24-hour TWAP

        let avg_cost = calc.estimate_twap_execution_cost(
            total_amount,
            liquidity,
            time_buckets
        ).unwrap();

        assert!(avg_cost > 0.0);
        assert!(avg_cost < 0.1); // Should be reasonable for TWAP execution
    }

    #[test]
    fn test_whale_tier_classification() {
        let calc = SlippageCalculator::new(SlippageParams::default());

        assert_eq!(calc.get_whale_tier_slippage(500_000 * Price::PRECISION), 0.001);
        assert_eq!(calc.get_whale_tier_slippage(5_000_000 * Price::PRECISION), 0.005);
        assert_eq!(calc.get_whale_tier_slippage(25_000_000 * Price::PRECISION), 0.015);
        assert_eq!(calc.get_whale_tier_slippage(75_000_000 * Price::PRECISION), 0.025);
        assert_eq!(calc.get_whale_tier_slippage(150_000_000 * Price::PRECISION), 0.05);
    }

    #[test]
    fn test_zero_amount_protection() {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let result = calc.calculate_slippage(0, 1000);
        assert!(matches!(result, Err(PriceError::InvalidPrice(_))));
    }

    #[test]
    fn test_zero_liquidity_protection() {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let result = calc.calculate_slippage(1000, 0);
        assert!(matches!(result, Err(PriceError::InvalidPrice(_))));
    }
}