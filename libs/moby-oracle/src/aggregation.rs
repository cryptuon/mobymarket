//! # Price Feed Aggregation
//!
//! This module provides sophisticated aggregation strategies for combining price data from multiple
//! oracle sources to produce reliable, accurate price feeds for whale trading operations.
//!
//! ## Features
//!
//! - Multiple aggregation strategies (median, weighted average, TWAP)
//! - Outlier detection and filtering
//! - Confidence scoring and data quality metrics
//! - Real-time consensus mechanisms
//! - Whale trading volume impact analysis

use crate::error::{OracleError, OracleResult};
use crate::sources::{DataPoint, DataSource};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;

/// Aggregation strategy for combining multiple price data points
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AggregationStrategy {
    /// Simple median of all valid prices
    Median,
    /// Weighted average based on source reliability
    WeightedAverage,
    /// Time-weighted average price (TWAP)
    TimeWeightedAverage { window: Duration },
    /// Volume-weighted average price (VWAP)
    VolumeWeightedAverage,
    /// Consensus-based aggregation requiring minimum agreement
    Consensus { min_sources: usize, threshold: Decimal },
    /// Custom aggregation with user-defined function
    Custom { name: String },
}

/// Aggregated price data with confidence metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPrice {
    /// Final aggregated price value
    pub price: Decimal,
    /// Trading symbol (e.g., "ETH/USD")
    pub symbol: String,
    /// Timestamp of aggregation
    pub timestamp: DateTime<Utc>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Number of sources contributing to aggregation
    pub source_count: usize,
    /// Sources that contributed data
    pub contributing_sources: Vec<DataSource>,
    /// Price deviation from previous aggregation
    pub deviation: Option<Decimal>,
    /// Aggregation strategy used
    pub strategy: AggregationStrategy,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// Whale trading impact analysis
    pub whale_impact: WhaleImpactAnalysis,
}

/// Data quality metrics for aggregated prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Standard deviation of source prices
    pub price_variance: Decimal,
    /// Maximum price difference between sources
    pub max_spread: Decimal,
    /// Percentage of outliers filtered
    pub outlier_rate: f64,
    /// Average latency of source data
    pub avg_latency_ms: u64,
    /// Source reliability scores
    pub source_reliability: HashMap<DataSource, f64>,
}

/// Whale trading impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleImpactAnalysis {
    /// Estimated price impact for large orders
    pub price_impact_bps: Decimal,
    /// Available liquidity depth
    pub liquidity_depth: Decimal,
    /// Market volatility indicator
    pub volatility_score: f64,
    /// Large order flow detection
    pub whale_activity_detected: bool,
    /// Recommended order size limits
    pub max_order_size: Option<Decimal>,
}

/// Configuration for the aggregation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Primary aggregation strategy
    pub strategy: AggregationStrategy,
    /// Fallback strategy if primary fails
    pub fallback_strategy: Option<AggregationStrategy>,
    /// Maximum age of data points to consider
    pub max_data_age: Duration,
    /// Minimum number of sources required
    pub min_sources: usize,
    /// Maximum allowed price deviation (percentage)
    pub max_deviation: Decimal,
    /// Outlier detection threshold (z-score)
    pub outlier_threshold: f64,
    /// Source weights for weighted aggregation
    pub source_weights: HashMap<DataSource, f64>,
    /// Whale trading specific settings
    pub whale_config: WhaleAggregationConfig,
}

/// Whale trading specific aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleAggregationConfig {
    /// Volume threshold for whale detection
    pub whale_volume_threshold: Decimal,
    /// Price impact calculation method
    pub impact_calculation: ImpactCalculationMethod,
    /// Liquidity analysis depth
    pub liquidity_depth_levels: usize,
    /// Volatility window for analysis
    pub volatility_window: Duration,
}

/// Method for calculating price impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactCalculationMethod {
    /// Linear price impact model
    Linear { coefficient: f64 },
    /// Square root price impact model
    SquareRoot { coefficient: f64 },
    /// Order book based calculation
    OrderBook,
    /// Historical regression model
    Historical { lookback_periods: usize },
}

/// Trait for implementing aggregation strategies
#[async_trait]
pub trait AggregationStrategy {
    /// Aggregate multiple data points into a single price
    async fn aggregate(
        &self,
        data_points: Vec<DataPoint>,
        config: &AggregationConfig,
    ) -> OracleResult<AggregatedPrice>;

    /// Validate aggregation parameters
    fn validate_config(&self, config: &AggregationConfig) -> OracleResult<()>;

    /// Calculate confidence score for aggregated price
    fn calculate_confidence(&self, data_points: &[DataPoint]) -> f64;
}

/// Main price aggregator
pub struct Aggregator {
    config: AggregationConfig,
    historical_prices: Vec<AggregatedPrice>,
}

impl Aggregator {
    /// Create new aggregator with configuration
    pub fn new(config: AggregationConfig) -> Self {
        Self {
            config,
            historical_prices: Vec::new(),
        }
    }

    /// Aggregate price data from multiple sources
    pub async fn aggregate_prices(
        &mut self,
        symbol: &str,
        data_points: Vec<DataPoint>,
    ) -> OracleResult<AggregatedPrice> {
        // Validate input data
        self.validate_data_points(&data_points)?;

        // Filter outliers
        let filtered_points = self.filter_outliers(data_points)?;

        // Check minimum source requirement
        if filtered_points.len() < self.config.min_sources {
            return Err(OracleError::InsufficientDataSources {
                required: self.config.min_sources,
                available: filtered_points.len(),
            });
        }

        // Apply aggregation strategy
        let aggregated = match &self.config.strategy {
            AggregationStrategy::Median => self.aggregate_median(&filtered_points).await?,
            AggregationStrategy::WeightedAverage => self.aggregate_weighted(&filtered_points).await?,
            AggregationStrategy::TimeWeightedAverage { window } => {
                self.aggregate_twap(&filtered_points, *window).await?
            }
            AggregationStrategy::VolumeWeightedAverage => {
                self.aggregate_vwap(&filtered_points).await?
            }
            AggregationStrategy::Consensus { min_sources, threshold } => {
                self.aggregate_consensus(&filtered_points, *min_sources, *threshold).await?
            }
            AggregationStrategy::Custom { name } => {
                return Err(OracleError::UnsupportedAggregationStrategy { strategy: name.clone() });
            }
        };

        // Store historical data
        self.historical_prices.push(aggregated.clone());

        // Limit historical data size
        if self.historical_prices.len() > 1000 {
            self.historical_prices.drain(0..100);
        }

        Ok(aggregated)
    }

    /// Validate input data points
    fn validate_data_points(&self, data_points: &[DataPoint]) -> OracleResult<()> {
        if data_points.is_empty() {
            return Err(OracleError::NoDataPointsProvided);
        }

        let now = Utc::now();
        for point in data_points {
            // Check data age
            let age = now.signed_duration_since(point.timestamp);
            if age > chrono::Duration::from_std(self.config.max_data_age)
                .map_err(|_| OracleError::InvalidTimeRange)?
            {
                return Err(OracleError::DataTooOld {
                    age_seconds: age.num_seconds() as u64,
                    max_age_seconds: self.config.max_data_age.as_secs(),
                });
            }

            // Validate price value
            if point.value <= Decimal::ZERO {
                return Err(OracleError::InvalidPriceValue { value: point.value });
            }
        }

        Ok(())
    }

    /// Filter outliers using statistical methods
    fn filter_outliers(&self, mut data_points: Vec<DataPoint>) -> OracleResult<Vec<DataPoint>> {
        if data_points.len() < 3 {
            return Ok(data_points);
        }

        // Calculate z-scores
        let prices: Vec<f64> = data_points.iter()
            .map(|p| p.value.to_string().parse::<f64>().unwrap_or(0.0))
            .collect();

        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return Ok(data_points);
        }

        // Filter based on z-score threshold
        data_points.retain(|point| {
            let price = point.value.to_string().parse::<f64>().unwrap_or(0.0);
            let z_score = (price - mean).abs() / std_dev;
            z_score <= self.config.outlier_threshold
        });

        Ok(data_points)
    }

    /// Aggregate using median strategy
    async fn aggregate_median(&self, data_points: &[DataPoint]) -> OracleResult<AggregatedPrice> {
        let mut prices: Vec<Decimal> = data_points.iter().map(|p| p.value).collect();
        prices.sort();

        let median_price = if prices.len() % 2 == 0 {
            let mid = prices.len() / 2;
            (prices[mid - 1] + prices[mid]) / Decimal::from(2)
        } else {
            prices[prices.len() / 2]
        };

        let quality_metrics = self.calculate_quality_metrics(data_points);
        let whale_impact = self.analyze_whale_impact(data_points, median_price).await?;
        let confidence = self.calculate_confidence_score(data_points, median_price);

        Ok(AggregatedPrice {
            price: median_price,
            symbol: data_points[0].symbol.clone(),
            timestamp: Utc::now(),
            confidence,
            source_count: data_points.len(),
            contributing_sources: data_points.iter().map(|p| p.source).collect(),
            deviation: self.calculate_deviation(median_price),
            strategy: AggregationStrategy::Median,
            quality_metrics,
            whale_impact,
        })
    }

    /// Aggregate using weighted average strategy
    async fn aggregate_weighted(&self, data_points: &[DataPoint]) -> OracleResult<AggregatedPrice> {
        let mut weighted_sum = Decimal::ZERO;
        let mut total_weight = Decimal::ZERO;

        for point in data_points {
            let weight = self.config.source_weights
                .get(&point.source)
                .unwrap_or(&1.0);
            let weight_decimal = Decimal::from_f64_retain(*weight)
                .ok_or(OracleError::InvalidWeight { weight: *weight })?;

            weighted_sum += point.value * weight_decimal;
            total_weight += weight_decimal;
        }

        if total_weight == Decimal::ZERO {
            return Err(OracleError::ZeroTotalWeight);
        }

        let weighted_price = weighted_sum / total_weight;
        let quality_metrics = self.calculate_quality_metrics(data_points);
        let whale_impact = self.analyze_whale_impact(data_points, weighted_price).await?;
        let confidence = self.calculate_confidence_score(data_points, weighted_price);

        Ok(AggregatedPrice {
            price: weighted_price,
            symbol: data_points[0].symbol.clone(),
            timestamp: Utc::now(),
            confidence,
            source_count: data_points.len(),
            contributing_sources: data_points.iter().map(|p| p.source).collect(),
            deviation: self.calculate_deviation(weighted_price),
            strategy: AggregationStrategy::WeightedAverage,
            quality_metrics,
            whale_impact,
        })
    }

    /// Aggregate using time-weighted average price (TWAP)
    async fn aggregate_twap(
        &self,
        data_points: &[DataPoint],
        window: Duration,
    ) -> OracleResult<AggregatedPrice> {
        let now = Utc::now();
        let window_start = now - chrono::Duration::from_std(window)
            .map_err(|_| OracleError::InvalidTimeRange)?;

        // Filter points within time window
        let window_points: Vec<_> = data_points.iter()
            .filter(|p| p.timestamp >= window_start)
            .collect();

        if window_points.is_empty() {
            return Err(OracleError::NoDataInTimeWindow {
                window_seconds: window.as_secs(),
            });
        }

        // Calculate time-weighted average
        let mut total_weighted_value = Decimal::ZERO;
        let mut total_time_weight = 0i64;

        for (i, point) in window_points.iter().enumerate() {
            let time_weight = if i == window_points.len() - 1 {
                // Last point - weight until now
                now.signed_duration_since(point.timestamp).num_seconds()
            } else {
                // Weight until next point
                window_points[i + 1].timestamp.signed_duration_since(point.timestamp).num_seconds()
            };

            if time_weight > 0 {
                total_weighted_value += point.value * Decimal::from(time_weight);
                total_time_weight += time_weight;
            }
        }

        if total_time_weight == 0 {
            return Err(OracleError::ZeroTimeWeight);
        }

        let twap_price = total_weighted_value / Decimal::from(total_time_weight);
        let quality_metrics = self.calculate_quality_metrics(data_points);
        let whale_impact = self.analyze_whale_impact(data_points, twap_price).await?;
        let confidence = self.calculate_confidence_score(data_points, twap_price);

        Ok(AggregatedPrice {
            price: twap_price,
            symbol: data_points[0].symbol.clone(),
            timestamp: now,
            confidence,
            source_count: data_points.len(),
            contributing_sources: data_points.iter().map(|p| p.source).collect(),
            deviation: self.calculate_deviation(twap_price),
            strategy: AggregationStrategy::TimeWeightedAverage { window },
            quality_metrics,
            whale_impact,
        })
    }

    /// Aggregate using volume-weighted average price (VWAP)
    async fn aggregate_vwap(&self, data_points: &[DataPoint]) -> OracleResult<AggregatedPrice> {
        let mut volume_weighted_sum = Decimal::ZERO;
        let mut total_volume = Decimal::ZERO;

        for point in data_points {
            let volume = point.volume.unwrap_or(Decimal::ONE);
            volume_weighted_sum += point.value * volume;
            total_volume += volume;
        }

        if total_volume == Decimal::ZERO {
            return Err(OracleError::ZeroTotalVolume);
        }

        let vwap_price = volume_weighted_sum / total_volume;
        let quality_metrics = self.calculate_quality_metrics(data_points);
        let whale_impact = self.analyze_whale_impact(data_points, vwap_price).await?;
        let confidence = self.calculate_confidence_score(data_points, vwap_price);

        Ok(AggregatedPrice {
            price: vwap_price,
            symbol: data_points[0].symbol.clone(),
            timestamp: Utc::now(),
            confidence,
            source_count: data_points.len(),
            contributing_sources: data_points.iter().map(|p| p.source).collect(),
            deviation: self.calculate_deviation(vwap_price),
            strategy: AggregationStrategy::VolumeWeightedAverage,
            quality_metrics,
            whale_impact,
        })
    }

    /// Aggregate using consensus strategy
    async fn aggregate_consensus(
        &self,
        data_points: &[DataPoint],
        min_sources: usize,
        threshold: Decimal,
    ) -> OracleResult<AggregatedPrice> {
        if data_points.len() < min_sources {
            return Err(OracleError::InsufficientDataSources {
                required: min_sources,
                available: data_points.len(),
            });
        }

        // Group prices within threshold
        let mut consensus_groups: Vec<Vec<&DataPoint>> = Vec::new();

        for point in data_points {
            let mut added_to_group = false;

            for group in &mut consensus_groups {
                if let Some(first_point) = group.first() {
                    let price_diff = (point.value - first_point.value).abs();
                    let relative_diff = price_diff / first_point.value;

                    if relative_diff <= threshold {
                        group.push(point);
                        added_to_group = true;
                        break;
                    }
                }
            }

            if !added_to_group {
                consensus_groups.push(vec![point]);
            }
        }

        // Find largest consensus group
        let consensus_group = consensus_groups.into_iter()
            .max_by_key(|group| group.len())
            .ok_or(OracleError::NoConsensusReached)?;

        if consensus_group.len() < min_sources {
            return Err(OracleError::InsufficientConsensus {
                required: min_sources,
                achieved: consensus_group.len(),
            });
        }

        // Calculate average of consensus group
        let sum: Decimal = consensus_group.iter().map(|p| p.value).sum();
        let consensus_price = sum / Decimal::from(consensus_group.len());

        let quality_metrics = self.calculate_quality_metrics(data_points);
        let whale_impact = self.analyze_whale_impact(data_points, consensus_price).await?;
        let confidence = self.calculate_confidence_score(data_points, consensus_price);

        Ok(AggregatedPrice {
            price: consensus_price,
            symbol: data_points[0].symbol.clone(),
            timestamp: Utc::now(),
            confidence,
            source_count: consensus_group.len(),
            contributing_sources: consensus_group.iter().map(|p| p.source).collect(),
            deviation: self.calculate_deviation(consensus_price),
            strategy: AggregationStrategy::Consensus { min_sources, threshold },
            quality_metrics,
            whale_impact,
        })
    }

    /// Calculate quality metrics for aggregated data
    fn calculate_quality_metrics(&self, data_points: &[DataPoint]) -> QualityMetrics {
        let prices: Vec<f64> = data_points.iter()
            .map(|p| p.value.to_string().parse::<f64>().unwrap_or(0.0))
            .collect();

        // Calculate variance
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        // Calculate spread
        let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_price = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let spread = max_price - min_price;

        // Calculate average latency
        let now = Utc::now();
        let total_latency: i64 = data_points.iter()
            .map(|p| now.signed_duration_since(p.timestamp).num_milliseconds())
            .sum();
        let avg_latency = if !data_points.is_empty() {
            (total_latency / data_points.len() as i64) as u64
        } else {
            0
        };

        // Calculate source reliability (simplified)
        let mut source_reliability = HashMap::new();
        for point in data_points {
            let reliability = if point.confidence > 0.9 { 0.95 }
            else if point.confidence > 0.8 { 0.85 }
            else if point.confidence > 0.7 { 0.75 }
            else { 0.65 };
            source_reliability.insert(point.source, reliability);
        }

        QualityMetrics {
            price_variance: Decimal::from_f64_retain(variance).unwrap_or_default(),
            max_spread: Decimal::from_f64_retain(spread).unwrap_or_default(),
            outlier_rate: 0.0, // Would be calculated during outlier filtering
            avg_latency_ms: avg_latency,
            source_reliability,
        }
    }

    /// Analyze whale trading impact
    async fn analyze_whale_impact(
        &self,
        data_points: &[DataPoint],
        price: Decimal,
    ) -> OracleResult<WhaleImpactAnalysis> {
        // Calculate total volume
        let total_volume: Decimal = data_points.iter()
            .map(|p| p.volume.unwrap_or_default())
            .sum();

        // Detect whale activity
        let whale_detected = total_volume >= self.config.whale_config.whale_volume_threshold;

        // Calculate price impact (simplified model)
        let price_impact_bps = if whale_detected {
            let impact_coefficient = 0.001; // 0.1% per $1M volume
            let volume_in_millions = total_volume / Decimal::from(1_000_000);
            volume_in_millions * Decimal::from_f64_retain(impact_coefficient).unwrap_or_default()
        } else {
            Decimal::ZERO
        };

        // Estimate liquidity depth (simplified)
        let liquidity_depth = total_volume * Decimal::from(2); // Assume 2x reported volume

        // Calculate volatility score
        let prices: Vec<f64> = data_points.iter()
            .map(|p| p.value.to_string().parse::<f64>().unwrap_or(0.0))
            .collect();
        let volatility_score = if prices.len() > 1 {
            let mean = prices.iter().sum::<f64>() / prices.len() as f64;
            let variance = prices.iter()
                .map(|&p| (p - mean).powi(2))
                .sum::<f64>() / prices.len() as f64;
            (variance.sqrt() / mean).min(1.0)
        } else {
            0.0
        };

        // Calculate max order size
        let max_order_size = if whale_detected {
            Some(liquidity_depth / Decimal::from(10)) // 10% of liquidity
        } else {
            None
        };

        Ok(WhaleImpactAnalysis {
            price_impact_bps,
            liquidity_depth,
            volatility_score,
            whale_activity_detected: whale_detected,
            max_order_size,
        })
    }

    /// Calculate confidence score for aggregated price
    fn calculate_confidence_score(&self, data_points: &[DataPoint], aggregated_price: Decimal) -> f64 {
        let source_count_factor = (data_points.len() as f64 / 10.0).min(1.0);

        // Calculate price consistency
        let avg_confidence: f64 = data_points.iter()
            .map(|p| p.confidence)
            .sum::<f64>() / data_points.len() as f64;

        // Calculate price spread factor
        let prices: Vec<Decimal> = data_points.iter().map(|p| p.value).collect();
        let max_price = prices.iter().max().unwrap_or(&Decimal::ZERO);
        let min_price = prices.iter().min().unwrap_or(&Decimal::ZERO);
        let spread = if *max_price > Decimal::ZERO {
            (*max_price - *min_price) / *max_price
        } else {
            Decimal::ZERO
        };

        let spread_factor = (1.0 - spread.to_string().parse::<f64>().unwrap_or(0.0)).max(0.0);

        // Combine factors
        (source_count_factor * 0.3 + avg_confidence * 0.4 + spread_factor * 0.3).min(1.0)
    }

    /// Calculate price deviation from previous aggregation
    fn calculate_deviation(&self, current_price: Decimal) -> Option<Decimal> {
        if let Some(last_price) = self.historical_prices.last() {
            let deviation = (current_price - last_price.price).abs() / last_price.price;
            Some(deviation)
        } else {
            None
        }
    }

    /// Get historical aggregated prices
    pub fn get_historical_prices(&self, limit: Option<usize>) -> Vec<&AggregatedPrice> {
        match limit {
            Some(n) => self.historical_prices.iter().rev().take(n).collect(),
            None => self.historical_prices.iter().collect(),
        }
    }

    /// Update aggregation configuration
    pub fn update_config(&mut self, config: AggregationConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AggregationConfig {
        &self.config
    }
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            strategy: AggregationStrategy::Median,
            fallback_strategy: Some(AggregationStrategy::WeightedAverage),
            max_data_age: Duration::from_secs(300), // 5 minutes
            min_sources: 3,
            max_deviation: Decimal::from_f64_retain(0.05).unwrap(), // 5%
            outlier_threshold: 2.0, // 2 standard deviations
            source_weights: HashMap::new(),
            whale_config: WhaleAggregationConfig::default(),
        }
    }
}

impl Default for WhaleAggregationConfig {
    fn default() -> Self {
        Self {
            whale_volume_threshold: Decimal::from(1_000_000), // $1M
            impact_calculation: ImpactCalculationMethod::Linear { coefficient: 0.001 },
            liquidity_depth_levels: 10,
            volatility_window: Duration::from_secs(3600), // 1 hour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::DataSource;

    fn create_test_data_point(source: DataSource, price: f64, confidence: f64) -> DataPoint {
        DataPoint {
            source,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from_f64_retain(price).unwrap(),
            timestamp: Utc::now(),
            confidence,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_median_aggregation() {
        let config = AggregationConfig::default();
        let mut aggregator = Aggregator::new(config);

        let data_points = vec![
            create_test_data_point(DataSource::Chainlink, 2000.0, 0.95),
            create_test_data_point(DataSource::Pyth, 2010.0, 0.90),
            create_test_data_point(DataSource::Band, 1990.0, 0.85),
        ];

        let result = aggregator.aggregate_prices("ETH/USD", data_points).await;
        assert!(result.is_ok());

        let aggregated = result.unwrap();
        assert_eq!(aggregated.price, Decimal::from(2000)); // Median of [1990, 2000, 2010]
        assert_eq!(aggregated.source_count, 3);
    }

    #[tokio::test]
    async fn test_weighted_aggregation() {
        let mut config = AggregationConfig::default();
        config.strategy = AggregationStrategy::WeightedAverage;
        config.source_weights.insert(DataSource::Chainlink, 0.5);
        config.source_weights.insert(DataSource::Pyth, 0.3);
        config.source_weights.insert(DataSource::Band, 0.2);

        let mut aggregator = Aggregator::new(config);

        let data_points = vec![
            create_test_data_point(DataSource::Chainlink, 2000.0, 0.95),
            create_test_data_point(DataSource::Pyth, 2010.0, 0.90),
            create_test_data_point(DataSource::Band, 1990.0, 0.85),
        ];

        let result = aggregator.aggregate_prices("ETH/USD", data_points).await;
        assert!(result.is_ok());

        let aggregated = result.unwrap();
        // Weighted average: (2000*0.5 + 2010*0.3 + 1990*0.2) = 2001
        assert_eq!(aggregated.price, Decimal::from(2001));
    }

    #[tokio::test]
    async fn test_insufficient_sources() {
        let mut config = AggregationConfig::default();
        config.min_sources = 5;

        let aggregator = Aggregator::new(config);

        let data_points = vec![
            create_test_data_point(DataSource::Chainlink, 2000.0, 0.95),
        ];

        let result = aggregator.aggregate_prices("ETH/USD", data_points).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OracleError::InsufficientDataSources { .. }));
    }

    #[tokio::test]
    async fn test_outlier_filtering() {
        let config = AggregationConfig::default();
        let mut aggregator = Aggregator::new(config);

        let data_points = vec![
            create_test_data_point(DataSource::Chainlink, 2000.0, 0.95),
            create_test_data_point(DataSource::Pyth, 2010.0, 0.90),
            create_test_data_point(DataSource::Band, 1990.0, 0.85),
            create_test_data_point(DataSource::API3, 5000.0, 0.80), // Outlier
        ];

        let result = aggregator.aggregate_prices("ETH/USD", data_points).await;
        assert!(result.is_ok());

        let aggregated = result.unwrap();
        // Should filter out the 5000 outlier and use median of [1990, 2000, 2010]
        assert_eq!(aggregated.price, Decimal::from(2000));
        assert_eq!(aggregated.source_count, 3); // Outlier filtered
    }

    #[tokio::test]
    async fn test_whale_impact_analysis() {
        let config = AggregationConfig::default();
        let mut aggregator = Aggregator::new(config);

        let data_points = vec![
            DataPoint {
                source: DataSource::Chainlink,
                symbol: "ETH/USD".to_string(),
                value: Decimal::from(2000),
                timestamp: Utc::now(),
                confidence: 0.95,
                volume: Some(Decimal::from(2_000_000)), // $2M volume - whale level
                metadata: HashMap::new(),
            },
        ];

        let result = aggregator.aggregate_prices("ETH/USD", data_points).await;
        assert!(result.is_ok());

        let aggregated = result.unwrap();
        assert!(aggregated.whale_impact.whale_activity_detected);
        assert!(aggregated.whale_impact.price_impact_bps > Decimal::ZERO);
    }
}