use crate::{PriceFeed, error::OracleError};
use moby_math::Price;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionParams {
    pub max_price_deviation: f64,
    pub max_volume_spike: f64,
    pub min_confidence_threshold: f64,
    pub correlation_threshold: f64,
    pub time_window_minutes: i64,
    pub min_sources_for_validation: usize,
}

impl Default for DetectionParams {
    fn default() -> Self {
        Self {
            max_price_deviation: 0.03,    // 3% max deviation
            max_volume_spike: 10.0,       // 10x volume spike
            min_confidence_threshold: 0.7, // 70% minimum confidence
            correlation_threshold: 0.8,    // 80% correlation between sources
            time_window_minutes: 60,       // 1 hour window
            min_sources_for_validation: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManipulationType {
    PriceSpike,
    VolumeSpike,
    SourceCorrelationBreak,
    StaleData,
    LowConfidence,
    SuspiciousPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManipulationAlert {
    pub alert_type: ManipulationType,
    pub severity: f64,
    pub description: String,
    pub affected_sources: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub recommended_action: String,
}

#[derive(Debug, Clone)]
pub struct PriceHistory {
    pub prices: VecDeque<(DateTime<Utc>, Price)>,
    pub volumes: VecDeque<(DateTime<Utc>, u64)>,
    pub max_history: usize,
}

impl PriceHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            prices: VecDeque::new(),
            volumes: VecDeque::new(),
            max_history,
        }
    }

    pub fn add_data(&mut self, timestamp: DateTime<Utc>, price: Price, volume: u64) {
        self.prices.push_back((timestamp, price));
        self.volumes.push_back((timestamp, volume));

        // Keep only recent history
        while self.prices.len() > self.max_history {
            self.prices.pop_front();
        }
        while self.volumes.len() > self.max_history {
            self.volumes.pop_front();
        }
    }

    pub fn get_recent_average(&self, minutes: i64) -> Option<f64> {
        let cutoff = Utc::now() - Duration::minutes(minutes);
        let recent_prices: Vec<f64> = self.prices
            .iter()
            .filter(|(timestamp, _)| *timestamp > cutoff)
            .map(|(_, price)| price.to_f64().unwrap_or(0.0))
            .collect();

        if recent_prices.is_empty() {
            None
        } else {
            Some(recent_prices.iter().sum::<f64>() / recent_prices.len() as f64)
        }
    }

    pub fn get_recent_volume_average(&self, minutes: i64) -> Option<f64> {
        let cutoff = Utc::now() - Duration::minutes(minutes);
        let recent_volumes: Vec<u64> = self.volumes
            .iter()
            .filter(|(timestamp, _)| *timestamp > cutoff)
            .map(|(_, volume)| *volume)
            .collect();

        if recent_volumes.is_empty() {
            None
        } else {
            Some(recent_volumes.iter().sum::<u64>() as f64 / recent_volumes.len() as f64)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManipulationDetector {
    params: DetectionParams,
    price_histories: HashMap<String, PriceHistory>,
    alert_history: VecDeque<ManipulationAlert>,
}

impl ManipulationDetector {
    pub fn new() -> Self {
        Self {
            params: DetectionParams::default(),
            price_histories: HashMap::new(),
            alert_history: VecDeque::new(),
        }
    }

    pub fn with_params(params: DetectionParams) -> Self {
        Self {
            params,
            price_histories: HashMap::new(),
            alert_history: VecDeque::new(),
        }
    }

    pub fn check_feeds(&self, feeds: &[PriceFeed]) -> Result<(), OracleError> {
        if feeds.len() < self.params.min_sources_for_validation {
            return Err(OracleError::aggregation_failed(&format!(
                "Insufficient sources for manipulation detection: {} < {}",
                feeds.len(), self.params.min_sources_for_validation
            )));
        }

        // Check for various manipulation patterns
        self.check_price_deviation(feeds)?;
        self.check_confidence_levels(feeds)?;
        self.check_data_staleness(feeds)?;
        self.check_source_correlation(feeds)?;

        Ok(())
    }

    fn check_price_deviation(&self, feeds: &[PriceFeed]) -> Result<(), OracleError> {
        let prices: Vec<f64> = feeds.iter()
            .map(|f| f.data.price.to_f64().unwrap_or(0.0))
            .collect();

        let mean_price = prices.iter().sum::<f64>() / prices.len() as f64;

        for (i, &price) in prices.iter().enumerate() {
            let deviation = (price - mean_price).abs() / mean_price;

            if deviation > self.params.max_price_deviation {
                return Err(OracleError::manipulation(&format!(
                    "Price deviation detected: source {} deviates {:.2}% from mean",
                    feeds[i].source_id, deviation * 100.0
                )));
            }
        }

        Ok(())
    }

    fn check_confidence_levels(&self, feeds: &[PriceFeed]) -> Result<(), OracleError> {
        for feed in feeds {
            if feed.data.confidence < self.params.min_confidence_threshold {
                return Err(OracleError::manipulation(&format!(
                    "Low confidence detected: source {} has confidence {:.2}",
                    feed.source_id, feed.data.confidence
                )));
            }
        }

        Ok(())
    }

    fn check_data_staleness(&self, feeds: &[PriceFeed]) -> Result<(), OracleError> {
        let now = Utc::now();
        let stale_threshold = Duration::minutes(self.params.time_window_minutes);

        for feed in feeds {
            let age = now - feed.data.timestamp;
            if age > stale_threshold {
                return Err(OracleError::stale(age.num_minutes() as u64));
            }
        }

        Ok(())
    }

    fn check_source_correlation(&self, feeds: &[PriceFeed]) -> Result<(), OracleError> {
        if feeds.len() < 3 {
            return Ok(()); // Can't check correlation with less than 3 sources
        }

        let prices: Vec<f64> = feeds.iter()
            .map(|f| f.data.price.to_f64().unwrap_or(0.0))
            .collect();

        // Calculate pairwise correlations (simplified - uses price variance as proxy)
        let mean_price = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|&p| (p - mean_price).powi(2))
            .sum::<f64>() / prices.len() as f64;

        let coefficient_of_variation = variance.sqrt() / mean_price;

        // High variation might indicate lack of correlation or manipulation
        if coefficient_of_variation > (1.0 - self.params.correlation_threshold) {
            return Err(OracleError::manipulation(&format!(
                "Source correlation break detected: CV = {:.4}",
                coefficient_of_variation
            )));
        }

        Ok(())
    }

    pub fn update_history(&mut self, feeds: &[PriceFeed]) {
        for feed in feeds {
            let history = self.price_histories
                .entry(feed.source_id.clone())
                .or_insert_with(|| PriceHistory::new(1000));

            history.add_data(
                feed.data.timestamp,
                feed.data.price,
                feed.data.volume_24h,
            );
        }
    }

    pub fn check_volume_spikes(&self, feeds: &[PriceFeed]) -> Result<Vec<ManipulationAlert>, OracleError> {
        let mut alerts = Vec::new();

        for feed in feeds {
            if let Some(history) = self.price_histories.get(&feed.source_id) {
                if let Some(avg_volume) = history.get_recent_volume_average(self.params.time_window_minutes) {
                    let current_volume = feed.data.volume_24h as f64;
                    let volume_ratio = current_volume / avg_volume;

                    if volume_ratio > self.params.max_volume_spike {
                        alerts.push(ManipulationAlert {
                            alert_type: ManipulationType::VolumeSpike,
                            severity: (volume_ratio / self.params.max_volume_spike).min(1.0),
                            description: format!(
                                "Volume spike detected: {}x normal volume on {}",
                                volume_ratio, feed.source_id
                            ),
                            affected_sources: vec![feed.source_id.clone()],
                            timestamp: Utc::now(),
                            recommended_action: "Reduce weight of this source temporarily".to_string(),
                        });
                    }
                }
            }
        }

        Ok(alerts)
    }

    pub fn check_price_patterns(&self, symbol: &str) -> Result<Vec<ManipulationAlert>, OracleError> {
        let mut alerts = Vec::new();

        // Check for suspicious price patterns across all sources
        let mut all_recent_prices = Vec::new();

        for (source_id, history) in &self.price_histories {
            let cutoff = Utc::now() - Duration::minutes(self.params.time_window_minutes);
            let recent_prices: Vec<(DateTime<Utc>, f64)> = history.prices
                .iter()
                .filter(|(timestamp, _)| *timestamp > cutoff)
                .map(|(timestamp, price)| (*timestamp, price.to_f64().unwrap_or(0.0)))
                .collect();

            if !recent_prices.is_empty() {
                all_recent_prices.extend(recent_prices);
            }
        }

        if all_recent_prices.len() > 10 {
            // Sort by timestamp
            all_recent_prices.sort_by_key(|(timestamp, _)| *timestamp);

            // Check for sudden jumps
            for window in all_recent_prices.windows(5) {
                let prices: Vec<f64> = window.iter().map(|(_, price)| *price).collect();
                let first_price = prices[0];
                let last_price = prices[prices.len() - 1];

                let change = (last_price - first_price).abs() / first_price;

                if change > self.params.max_price_deviation * 2.0 {
                    alerts.push(ManipulationAlert {
                        alert_type: ManipulationType::SuspiciousPattern,
                        severity: (change / (self.params.max_price_deviation * 2.0)).min(1.0),
                        description: format!(
                            "Suspicious price pattern detected for {}: {:.2}% change in short window",
                            symbol, change * 100.0
                        ),
                        affected_sources: vec!["multiple".to_string()],
                        timestamp: Utc::now(),
                        recommended_action: "Investigate price feeds and consider circuit breaker".to_string(),
                    });
                }
            }
        }

        Ok(alerts)
    }

    pub fn get_source_reliability(&self, source_id: &str) -> f64 {
        // Calculate reliability score based on historical performance
        if let Some(history) = self.price_histories.get(source_id) {
            let recent_count = history.prices.len().min(100); // Last 100 data points
            if recent_count == 0 {
                return 0.0;
            }

            // Simple reliability: inverse of coefficient of variation
            if let Some(avg_price) = history.get_recent_average(self.params.time_window_minutes) {
                let recent_prices: Vec<f64> = history.prices
                    .iter()
                    .rev()
                    .take(recent_count)
                    .map(|(_, price)| price.to_f64().unwrap_or(0.0))
                    .collect();

                let variance = recent_prices.iter()
                    .map(|&p| (p - avg_price).powi(2))
                    .sum::<f64>() / recent_prices.len() as f64;

                let cv = variance.sqrt() / avg_price;
                return (1.0 - cv.min(1.0)).max(0.0);
            }
        }

        0.5 // Default neutral reliability
    }

    pub fn add_alert(&mut self, alert: ManipulationAlert) {
        self.alert_history.push_back(alert);

        // Keep only recent alerts
        while self.alert_history.len() > 1000 {
            self.alert_history.pop_front();
        }
    }

    pub fn get_recent_alerts(&self, minutes: i64) -> Vec<&ManipulationAlert> {
        let cutoff = Utc::now() - Duration::minutes(minutes);
        self.alert_history
            .iter()
            .filter(|alert| alert.timestamp > cutoff)
            .collect()
    }
}

impl Default for ManipulationDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PriceData, sources::OracleSource};

    fn create_normal_feeds() -> Vec<PriceFeed> {
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
    fn test_normal_feeds_pass() {
        let detector = ManipulationDetector::new();
        let feeds = create_normal_feeds();

        let result = detector.check_feeds(&feeds);
        assert!(result.is_ok());
    }

    #[test]
    fn test_price_deviation_detection() {
        let detector = ManipulationDetector::new();
        let mut feeds = create_normal_feeds();

        // Add outlier price
        feeds[0].data.price = Price::from_float(50_000.0, 8).unwrap(); // Way off

        let result = detector.check_feeds(&feeds);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Price deviation"));
    }

    #[test]
    fn test_low_confidence_detection() {
        let detector = ManipulationDetector::new();
        let mut feeds = create_normal_feeds();

        // Set low confidence
        feeds[0].data.confidence = 0.5; // Below 0.7 threshold

        let result = detector.check_feeds(&feeds);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Low confidence"));
    }

    #[test]
    fn test_stale_data_detection() {
        let detector = ManipulationDetector::new();
        let mut feeds = create_normal_feeds();

        // Set old timestamp
        feeds[0].data.timestamp = Utc::now() - Duration::hours(2);

        let result = detector.check_feeds(&feeds);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Stale data"));
    }

    #[test]
    fn test_insufficient_sources() {
        let detector = ManipulationDetector::new();
        let feeds = vec![create_normal_feeds()[0].clone()]; // Only one source

        let result = detector.check_feeds(&feeds);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Insufficient sources"));
    }

    #[test]
    fn test_price_history() {
        let mut history = PriceHistory::new(10);
        let price = Price::from_float(43_250.0, 8).unwrap();

        history.add_data(Utc::now(), price, 1000);
        assert_eq!(history.prices.len(), 1);

        // Add more than max
        for i in 0..15 {
            let timestamp = Utc::now() + Duration::minutes(i);
            history.add_data(timestamp, price, 1000);
        }

        assert_eq!(history.prices.len(), 10); // Should be limited to max_history
    }

    #[test]
    fn test_volume_spike_detection() {
        let mut detector = ManipulationDetector::new();
        let feeds = create_normal_feeds();

        // Add normal history
        detector.update_history(&feeds);

        // Create feeds with volume spike
        let mut spike_feeds = feeds.clone();
        spike_feeds[0].data.volume_24h = 2_000_000_000 * Price::PRECISION; // 20x normal

        let alerts = detector.check_volume_spikes(&spike_feeds).unwrap();
        assert!(!alerts.is_empty());
        assert!(matches!(alerts[0].alert_type, ManipulationType::VolumeSpike));
    }

    #[test]
    fn test_source_reliability() {
        let mut detector = ManipulationDetector::new();
        let feeds = create_normal_feeds();

        detector.update_history(&feeds);

        let reliability = detector.get_source_reliability("chainlink");
        assert!(reliability > 0.0);
        assert!(reliability <= 1.0);
    }

    #[test]
    fn test_alert_management() {
        let mut detector = ManipulationDetector::new();

        let alert = ManipulationAlert {
            alert_type: ManipulationType::PriceSpike,
            severity: 0.8,
            description: "Test alert".to_string(),
            affected_sources: vec!["test".to_string()],
            timestamp: Utc::now(),
            recommended_action: "Test action".to_string(),
        };

        detector.add_alert(alert);
        assert_eq!(detector.alert_history.len(), 1);

        let recent = detector.get_recent_alerts(60);
        assert_eq!(recent.len(), 1);
    }
}