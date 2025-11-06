use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, ws::WebSocket},
    http::{StatusCode, HeaderMap},
    response::{Json, Response},
    routing::{get, post, put, delete},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use tokio_tungstenite::tungstenite::Message;

use crate::{
    Result, PlatformError,
    strategies::{TradeRequest, StrategyResult, StrategyParameters},
    analytics::{MarketAnalytics, WhaleTracking, RealTimeMonitoring, TradingSignals, MarketIntelligence},
    revenue::{RevenueMetrics, YieldRecommendation},
    platform::MobyMarket,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key_id: String,
    pub user_id: String,
    pub permissions: Vec<ApiPermission>,
    pub rate_limit: RateLimit,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiPermission {
    ReadMarketData,
    ExecuteTrades,
    AccessAnalytics,
    ManageWallet,
    AccessWhaleData,
    CreateStrategies,
    AccessPremiumFeatures,
    AdminAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub concurrent_requests: u32,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            concurrent_requests: 10,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
            request_id: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TradeExecutionRequest {
    pub token_in: String,
    pub token_out: String,
    pub amount_in: Decimal,
    pub max_slippage: Option<Decimal>,
    pub strategy_preference: Option<String>,
    pub privacy_level: Option<String>,
    pub cross_chain_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TradeExecutionResponse {
    pub trade_id: String,
    pub status: String,
    pub estimated_output: Decimal,
    pub estimated_slippage: Decimal,
    pub estimated_gas_fee: Decimal,
    pub execution_steps: Vec<ExecutionStepResponse>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionStepResponse {
    pub step_id: String,
    pub description: String,
    pub status: String,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub timeframe: Option<String>,
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketAuth {
    pub api_key: String,
    pub subscriptions: Vec<String>,
}

pub struct ApiServer {
    router: Router,
    platform: Arc<RwLock<MobyMarket>>,
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
    websocket_connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
}

#[derive(Debug)]
struct RateLimiter {
    requests_this_minute: u32,
    requests_this_hour: u32,
    requests_this_day: u32,
    last_reset_minute: DateTime<Utc>,
    last_reset_hour: DateTime<Utc>,
    last_reset_day: DateTime<Utc>,
}

#[derive(Debug)]
struct WebSocketConnection {
    user_id: String,
    subscriptions: Vec<String>,
    last_heartbeat: DateTime<Utc>,
}

impl ApiServer {
    pub fn new(platform: Arc<RwLock<MobyMarket>>) -> Self {
        let api_keys = Arc::new(RwLock::new(HashMap::new()));
        let rate_limiters = Arc::new(RwLock::new(HashMap::new()));
        let websocket_connections = Arc::new(RwLock::new(HashMap::new()));

        let app_state = AppState {
            platform: platform.clone(),
            api_keys: api_keys.clone(),
            rate_limiters: rate_limiters.clone(),
            websocket_connections: websocket_connections.clone(),
        };

        let router = create_router(app_state);

        Self {
            router,
            platform,
            api_keys,
            rate_limiters,
            websocket_connections,
        }
    }

    pub async fn serve(self, port: u16) -> Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| PlatformError::InternalError {
                details: format!("Failed to bind to {}: {}", addr, e),
            })?;

        println!("=€ Moby Market API Server running on http://{}", addr);

        axum::serve(listener, self.router).await
            .map_err(|e| PlatformError::InternalError {
                details: format!("Server error: {}", e),
            })?;

        Ok(())
    }

    pub async fn create_api_key(
        &self,
        user_id: String,
        permissions: Vec<ApiPermission>,
        rate_limit: Option<RateLimit>,
    ) -> String {
        let key_id = format!("mk_{}", Uuid::new_v4().simple());
        let api_key = ApiKey {
            key_id: key_id.clone(),
            user_id,
            permissions,
            rate_limit: rate_limit.unwrap_or_default(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            is_active: true,
        };

        self.api_keys.write().await.insert(key_id.clone(), api_key);
        key_id
    }
}

#[derive(Clone)]
struct AppState {
    platform: Arc<RwLock<MobyMarket>>,
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
    websocket_connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
}

fn create_router(state: AppState) -> Router {
    Router::new()
        // Market Data Endpoints
        .route("/api/v1/market/analytics", get(get_market_analytics))
        .route("/api/v1/market/whale-tracking", get(get_whale_tracking))
        .route("/api/v1/market/signals", get(get_trading_signals))
        .route("/api/v1/market/intelligence", get(get_market_intelligence))

        // Trading Endpoints
        .route("/api/v1/trading/execute", post(execute_trade))
        .route("/api/v1/trading/strategy/analyze", post(analyze_strategy))
        .route("/api/v1/trading/history", get(get_trading_history))
        .route("/api/v1/trading/positions", get(get_positions))

        // Yield & Revenue Endpoints
        .route("/api/v1/yield/opportunities", get(get_yield_opportunities))
        .route("/api/v1/yield/recommendations", post(get_yield_recommendations))
        .route("/api/v1/revenue/metrics", get(get_revenue_metrics))

        // User & Account Endpoints
        .route("/api/v1/user/profile", get(get_user_profile))
        .route("/api/v1/user/api-keys", post(create_user_api_key))
        .route("/api/v1/user/subscription", get(get_user_subscription))
        .route("/api/v1/user/subscription", post(update_user_subscription))

        // Analytics Endpoints
        .route("/api/v1/analytics/monitoring", get(get_monitoring_data))
        .route("/api/v1/analytics/performance", get(get_performance_metrics))
        .route("/api/v1/analytics/whale/:whale_id", get(get_whale_details))

        // WebSocket Endpoint
        .route("/api/v1/ws", get(websocket_handler))

        // Health & Status
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/status", get(system_status))

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .into_inner()
        )
        .with_state(state)
}

// Market Data Handlers
async fn get_market_analytics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<MarketAnalytics>>, StatusCode> {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let platform = state.platform.read().await;
    match platform.get_analytics_engine().get_current_market_analytics().await {
        Some(analytics) => Ok(Json(ApiResponse::success(analytics))),
        None => Ok(Json(ApiResponse::error("No market analytics available".to_string()))),
    }
}

async fn get_whale_tracking(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<WhaleTracking>>, StatusCode> {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let platform = state.platform.read().await;
    match platform.get_analytics_engine().get_current_whale_tracking().await {
        Some(tracking) => Ok(Json(ApiResponse::success(tracking))),
        None => Ok(Json(ApiResponse::error("No whale tracking data available".to_string()))),
    }
}

async fn get_trading_signals(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<TradingSignals>>, StatusCode> {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let platform = state.platform.read().await;
    match platform.get_analytics_engine().get_current_signals().await {
        Some(signals) => Ok(Json(ApiResponse::success(signals))),
        None => Ok(Json(ApiResponse::error("No trading signals available".to_string()))),
    }
}

async fn get_market_intelligence(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MarketIntelligence>>, StatusCode> {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let platform = state.platform.read().await;
    match platform.get_analytics_engine().get_current_intelligence().await {
        Some(intelligence) => Ok(Json(ApiResponse::success(intelligence))),
        None => Ok(Json(ApiResponse::error("No market intelligence available".to_string()))),
    }
}

// Trading Handlers
async fn execute_trade(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TradeExecutionRequest>,
) -> Result<Json<ApiResponse<TradeExecutionResponse>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !api_key.permissions.contains(&ApiPermission::ExecuteTrades) {
        return Err(StatusCode::FORBIDDEN);
    }

    let trade_request = TradeRequest {
        id: Uuid::new_v4(),
        user_id: api_key.user_id.clone(),
        token_in: request.token_in,
        token_out: request.token_out,
        amount_in: request.amount_in,
        target_chains: vec!["ethereum".to_string()], // Default
        max_slippage: request.max_slippage.unwrap_or(Decimal::new(5, 2)), // 5%
        deadline: Utc::now() + chrono::Duration::hours(1),
        privacy_requirements: match request.privacy_level.as_deref() {
            Some("high") => moby_privacy::PrivacyLevel::High,
            Some("medium") => moby_privacy::PrivacyLevel::Medium,
            _ => moby_privacy::PrivacyLevel::Low,
        },
        strategy_preferences: request.strategy_preference.map(|s| vec![s]).unwrap_or_default(),
    };

    let mut platform = state.platform.write().await;
    match platform.execute_whale_trade(trade_request).await {
        Ok(result) => {
            let response = TradeExecutionResponse {
                trade_id: result.trade_id.to_string(),
                status: "executed".to_string(),
                estimated_output: result.output_amount,
                estimated_slippage: result.actual_slippage,
                estimated_gas_fee: result.gas_used,
                execution_steps: result.execution_path.iter().map(|step| ExecutionStepResponse {
                    step_id: step.step_id.to_string(),
                    description: format!("{:?}", step.action),
                    status: "completed".to_string(),
                    transaction_hash: Some("0x...".to_string()), // Would be real hash
                }).collect(),
            };
            Ok(Json(ApiResponse::success(response)))
        },
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

async fn analyze_strategy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TradeExecutionRequest>,
) -> Result<Json<ApiResponse<StrategyResult>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let trade_request = TradeRequest {
        id: Uuid::new_v4(),
        user_id: api_key.user_id.clone(),
        token_in: request.token_in,
        token_out: request.token_out,
        amount_in: request.amount_in,
        target_chains: vec!["ethereum".to_string()],
        max_slippage: request.max_slippage.unwrap_or(Decimal::new(5, 2)),
        deadline: Utc::now() + chrono::Duration::hours(1),
        privacy_requirements: moby_privacy::PrivacyLevel::Medium,
        strategy_preferences: vec![],
    };

    let platform = state.platform.read().await;
    match platform.analyze_trade_strategy(trade_request, StrategyParameters::default()).await {
        Ok(result) => Ok(Json(ApiResponse::success(result))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

async fn get_trading_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Mock trading history
    let history = vec![
        serde_json::json!({
            "trade_id": "trade_001",
            "timestamp": Utc::now() - chrono::Duration::hours(2),
            "pair": "ETH/USDC",
            "amount": "5000000",
            "profit": "115000",
            "status": "completed"
        }),
        serde_json::json!({
            "trade_id": "trade_002",
            "timestamp": Utc::now() - chrono::Duration::hours(6),
            "pair": "BTC/USDC",
            "amount": "2000000",
            "profit": "45000",
            "status": "completed"
        }),
    ];

    Ok(Json(ApiResponse::success(history)))
}

async fn get_positions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Mock positions
    let positions = vec![
        serde_json::json!({
            "position_id": "pos_001",
            "asset": "ETH",
            "amount": "100.5",
            "value_usd": "320000",
            "pnl": "15000",
            "pnl_percentage": "4.9"
        }),
    ];

    Ok(Json(ApiResponse::success(positions)))
}

// Yield & Revenue Handlers
async fn get_yield_opportunities(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Mock yield opportunities
    let opportunities = vec![
        serde_json::json!({
            "protocol": "Uniswap V3",
            "pair": "ETH/USDC",
            "apy": "28.5",
            "tvl": "500000000",
            "risk_score": "0.3"
        }),
    ];

    Ok(Json(ApiResponse::success(opportunities)))
}

async fn get_yield_recommendations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(portfolio): Json<HashMap<String, Decimal>>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Mock recommendations
    let recommendations = vec![
        serde_json::json!({
            "asset": "ETH",
            "recommended_strategy": "Uniswap V3 LP",
            "estimated_apy": "28.5",
            "recommended_amount": "80000",
            "confidence": "0.85"
        }),
    ];

    Ok(Json(ApiResponse::success(recommendations)))
}

async fn get_revenue_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<RevenueMetrics>>, StatusCode> {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let platform = state.platform.read().await;
    let metrics = platform.get_revenue_engine().get_revenue_metrics().await;
    Ok(Json(ApiResponse::success(metrics)))
}

// User & Account Handlers
async fn get_user_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let profile = serde_json::json!({
        "user_id": api_key.user_id,
        "tier": "whale",
        "total_volume": "250000000",
        "total_profit": "2800000",
        "win_rate": "0.85",
        "created_at": "2024-01-01T00:00:00Z"
    });

    Ok(Json(ApiResponse::success(profile)))
}

async fn create_user_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Create new API key (simplified)
    let new_key = format!("mk_{}", Uuid::new_v4().simple());
    let response = serde_json::json!({
        "api_key": new_key,
        "permissions": ["ReadMarketData", "ExecuteTrades"],
        "rate_limit": {
            "requests_per_minute": 100,
            "requests_per_hour": 1000
        }
    });

    Ok(Json(ApiResponse::success(response)))
}

async fn get_user_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let subscription = serde_json::json!({
        "plan": "whale",
        "status": "active",
        "next_billing": "2024-12-01T00:00:00Z",
        "features_used": ["PriorityExecution", "AdvancedAnalytics"]
    });

    Ok(Json(ApiResponse::success(subscription)))
}

async fn update_user_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let response = serde_json::json!({
        "status": "updated",
        "new_plan": request["plan"],
        "effective_date": Utc::now()
    });

    Ok(Json(ApiResponse::success(response)))
}

// Analytics Handlers
async fn get_monitoring_data(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<RealTimeMonitoring>>, StatusCode> {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let platform = state.platform.read().await;
    match platform.get_analytics_engine().get_current_monitoring().await {
        Some(monitoring) => Ok(Json(ApiResponse::success(monitoring))),
        None => Ok(Json(ApiResponse::error("No monitoring data available".to_string()))),
    }
}

async fn get_performance_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let metrics = serde_json::json!({
        "total_return": "0.185",
        "sharpe_ratio": "1.85",
        "max_drawdown": "0.08",
        "win_rate": "0.78",
        "trades_executed": "8950",
        "avg_execution_time": "850ms"
    });

    Ok(Json(ApiResponse::success(metrics)))
}

async fn get_whale_details(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(whale_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let api_key = authenticate_request(&state, &headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !api_key.permissions.contains(&ApiPermission::AccessWhaleData) {
        return Err(StatusCode::FORBIDDEN);
    }

    let whale_details = serde_json::json!({
        "whale_id": whale_id,
        "portfolio_value": "50000000",
        "recent_trades": 15,
        "preferred_chains": ["ethereum", "arbitrum"],
        "risk_profile": "balanced",
        "influence_score": "0.85"
    });

    Ok(Json(ApiResponse::success(whale_details)))
}

// WebSocket Handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Err(_) = authenticate_request(&state, &headers).await {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Handle WebSocket messages
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Send periodic updates
                let update = serde_json::json!({
                    "type": "market_update",
                    "timestamp": Utc::now(),
                    "data": {
                        "price_eth": "3200.50",
                        "volume_24h": "320000000"
                    }
                });

                if sender.send(Message::Text(update.to_string())).await.is_err() {
                    break;
                }
            },
            msg = receiver.next() => {
                if msg.is_none() {
                    break;
                }
            }
        }
    }
}

// Health & Status Handlers
async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    let health = serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "uptime": "99.95%",
        "version": "0.1.0"
    });
    Json(ApiResponse::success(health))
}

async fn system_status(
    State(state): State<AppState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let status = serde_json::json!({
        "platform": "operational",
        "components": {
            "privacy": "healthy",
            "governance": "healthy",
            "bridge": "healthy",
            "oracle": "healthy",
            "dex": "healthy"
        },
        "metrics": {
            "active_users": 2850,
            "total_volume_24h": "320000000",
            "trades_per_second": 12.5
        }
    });
    Json(ApiResponse::success(status))
}

// Authentication & Authorization
async fn authenticate_request(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<ApiKey, PlatformError> {
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| PlatformError::AuthenticationFailed {
            user_id: "unknown".to_string(),
        })?;

    let api_keys = state.api_keys.read().await;
    let api_key = api_keys.get(auth_header)
        .ok_or_else(|| PlatformError::AuthenticationFailed {
            user_id: "unknown".to_string(),
        })?;

    if !api_key.is_active {
        return Err(PlatformError::AuthenticationFailed {
            user_id: api_key.user_id.clone(),
        });
    }

    if let Some(expires_at) = api_key.expires_at {
        if Utc::now() > expires_at {
            return Err(PlatformError::AuthenticationFailed {
                user_id: api_key.user_id.clone(),
            });
        }
    }

    Ok(api_key.clone())
}

// REST API Client for external integrations
pub struct ApiClient {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_market_analytics(&self) -> Result<MarketAnalytics> {
        let url = format!("{}/api/v1/market/analytics", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| PlatformError::ApiRequestFailed {
                endpoint: url.clone(),
                status: 0,
            })?;

        let api_response: ApiResponse<MarketAnalytics> = response.json().await
            .map_err(|e| PlatformError::ApiRequestFailed {
                endpoint: url,
                status: 0,
            })?;

        api_response.data.ok_or_else(|| PlatformError::ApiRequestFailed {
            endpoint: "market/analytics".to_string(),
            status: 404,
        })
    }

    pub async fn execute_trade(&self, request: TradeExecutionRequest) -> Result<TradeExecutionResponse> {
        let url = format!("{}/api/v1/trading/execute", self.base_url);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| PlatformError::ApiRequestFailed {
                endpoint: url.clone(),
                status: 0,
            })?;

        let api_response: ApiResponse<TradeExecutionResponse> = response.json().await
            .map_err(|e| PlatformError::ApiRequestFailed {
                endpoint: url,
                status: 0,
            })?;

        api_response.data.ok_or_else(|| PlatformError::ApiRequestFailed {
            endpoint: "trading/execute".to_string(),
            status: 404,
        })
    }
}

// WebSocket Server for real-time data streaming
pub struct WebSocketServer {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    broadcast_sender: tokio::sync::broadcast::Sender<String>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        let (broadcast_sender, _) = tokio::sync::broadcast::channel(1000);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_sender,
        }
    }

    pub async fn broadcast_market_update(&self, data: serde_json::Value) -> Result<()> {
        let message = serde_json::json!({
            "type": "market_update",
            "timestamp": Utc::now(),
            "data": data
        }).to_string();

        self.broadcast_sender.send(message)
            .map_err(|e| PlatformError::InternalError {
                details: format!("Failed to broadcast: {}", e),
            })?;

        Ok(())
    }

    pub async fn broadcast_trade_update(&self, trade_data: serde_json::Value) -> Result<()> {
        let message = serde_json::json!({
            "type": "trade_update",
            "timestamp": Utc::now(),
            "data": trade_data
        }).to_string();

        self.broadcast_sender.send(message)
            .map_err(|e| PlatformError::InternalError {
                details: format!("Failed to broadcast: {}", e),
            })?;

        Ok(())
    }
}

pub struct RestApiHandler {
    server: ApiServer,
    client: Option<ApiClient>,
}

impl RestApiHandler {
    pub fn new(platform: Arc<RwLock<MobyMarket>>) -> Self {
        Self {
            server: ApiServer::new(platform),
            client: None,
        }
    }

    pub fn with_client(mut self, base_url: String, api_key: String) -> Self {
        self.client = Some(ApiClient::new(base_url, api_key));
        self
    }

    pub async fn start_server(self, port: u16) -> Result<()> {
        self.server.serve(port).await
    }

    pub fn get_client(&self) -> Option<&ApiClient> {
        self.client.as_ref()
    }
}