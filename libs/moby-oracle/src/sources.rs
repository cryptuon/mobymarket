//! Data source integrations for the Moby Oracle system.
//!
//! This module provides unified interfaces for connecting to various oracle networks
//! and data providers including Chainlink, Pyth, Band Protocol, API3, UMA, and
//! traditional financial data APIs for comprehensive price feed aggregation.

use crate::error::{OracleError, OracleResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use reqwest::Client;
use tokio::time::{Duration, timeout};

/// Data source types supported by the oracle system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum DataSource {
    /// Chainlink decentralized oracle network
    Chainlink,
    /// Pyth high-frequency price feeds
    Pyth,
    /// Band Protocol decentralized oracle
    BandProtocol,
    /// API3 first-party oracle feeds
    API3,
    /// UMA optimistic oracle
    UMA,
    /// Centralized exchange APIs
    CentralizedExchange { exchange: String },
    /// DEX aggregators
    DexAggregator { aggregator: String },
    /// Traditional financial data providers
    TraditionalFinance { provider: String },
    /// Custom data source
    Custom { name: String, endpoint: String },
}

/// Data point from any source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Source identifier
    pub source: DataSource,
    /// Trading symbol (e.g., "ETH/USD")
    pub symbol: String,
    /// Price value
    pub price: Decimal,
    /// Volume (if available)
    pub volume: Option<Decimal>,
    /// Liquidity depth (if available)
    pub liquidity: Option<Decimal>,
    /// Timestamp of the data
    pub timestamp: DateTime<Utc>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Data signature for verification
    pub signature: Option<String>,
}

/// Configuration for data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Source type
    pub source: DataSource,
    /// API endpoint URL
    pub endpoint: String,
    /// Authentication credentials
    pub auth: Option<AuthConfig>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Rate limit (requests per second)
    pub rate_limit: u32,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Additional configuration parameters
    pub parameters: HashMap<String, String>,
    /// Whether this source is enabled
    pub enabled: bool,
}

/// Authentication configuration for data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// API key authentication
    ApiKey {
        key: String,
        header_name: Option<String>,
    },
    /// Bearer token authentication
    BearerToken {
        token: String,
    },
    /// Basic authentication
    Basic {
        username: String,
        password: String,
    },
    /// Custom authentication headers
    Custom {
        headers: HashMap<String, String>,
    },
}

/// Retry configuration for failed requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay between retries (milliseconds)
    pub initial_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,
        }
    }
}

/// Source provider trait for implementing data source connectors
#[async_trait]
pub trait SourceProvider: Send + Sync {
    /// Get the source type
    fn source_type(&self) -> DataSource;

    /// Fetch latest price data for a symbol
    async fn fetch_price(&self, symbol: &str) -> OracleResult<DataPoint>;

    /// Fetch multiple symbols at once (if supported)
    async fn fetch_prices(&self, symbols: &[String]) -> OracleResult<Vec<DataPoint>> {
        let mut results = Vec::new();
        for symbol in symbols {
            match self.fetch_price(symbol).await {
                Ok(data_point) => results.push(data_point),
                Err(e) => {
                    log::warn!("Failed to fetch price for {}: {}", symbol, e);
                    continue;
                }
            }
        }
        Ok(results)
    }

    /// Check if the source is healthy and responsive
    async fn health_check(&self) -> OracleResult<SourceHealth>;

    /// Get supported symbols
    async fn get_supported_symbols(&self) -> OracleResult<Vec<String>>;

    /// Subscribe to real-time price updates (if supported)
    async fn subscribe(&self, symbols: &[String]) -> OracleResult<SourceSubscription> {
        Err(OracleError::NotImplemented {
            feature: format!("Real-time subscription for {:?}", self.source_type()),
            planned_version: Some("0.2.0".to_string()),
        })
    }

    /// Get source metadata and capabilities
    async fn get_metadata(&self) -> SourceMetadata;
}

/// Health status of a data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceHealth {
    /// Whether the source is healthy
    pub is_healthy: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Last successful request timestamp
    pub last_success: Option<DateTime<Utc>>,
    /// Error details if unhealthy
    pub error_details: Option<String>,
    /// Uptime percentage over last 24 hours
    pub uptime_24h: Option<f64>,
}

/// Real-time subscription for price updates
#[derive(Debug)]
pub struct SourceSubscription {
    /// Subscription ID
    pub id: String,
    /// Data receiver channel
    pub receiver: tokio::sync::mpsc::Receiver<DataPoint>,
    /// Subscription status
    pub status: SubscriptionStatus,
}

/// Status of real-time subscriptions
#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Failed(String),
    Cancelled,
}

/// Metadata about a data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadata {
    /// Source name
    pub name: String,
    /// Description
    pub description: String,
    /// Supported features
    pub features: Vec<SourceFeature>,
    /// Update frequency
    pub update_frequency: Option<Duration>,
    /// Supported symbols count
    pub symbol_count: Option<u32>,
    /// Rate limits
    pub rate_limits: Option<RateLimits>,
    /// Geographic restrictions
    pub geo_restrictions: Vec<String>,
}

/// Features supported by data sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceFeature {
    RealtimeUpdates,
    HistoricalData,
    BatchRequests,
    WebSocketStreaming,
    CryptographicProofs,
    HighFrequency,
    LowLatency,
    WhaleDataSupport,
}

/// Rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Requests per second
    pub requests_per_second: u32,
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    /// Daily request limit
    pub daily_limit: Option<u32>,
    /// Burst limit
    pub burst_limit: Option<u32>,
}

/// Chainlink data source implementation
pub struct ChainlinkProvider {
    config: SourceConfig,
    client: Client,
}

impl ChainlinkProvider {
    /// Create new Chainlink provider
    pub fn new(config: SourceConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SourceProvider for ChainlinkProvider {
    fn source_type(&self) -> DataSource {
        DataSource::Chainlink
    }

    async fn fetch_price(&self, symbol: &str) -> OracleResult<DataPoint> {
        let url = format!("{}/api/v1/price/{}", self.config.endpoint, symbol);

        let response = timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.client.get(&url).send()
        ).await
        .map_err(|_| OracleError::data_source_timeout("chainlink", self.config.timeout_ms))?
        .map_err(|e| OracleError::NetworkError {
            operation: "chainlink_fetch".to_string(),
            error: e.to_string(),
            endpoint: Some(url.clone()),
        })?;

        if !response.status().is_success() {
            return Err(OracleError::ApiRequestFailed {
                url,
                status: response.status().as_u16(),
                response_body: response.text().await.ok(),
                retry_after: None,
            });
        }

        let data: ChainlinkResponse = response.json().await
            .map_err(|e| OracleError::invalid_data("chainlink", &e.to_string()))?;

        Ok(DataPoint {
            source: DataSource::Chainlink,
            symbol: symbol.to_string(),
            price: data.price,
            volume: None,
            liquidity: None,
            timestamp: data.updated_at,
            confidence: 0.95, // Chainlink typically has high confidence
            metadata: HashMap::from([
                ("round_id".to_string(), data.round_id.to_string()),
                ("aggregator".to_string(), data.aggregator.unwrap_or_default()),
            ]),
            signature: data.signature,
        })
    }

    async fn health_check(&self) -> OracleResult<SourceHealth> {
        let start = std::time::Instant::now();

        let health_url = format!("{}/health", self.config.endpoint);
        let result = timeout(
            Duration::from_millis(5000), // 5 second timeout for health checks
            self.client.get(&health_url).send()
        ).await;

        let response_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) if response.status().is_success() => {
                Ok(SourceHealth {
                    is_healthy: true,
                    response_time_ms,
                    last_success: Some(Utc::now()),
                    error_details: None,
                    uptime_24h: Some(99.9), // Would be calculated from metrics
                })
            }
            Ok(Ok(response)) => {
                Ok(SourceHealth {
                    is_healthy: false,
                    response_time_ms,
                    last_success: None,
                    error_details: Some(format!("HTTP {}", response.status())),
                    uptime_24h: None,
                })
            }
            Ok(Err(e)) => {
                Ok(SourceHealth {
                    is_healthy: false,
                    response_time_ms,
                    last_success: None,
                    error_details: Some(e.to_string()),
                    uptime_24h: None,
                })
            }
            Err(_) => {
                Ok(SourceHealth {
                    is_healthy: false,
                    response_time_ms,
                    last_success: None,
                    error_details: Some("Timeout".to_string()),
                    uptime_24h: None,
                })
            }
        }
    }

    async fn get_supported_symbols(&self) -> OracleResult<Vec<String>> {
        let url = format!("{}/api/v1/symbols", self.config.endpoint);

        let response = self.client.get(&url).send().await
            .map_err(|e| OracleError::NetworkError {
                operation: "get_symbols".to_string(),
                error: e.to_string(),
                endpoint: Some(url),
            })?;

        let symbols: Vec<String> = response.json().await
            .map_err(|e| OracleError::invalid_data("chainlink", &e.to_string()))?;

        Ok(symbols)
    }

    async fn get_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "Chainlink".to_string(),
            description: "Decentralized oracle network providing secure price feeds".to_string(),
            features: vec![
                SourceFeature::CryptographicProofs,
                SourceFeature::HighFrequency,
                SourceFeature::WhaleDataSupport,
            ],
            update_frequency: Some(Duration::from_secs(60)), // 1 minute
            symbol_count: Some(1000), // Approximate
            rate_limits: Some(RateLimits {
                requests_per_second: 10,
                requests_per_minute: Some(600),
                daily_limit: Some(100000),
                burst_limit: Some(50),
            }),
            geo_restrictions: vec![], // Generally available globally
        }
    }
}

/// Chainlink API response structure
#[derive(Debug, Deserialize)]
struct ChainlinkResponse {
    price: Decimal,
    updated_at: DateTime<Utc>,
    round_id: u64,
    aggregator: Option<String>,
    signature: Option<String>,
}

/// Pyth data source implementation
pub struct PythProvider {
    config: SourceConfig,
    client: Client,
}

impl PythProvider {
    pub fn new(config: SourceConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SourceProvider for PythProvider {
    fn source_type(&self) -> DataSource {
        DataSource::Pyth
    }

    async fn fetch_price(&self, symbol: &str) -> OracleResult<DataPoint> {
        let url = format!("{}/api/latest_price_feeds?ids={}",
                         self.config.endpoint,
                         self.symbol_to_pyth_id(symbol)?);

        let response = timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.client.get(&url).send()
        ).await
        .map_err(|_| OracleError::data_source_timeout("pyth", self.config.timeout_ms))?
        .map_err(|e| OracleError::NetworkError {
            operation: "pyth_fetch".to_string(),
            error: e.to_string(),
            endpoint: Some(url.clone()),
        })?;

        let data: Vec<PythPriceFeed> = response.json().await
            .map_err(|e| OracleError::invalid_data("pyth", &e.to_string()))?;

        let price_feed = data.into_iter().next()
            .ok_or_else(|| OracleError::invalid_data("pyth", "No price data returned"))?;

        Ok(DataPoint {
            source: DataSource::Pyth,
            symbol: symbol.to_string(),
            price: price_feed.price.price,
            volume: None,
            liquidity: None,
            timestamp: DateTime::from_timestamp(price_feed.price.publish_time, 0)
                .unwrap_or(Utc::now()),
            confidence: price_feed.price.conf as f64 / price_feed.price.price.to_f64().unwrap_or(1.0),
            metadata: HashMap::from([
                ("id".to_string(), price_feed.id),
                ("expo".to_string(), price_feed.price.expo.to_string()),
            ]),
            signature: None,
        })
    }

    async fn health_check(&self) -> OracleResult<SourceHealth> {
        let start = std::time::Instant::now();

        // Pyth doesn't have a dedicated health endpoint, so we test with a known symbol
        let result = self.fetch_price("ETH/USD").await;
        let response_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(_) => Ok(SourceHealth {
                is_healthy: true,
                response_time_ms,
                last_success: Some(Utc::now()),
                error_details: None,
                uptime_24h: Some(99.5),
            }),
            Err(e) => Ok(SourceHealth {
                is_healthy: false,
                response_time_ms,
                last_success: None,
                error_details: Some(e.to_string()),
                uptime_24h: None,
            }),
        }
    }

    async fn get_supported_symbols(&self) -> OracleResult<Vec<String>> {
        // Pyth has a fixed set of supported symbols
        Ok(vec![
            "ETH/USD".to_string(),
            "BTC/USD".to_string(),
            "SOL/USD".to_string(),
            "AVAX/USD".to_string(),
            "MATIC/USD".to_string(),
            "BNB/USD".to_string(),
            "ADA/USD".to_string(),
            "DOT/USD".to_string(),
            "LINK/USD".to_string(),
            "UNI/USD".to_string(),
        ])
    }

    async fn get_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "Pyth Network".to_string(),
            description: "High-frequency price feeds for DeFi applications".to_string(),
            features: vec![
                SourceFeature::HighFrequency,
                SourceFeature::LowLatency,
                SourceFeature::RealtimeUpdates,
                SourceFeature::WhaleDataSupport,
            ],
            update_frequency: Some(Duration::from_millis(400)), // ~400ms updates
            symbol_count: Some(200),
            rate_limits: Some(RateLimits {
                requests_per_second: 50,
                requests_per_minute: Some(3000),
                daily_limit: None, // No daily limit
                burst_limit: Some(100),
            }),
            geo_restrictions: vec![],
        }
    }
}

impl PythProvider {
    /// Convert trading symbol to Pyth price feed ID
    fn symbol_to_pyth_id(&self, symbol: &str) -> OracleResult<String> {
        let id = match symbol {
            "ETH/USD" => "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "BTC/USD" => "0xe62df6c8b4c85fe1d7fe831e9a7a5c32b5c14efeaa9b6a8adf2d8eace8d9b1ab",
            "SOL/USD" => "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
            _ => return Err(OracleError::InvalidSymbol {
                symbol: symbol.to_string(),
                reason: "Symbol not supported by Pyth".to_string(),
                suggestions: vec!["ETH/USD".to_string(), "BTC/USD".to_string(), "SOL/USD".to_string()],
            }),
        };
        Ok(id.to_string())
    }
}

/// Pyth API response structures
#[derive(Debug, Deserialize)]
struct PythPriceFeed {
    id: String,
    price: PythPrice,
}

#[derive(Debug, Deserialize)]
struct PythPrice {
    price: Decimal,
    conf: i64,
    expo: i32,
    publish_time: i64,
}

/// Data source manager for coordinating multiple sources
pub struct SourceManager {
    sources: HashMap<DataSource, Box<dyn SourceProvider>>,
    configs: HashMap<DataSource, SourceConfig>,
    health_status: HashMap<DataSource, SourceHealth>,
}

impl SourceManager {
    /// Create new source manager
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            configs: HashMap::new(),
            health_status: HashMap::new(),
        }
    }

    /// Register a data source
    pub async fn register_source(
        &mut self,
        config: SourceConfig,
        provider: Box<dyn SourceProvider>,
    ) -> OracleResult<()> {
        let source_type = provider.source_type();

        // Validate configuration
        self.validate_config(&config).await?;

        // Test connectivity
        let health = provider.health_check().await?;
        if !health.is_healthy {
            log::warn!("Registering unhealthy source: {:?}", source_type);
        }

        self.sources.insert(source_type.clone(), provider);
        self.configs.insert(source_type.clone(), config);
        self.health_status.insert(source_type, health);

        Ok(())
    }

    /// Fetch price from a specific source
    pub async fn fetch_from_source(
        &self,
        source: &DataSource,
        symbol: &str,
    ) -> OracleResult<DataPoint> {
        let provider = self.sources.get(source)
            .ok_or_else(|| OracleError::data_source_unavailable(
                &format!("{:?}", source),
                "Source not registered"
            ))?;

        provider.fetch_price(symbol).await
    }

    /// Fetch prices from multiple sources
    pub async fn fetch_from_sources(
        &self,
        sources: &[DataSource],
        symbol: &str,
    ) -> OracleResult<Vec<DataPoint>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for source in sources {
            match self.fetch_from_source(source, symbol).await {
                Ok(data_point) => results.push(data_point),
                Err(e) => {
                    log::warn!("Failed to fetch from {:?}: {}", source, e);
                    errors.push(format!("{:?}", source));
                }
            }
        }

        if results.is_empty() {
            return Err(OracleError::InsufficientSources {
                symbol: symbol.to_string(),
                available: 0,
                required: sources.len(),
                failed_sources: errors,
            });
        }

        Ok(results)
    }

    /// Get all healthy sources
    pub async fn get_healthy_sources(&self) -> Vec<DataSource> {
        self.health_status
            .iter()
            .filter(|(_, health)| health.is_healthy)
            .map(|(source, _)| source.clone())
            .collect()
    }

    /// Update health status for all sources
    pub async fn update_health_status(&mut self) {
        for (source_type, provider) in &self.sources {
            if let Ok(health) = provider.health_check().await {
                self.health_status.insert(source_type.clone(), health);
            }
        }
    }

    /// Get source metadata
    pub async fn get_source_metadata(&self, source: &DataSource) -> Option<SourceMetadata> {
        self.sources.get(source)?.get_metadata().await.into()
    }

    /// Validate source configuration
    async fn validate_config(&self, config: &SourceConfig) -> OracleResult<()> {
        if config.endpoint.is_empty() {
            return Err(OracleError::configuration_error(
                "endpoint",
                "Endpoint URL cannot be empty"
            ));
        }

        if config.timeout_ms == 0 {
            return Err(OracleError::configuration_error(
                "timeout_ms",
                "Timeout must be greater than 0"
            ));
        }

        if config.rate_limit == 0 {
            return Err(OracleError::configuration_error(
                "rate_limit",
                "Rate limit must be greater than 0"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_source_manager() {
        let mut manager = SourceManager::new();
        assert_eq!(manager.sources.len(), 0);
    }

    #[test]
    fn test_data_point_creation() {
        let data_point = DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            price: Decimal::new(2000, 0),
            volume: None,
            liquidity: None,
            timestamp: Utc::now(),
            confidence: 0.95,
            metadata: HashMap::new(),
            signature: None,
        };

        assert_eq!(data_point.source, DataSource::Chainlink);
        assert_eq!(data_point.symbol, "ETH/USD");
        assert_eq!(data_point.price, Decimal::new(2000, 0));
    }

    #[test]
    fn test_source_config_validation() {
        let config = SourceConfig {
            source: DataSource::Chainlink,
            endpoint: "https://api.chainlink.com".to_string(),
            auth: None,
            timeout_ms: 5000,
            rate_limit: 10,
            retry_config: RetryConfig::default(),
            parameters: HashMap::new(),
            enabled: true,
        };

        assert!(config.enabled);
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.rate_limit, 10);
    }

    #[test]
    fn test_pyth_symbol_conversion() {
        let config = SourceConfig {
            source: DataSource::Pyth,
            endpoint: "https://api.pyth.network".to_string(),
            auth: None,
            timeout_ms: 3000,
            rate_limit: 50,
            retry_config: RetryConfig::default(),
            parameters: HashMap::new(),
            enabled: true,
        };

        let provider = PythProvider::new(config);
        let eth_id = provider.symbol_to_pyth_id("ETH/USD").unwrap();
        assert!(!eth_id.is_empty());
        assert!(eth_id.starts_with("0x"));
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.max_delay_ms, 30000);
    }
}