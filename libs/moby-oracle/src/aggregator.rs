use crate::{PriceFeed, error::OracleError};
use moby_math::Price;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationMethod {
    WeightedAverage,
    MedianWithTrimming { trim_percent: f64 },
    VolumeWeighted,
    ConfidenceWeighted,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct WeightedPrice {
    pub price: Price,
    pub weight: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct PriceAggregator {
    method: AggregationMethod,
    outlier_threshold: f64,
    min_sources: usize,
}

impl PriceAggregator {
    pub fn new() -> Self {
        Self {
            method: AggregationMethod::Hybrid,
            outlier_threshold: 0.05, // 5% deviation threshold
            min_sources: 3,
        }
    }

    pub fn with_method(method: AggregationMethod) -> Self {
        Self {
            method,
            outlier_threshold: 0.05,
            min_sources: 3,
        }
    }

    pub fn with_outlier_threshold(mut self, threshold: f64) -> Self {
        self.outlier_threshold = threshold;
        self
    }

    pub fn with_min_sources(mut self, min_sources: usize) -> Self {
        self.min_sources = min_sources;
        self
    }

    pub fn aggregate(&self, feeds: &[PriceFeed]) -> Result<Price, OracleError> {
        if feeds.is_empty() {
            return Err(OracleError::NoDataAvailable);
        }

        if feeds.len() < self.min_sources {
            return Err(OracleError::aggregation_failed(&format!(
                "Insufficient sources: {} < {}", feeds.len(), self.min_sources
            )));
        }

        // Remove outliers first
        let filtered_feeds = self.filter_outliers(feeds)?;

        if filtered_feeds.is_empty() {
            return Err(OracleError::aggregation_failed("All feeds filtered as outliers"));
        }

        match &self.method {
            AggregationMethod::WeightedAverage => self.weighted_average(&filtered_feeds),
            AggregationMethod::MedianWithTrimming { trim_percent } => {
                self.median_with_trimming(&filtered_feeds, *trim_percent)
            },
            AggregationMethod::VolumeWeighted => self.volume_weighted(&filtered_feeds),
            AggregationMethod::ConfidenceWeighted => self.confidence_weighted(&filtered_feeds),
            AggregationMethod::Hybrid => self.hybrid_aggregation(&filtered_feeds),
        }
    }

    fn filter_outliers(&self, feeds: &[PriceFeed]) -> Result<Vec<PriceFeed>, OracleError> {
        if feeds.len() < 3 {
            // Can't detect outliers with less than 3 data points
            return Ok(feeds.to_vec());
        }

        // Calculate median price for outlier detection
        let mut prices: Vec<f64> = feeds.iter()
            .map(|f| f.data.price.to_f64().unwrap_or(0.0))
            .collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if prices.len() % 2 == 0 {
            (prices[prices.len() / 2 - 1] + prices[prices.len() / 2]) / 2.0
        } else {
            prices[prices.len() / 2]
        };

        // Filter out prices that deviate more than threshold from median
        let filtered: Vec<PriceFeed> = feeds.iter()
            .filter(|feed| {
                let price = feed.data.price.to_f64().unwrap_or(0.0);
                let deviation = (price - median).abs() / median;
                deviation <= self.outlier_threshold
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    fn weighted_average(&self, feeds: &[PriceFeed]) -> Result<Price, OracleError> {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for feed in feeds {
            let price = feed.data.price.to_f64()
                .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

            weighted_sum += price * feed.weight;
            total_weight += feed.weight;
        }

        if total_weight == 0.0 {
            return Err(OracleError::aggregation_failed("Zero total weight"));
        }

        let avg_price = weighted_sum / total_weight;
        Price::from_float(avg_price, 8)
            .map_err(|e| OracleError::invalid_data(&e.to_string()))
    }

    fn median_with_trimming(&self, feeds: &[PriceFeed], trim_percent: f64) -> Result<Price, OracleError> {
        let mut prices: Vec<f64> = feeds.iter()
            .map(|f| f.data.price.to_f64().unwrap_or(0.0))
            .collect();

        prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Trim extreme values
        let trim_count = ((prices.len() as f64) * trim_percent / 100.0).floor() as usize;
        let start = trim_count;
        let end = prices.len().saturating_sub(trim_count);

        if start >= end {
            return Err(OracleError::aggregation_failed("Over-trimming"));
        }

        let trimmed = &prices[start..end];
        let median = if trimmed.len() % 2 == 0 {
            (trimmed[trimmed.len() / 2 - 1] + trimmed[trimmed.len() / 2]) / 2.0
        } else {
            trimmed[trimmed.len() / 2]
        };

        Price::from_float(median, 8)
            .map_err(|e| OracleError::invalid_data(&e.to_string()))
    }

    fn volume_weighted(&self, feeds: &[PriceFeed]) -> Result<Price, OracleError> {
        let mut weighted_sum = 0.0;
        let mut total_volume = 0u64;

        for feed in feeds {
            let price = feed.data.price.to_f64()
                .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

            weighted_sum += price * (feed.data.volume_24h as f64);
            total_volume += feed.data.volume_24h;
        }

        if total_volume == 0 {
            return Err(OracleError::aggregation_failed("Zero total volume"));
        }

        let vwap = weighted_sum / (total_volume as f64);
        Price::from_float(vwap, 8)
            .map_err(|e| OracleError::invalid_data(&e.to_string()))
    }

    fn confidence_weighted(&self, feeds: &[PriceFeed]) -> Result<Price, OracleError> {
        let mut weighted_sum = 0.0;
        let mut total_confidence = 0.0;

        for feed in feeds {
            let price = feed.data.price.to_f64()
                .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

            let confidence = feed.data.confidence;
            weighted_sum += price * confidence;
            total_confidence += confidence;
        }

        if total_confidence == 0.0 {
            return Err(OracleError::aggregation_failed("Zero total confidence"));
        }

        let avg_price = weighted_sum / total_confidence;
        Price::from_float(avg_price, 8)
            .map_err(|e| OracleError::invalid_data(&e.to_string()))
    }

    fn hybrid_aggregation(&self, feeds: &[PriceFeed]) -> Result<Price, OracleError> {
        // Hybrid approach: combine multiple methods with weights

        // 1. Get confidence-weighted price (40% weight)
        let confidence_price = self.confidence_weighted(feeds)?;

        // 2. Get volume-weighted price (30% weight)
        let volume_price = self.volume_weighted(feeds)?;

        // 3. Get median price (30% weight)
        let median_price = self.median_with_trimming(feeds, 10.0)?;

        // Combine with weights
        let conf_weight = 0.4;
        let vol_weight = 0.3;
        let median_weight = 0.3;

        let conf_val = confidence_price.to_f64()
            .map_err(|e| OracleError::invalid_data(&e.to_string()))?;
        let vol_val = volume_price.to_f64()
            .map_err(|e| OracleError::invalid_data(&e.to_string()))?;
        let median_val = median_price.to_f64()
            .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

        let hybrid_price = (conf_val * conf_weight) +
                          (vol_val * vol_weight) +
                          (median_val * median_weight);

        Price::from_float(hybrid_price, 8)
            .map_err(|e| OracleError::invalid_data(&e.to_string()))
    }

    pub fn get_price_statistics(&self, feeds: &[PriceFeed]) -> Result<PriceStatistics, OracleError> {
        if feeds.is_empty() {
            return Err(OracleError::NoDataAvailable);
        }

        let prices: Vec<f64> = feeds.iter()
            .map(|f| f.data.price.to_f64().unwrap_or(0.0))
            .collect();

        let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_price = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let mean_price = prices.iter().sum::<f64>() / prices.len() as f64;

        // Calculate standard deviation
        let variance = prices.iter()
            .map(|&price| (price - mean_price).powi(2))
            .sum::<f64>() / prices.len() as f64;
        let std_dev = variance.sqrt();

        let price_spread = (max_price - min_price) / mean_price;

        Ok(PriceStatistics {
            min_price: Price::from_float(min_price, 8)?,
            max_price: Price::from_float(max_price, 8)?,
            mean_price: Price::from_float(mean_price, 8)?,
            std_deviation: std_dev,
            price_spread,
            source_count: feeds.len(),
        })
    }
}

impl Default for PriceAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PriceStatistics {
    pub min_price: Price,
    pub max_price: Price,
    pub mean_price: Price,
    pub std_deviation: f64,
    pub price_spread: f64,
    pub source_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PriceData, sources::OracleSource};
    use chrono::Utc;

    fn create_test_feeds() -> Vec<PriceFeed> {
        vec![
            PriceFeed {
                source_id: "chainlink".to_string(),
                symbol: "BTC".to_string(),
                data: PriceData {
                    price: Price::from_float(43_250.0, 8).unwrap(),
                    volume_24h: 100_000_000 * Price::PRECISION,
                    timestamp: Utc::now(),
                    confidence: 0.95,
                },
                weight: 0.3,
            },
            PriceFeed {
                source_id: "pyth".to_string(),
                symbol: "BTC".to_string(),
                data: PriceData {
                    price: Price::from_float(43_275.0, 8).unwrap(),
                    volume_24h: 80_000_000 * Price::PRECISION,
                    timestamp: Utc::now(),
                    confidence: 0.92,
                },
                weight: 0.25,
            },
            PriceFeed {
                source_id: "binance".to_string(),
                symbol: "BTC".to_string(),
                data: PriceData {
                    price: Price::from_float(43_240.0, 8).unwrap(),
                    volume_24h: 200_000_000 * Price::PRECISION,
                    timestamp: Utc::now(),
                    confidence: 0.88,
                },
                weight: 0.2,
            },
        ]
    }

    #[test]
    fn test_weighted_average() {
        let aggregator = PriceAggregator::with_method(AggregationMethod::WeightedAverage);
        let feeds = create_test_feeds();

        let result = aggregator.aggregate(&feeds).unwrap();
        let price_value = result.to_f64().unwrap();

        // Should be close to weighted average
        assert!(price_value > 43_200.0);
        assert!(price_value < 43_300.0);
    }

    #[test]
    fn test_median_aggregation() {
        let aggregator = PriceAggregator::with_method(
            AggregationMethod::MedianWithTrimming { trim_percent: 0.0 }
        );
        let feeds = create_test_feeds();

        let result = aggregator.aggregate(&feeds).unwrap();
        let price_value = result.to_f64().unwrap();

        // Should be close to median (43_250.0)
        assert!((price_value - 43_250.0).abs() < 10.0);
    }

    #[test]
    fn test_volume_weighted() {
        let aggregator = PriceAggregator::with_method(AggregationMethod::VolumeWeighted);
        let feeds = create_test_feeds();

        let result = aggregator.aggregate(&feeds).unwrap();
        let price_value = result.to_f64().unwrap();

        // Should be closer to Binance price due to higher volume
        assert!(price_value > 43_200.0);
        assert!(price_value < 43_280.0);
    }

    #[test]
    fn test_confidence_weighted() {
        let aggregator = PriceAggregator::with_method(AggregationMethod::ConfidenceWeighted);
        let feeds = create_test_feeds();

        let result = aggregator.aggregate(&feeds).unwrap();
        let price_value = result.to_f64().unwrap();

        // Should be closer to Chainlink price due to higher confidence
        assert!(price_value > 43_240.0);
        assert!(price_value < 43_270.0);
    }

    #[test]
    fn test_hybrid_aggregation() {
        let aggregator = PriceAggregator::new(); // Uses hybrid by default
        let feeds = create_test_feeds();

        let result = aggregator.aggregate(&feeds).unwrap();
        assert!(result.to_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_outlier_filtering() {
        let mut feeds = create_test_feeds();

        // Add outlier
        feeds.push(PriceFeed {
            source_id: "outlier".to_string(),
            symbol: "BTC".to_string(),
            data: PriceData {
                price: Price::from_float(50_000.0, 8).unwrap(), // Way off
                volume_24h: 10_000_000 * Price::PRECISION,
                timestamp: Utc::now(),
                confidence: 0.5,
            },
            weight: 0.1,
        });

        let aggregator = PriceAggregator::new().with_outlier_threshold(0.05);
        let filtered = aggregator.filter_outliers(&feeds).unwrap();

        // Outlier should be filtered out
        assert_eq!(filtered.len(), 3);
        assert!(!filtered.iter().any(|f| f.source_id == "outlier"));
    }

    #[test]
    fn test_insufficient_sources() {
        let aggregator = PriceAggregator::new().with_min_sources(5);
        let feeds = create_test_feeds(); // Only 3 feeds

        let result = aggregator.aggregate(&feeds);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_statistics() {
        let aggregator = PriceAggregator::new();
        let feeds = create_test_feeds();

        let stats = aggregator.get_price_statistics(&feeds).unwrap();

        assert_eq!(stats.source_count, 3);
        assert!(stats.price_spread > 0.0);
        assert!(stats.std_deviation > 0.0);

        let min_val = stats.min_price.to_f64().unwrap();
        let max_val = stats.max_price.to_f64().unwrap();
        assert!(min_val < max_val);
    }

    #[test]
    fn test_empty_feeds() {
        let aggregator = PriceAggregator::new();
        let feeds = vec![];

        let result = aggregator.aggregate(&feeds);
        assert!(result.is_err());
    }
}