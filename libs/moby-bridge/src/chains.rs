//! Chain abstraction and multi-chain support for the Moby Bridge system.
//!
//! This module provides a unified interface for interacting with different blockchain networks,
//! enabling seamless cross-chain transfers and operations. It includes chain-specific implementations,
//! chain discovery mechanisms, and standardized APIs for whale trading operations.

use crate::error::{BridgeError, BridgeResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Unique identifier for blockchain networks
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainId(pub String);

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ChainId {
    fn from(s: &str) -> Self {
        ChainId(s.to_string())
    }
}

impl From<String> for ChainId {
    fn from(s: String) -> Self {
        ChainId(s)
    }
}

/// Chain type categorization for different blockchain architectures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChainType {
    /// Ethereum Virtual Machine compatible chains
    EVM {
        chain_id: u64,
        network_name: String,
    },
    /// Solana-based chains
    Solana {
        cluster: String,
        commitment: String,
    },
    /// Bitcoin-like UTXO chains
    Bitcoin {
        network: String,
        address_type: String,
    },
    /// Cosmos SDK based chains
    Cosmos {
        chain_id: String,
        bech32_prefix: String,
    },
    /// Polkadot parachains
    Substrate {
        para_id: Option<u32>,
        relay_chain: String,
    },
    /// Custom or other chain types
    Custom {
        protocol: String,
        version: String,
        metadata: HashMap<String, String>,
    },
}

/// Current operational status of a blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChainStatus {
    /// Chain is fully operational
    Active,
    /// Chain has minor issues but still functional
    Degraded {
        reason: String,
        estimated_recovery: Option<DateTime<Utc>>,
    },
    /// Chain is experiencing major issues
    Impaired {
        reason: String,
        affected_operations: Vec<String>,
    },
    /// Chain is temporarily offline
    Maintenance {
        start_time: DateTime<Utc>,
        estimated_end: Option<DateTime<Utc>>,
        reason: String,
    },
    /// Chain is permanently disabled
    Deprecated {
        deprecated_at: DateTime<Utc>,
        migration_target: Option<ChainId>,
    },
}

/// Supported token standards on each chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenStandard {
    /// Native chain token (ETH, SOL, BTC, etc.)
    Native,
    /// ERC-20 compatible tokens
    ERC20 { contract_address: String },
    /// Solana Program Library tokens
    SPL { mint_address: String },
    /// Cosmos SDK native tokens
    IBC { denom: String },
    /// Custom token implementation
    Custom {
        standard: String,
        identifier: String,
        metadata: HashMap<String, String>,
    },
}

/// Bridge-specific configuration for each chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Whether this chain supports incoming transfers
    pub supports_incoming: bool,
    /// Whether this chain supports outgoing transfers
    pub supports_outgoing: bool,
    /// Minimum transfer amount (in smallest unit)
    pub min_transfer_amount: u64,
    /// Maximum transfer amount (in smallest unit)
    pub max_transfer_amount: u64,
    /// Bridge fee percentage (0.0 to 1.0)
    pub bridge_fee_rate: Decimal,
    /// Fixed bridge fee (in smallest unit)
    pub bridge_fee_fixed: u64,
    /// Confirmation requirements for finality
    pub confirmation_blocks: u32,
    /// Estimated time for transaction finality
    pub finality_time_seconds: u32,
    /// Supported privacy levels
    pub privacy_levels: Vec<String>,
    /// Whale trading optimizations enabled
    pub whale_optimizations: bool,
}

/// Real-time chain metrics and health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetrics {
    /// Current block height
    pub block_height: u64,
    /// Average block time in seconds
    pub avg_block_time: u32,
    /// Current gas/fee price
    pub current_fee_level: Decimal,
    /// Total value locked in bridge contracts
    pub tvl_usd: Decimal,
    /// Number of active validators/nodes
    pub validator_count: u32,
    /// Network hash rate or stake weight
    pub network_security_metric: Decimal,
    /// Last successful bridge transaction
    pub last_bridge_tx: Option<DateTime<Utc>>,
    /// 24h transaction volume
    pub daily_volume_usd: Decimal,
    /// Current network congestion level (0.0 to 1.0)
    pub congestion_level: f32,
}

/// Comprehensive chain configuration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Unique chain identifier
    pub chain_id: ChainId,
    /// Human-readable chain name
    pub name: String,
    /// Chain type and technical details
    pub chain_type: ChainType,
    /// Current operational status
    pub status: ChainStatus,
    /// Network endpoints and connection info
    pub endpoints: Vec<ChainEndpoint>,
    /// Supported token standards
    pub supported_tokens: Vec<TokenStandard>,
    /// Bridge-specific configuration
    pub bridge_config: BridgeConfig,
    /// Real-time metrics
    pub metrics: ChainMetrics,
    /// Additional chain-specific metadata
    pub metadata: HashMap<String, String>,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Network endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainEndpoint {
    /// Endpoint URL
    pub url: String,
    /// Endpoint type (RPC, WebSocket, etc.)
    pub endpoint_type: EndpointType,
    /// Priority for load balancing (higher = preferred)
    pub priority: u8,
    /// Whether this endpoint is currently healthy
    pub is_healthy: bool,
    /// Last health check timestamp
    pub last_health_check: DateTime<Utc>,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u32,
}

/// Types of network endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointType {
    /// JSON-RPC endpoint
    JsonRpc,
    /// WebSocket endpoint
    WebSocket,
    /// REST API endpoint
    RestApi,
    /// GraphQL endpoint
    GraphQL,
    /// Custom protocol endpoint
    Custom(String),
}

/// Abstract interface for blockchain interactions
#[async_trait]
pub trait Chain: Send + Sync {
    /// Get chain configuration
    async fn get_config(&self) -> BridgeResult<ChainConfig>;

    /// Get current chain status
    async fn get_status(&self) -> BridgeResult<ChainStatus>;

    /// Get real-time chain metrics
    async fn get_metrics(&self) -> BridgeResult<ChainMetrics>;

    /// Validate an address format for this chain
    async fn validate_address(&self, address: &str) -> BridgeResult<bool>;

    /// Get token balance for an address
    async fn get_token_balance(&self, address: &str, token: &TokenStandard) -> BridgeResult<u64>;

    /// Get transaction details by hash
    async fn get_transaction(&self, tx_hash: &str) -> BridgeResult<ChainTransaction>;

    /// Submit a transaction to the network
    async fn submit_transaction(&self, tx_data: &[u8]) -> BridgeResult<String>;

    /// Get transaction confirmations
    async fn get_confirmations(&self, tx_hash: &str) -> BridgeResult<u32>;

    /// Estimate transaction fees
    async fn estimate_fees(&self, transaction_data: &TransactionRequest) -> BridgeResult<FeeEstimate>;

    /// Subscribe to new blocks (if supported)
    async fn subscribe_blocks(&self) -> BridgeResult<BlockSubscription>;

    /// Get bridge contract address
    async fn get_bridge_contract(&self) -> BridgeResult<String>;

    /// Check if whale optimizations are available
    async fn supports_whale_optimizations(&self) -> bool;
}

/// Transaction information from any chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTransaction {
    /// Transaction hash
    pub hash: String,
    /// Block number (None if pending)
    pub block_number: Option<u64>,
    /// Transaction index in block
    pub transaction_index: Option<u32>,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: Option<String>,
    /// Transaction value
    pub value: u64,
    /// Gas/fee paid
    pub fee_paid: u64,
    /// Transaction status
    pub status: TransactionStatus,
    /// Number of confirmations
    pub confirmations: u32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional chain-specific data
    pub chain_specific_data: HashMap<String, String>,
}

/// Transaction status across different chains
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    /// Transaction is pending in mempool
    Pending,
    /// Transaction is confirmed in a block
    Confirmed,
    /// Transaction failed
    Failed { reason: String },
    /// Transaction was dropped/replaced
    Dropped,
}

/// Fee estimation for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    /// Estimated total fee in smallest unit
    pub total_fee: u64,
    /// Base fee component
    pub base_fee: u64,
    /// Priority fee component
    pub priority_fee: u64,
    /// Estimated confirmation time in seconds
    pub estimated_confirmation_seconds: u32,
    /// Fee level (slow, standard, fast)
    pub fee_level: FeeLevel,
}

/// Fee priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeeLevel {
    Slow,
    Standard,
    Fast,
    Urgent,
}

/// Transaction request for fee estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Transfer amount
    pub amount: u64,
    /// Token to transfer
    pub token: TokenStandard,
    /// Additional transaction data
    pub data: Option<Vec<u8>>,
}

/// Block subscription for real-time updates
#[derive(Debug)]
pub struct BlockSubscription {
    /// Subscription ID
    pub id: String,
    /// Block receiver channel
    pub receiver: tokio::sync::mpsc::Receiver<BlockInfo>,
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    /// Block number
    pub number: u64,
    /// Block hash
    pub hash: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Number of transactions
    pub transaction_count: u32,
    /// Parent block hash
    pub parent_hash: String,
}

/// Chain registry for managing multiple blockchain networks
#[derive(Debug)]
pub struct ChainRegistry {
    /// Registered chains
    chains: HashMap<ChainId, Box<dyn Chain>>,
    /// Chain configurations
    configs: HashMap<ChainId, ChainConfig>,
    /// Health monitoring
    health_monitor: ChainHealthMonitor,
}

impl ChainRegistry {
    /// Create a new chain registry
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
            configs: HashMap::new(),
            health_monitor: ChainHealthMonitor::new(),
        }
    }

    /// Register a new chain
    pub async fn register_chain(
        &mut self,
        chain_id: ChainId,
        chain: Box<dyn Chain>,
    ) -> BridgeResult<()> {
        // Get initial configuration
        let config = chain.get_config().await?;

        // Validate chain configuration
        self.validate_chain_config(&config)?;

        // Store chain and config
        self.chains.insert(chain_id.clone(), chain);
        self.configs.insert(chain_id.clone(), config);

        // Start health monitoring
        self.health_monitor.start_monitoring(&chain_id).await?;

        Ok(())
    }

    /// Get a chain by ID
    pub fn get_chain(&self, chain_id: &ChainId) -> BridgeResult<&dyn Chain> {
        self.chains
            .get(chain_id)
            .map(|c| c.as_ref())
            .ok_or_else(|| BridgeError::ChainNotFound {
                chain_id: chain_id.clone(),
                available_chains: self.chains.keys().cloned().collect(),
            })
    }

    /// Get chain configuration
    pub fn get_chain_config(&self, chain_id: &ChainId) -> BridgeResult<&ChainConfig> {
        self.configs
            .get(chain_id)
            .ok_or_else(|| BridgeError::ChainNotFound {
                chain_id: chain_id.clone(),
                available_chains: self.chains.keys().cloned().collect(),
            })
    }

    /// List all registered chains
    pub fn list_chains(&self) -> Vec<ChainId> {
        self.chains.keys().cloned().collect()
    }

    /// Get chains by status
    pub fn get_chains_by_status(&self, status: &ChainStatus) -> Vec<ChainId> {
        self.configs
            .iter()
            .filter(|(_, config)| std::mem::discriminant(&config.status) == std::mem::discriminant(status))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get active chains supporting whale transfers
    pub fn get_whale_enabled_chains(&self) -> Vec<ChainId> {
        self.configs
            .iter()
            .filter(|(_, config)| {
                matches!(config.status, ChainStatus::Active) &&
                config.bridge_config.whale_optimizations
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Update chain configuration
    pub async fn update_chain_config(
        &mut self,
        chain_id: &ChainId,
        config: ChainConfig,
    ) -> BridgeResult<()> {
        // Validate the updated configuration
        self.validate_chain_config(&config)?;

        // Update stored configuration
        self.configs.insert(chain_id.clone(), config);

        Ok(())
    }

    /// Remove a chain from registry
    pub async fn remove_chain(&mut self, chain_id: &ChainId) -> BridgeResult<()> {
        // Stop health monitoring
        self.health_monitor.stop_monitoring(chain_id).await?;

        // Remove from registry
        self.chains.remove(chain_id);
        self.configs.remove(chain_id);

        Ok(())
    }

    /// Validate chain configuration
    fn validate_chain_config(&self, config: &ChainConfig) -> BridgeResult<()> {
        // Validate bridge configuration
        if config.bridge_config.min_transfer_amount >= config.bridge_config.max_transfer_amount {
            return Err(BridgeError::InvalidChainConfig {
                chain_id: config.chain_id.clone(),
                reason: "Minimum transfer amount must be less than maximum".to_string(),
            });
        }

        // Validate fee rates
        if config.bridge_config.bridge_fee_rate < Decimal::ZERO ||
           config.bridge_config.bridge_fee_rate > Decimal::ONE {
            return Err(BridgeError::InvalidChainConfig {
                chain_id: config.chain_id.clone(),
                reason: "Bridge fee rate must be between 0.0 and 1.0".to_string(),
            });
        }

        // Validate endpoints
        if config.endpoints.is_empty() {
            return Err(BridgeError::InvalidChainConfig {
                chain_id: config.chain_id.clone(),
                reason: "At least one endpoint must be configured".to_string(),
            });
        }

        Ok(())
    }

    /// Get overall registry health
    pub async fn get_registry_health(&self) -> ChainRegistryHealth {
        let total_chains = self.chains.len();
        let active_chains = self.get_chains_by_status(&ChainStatus::Active).len();
        let healthy_chains = self.health_monitor.get_healthy_chains().await.len();

        ChainRegistryHealth {
            total_chains,
            active_chains,
            healthy_chains,
            health_percentage: if total_chains > 0 {
                (healthy_chains as f32 / total_chains as f32) * 100.0
            } else {
                0.0
            },
            last_updated: Utc::now(),
        }
    }
}

/// Health information for the chain registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRegistryHealth {
    /// Total number of registered chains
    pub total_chains: usize,
    /// Number of active chains
    pub active_chains: usize,
    /// Number of healthy chains
    pub healthy_chains: usize,
    /// Overall health percentage
    pub health_percentage: f32,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Chain health monitoring system
#[derive(Debug)]
pub struct ChainHealthMonitor {
    /// Monitoring tasks
    monitoring_tasks: HashMap<ChainId, tokio::task::JoinHandle<()>>,
    /// Health status cache
    health_status: HashMap<ChainId, ChainHealthStatus>,
}

impl ChainHealthMonitor {
    /// Create new health monitor
    pub fn new() -> Self {
        Self {
            monitoring_tasks: HashMap::new(),
            health_status: HashMap::new(),
        }
    }

    /// Start monitoring a chain
    pub async fn start_monitoring(&mut self, chain_id: &ChainId) -> BridgeResult<()> {
        let id = chain_id.clone();

        // Create monitoring task
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;
                // Health check logic would go here
                // This is a simplified version
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        self.monitoring_tasks.insert(chain_id.clone(), task);

        Ok(())
    }

    /// Stop monitoring a chain
    pub async fn stop_monitoring(&mut self, chain_id: &ChainId) -> BridgeResult<()> {
        if let Some(task) = self.monitoring_tasks.remove(chain_id) {
            task.abort();
        }

        self.health_status.remove(chain_id);

        Ok(())
    }

    /// Get healthy chains
    pub async fn get_healthy_chains(&self) -> Vec<ChainId> {
        self.health_status
            .iter()
            .filter(|(_, status)| status.is_healthy)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

/// Health status for individual chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainHealthStatus {
    /// Whether the chain is healthy
    pub is_healthy: bool,
    /// Last successful health check
    pub last_successful_check: DateTime<Utc>,
    /// Consecutive failed checks
    pub consecutive_failures: u32,
    /// Current response time in milliseconds
    pub response_time_ms: u32,
    /// Error details if unhealthy
    pub error_details: Option<String>,
}

/// Pre-configured chain implementations for popular networks
pub mod implementations {
    use super::*;

    /// Create Ethereum mainnet configuration
    pub fn ethereum_mainnet() -> ChainConfig {
        ChainConfig {
            chain_id: ChainId::from("ethereum"),
            name: "Ethereum Mainnet".to_string(),
            chain_type: ChainType::EVM {
                chain_id: 1,
                network_name: "mainnet".to_string(),
            },
            status: ChainStatus::Active,
            endpoints: vec![
                ChainEndpoint {
                    url: "https://eth-mainnet.public.blastapi.io".to_string(),
                    endpoint_type: EndpointType::JsonRpc,
                    priority: 100,
                    is_healthy: true,
                    last_health_check: Utc::now(),
                    avg_response_time_ms: 200,
                },
            ],
            supported_tokens: vec![
                TokenStandard::Native,
                TokenStandard::ERC20 { contract_address: "0xA0b86a33E6441c8044f32a78e9ed9ED4E7E5CfC2".to_string() }, // USDC
            ],
            bridge_config: BridgeConfig {
                supports_incoming: true,
                supports_outgoing: true,
                min_transfer_amount: 1000000, // 1 USDC
                max_transfer_amount: 100000000000000, // 100M USDC
                bridge_fee_rate: Decimal::new(1, 3), // 0.1%
                bridge_fee_fixed: 0,
                confirmation_blocks: 12,
                finality_time_seconds: 180,
                privacy_levels: vec!["public".to_string(), "confidential".to_string()],
                whale_optimizations: true,
            },
            metrics: ChainMetrics {
                block_height: 18000000,
                avg_block_time: 12,
                current_fee_level: Decimal::new(20, 9), // 20 gwei
                tvl_usd: Decimal::new(50000000, 0), // $50M
                validator_count: 500000,
                network_security_metric: Decimal::new(20000000, 0), // ETH staked
                last_bridge_tx: Some(Utc::now()),
                daily_volume_usd: Decimal::new(10000000, 0), // $10M
                congestion_level: 0.3,
            },
            metadata: HashMap::from([
                ("explorer".to_string(), "https://etherscan.io".to_string()),
                ("docs".to_string(), "https://ethereum.org/en/developers/docs/".to_string()),
            ]),
            last_updated: Utc::now(),
        }
    }

    /// Create Solana mainnet configuration
    pub fn solana_mainnet() -> ChainConfig {
        ChainConfig {
            chain_id: ChainId::from("solana"),
            name: "Solana Mainnet".to_string(),
            chain_type: ChainType::Solana {
                cluster: "mainnet-beta".to_string(),
                commitment: "confirmed".to_string(),
            },
            status: ChainStatus::Active,
            endpoints: vec![
                ChainEndpoint {
                    url: "https://api.mainnet-beta.solana.com".to_string(),
                    endpoint_type: EndpointType::JsonRpc,
                    priority: 100,
                    is_healthy: true,
                    last_health_check: Utc::now(),
                    avg_response_time_ms: 150,
                },
            ],
            supported_tokens: vec![
                TokenStandard::Native,
                TokenStandard::SPL { mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string() }, // USDC
            ],
            bridge_config: BridgeConfig {
                supports_incoming: true,
                supports_outgoing: true,
                min_transfer_amount: 1000000, // 1 USDC
                max_transfer_amount: 100000000000000, // 100M USDC
                bridge_fee_rate: Decimal::new(5, 4), // 0.05%
                bridge_fee_fixed: 0,
                confirmation_blocks: 1,
                finality_time_seconds: 13,
                privacy_levels: vec!["public".to_string()],
                whale_optimizations: true,
            },
            metrics: ChainMetrics {
                block_height: 200000000,
                avg_block_time: 400, // 400ms
                current_fee_level: Decimal::new(5000, 9), // 5000 lamports
                tvl_usd: Decimal::new(30000000, 0), // $30M
                validator_count: 1500,
                network_security_metric: Decimal::new(400000000, 0), // SOL staked
                last_bridge_tx: Some(Utc::now()),
                daily_volume_usd: Decimal::new(5000000, 0), // $5M
                congestion_level: 0.2,
            },
            metadata: HashMap::from([
                ("explorer".to_string(), "https://solscan.io".to_string()),
                ("docs".to_string(), "https://docs.solana.com".to_string()),
            ]),
            last_updated: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_chain_registry_creation() {
        let registry = ChainRegistry::new();
        assert_eq!(registry.list_chains().len(), 0);
    }

    #[tokio::test]
    async fn test_chain_config_validation() {
        let registry = ChainRegistry::new();

        // Test invalid min/max transfer amounts
        let mut config = implementations::ethereum_mainnet();
        config.bridge_config.min_transfer_amount = 1000;
        config.bridge_config.max_transfer_amount = 500;

        let result = registry.validate_chain_config(&config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fee_validation() {
        let registry = ChainRegistry::new();

        // Test invalid fee rate
        let mut config = implementations::ethereum_mainnet();
        config.bridge_config.bridge_fee_rate = Decimal::new(15, 1); // 1.5 (invalid)

        let result = registry.validate_chain_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_id_display() {
        let chain_id = ChainId::from("ethereum");
        assert_eq!(format!("{}", chain_id), "ethereum");
    }

    #[test]
    fn test_chain_id_from_string() {
        let chain_id = ChainId::from("solana".to_string());
        assert_eq!(chain_id.0, "solana");
    }

    #[test]
    fn test_predefined_configs() {
        let eth_config = implementations::ethereum_mainnet();
        assert_eq!(eth_config.chain_id.0, "ethereum");
        assert_eq!(eth_config.name, "Ethereum Mainnet");

        let sol_config = implementations::solana_mainnet();
        assert_eq!(sol_config.chain_id.0, "solana");
        assert_eq!(sol_config.name, "Solana Mainnet");
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let mut monitor = ChainHealthMonitor::new();
        let chain_id = ChainId::from("test-chain");

        let result = monitor.start_monitoring(&chain_id).await;
        assert!(result.is_ok());

        let result = monitor.stop_monitoring(&chain_id).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_transaction_status() {
        let status = TransactionStatus::Pending;
        assert_eq!(status, TransactionStatus::Pending);

        let failed_status = TransactionStatus::Failed {
            reason: "Insufficient gas".to_string(),
        };
        assert!(matches!(failed_status, TransactionStatus::Failed { .. }));
    }

    #[test]
    fn test_chain_type_variants() {
        let evm_chain = ChainType::EVM {
            chain_id: 1,
            network_name: "mainnet".to_string(),
        };
        assert!(matches!(evm_chain, ChainType::EVM { .. }));

        let solana_chain = ChainType::Solana {
            cluster: "mainnet-beta".to_string(),
            commitment: "confirmed".to_string(),
        };
        assert!(matches!(solana_chain, ChainType::Solana { .. }));
    }

    #[test]
    fn test_token_standards() {
        let native = TokenStandard::Native;
        assert!(matches!(native, TokenStandard::Native));

        let erc20 = TokenStandard::ERC20 {
            contract_address: "0x1234...".to_string(),
        };
        assert!(matches!(erc20, TokenStandard::ERC20 { .. }));

        let spl = TokenStandard::SPL {
            mint_address: "mint123...".to_string(),
        };
        assert!(matches!(spl, TokenStandard::SPL { .. }));
    }
}