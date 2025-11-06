//! # Unified Moby Market Platform
//!
//! The main platform orchestrator that coordinates all components and provides
//! the primary interface for whale trading operations.

use crate::{Result, PlatformError, integration::ComponentManager, strategies::StrategyEngine};
use moby_privacy::PrivacyEngine;
use moby_governance::GovernanceEngine;
use moby_bridge::BridgeEngine;
use moby_oracle::OracleEngine;
use moby_dex::DEXEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Main platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Platform name and version
    pub platform_info: PlatformInfo,

    /// Database connection configuration
    pub database: DatabaseConfig,

    /// API server configuration
    pub api: ApiConfig,

    /// Component-specific configurations
    pub components: ComponentConfigs,

    /// Trading parameters
    pub trading: TradingConfig,

    /// Revenue and fee configuration
    pub revenue: RevenueConfig,

    /// Security and compliance settings
    pub security: SecurityConfig,

    /// Monitoring and analytics
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub name: String,
    pub version: String,
    pub environment: String, // dev, staging, production
    pub deployment_id: String,
    pub launched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub cors_enabled: bool,
    pub rate_limit_per_minute: u32,
    pub websocket_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfigs {
    pub privacy_enabled: bool,
    pub governance_enabled: bool,
    pub bridge_enabled: bool,
    pub oracle_enabled: bool,
    pub dex_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub max_trade_size_usd: f64,
    pub min_trade_size_usd: f64,
    pub whale_threshold_usd: f64,
    pub max_slippage_percentage: f64,
    pub max_price_impact_percentage: f64,
    pub default_deadline_minutes: u32,
    pub mev_protection_enabled: bool,
    pub cross_chain_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueConfig {
    pub platform_fee_percentage: f64,
    pub whale_fee_discount_percentage: f64,
    pub liquidity_provision_fee_percentage: f64,
    pub cross_chain_fee_percentage: f64,
    pub premium_features_monthly_usd: f64,
    pub governance_revenue_share_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub kyc_required_threshold_usd: f64,
    pub sanctions_screening_enabled: bool,
    pub geographic_restrictions: Vec<String>,
    pub blacklisted_addresses: Vec<String>,
    pub max_daily_volume_usd: f64,
    pub audit_logging_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub prometheus_endpoint: String,
    pub health_check_interval_seconds: u32,
    pub alert_webhook_url: Option<String>,
    pub performance_tracking_enabled: bool,
}

/// Current state of the platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformState {
    pub status: PlatformStatus,
    pub uptime_seconds: u64,
    pub active_users: u32,
    pub total_trades_24h: u32,
    pub total_volume_24h_usd: Decimal,
    pub total_fees_24h_usd: Decimal,
    pub component_status: HashMap<String, ComponentStatus>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlatformStatus {
    Initializing,
    Healthy,
    Degraded,
    Maintenance,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub enabled: bool,
    pub healthy: bool,
    pub last_health_check: DateTime<Utc>,
    pub error_message: Option<String>,
}

/// Trade execution request with all parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRequest {
    pub trade_id: String,
    pub user_id: String,
    pub pair: String,
    pub amount_in: Decimal,
    pub token_in: String,
    pub min_amount_out: Decimal,
    pub max_slippage: f64,
    pub deadline: DateTime<Utc>,
    pub privacy_level: PrivacyLevel,
    pub strategy: TradingStrategy,
    pub cross_chain_enabled: bool,
    pub mev_protection_enabled: bool,
    pub split_trade: bool,
    pub max_parts: u32,
    pub custom_parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    None,       // Public trades visible on-chain
    Basic,      // Basic obfuscation
    Enhanced,   // Advanced privacy techniques
    Full,       // Complete zero-knowledge privacy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingStrategy {
    MinimizeSlippage,
    MinimizeFees,
    MaximizeSpeed,
    MaximizePrivacy,
    CrossChainArbitrage,
    YieldOptimization,
    Custom(String),
}

/// Complete trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub trade_id: String,
    pub status: TradeStatus,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub fees_paid: Decimal,
    pub slippage: f64,
    pub price_impact: f64,
    pub execution_time_ms: u64,
    pub routes_used: Vec<TradeRoute>,
    pub privacy_applied: bool,
    pub cross_chain_executed: bool,
    pub mev_protection_triggered: bool,
    pub profit_usd: Decimal,
    pub gas_costs: HashMap<String, Decimal>,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeStatus {
    Pending,
    Executing,
    Completed,
    PartiallyFilled,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRoute {
    pub chain: String,
    pub dex: String,
    pub pool_id: String,
    pub amount_percentage: f64,
    pub estimated_output: Decimal,
    pub fees: Decimal,
}

/// Main platform orchestrator
pub struct MobyMarket {
    config: PlatformConfig,
    state: PlatformState,
    component_manager: ComponentManager,
    strategy_engine: StrategyEngine,

    // Core components
    privacy_engine: Option<PrivacyEngine>,
    governance_engine: Option<GovernanceEngine>,
    bridge_engine: Option<BridgeEngine>,
    oracle_engine: Option<OracleEngine>,
    dex_engine: Option<DEXEngine>,

    // Runtime state
    startup_time: DateTime<Utc>,
    active_trades: HashMap<String, TradeRequest>,
    user_sessions: HashMap<String, UserSession>,
}

#[derive(Debug, Clone)]
struct UserSession {
    user_id: String,
    authenticated_at: DateTime<Utc>,
    permissions: Vec<String>,
    active_trades: Vec<String>,
    total_volume: Decimal,
}

impl MobyMarket {
    /// Create a new Moby Market platform instance
    pub async fn new() -> Result<Self> {
        let config = Self::default_config();
        Self::with_config(config).await
    }

    /// Create platform with custom configuration
    pub async fn with_config(config: PlatformConfig) -> Result<Self> {
        let startup_time = Utc::now();

        // Initialize component manager
        let component_manager = ComponentManager::new(&config).await
            .map_err(|e| PlatformError::ComponentInitializationFailed {
                component: format!("ComponentManager: {}", e),
            })?;

        // Initialize strategy engine
        let strategy_engine = StrategyEngine::new(&config.trading).await
            .map_err(|e| PlatformError::ComponentInitializationFailed {
                component: format!("StrategyEngine: {}", e),
            })?;

        // Initialize core components based on configuration
        let privacy_engine = if config.components.privacy_enabled {
            Some(PrivacyEngine::new().await.map_err(|e| {
                PlatformError::ComponentInitializationFailed {
                    component: format!("PrivacyEngine: {}", e),
                }
            })?)
        } else {
            None
        };

        let governance_engine = if config.components.governance_enabled {
            Some(GovernanceEngine::new().await.map_err(|e| {
                PlatformError::ComponentInitializationFailed {
                    component: format!("GovernanceEngine: {}", e),
                }
            })?)
        } else {
            None
        };

        let bridge_engine = if config.components.bridge_enabled {
            Some(BridgeEngine::new().await.map_err(|e| {
                PlatformError::ComponentInitializationFailed {
                    component: format!("BridgeEngine: {}", e),
                }
            })?)
        } else {
            None
        };

        let oracle_engine = if config.components.oracle_enabled {
            Some(OracleEngine::new().await.map_err(|e| {
                PlatformError::ComponentInitializationFailed {
                    component: format!("OracleEngine: {}", e),
                }
            })?)
        } else {
            None
        };

        let dex_engine = if config.components.dex_enabled {
            Some(DEXEngine::new().await.map_err(|e| {
                PlatformError::ComponentInitializationFailed {
                    component: format!("DEXEngine: {}", e),
                }
            })?)
        } else {
            None
        };

        // Initialize platform state
        let mut component_status = HashMap::new();
        component_status.insert("privacy".to_string(), ComponentStatus {
            enabled: privacy_engine.is_some(),
            healthy: true,
            last_health_check: startup_time,
            error_message: None,
        });
        component_status.insert("governance".to_string(), ComponentStatus {
            enabled: governance_engine.is_some(),
            healthy: true,
            last_health_check: startup_time,
            error_message: None,
        });
        component_status.insert("bridge".to_string(), ComponentStatus {
            enabled: bridge_engine.is_some(),
            healthy: true,
            last_health_check: startup_time,
            error_message: None,
        });
        component_status.insert("oracle".to_string(), ComponentStatus {
            enabled: oracle_engine.is_some(),
            healthy: true,
            last_health_check: startup_time,
            error_message: None,
        });
        component_status.insert("dex".to_string(), ComponentStatus {
            enabled: dex_engine.is_some(),
            healthy: true,
            last_health_check: startup_time,
            error_message: None,
        });

        let state = PlatformState {
            status: PlatformStatus::Healthy,
            uptime_seconds: 0,
            active_users: 0,
            total_trades_24h: 0,
            total_volume_24h_usd: Decimal::ZERO,
            total_fees_24h_usd: Decimal::ZERO,
            component_status,
            last_updated: startup_time,
        };

        Ok(Self {
            config,
            state,
            component_manager,
            strategy_engine,
            privacy_engine,
            governance_engine,
            bridge_engine,
            oracle_engine,
            dex_engine,
            startup_time,
            active_trades: HashMap::new(),
            user_sessions: HashMap::new(),
        })
    }

    /// Execute a whale trade with full platform integration
    pub async fn execute_whale_trade(&mut self, request: TradeRequest) -> Result<TradeResult> {
        let start_time = std::time::Instant::now();

        // Validate request
        self.validate_trade_request(&request).await?;

        // Register active trade
        self.active_trades.insert(request.trade_id.clone(), request.clone());

        // Step 1: Get real-time pricing from oracle
        let pricing_data = if let Some(ref oracle) = self.oracle_engine {
            oracle.get_pricing(&request.pair).await?
        } else {
            return Err(PlatformError::ComponentInitializationFailed {
                component: "Oracle engine required for pricing".to_string(),
            });
        };

        // Step 2: Analyze optimal strategy using strategy engine
        let strategy_params = self.strategy_engine.analyze_trade(
            &request,
            &pricing_data,
        ).await?;

        // Step 3: Apply privacy protection if requested
        let privacy_result = if request.privacy_level != PrivacyLevel::None {
            if let Some(ref privacy) = self.privacy_engine {
                Some(privacy.prepare_private_trade(&request).await?)
            } else {
                return Err(PlatformError::PrivacyError {
                    operation: "Privacy engine not available".to_string(),
                });
            }
        } else {
            None
        };

        // Step 4: Check for cross-chain opportunities
        let cross_chain_routes = if request.cross_chain_enabled {
            if let Some(ref bridge) = self.bridge_engine {
                bridge.find_optimal_routes(&request).await?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Step 5: Execute trade through DEX engine
        let execution_result = if let Some(ref dex) = self.dex_engine {
            dex.execute_optimized_trade(
                &request,
                &strategy_params,
                privacy_result.as_ref(),
                &cross_chain_routes,
            ).await?
        } else {
            return Err(PlatformError::DEXError {
                dex: "platform".to_string(),
                operation: "DEX engine required for execution".to_string(),
            });
        };

        // Step 6: Update platform state and metrics
        self.update_trade_metrics(&execution_result);

        // Step 7: Remove from active trades
        self.active_trades.remove(&request.trade_id);

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Construct final result
        let trade_result = TradeResult {
            trade_id: request.trade_id,
            status: TradeStatus::Completed,
            amount_in: execution_result.amount_in,
            amount_out: execution_result.amount_out,
            fees_paid: execution_result.total_fees,
            slippage: execution_result.actual_slippage,
            price_impact: execution_result.price_impact,
            execution_time_ms: execution_time,
            routes_used: execution_result.routes,
            privacy_applied: privacy_result.is_some(),
            cross_chain_executed: !cross_chain_routes.is_empty(),
            mev_protection_triggered: execution_result.mev_protection_used,
            profit_usd: execution_result.estimated_profit,
            gas_costs: execution_result.gas_costs,
            executed_at: Utc::now(),
        };

        Ok(trade_result)
    }

    /// Create a trade request builder for fluent API
    pub fn trade(&self) -> TradeRequestBuilder {
        TradeRequestBuilder::new()
    }

    /// Get current platform state
    pub fn get_state(&self) -> &PlatformState {
        &self.state
    }

    /// Get platform configuration
    pub fn get_config(&self) -> &PlatformConfig {
        &self.config
    }

    /// Get real-time market analytics
    pub async fn get_market_analytics(&self) -> Result<MarketAnalytics> {
        // Aggregate data from all components
        let whale_activity = if let Some(ref oracle) = self.oracle_engine {
            oracle.get_whale_activity().await?
        } else {
            Default::default()
        };

        let cross_chain_opportunities = if let Some(ref bridge) = self.bridge_engine {
            bridge.get_arbitrage_opportunities().await?
        } else {
            Vec::new()
        };

        let dex_liquidity = if let Some(ref dex) = self.dex_engine {
            dex.get_liquidity_overview().await?
        } else {
            Default::default()
        };

        Ok(MarketAnalytics {
            whale_activity,
            cross_chain_opportunities,
            dex_liquidity,
            platform_metrics: self.state.clone(),
            generated_at: Utc::now(),
        })
    }

    /// Authenticate and create user session
    pub async fn authenticate_user(&mut self, credentials: UserCredentials) -> Result<String> {
        // Validate credentials (simplified)
        let user_id = credentials.user_id.clone();

        // Create session
        let session = UserSession {
            user_id: user_id.clone(),
            authenticated_at: Utc::now(),
            permissions: credentials.permissions,
            active_trades: Vec::new(),
            total_volume: Decimal::ZERO,
        };

        self.user_sessions.insert(user_id.clone(), session);

        // Return session token (simplified)
        Ok(format!("session_{}", Uuid::new_v4()))
    }

    /// Update platform health status
    pub async fn update_health_status(&mut self) -> Result<()> {
        let now = Utc::now();

        // Check each component's health
        for (name, status) in &mut self.state.component_status {
            match name.as_str() {
                "privacy" => {
                    if let Some(ref privacy) = self.privacy_engine {
                        status.healthy = privacy.health_check().await.is_ok();
                    }
                }
                "governance" => {
                    if let Some(ref governance) = self.governance_engine {
                        status.healthy = governance.health_check().await.is_ok();
                    }
                }
                "bridge" => {
                    if let Some(ref bridge) = self.bridge_engine {
                        status.healthy = bridge.health_check().await.is_ok();
                    }
                }
                "oracle" => {
                    if let Some(ref oracle) = self.oracle_engine {
                        status.healthy = oracle.health_check().await.is_ok();
                    }
                }
                "dex" => {
                    if let Some(ref dex) = self.dex_engine {
                        status.healthy = dex.health_check().await.is_ok();
                    }
                }
                _ => {}
            }
            status.last_health_check = now;
        }

        // Update overall platform status
        let all_healthy = self.state.component_status.values()
            .filter(|s| s.enabled)
            .all(|s| s.healthy);

        self.state.status = if all_healthy {
            PlatformStatus::Healthy
        } else {
            PlatformStatus::Degraded
        };

        self.state.uptime_seconds = now.signed_duration_since(self.startup_time).num_seconds() as u64;
        self.state.last_updated = now;

        Ok(())
    }

    /// Get default platform configuration
    fn default_config() -> PlatformConfig {
        PlatformConfig {
            platform_info: PlatformInfo {
                name: crate::PLATFORM_NAME.to_string(),
                version: crate::PLATFORM_VERSION.to_string(),
                environment: "development".to_string(),
                deployment_id: Uuid::new_v4().to_string(),
                launched_at: Utc::now(),
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/moby_market".to_string(),
                max_connections: 20,
                timeout_seconds: 30,
                retry_attempts: 3,
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                tls_enabled: false,
                cors_enabled: true,
                rate_limit_per_minute: 1000,
                websocket_enabled: true,
            },
            components: ComponentConfigs {
                privacy_enabled: true,
                governance_enabled: true,
                bridge_enabled: true,
                oracle_enabled: true,
                dex_enabled: true,
            },
            trading: TradingConfig {
                max_trade_size_usd: crate::DEFAULT_MAX_TRADE_SIZE,
                min_trade_size_usd: crate::DEFAULT_MIN_TRADE_SIZE,
                whale_threshold_usd: crate::DEFAULT_WHALE_THRESHOLD,
                max_slippage_percentage: crate::DEFAULT_MAX_SLIPPAGE,
                max_price_impact_percentage: 0.10, // 10%
                default_deadline_minutes: 30,
                mev_protection_enabled: true,
                cross_chain_enabled: true,
            },
            revenue: RevenueConfig {
                platform_fee_percentage: crate::DEFAULT_PLATFORM_FEE,
                whale_fee_discount_percentage: 0.2, // 20% discount for whales
                liquidity_provision_fee_percentage: 0.1, // 10% of trading fees
                cross_chain_fee_percentage: 0.05, // 5% for cross-chain operations
                premium_features_monthly_usd: 1000.0,
                governance_revenue_share_percentage: 30.0, // 30% to governance
            },
            security: SecurityConfig {
                kyc_required_threshold_usd: 100_000.0,
                sanctions_screening_enabled: true,
                geographic_restrictions: vec!["US".to_string(), "CN".to_string()],
                blacklisted_addresses: Vec::new(),
                max_daily_volume_usd: 10_000_000.0,
                audit_logging_enabled: true,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                prometheus_endpoint: "/metrics".to_string(),
                health_check_interval_seconds: 30,
                alert_webhook_url: None,
                performance_tracking_enabled: true,
            },
        }
    }

    async fn validate_trade_request(&self, request: &TradeRequest) -> Result<()> {
        // Amount validation
        let amount_usd = request.amount_in.to_string().parse::<f64>().unwrap_or(0.0);
        if amount_usd < self.config.trading.min_trade_size_usd {
            return Err(PlatformError::ConfigurationError {
                parameter: "trade_amount".to_string(),
                value: format!("${} below minimum ${}", amount_usd, self.config.trading.min_trade_size_usd),
            });
        }

        if amount_usd > self.config.trading.max_trade_size_usd {
            return Err(PlatformError::ConfigurationError {
                parameter: "trade_amount".to_string(),
                value: format!("${} exceeds maximum ${}", amount_usd, self.config.trading.max_trade_size_usd),
            });
        }

        // Deadline validation
        if request.deadline < Utc::now() {
            return Err(PlatformError::ConfigurationError {
                parameter: "deadline".to_string(),
                value: "Trade deadline is in the past".to_string(),
            });
        }

        // User session validation
        if !self.user_sessions.contains_key(&request.user_id) {
            return Err(PlatformError::AuthenticationFailed {
                user_id: request.user_id.clone(),
            });
        }

        Ok(())
    }

    fn update_trade_metrics(&mut self, execution_result: &ExecutionResult) {
        self.state.total_trades_24h += 1;
        self.state.total_volume_24h_usd += execution_result.amount_in;
        self.state.total_fees_24h_usd += execution_result.total_fees;
        self.state.last_updated = Utc::now();
    }
}

/// Builder for creating trade requests with fluent API
pub struct TradeRequestBuilder {
    trade_id: String,
    user_id: Option<String>,
    pair: Option<String>,
    amount_in: Option<Decimal>,
    token_in: Option<String>,
    min_amount_out: Option<Decimal>,
    max_slippage: f64,
    deadline: Option<DateTime<Utc>>,
    privacy_level: PrivacyLevel,
    strategy: TradingStrategy,
    cross_chain_enabled: bool,
    mev_protection_enabled: bool,
    split_trade: bool,
    max_parts: u32,
    custom_parameters: HashMap<String, serde_json::Value>,
}

impl TradeRequestBuilder {
    pub fn new() -> Self {
        Self {
            trade_id: Uuid::new_v4().to_string(),
            user_id: None,
            pair: None,
            amount_in: None,
            token_in: None,
            min_amount_out: None,
            max_slippage: 0.05, // 5% default
            deadline: None,
            privacy_level: PrivacyLevel::None,
            strategy: TradingStrategy::MinimizeSlippage,
            cross_chain_enabled: false,
            mev_protection_enabled: true,
            split_trade: false,
            max_parts: 1,
            custom_parameters: HashMap::new(),
        }
    }

    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn pair(mut self, pair: &str) -> Self {
        self.pair = Some(pair.to_string());
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount_in = Some(Decimal::from(amount));
        self
    }

    pub fn token_in(mut self, token: &str) -> Self {
        self.token_in = Some(token.to_string());
        self
    }

    pub fn min_amount_out(mut self, amount: u64) -> Self {
        self.min_amount_out = Some(Decimal::from(amount));
        self
    }

    pub fn max_slippage(mut self, slippage: f64) -> Self {
        self.max_slippage = slippage;
        self
    }

    pub fn deadline_minutes(mut self, minutes: i64) -> Self {
        self.deadline = Some(Utc::now() + chrono::Duration::minutes(minutes));
        self
    }

    pub fn privacy_level(mut self, level: PrivacyLevel) -> Self {
        self.privacy_level = level;
        self
    }

    pub fn strategy(mut self, strategy: TradingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn cross_chain_enabled(mut self, enabled: bool) -> Self {
        self.cross_chain_enabled = enabled;
        self
    }

    pub fn mev_protection(mut self, enabled: bool) -> Self {
        self.mev_protection_enabled = enabled;
        self
    }

    pub fn split_trade(mut self, enabled: bool, max_parts: u32) -> Self {
        self.split_trade = enabled;
        self.max_parts = max_parts;
        self
    }

    pub fn build(self) -> Result<TradeRequest> {
        Ok(TradeRequest {
            trade_id: self.trade_id,
            user_id: self.user_id.ok_or_else(|| PlatformError::ConfigurationError {
                parameter: "user_id".to_string(),
                value: "missing".to_string(),
            })?,
            pair: self.pair.ok_or_else(|| PlatformError::ConfigurationError {
                parameter: "pair".to_string(),
                value: "missing".to_string(),
            })?,
            amount_in: self.amount_in.ok_or_else(|| PlatformError::ConfigurationError {
                parameter: "amount_in".to_string(),
                value: "missing".to_string(),
            })?,
            token_in: self.token_in.ok_or_else(|| PlatformError::ConfigurationError {
                parameter: "token_in".to_string(),
                value: "missing".to_string(),
            })?,
            min_amount_out: self.min_amount_out.unwrap_or(Decimal::ZERO),
            max_slippage: self.max_slippage,
            deadline: self.deadline.unwrap_or_else(|| Utc::now() + chrono::Duration::minutes(30)),
            privacy_level: self.privacy_level,
            strategy: self.strategy,
            cross_chain_enabled: self.cross_chain_enabled,
            mev_protection_enabled: self.mev_protection_enabled,
            split_trade: self.split_trade,
            max_parts: self.max_parts,
            custom_parameters: self.custom_parameters,
        })
    }
}

// Supporting types for demonstration
#[derive(Debug, Clone)]
pub struct UserCredentials {
    pub user_id: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MarketAnalytics {
    pub whale_activity: serde_json::Value,
    pub cross_chain_opportunities: Vec<serde_json::Value>,
    pub dex_liquidity: serde_json::Value,
    pub platform_metrics: PlatformState,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub total_fees: Decimal,
    pub actual_slippage: f64,
    pub price_impact: f64,
    pub routes: Vec<TradeRoute>,
    pub mev_protection_used: bool,
    pub estimated_profit: Decimal,
    pub gas_costs: HashMap<String, Decimal>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_platform_initialization() {
        let platform = MobyMarket::new().await;
        assert!(platform.is_ok());

        let platform = platform.unwrap();
        assert_eq!(platform.get_state().status, PlatformStatus::Healthy);
    }

    #[test]
    fn test_trade_request_builder() {
        let request = TradeRequestBuilder::new()
            .user_id("user123")
            .pair("ETH/USDC")
            .amount(1000000) // $1M
            .token_in("ETH")
            .privacy_level(PrivacyLevel::Full)
            .strategy(TradingStrategy::MinimizeSlippage)
            .cross_chain_enabled(true)
            .build();

        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.user_id, "user123");
        assert_eq!(request.pair, "ETH/USDC");
        assert_eq!(request.privacy_level, PrivacyLevel::Full);
        assert!(request.cross_chain_enabled);
    }

    #[test]
    fn test_default_config() {
        let config = MobyMarket::default_config();
        assert_eq!(config.platform_info.name, "Moby Market");
        assert!(config.components.privacy_enabled);
        assert!(config.components.governance_enabled);
        assert!(config.trading.mev_protection_enabled);
    }
}