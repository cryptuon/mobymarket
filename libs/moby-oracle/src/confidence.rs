use crate::{PriceFeed, error::OracleError};
use moby_math::Price;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceMetrics {
    pub source_agreement: f64,
    pub data_freshness: f64,
    pub volume_consistency: f64,
    pub source_reliability: f64,
    pub price_stability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub overall_score: f64,
    pub metrics: ConfidenceMetrics,
    pub individual_scores: HashMap<String, f64>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_confidence(confidence: f64) -> Self {
        match confidence {
            c if c >= 0.9 => RiskLevel::Low,
            c if c >= 0.7 => RiskLevel::Medium,
            c if c >= 0.5 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    pub fn whale_trading_threshold(&self) -> f64 {
        match self {
            RiskLevel::Low => 0.0,      // No restrictions
            RiskLevel::Medium => 0.02,   // 2% slippage buffer
            RiskLevel::High => 0.05,     // 5% slippage buffer
            RiskLevel::Critical => 1.0,  // Block whale trades
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfidenceCalculator {
    source_weights: HashMap<String, f64>,
    historical_performance: HashMap<String, f64>,
}

impl ConfidenceCalculator {
    pub fn new() -> Self {
        let mut source_weights = HashMap::new();
        source_weights.insert("chainlink".to_string(), 1.0);
        source_weights.insert("pyth".to_string(), 0.95);
        source_weights.insert("switchboard".to_string(), 0.9);
        source_weights.insert("binance".to_string(), 0.85);
        source_weights.insert("coingecko".to_string(), 0.8);
        source_weights.insert("ftx".to_string(), 0.82);

        Self {
            source_weights,
            historical_performance: HashMap::new(),
        }
    }

    pub fn calculate(
        &self,
        feeds: &[PriceFeed],
        aggregated_price: &Price,
    ) -> Result<ConfidenceScore, OracleError> {
        if feeds.is_empty() {
            return Err(OracleError::NoDataAvailable);
        }

        let source_agreement = self.calculate_source_agreement(feeds, aggregated_price)?;
        let data_freshness = self.calculate_data_freshness(feeds)?;
        let volume_consistency = self.calculate_volume_consistency(feeds)?;
        let source_reliability = self.calculate_source_reliability(feeds)?;
        let price_stability = self.calculate_price_stability(feeds)?;

        let metrics = ConfidenceMetrics {
            source_agreement,
            data_freshness,
            volume_consistency,
            source_reliability,
            price_stability,
        };

        // Weighted combination of metrics
        let overall_score = self.combine_metrics(&metrics);

        let individual_scores = self.calculate_individual_scores(feeds, aggregated_price)?;
        let risk_level = RiskLevel::from_confidence(overall_score);

        Ok(ConfidenceScore {
            overall_score,
            metrics,
            individual_scores,
            risk_level,
        })
    }

    fn calculate_source_agreement(
        &self,
        feeds: &[PriceFeed],
        aggregated_price: &Price,
    ) -> Result<f64, OracleError> {
        let agg_price = aggregated_price.to_f64()
            .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

        let mut total_weighted_deviation = 0.0;
        let mut total_weight = 0.0;

        for feed in feeds {
            let price = feed.data.price.to_f64()
                .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

            let deviation = (price - agg_price).abs() / agg_price;
            let weight = self.source_weights.get(&feed.source_id).unwrap_or(&0.5);

            total_weighted_deviation += deviation * weight;
            total_weight += weight;
        }

        if total_weight == 0.0 {
            return Ok(0.0);
        }

        let avg_deviation = total_weighted_deviation / total_weight;

        // Convert deviation to agreement score (inverse relationship)
        let agreement = (1.0 - (avg_deviation * 10.0)).max(0.0).min(1.0);

        Ok(agreement)
    }

    fn calculate_data_freshness(&self, feeds: &[PriceFeed]) -> Result<f64, OracleError> {
        use chrono::{Utc, Duration};

        let now = Utc::now();
        let mut total_freshness = 0.0;
        let mut count = 0;

        for feed in feeds {
            let age = now - feed.data.timestamp;
            let age_minutes = age.num_minutes() as f64;

            // Freshness score decreases exponentially with age
            let freshness = (-age_minutes / 30.0).exp(); // Half-life of 30 minutes

            total_freshness += freshness;
            count += 1;
        }

        if count == 0 {
            return Ok(0.0);
        }

        Ok(total_freshness / count as f64)
    }

    fn calculate_volume_consistency(&self, feeds: &[PriceFeed]) -> Result<f64, OracleError> {
        if feeds.len() < 2 {
            return Ok(1.0); // Single source is perfectly consistent with itself
        }

        let volumes: Vec<f64> = feeds.iter()
            .map(|f| f.data.volume_24h as f64)
            .collect();

        let mean_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;

        if mean_volume == 0.0 {
            return Ok(0.0);
        }

        // Calculate coefficient of variation
        let variance = volumes.iter()
            .map(|&v| (v - mean_volume).powi(2))
            .sum::<f64>() / volumes.len() as f64;

        let cv = variance.sqrt() / mean_volume;

        // Convert CV to consistency score (lower CV = higher consistency)
        let consistency = (1.0 - cv.min(2.0) / 2.0).max(0.0);

        Ok(consistency)
    }

    fn calculate_source_reliability(&self, feeds: &[PriceFeed]) -> Result<f64, OracleError> {
        let mut total_reliability = 0.0;
        let mut total_weight = 0.0;

        for feed in feeds {
            // Combine intrinsic confidence with source weight
            let source_weight = self.source_weights.get(&feed.source_id).unwrap_or(&0.5);
            let historical_perf = self.historical_performance.get(&feed.source_id).unwrap_or(&0.8);

            let reliability = feed.data.confidence * source_weight * historical_perf;

            total_reliability += reliability;
            total_weight += 1.0;
        }

        if total_weight == 0.0 {
            return Ok(0.0);
        }

        Ok(total_reliability / total_weight)
    }

    fn calculate_price_stability(&self, feeds: &[PriceFeed]) -> Result<f64, OracleError> {
        if feeds.len() < 2 {
            return Ok(1.0);
        }

        let prices: Vec<f64> = feeds.iter()
            .map(|f| f.data.price.to_f64().unwrap_or(0.0))
            .collect();

        let mean_price = prices.iter().sum::<f64>() / prices.len() as f64;

        if mean_price == 0.0 {
            return Ok(0.0);
        }

        // Calculate coefficient of variation for prices
        let variance = prices.iter()
            .map(|&p| (p - mean_price).powi(2))
            .sum::<f64>() / prices.len() as f64;

        let cv = variance.sqrt() / mean_price;

        // Stability score (lower variation = higher stability)
        let stability = (1.0 - cv * 20.0).max(0.0).min(1.0);

        Ok(stability)
    }

    fn combine_metrics(&self, metrics: &ConfidenceMetrics) -> f64 {
        // Weighted combination of all metrics
        let weights = MetricWeights {
            source_agreement: 0.3,
            data_freshness: 0.2,
            volume_consistency: 0.15,
            source_reliability: 0.25,
            price_stability: 0.1,
        };

        let score = metrics.source_agreement * weights.source_agreement +
                   metrics.data_freshness * weights.data_freshness +
                   metrics.volume_consistency * weights.volume_consistency +
                   metrics.source_reliability * weights.source_reliability +
                   metrics.price_stability * weights.price_stability;

        score.max(0.0).min(1.0)
    }

    fn calculate_individual_scores(
        &self,
        feeds: &[PriceFeed],
        aggregated_price: &Price,
    ) -> Result<HashMap<String, f64>, OracleError> {
        let mut scores = HashMap::new();
        let agg_price = aggregated_price.to_f64()
            .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

        for feed in feeds {
            let price = feed.data.price.to_f64()
                .map_err(|e| OracleError::invalid_data(&e.to_string()))?;

            // Individual score based on multiple factors
            let price_accuracy = {
                let deviation = (price - agg_price).abs() / agg_price;
                (1.0 - deviation * 10.0).max(0.0).min(1.0)
            };

            let source_weight = self.source_weights.get(&feed.source_id).unwrap_or(&0.5);
            let intrinsic_confidence = feed.data.confidence;

            // Data freshness for this specific feed
            let freshness = {
                use chrono::{Utc, Duration};
                let age = Utc::now() - feed.data.timestamp;
                let age_minutes = age.num_minutes() as f64;
                (-age_minutes / 30.0).exp()
            };

            // Combined individual score
            let individual_score = (price_accuracy * 0.4 +
                                  source_weight * 0.3 +
                                  intrinsic_confidence * 0.2 +
                                  freshness * 0.1)
                                  .max(0.0).min(1.0);

            scores.insert(feed.source_id.clone(), individual_score);
        }

        Ok(scores)
    }

    pub fn update_historical_performance(&mut self, source_id: &str, performance: f64) {
        self.historical_performance.insert(source_id.to_string(), performance);
    }

    pub fn get_whale_trading_recommendation(&self, confidence: &ConfidenceScore) -> TradingRecommendation {
        match confidence.risk_level {
            RiskLevel::Low => TradingRecommendation {
                allow_trading: true,
                max_position_size: 1.0,
                required_slippage_buffer: 0.0,
                recommended_execution: ExecutionStrategy::Market,
                message: "High confidence - proceed with normal execution".to_string(),
            },
            RiskLevel::Medium => TradingRecommendation {
                allow_trading: true,
                max_position_size: 0.7,
                required_slippage_buffer: 0.02,
                recommended_execution: ExecutionStrategy::TWAP,
                message: "Medium confidence - use TWAP execution".to_string(),
            },
            RiskLevel::High => TradingRecommendation {
                allow_trading: true,
                max_position_size: 0.3,
                required_slippage_buffer: 0.05,
                recommended_execution: ExecutionStrategy::Conservative,
                message: "Low confidence - conservative execution only".to_string(),
            },
            RiskLevel::Critical => TradingRecommendation {
                allow_trading: false,
                max_position_size: 0.0,
                required_slippage_buffer: 1.0,
                recommended_execution: ExecutionStrategy::Block,
                message: "Critical risk - block all whale trades".to_string(),
            },
        }
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct MetricWeights {
    source_agreement: f64,
    data_freshness: f64,
    volume_consistency: f64,
    source_reliability: f64,
    price_stability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingRecommendation {
    pub allow_trading: bool,
    pub max_position_size: f64,
    pub required_slippage_buffer: f64,
    pub recommended_execution: ExecutionStrategy,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Market,
    TWAP,
    Conservative,
    Block,
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
    fn test_confidence_calculation() {
        let calculator = ConfidenceCalculator::new();
        let feeds = create_test_feeds();
        let aggregated_price = Price::from_float(43_255.0, 8).unwrap();

        let confidence = calculator.calculate(&feeds, &aggregated_price).unwrap();

        assert!(confidence.overall_score > 0.0);
        assert!(confidence.overall_score <= 1.0);
        assert_eq!(confidence.individual_scores.len(), 3);
    }

    #[test]
    fn test_source_agreement() {
        let calculator = ConfidenceCalculator::new();
        let feeds = create_test_feeds();
        let aggregated_price = Price::from_float(43_255.0, 8).unwrap();

        let agreement = calculator.calculate_source_agreement(&feeds, &aggregated_price).unwrap();

        assert!(agreement > 0.8); // Prices are close, should have high agreement
        assert!(agreement <= 1.0);
    }

    #[test]
    fn test_data_freshness() {
        let calculator = ConfidenceCalculator::new();
        let feeds = create_test_feeds();

        let freshness = calculator.calculate_data_freshness(&feeds).unwrap();

        assert!(freshness > 0.9); // Recent timestamps should have high freshness
        assert!(freshness <= 1.0);
    }

    #[test]
    fn test_volume_consistency() {
        let calculator = ConfidenceCalculator::new();
        let feeds = create_test_feeds();

        let consistency = calculator.calculate_volume_consistency(&feeds).unwrap();

        assert!(consistency > 0.0);
        assert!(consistency <= 1.0);
    }

    #[test]
    fn test_price_stability() {
        let calculator = ConfidenceCalculator::new();
        let feeds = create_test_feeds();

        let stability = calculator.calculate_price_stability(&feeds).unwrap();

        assert!(stability > 0.8); // Similar prices should be stable
        assert!(stability <= 1.0);
    }

    #[test]
    fn test_risk_level_classification() {
        assert!(matches!(RiskLevel::from_confidence(0.95), RiskLevel::Low));
        assert!(matches!(RiskLevel::from_confidence(0.75), RiskLevel::Medium));
        assert!(matches!(RiskLevel::from_confidence(0.55), RiskLevel::High));
        assert!(matches!(RiskLevel::from_confidence(0.25), RiskLevel::Critical));
    }

    #[test]
    fn test_whale_trading_thresholds() {
        assert_eq!(RiskLevel::Low.whale_trading_threshold(), 0.0);
        assert_eq!(RiskLevel::Medium.whale_trading_threshold(), 0.02);
        assert_eq!(RiskLevel::High.whale_trading_threshold(), 0.05);
        assert_eq!(RiskLevel::Critical.whale_trading_threshold(), 1.0);
    }

    #[test]
    fn test_trading_recommendation() {
        let calculator = ConfidenceCalculator::new();
        let feeds = create_test_feeds();
        let aggregated_price = Price::from_float(43_255.0, 8).unwrap();

        let confidence = calculator.calculate(&feeds, &aggregated_price).unwrap();
        let recommendation = calculator.get_whale_trading_recommendation(&confidence);

        assert!(recommendation.allow_trading); // High confidence feeds should allow trading
        assert!(recommendation.max_position_size > 0.0);
    }

    #[test]
    fn test_low_confidence_scenario() {
        let calculator = ConfidenceCalculator::new();
        let mut feeds = create_test_feeds();

        // Modify to create low confidence scenario - make prices diverge significantly
        feeds[0].data.confidence = 0.3;
        feeds[1].data.confidence = 0.4;
        feeds[2].data.confidence = 0.2;

        // Also make prices diverge to reduce source agreement
        feeds[0].data.price = Price::from_float(42_000.0, 8).unwrap();
        feeds[1].data.price = Price::from_float(44_500.0, 8).unwrap();
        feeds[2].data.price = Price::from_float(43_800.0, 8).unwrap();

        let aggregated_price = Price::from_float(43_255.0, 8).unwrap();
        let confidence = calculator.calculate(&feeds, &aggregated_price).unwrap();

        assert!(confidence.overall_score < 0.8); // Should be lower confidence
        // Risk level depends on overall score calculation
    }

    #[test]
    fn test_historical_performance_update() {
        let mut calculator = ConfidenceCalculator::new();

        calculator.update_historical_performance("chainlink", 0.95);
        calculator.update_historical_performance("pyth", 0.85);

        assert_eq!(calculator.historical_performance.get("chainlink"), Some(&0.95));
        assert_eq!(calculator.historical_performance.get("pyth"), Some(&0.85));
    }

    #[test]
    fn test_individual_scores() {
        let calculator = ConfidenceCalculator::new();
        let feeds = create_test_feeds();
        let aggregated_price = Price::from_float(43_255.0, 8).unwrap();

        let scores = calculator.calculate_individual_scores(&feeds, &aggregated_price).unwrap();

        assert_eq!(scores.len(), 3);

        // Chainlink should have highest score due to weight and confidence
        let chainlink_score = scores.get("chainlink").unwrap();
        let binance_score = scores.get("binance").unwrap();

        assert!(chainlink_score > binance_score);
    }

    #[test]
    fn test_empty_feeds() {
        let calculator = ConfidenceCalculator::new();
        let feeds = vec![];
        let aggregated_price = Price::from_float(43_255.0, 8).unwrap();

        let result = calculator.calculate(&feeds, &aggregated_price);
        assert!(result.is_err());
    }
}