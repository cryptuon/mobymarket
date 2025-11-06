//! Cross-chain communication protocols for the Moby Bridge system.
//!
//! This module provides standardized protocols for secure, efficient cross-chain
//! communication between different blockchain networks. It includes message formatting,
//! routing, validation, and delivery confirmation mechanisms optimized for whale
//! trading operations.

use crate::error::{BridgeError, BridgeResult};
use crate::chains::{ChainId, TokenStandard};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sha2::{Sha256, Digest};

/// Protocol type enumeration for different cross-chain communication mechanisms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtocolType {
    /// Lock-and-mint protocol for token bridging
    LockAndMint {
        /// Lock contract on source chain
        lock_contract: String,
        /// Mint contract on destination chain
        mint_contract: String,
        /// Escrow configuration
        escrow_config: EscrowConfig,
    },
    /// Burn-and-mint protocol for native tokens
    BurnAndMint {
        /// Burn contract on source chain
        burn_contract: String,
        /// Mint contract on destination chain
        mint_contract: String,
        /// Token supply management
        supply_control: SupplyControlConfig,
    },
    /// Atomic swap protocol
    AtomicSwap {
        /// Hash time-locked contract address
        htlc_contract: String,
        /// Timeout configuration
        timeout_config: TimeoutConfig,
        /// Secret management
        secret_config: SecretConfig,
    },
    /// Validator-based consensus protocol
    ValidatorConsensus {
        /// Validator set configuration
        validator_set: ValidatorSetConfig,
        /// Consensus threshold
        threshold: ConsensusThreshold,
        /// Slashing conditions
        slashing_config: SlashingConfig,
    },
    /// Optimistic bridge protocol
    Optimistic {
        /// Challenge period in seconds
        challenge_period: u64,
        /// Bond amount required
        bond_amount: u64,
        /// Fraud proof system
        fraud_proof_config: FraudProofConfig,
    },
    /// Zero-knowledge proof protocol
    ZkProof {
        /// Proof system type
        proof_system: ZkProofSystem,
        /// Circuit configuration
        circuit_config: CircuitConfig,
        /// Verification key
        verification_key: String,
    },
}

/// Message types for cross-chain communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// Transfer initiation message
    TransferInit {
        /// Transfer details
        transfer: TransferMessage,
        /// Proof of source transaction
        source_proof: SourceProof,
    },
    /// Transfer confirmation message
    TransferConfirm {
        /// Original transfer ID
        transfer_id: String,
        /// Destination transaction hash
        dest_tx_hash: String,
        /// Confirmation proof
        confirmation_proof: ConfirmationProof,
    },
    /// Transfer failure notification
    TransferFailed {
        /// Original transfer ID
        transfer_id: String,
        /// Failure reason
        reason: String,
        /// Recovery instructions
        recovery_action: RecoveryAction,
    },
    /// Validator set update
    ValidatorUpdate {
        /// New validator set
        new_validators: Vec<ValidatorInfo>,
        /// Effective timestamp
        effective_at: DateTime<Utc>,
        /// Signature threshold
        signature_threshold: u32,
    },
    /// Emergency pause message
    EmergencyPause {
        /// Pause duration
        duration: Option<u64>,
        /// Pause reason
        reason: String,
        /// Emergency authority signature
        authority_signature: String,
    },
    /// Protocol parameter update
    ParameterUpdate {
        /// Parameter name
        parameter: String,
        /// New value
        new_value: String,
        /// Effective timestamp
        effective_at: DateTime<Utc>,
    },
    /// Health check ping
    HealthPing {
        /// Ping timestamp
        timestamp: DateTime<Utc>,
        /// Chain status
        chain_status: String,
    },
    /// Batch operation for multiple transfers
    BatchOperation {
        /// List of operations
        operations: Vec<BatchOperation>,
        /// Batch metadata
        batch_metadata: BatchMetadata,
    },
}

/// Core transfer message structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransferMessage {
    /// Unique transfer identifier
    pub transfer_id: String,
    /// Source chain ID
    pub source_chain: ChainId,
    /// Destination chain ID
    pub dest_chain: ChainId,
    /// Sender address on source chain
    pub sender: String,
    /// Recipient address on destination chain
    pub recipient: String,
    /// Token being transferred
    pub token: TokenStandard,
    /// Transfer amount in smallest unit
    pub amount: u64,
    /// Bridge fee amount
    pub fee_amount: u64,
    /// Transfer deadline
    pub deadline: DateTime<Utc>,
    /// Nonce for replay protection
    pub nonce: u64,
    /// Additional transfer data
    pub data: Option<Vec<u8>>,
    /// Privacy level for transfer
    pub privacy_level: PrivacyLevel,
    /// Whale transfer optimizations
    pub whale_optimizations: Option<WhaleTransferConfig>,
}

/// Privacy levels for cross-chain transfers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivacyLevel {
    /// Fully public transfer
    Public,
    /// Confidential amounts
    ConfidentialAmount,
    /// Anonymous participants
    Anonymous,
    /// Full privacy with ZK proofs
    FullPrivacy,
}

/// Configuration for whale transfer optimizations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhaleTransferConfig {
    /// Use dedicated validator set
    pub use_dedicated_validators: bool,
    /// Priority routing
    pub priority_routing: bool,
    /// Enhanced monitoring
    pub enhanced_monitoring: bool,
    /// Custom confirmation requirements
    pub custom_confirmations: Option<u32>,
    /// Liquidity pre-allocation
    pub liquidity_prealloc: bool,
}

/// Proof of source transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceProof {
    /// Transaction hash on source chain
    pub tx_hash: String,
    /// Block number containing transaction
    pub block_number: u64,
    /// Merkle proof of inclusion
    pub merkle_proof: Vec<String>,
    /// Block header hash
    pub block_header: String,
    /// Transaction receipt proof
    pub receipt_proof: Vec<u8>,
}

/// Confirmation proof for completed transfers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfirmationProof {
    /// Destination transaction hash
    pub dest_tx_hash: String,
    /// Block number on destination
    pub dest_block_number: u64,
    /// Validator signatures
    pub validator_signatures: Vec<ValidatorSignature>,
    /// Proof timestamp
    pub timestamp: DateTime<Utc>,
}

/// Recovery action for failed transfers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecoveryAction {
    /// Retry the transfer
    Retry {
        /// Retry parameters
        retry_params: RetryParams,
    },
    /// Refund to sender
    Refund {
        /// Refund transaction hash
        refund_tx_hash: String,
    },
    /// Manual intervention required
    ManualIntervention {
        /// Instructions for manual resolution
        instructions: String,
        /// Support ticket ID
        ticket_id: String,
    },
}

/// Validator signature structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidatorSignature {
    /// Validator public key
    pub validator_pubkey: String,
    /// Signature over message hash
    pub signature: String,
    /// Signature timestamp
    pub timestamp: DateTime<Utc>,
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidatorInfo {
    /// Validator public key
    pub public_key: String,
    /// Validator address
    pub address: String,
    /// Staking power/weight
    pub power: u64,
    /// Commission rate
    pub commission: Decimal,
    /// Status (active, inactive, slashed)
    pub status: ValidatorStatus,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Validator status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Slashed { reason: String },
    Retiring { retirement_date: DateTime<Utc> },
}

/// Batch operation for efficient processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchOperation {
    /// Operation type
    pub operation_type: BatchOperationType,
    /// Operation data
    pub data: Vec<u8>,
    /// Operation weight for fee calculation
    pub weight: u32,
}

/// Types of batch operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchOperationType {
    Transfer,
    Refund,
    Slash,
    UpdateValidator,
    UpdateParameter,
}

/// Metadata for batch operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchMetadata {
    /// Batch ID
    pub batch_id: String,
    /// Total operations in batch
    pub operation_count: u32,
    /// Total weight of batch
    pub total_weight: u32,
    /// Batch timeout
    pub timeout: DateTime<Utc>,
    /// Priority level
    pub priority: BatchPriority,
}

/// Batch priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Cross-chain message with full protocol details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Message ID
    pub message_id: String,
    /// Protocol version
    pub version: String,
    /// Source chain
    pub source_chain: ChainId,
    /// Destination chain
    pub dest_chain: ChainId,
    /// Message type and payload
    pub message_type: MessageType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// Message priority
    pub priority: MessagePriority,
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
    /// Message hash for integrity
    pub message_hash: String,
    /// Route information
    pub route: MessageRoute,
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Urgent,
    Emergency,
}

/// Message routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRoute {
    /// Direct connection or via relay
    pub route_type: RouteType,
    /// Intermediate hops
    pub hops: Vec<ChainId>,
    /// Estimated delivery time
    pub estimated_delivery_seconds: u32,
    /// Route cost
    pub route_cost: u64,
}

/// Types of message routes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteType {
    /// Direct chain-to-chain
    Direct,
    /// Via relay chain
    Relay { relay_chain: ChainId },
    /// Multi-hop routing
    MultiHop,
    /// Emergency route
    Emergency,
}

/// Retry configuration for failed messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Initial retry delay in seconds
    pub initial_delay_seconds: u32,
    /// Backoff multiplier
    pub backoff_multiplier: f32,
    /// Maximum delay between retries
    pub max_delay_seconds: u32,
}

/// Retry parameters for recovery actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryParams {
    /// Delay before retry
    pub delay_seconds: u32,
    /// Modified transfer parameters
    pub modified_params: HashMap<String, String>,
}

/// Protocol interface for cross-chain communication
#[async_trait]
pub trait Protocol: Send + Sync {
    /// Get protocol type and configuration
    async fn get_protocol_type(&self) -> ProtocolType;

    /// Send a message to destination chain
    async fn send_message(
        &self,
        message: ProtocolMessage,
        destination: &ChainId,
    ) -> BridgeResult<String>;

    /// Receive and process incoming messages
    async fn receive_message(&self, message_data: &[u8]) -> BridgeResult<ProtocolMessage>;

    /// Validate message authenticity and integrity
    async fn validate_message(&self, message: &ProtocolMessage) -> BridgeResult<bool>;

    /// Get message status
    async fn get_message_status(&self, message_id: &str) -> BridgeResult<MessageStatus>;

    /// Subscribe to incoming messages
    async fn subscribe_messages(&self) -> BridgeResult<MessageSubscription>;

    /// Get protocol statistics
    async fn get_protocol_stats(&self) -> BridgeResult<ProtocolStats>;

    /// Handle protocol upgrades
    async fn handle_upgrade(&self, upgrade_data: &[u8]) -> BridgeResult<()>;
}

/// Message delivery status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    /// Message is pending delivery
    Pending,
    /// Message is in transit
    InTransit {
        /// Current location/hop
        current_hop: ChainId,
        /// Estimated arrival time
        estimated_arrival: DateTime<Utc>,
    },
    /// Message delivered successfully
    Delivered {
        /// Delivery timestamp
        delivered_at: DateTime<Utc>,
        /// Delivery confirmation
        confirmation_hash: String,
    },
    /// Message delivery failed
    Failed {
        /// Failure reason
        reason: String,
        /// Failure timestamp
        failed_at: DateTime<Utc>,
        /// Recovery options
        recovery_options: Vec<RecoveryAction>,
    },
    /// Message expired
    Expired {
        /// Expiration timestamp
        expired_at: DateTime<Utc>,
    },
}

/// Message subscription for real-time updates
#[derive(Debug)]
pub struct MessageSubscription {
    /// Subscription ID
    pub id: String,
    /// Message receiver channel
    pub receiver: tokio::sync::mpsc::Receiver<ProtocolMessage>,
}

/// Protocol performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolStats {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Successful deliveries
    pub successful_deliveries: u64,
    /// Failed deliveries
    pub failed_deliveries: u64,
    /// Average delivery time in seconds
    pub avg_delivery_time_seconds: f32,
    /// Current pending messages
    pub pending_messages: u32,
    /// Protocol uptime percentage
    pub uptime_percentage: f32,
    /// Last 24h throughput
    pub daily_throughput: u64,
}

/// Configuration structures for different protocol types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EscrowConfig {
    /// Escrow timeout in seconds
    pub timeout_seconds: u64,
    /// Required confirmations
    pub required_confirmations: u32,
    /// Multi-signature threshold
    pub multisig_threshold: u32,
    /// Escrow guardian addresses
    pub guardians: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupplyControlConfig {
    /// Maximum supply cap
    pub max_supply: Option<u64>,
    /// Burn rate for supply control
    pub burn_rate: Decimal,
    /// Supply oracle address
    pub supply_oracle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeoutConfig {
    /// Lock timeout in seconds
    pub lock_timeout_seconds: u64,
    /// Refund timeout in seconds
    pub refund_timeout_seconds: u64,
    /// Grace period for delays
    pub grace_period_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretConfig {
    /// Secret length in bytes
    pub secret_length: u32,
    /// Hash algorithm for secret
    pub hash_algorithm: String,
    /// Secret derivation method
    pub derivation_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidatorSetConfig {
    /// Minimum validators required
    pub min_validators: u32,
    /// Maximum validators allowed
    pub max_validators: u32,
    /// Validator rotation period
    pub rotation_period_seconds: u64,
    /// Staking requirements
    pub min_stake: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsensusThreshold {
    /// Signature threshold (e.g., 2/3)
    pub numerator: u32,
    pub denominator: u32,
    /// Minimum absolute number
    pub minimum_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlashingConfig {
    /// Slashing conditions
    pub conditions: Vec<SlashingCondition>,
    /// Slashing penalties
    pub penalties: Vec<SlashingPenalty>,
    /// Appeal process
    pub appeal_period_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlashingCondition {
    /// Condition type
    pub condition_type: String,
    /// Severity level
    pub severity: SlashingSeverity,
    /// Detection parameters
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlashingSeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlashingPenalty {
    /// Penalty amount or percentage
    pub amount: SlashingAmount,
    /// Jail time in seconds
    pub jail_time_seconds: u64,
    /// Additional restrictions
    pub restrictions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlashingAmount {
    Fixed(u64),
    Percentage(Decimal),
    Progressive { base: u64, multiplier: Decimal },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FraudProofConfig {
    /// Proof system type
    pub proof_system: String,
    /// Challenge window in seconds
    pub challenge_window_seconds: u64,
    /// Required evidence types
    pub required_evidence: Vec<String>,
    /// Verification method
    pub verification_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZkProofSystem {
    Groth16,
    PLONK,
    STARKs,
    Bulletproofs,
    Custom { name: String, version: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CircuitConfig {
    /// Circuit constraints
    pub constraints: u32,
    /// Public inputs count
    pub public_inputs: u32,
    /// Private inputs count
    pub private_inputs: u32,
    /// Circuit hash for verification
    pub circuit_hash: String,
}

/// Protocol message builder for constructing standardized messages
pub struct ProtocolMessageBuilder {
    message: ProtocolMessage,
}

impl ProtocolMessageBuilder {
    /// Create new message builder
    pub fn new(source_chain: ChainId, dest_chain: ChainId) -> Self {
        let message_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        Self {
            message: ProtocolMessage {
                message_id: message_id.clone(),
                version: "1.0".to_string(),
                source_chain,
                dest_chain: dest_chain.clone(),
                message_type: MessageType::HealthPing {
                    timestamp,
                    chain_status: "unknown".to_string(),
                },
                timestamp,
                expires_at: timestamp + chrono::Duration::hours(1),
                priority: MessagePriority::Normal,
                retry_config: None,
                message_hash: Self::calculate_message_hash(&message_id, &timestamp),
                route: MessageRoute {
                    route_type: RouteType::Direct,
                    hops: vec![dest_chain],
                    estimated_delivery_seconds: 30,
                    route_cost: 1000,
                },
            },
        }
    }

    /// Set message type
    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.message.message_type = message_type;
        self.recalculate_hash();
        self
    }

    /// Set message priority
    pub fn priority(mut self, priority: MessagePriority) -> Self {
        self.message.priority = priority;
        self.recalculate_hash();
        self
    }

    /// Set expiration time
    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.message.expires_at = expires_at;
        self.recalculate_hash();
        self
    }

    /// Set retry configuration
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.message.retry_config = Some(retry_config);
        self.recalculate_hash();
        self
    }

    /// Set message route
    pub fn route(mut self, route: MessageRoute) -> Self {
        self.message.route = route;
        self.recalculate_hash();
        self
    }

    /// Build the final message
    pub fn build(self) -> ProtocolMessage {
        self.message
    }

    /// Calculate message hash for integrity verification
    fn calculate_message_hash(message_id: &str, timestamp: &DateTime<Utc>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(message_id.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Recalculate message hash after changes
    fn recalculate_hash(&mut self) {
        self.message.message_hash = Self::calculate_message_hash(
            &self.message.message_id,
            &self.message.timestamp,
        );
    }
}

/// Protocol router for managing message routing across chains
pub struct ProtocolRouter {
    /// Registered protocols by chain
    protocols: HashMap<ChainId, Box<dyn Protocol>>,
    /// Routing table for optimal paths
    routing_table: RoutingTable,
    /// Message cache for deduplication
    message_cache: MessageCache,
}

impl ProtocolRouter {
    /// Create new protocol router
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
            routing_table: RoutingTable::new(),
            message_cache: MessageCache::new(),
        }
    }

    /// Register a protocol for a chain
    pub async fn register_protocol(
        &mut self,
        chain_id: ChainId,
        protocol: Box<dyn Protocol>,
    ) -> BridgeResult<()> {
        self.protocols.insert(chain_id.clone(), protocol);
        self.routing_table.add_chain(chain_id).await?;
        Ok(())
    }

    /// Route a message to its destination
    pub async fn route_message(&self, message: ProtocolMessage) -> BridgeResult<String> {
        // Check message cache for deduplication
        if self.message_cache.contains(&message.message_id) {
            return Err(BridgeError::DuplicateMessage {
                message_id: message.message_id,
            });
        }

        // Find optimal route
        let route = self.routing_table.find_route(
            &message.source_chain,
            &message.dest_chain,
        ).await?;

        // Get appropriate protocol
        let protocol = self.protocols
            .get(&message.dest_chain)
            .ok_or_else(|| BridgeError::ProtocolNotFound {
                chain_id: message.dest_chain.clone(),
            })?;

        // Send message
        let result = protocol.send_message(message.clone(), &message.dest_chain).await?;

        // Cache message for deduplication
        self.message_cache.insert(message.message_id.clone());

        Ok(result)
    }

    /// Get protocol statistics for all chains
    pub async fn get_all_protocol_stats(&self) -> HashMap<ChainId, ProtocolStats> {
        let mut stats = HashMap::new();

        for (chain_id, protocol) in &self.protocols {
            if let Ok(protocol_stats) = protocol.get_protocol_stats().await {
                stats.insert(chain_id.clone(), protocol_stats);
            }
        }

        stats
    }
}

/// Routing table for finding optimal message paths
#[derive(Debug)]
pub struct RoutingTable {
    /// Direct connections between chains
    direct_routes: HashMap<(ChainId, ChainId), RouteInfo>,
    /// Multi-hop routes
    multi_hop_routes: HashMap<(ChainId, ChainId), Vec<RouteInfo>>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            direct_routes: HashMap::new(),
            multi_hop_routes: HashMap::new(),
        }
    }

    pub async fn add_chain(&mut self, _chain_id: ChainId) -> BridgeResult<()> {
        // Route discovery logic would go here
        Ok(())
    }

    pub async fn find_route(&self, source: &ChainId, dest: &ChainId) -> BridgeResult<RouteInfo> {
        // Check direct route first
        if let Some(route) = self.direct_routes.get(&(source.clone(), dest.clone())) {
            return Ok(route.clone());
        }

        // Find multi-hop route
        if let Some(routes) = self.multi_hop_routes.get(&(source.clone(), dest.clone())) {
            if let Some(best_route) = routes.first() {
                return Ok(best_route.clone());
            }
        }

        Err(BridgeError::RouteNotFound {
            source: source.clone(),
            destination: dest.clone(),
        })
    }
}

/// Route information for message delivery
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub hops: Vec<ChainId>,
    pub estimated_time_seconds: u32,
    pub cost: u64,
    pub reliability: f32,
}

/// Message cache for deduplication
#[derive(Debug)]
pub struct MessageCache {
    cache: std::collections::HashSet<String>,
}

impl MessageCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashSet::new(),
        }
    }

    pub fn contains(&self, message_id: &str) -> bool {
        self.cache.contains(message_id)
    }

    pub fn insert(&mut self, message_id: String) {
        self.cache.insert(message_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_message_builder() {
        let source = ChainId::from("ethereum");
        let dest = ChainId::from("solana");

        let message = ProtocolMessageBuilder::new(source.clone(), dest.clone())
            .priority(MessagePriority::High)
            .expires_at(Utc::now() + chrono::Duration::hours(2))
            .build();

        assert_eq!(message.source_chain, source);
        assert_eq!(message.dest_chain, dest);
        assert_eq!(message.priority, MessagePriority::High);
    }

    #[test]
    fn test_transfer_message() {
        let transfer = TransferMessage {
            transfer_id: "test-transfer".to_string(),
            source_chain: ChainId::from("ethereum"),
            dest_chain: ChainId::from("solana"),
            sender: "0x123...".to_string(),
            recipient: "abc123...".to_string(),
            token: TokenStandard::Native,
            amount: 1000000,
            fee_amount: 1000,
            deadline: Utc::now() + chrono::Duration::hours(1),
            nonce: 1,
            data: None,
            privacy_level: PrivacyLevel::Public,
            whale_optimizations: None,
        };

        assert_eq!(transfer.transfer_id, "test-transfer");
        assert_eq!(transfer.amount, 1000000);
        assert_eq!(transfer.privacy_level, PrivacyLevel::Public);
    }

    #[test]
    fn test_protocol_types() {
        let lock_mint = ProtocolType::LockAndMint {
            lock_contract: "0xabc...".to_string(),
            mint_contract: "0xdef...".to_string(),
            escrow_config: EscrowConfig {
                timeout_seconds: 3600,
                required_confirmations: 12,
                multisig_threshold: 2,
                guardians: vec!["0x111...".to_string(), "0x222...".to_string()],
            },
        };

        assert!(matches!(lock_mint, ProtocolType::LockAndMint { .. }));
    }

    #[test]
    fn test_message_status() {
        let pending = MessageStatus::Pending;
        assert_eq!(pending, MessageStatus::Pending);

        let delivered = MessageStatus::Delivered {
            delivered_at: Utc::now(),
            confirmation_hash: "0xconf123...".to_string(),
        };
        assert!(matches!(delivered, MessageStatus::Delivered { .. }));
    }

    #[tokio::test]
    async fn test_protocol_router() {
        let router = ProtocolRouter::new();
        assert_eq!(router.protocols.len(), 0);
    }

    #[tokio::test]
    async fn test_routing_table() {
        let mut table = RoutingTable::new();
        let chain_id = ChainId::from("test-chain");

        let result = table.add_chain(chain_id).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_cache() {
        let mut cache = MessageCache::new();
        let message_id = "test-message-123";

        assert!(!cache.contains(message_id));
        cache.insert(message_id.to_string());
        assert!(cache.contains(message_id));
    }

    #[test]
    fn test_consensus_threshold() {
        let threshold = ConsensusThreshold {
            numerator: 2,
            denominator: 3,
            minimum_count: 5,
        };

        assert_eq!(threshold.numerator, 2);
        assert_eq!(threshold.denominator, 3);
        assert_eq!(threshold.minimum_count, 5);
    }

    #[test]
    fn test_whale_transfer_config() {
        let config = WhaleTransferConfig {
            use_dedicated_validators: true,
            priority_routing: true,
            enhanced_monitoring: true,
            custom_confirmations: Some(20),
            liquidity_prealloc: true,
        };

        assert!(config.use_dedicated_validators);
        assert!(config.priority_routing);
        assert_eq!(config.custom_confirmations, Some(20));
    }

    #[test]
    fn test_privacy_levels() {
        let levels = vec![
            PrivacyLevel::Public,
            PrivacyLevel::ConfidentialAmount,
            PrivacyLevel::Anonymous,
            PrivacyLevel::FullPrivacy,
        ];

        assert_eq!(levels.len(), 4);
        assert_eq!(levels[0], PrivacyLevel::Public);
        assert_eq!(levels[3], PrivacyLevel::FullPrivacy);
    }
}