//! Zero-knowledge circuits for private trading operations
//!
//! This module defines the ZK circuits used throughout the privacy system:
//! - Private trade validation circuits
//! - Range proof circuits
//! - Nullifier derivation circuits
//! - Mixing protocol circuits
//! - Compliance disclosure circuits

use crate::{
    error::{PrivacyError, PrivacyResult},
    engine::TradeSecret,
};
use moby_types::{AccountKey, WhaleAmount, TradeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for circuit compilation and usage
#[derive(Debug, Clone)]
pub struct CircuitConfig {
    /// Circuit compilation backend
    pub backend: CircuitBackend,
    /// Maximum number of constraints per circuit
    pub max_constraints: usize,
    /// Whether to enable circuit optimizations
    pub enable_optimizations: bool,
    /// Proof system to use
    pub proof_system: CircuitProofSystem,
    /// Circuit parameter cache size
    pub cache_size: usize,
    /// Whether to use universal setup
    pub universal_setup: bool,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            backend: CircuitBackend::Arkworks,
            max_constraints: 1_000_000,
            enable_optimizations: true,
            proof_system: CircuitProofSystem::Groth16,
            cache_size: 100,
            universal_setup: false,
        }
    }
}

/// Supported circuit compilation backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBackend {
    /// Arkworks R1CS backend
    Arkworks,
    /// Circom backend
    Circom,
    /// Noir backend
    Noir,
    /// Custom backend
    Custom,
}

/// Supported proof systems for circuits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitProofSystem {
    Groth16,
    Plonk,
    Marlin,
    Sonic,
}

/// Circuit definition and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    /// Circuit identifier
    pub id: String,
    /// Circuit name
    pub name: String,
    /// Circuit description
    pub description: String,
    /// Number of public inputs
    pub public_inputs: usize,
    /// Number of private inputs
    pub private_inputs: usize,
    /// Number of constraints
    pub constraint_count: usize,
    /// Circuit parameters (serialized)
    pub parameters: Vec<u8>,
    /// Proving key (if available)
    pub proving_key: Option<Vec<u8>>,
    /// Verification key
    pub verification_key: Vec<u8>,
    /// Circuit compilation timestamp
    pub compiled_at: u64,
    /// Circuit version
    pub version: String,
}

/// Private trade validation circuit
#[derive(Debug, Clone)]
pub struct PrivateTradeCircuit {
    /// Trade amount (private)
    pub amount: WhaleAmount,
    /// Sender account (private)
    pub sender: AccountKey,
    /// Recipient account (private)
    pub recipient: AccountKey,
    /// Trade secret/randomness (private)
    pub secret: TradeSecret,
    /// Trade commitment (public)
    pub commitment: [u8; 32],
    /// Nullifier (public)
    pub nullifier: [u8; 32],
    /// Merkle root (public)
    pub merkle_root: [u8; 32],
}

/// Range proof circuit for amount validation
#[derive(Debug, Clone)]
pub struct RangeProofCircuit {
    /// Value being range-proven (private)
    pub value: u64,
    /// Randomness for commitment (private)
    pub randomness: TradeSecret,
    /// Value commitment (public)
    pub commitment: [u8; 32],
    /// Minimum allowed value (public)
    pub min_value: u64,
    /// Maximum allowed value (public)
    pub max_value: u64,
    /// Number of bits for range
    pub range_bits: usize,
}

/// Nullifier derivation circuit
#[derive(Debug, Clone)]
pub struct NullifierCircuit {
    /// Account secret key (private)
    pub secret_key: [u8; 32],
    /// Trade ID (private)
    pub trade_id: TradeId,
    /// Additional entropy (private)
    pub entropy: [u8; 32],
    /// Sequence number (private)
    pub sequence: u64,
    /// Derived nullifier (public)
    pub nullifier: [u8; 32],
    /// Account public key (public)
    pub public_key: AccountKey,
}

/// Mixing protocol circuit
#[derive(Debug, Clone)]
pub struct MixingCircuit {
    /// Input commitments (private)
    pub input_commitments: Vec<[u8; 32]>,
    /// Input amounts (private)
    pub input_amounts: Vec<u64>,
    /// Input randomness (private)
    pub input_randomness: Vec<TradeSecret>,
    /// Output commitments (public)
    pub output_commitments: Vec<[u8; 32]>,
    /// Mixing fee (public)
    pub mixing_fee: u64,
    /// Merkle tree root (public)
    pub merkle_root: [u8; 32],
}

/// Privacy pool membership circuit
#[derive(Debug, Clone)]
pub struct PoolMembershipCircuit {
    /// Deposit amount (private)
    pub amount: u64,
    /// Deposit secret (private)
    pub secret: TradeSecret,
    /// Depositor account (private)
    pub depositor: AccountKey,
    /// Merkle path (private)
    pub merkle_path: Vec<[u8; 32]>,
    /// Leaf index (private)
    pub leaf_index: usize,
    /// Commitment (public)
    pub commitment: [u8; 32],
    /// Merkle root (public)
    pub merkle_root: [u8; 32],
    /// Nullifier (public)
    pub nullifier: [u8; 32],
}

/// Compliance disclosure circuit
#[derive(Debug, Clone)]
pub struct ComplianceCircuit {
    /// Original transaction data (private)
    pub transaction_data: Vec<u8>,
    /// Disclosure key (private)
    pub disclosure_key: [u8; 32],
    /// Officer authorization (private)
    pub officer_auth: [u8; 64],
    /// Disclosed information (public)
    pub disclosed_info: Vec<u8>,
    /// Disclosure commitment (public)
    pub disclosure_commitment: [u8; 32],
}

/// Circuit statistics and performance metrics
#[derive(Debug, Clone, Default)]
pub struct CircuitStats {
    /// Total circuits compiled
    pub circuits_compiled: u64,
    /// Total proofs generated using circuits
    pub proofs_generated: u64,
    /// Total proof verifications
    pub verifications_performed: u64,
    /// Average proof generation time (ms)
    pub avg_proof_time: f64,
    /// Average verification time (ms)
    pub avg_verification_time: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Circuit compilation success rate
    pub compilation_success_rate: f64,
}

/// Circuit manager for compiling and managing ZK circuits
pub struct CircuitManager {
    config: CircuitConfig,
    circuits: tokio::sync::RwLock<HashMap<String, Circuit>>,
    stats: tokio::sync::RwLock<CircuitStats>,
    parameter_cache: tokio::sync::RwLock<HashMap<String, Vec<u8>>>,
}

impl CircuitManager {
    /// Create a new circuit manager
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            config,
            circuits: tokio::sync::RwLock::new(HashMap::new()),
            stats: tokio::sync::RwLock::new(CircuitStats::default()),
            parameter_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Compile a private trade circuit
    pub async fn compile_private_trade_circuit(&self) -> PrivacyResult<String> {
        let circuit_id = "private_trade_v1".to_string();

        let circuit = Circuit {
            id: circuit_id.clone(),
            name: "Private Trade Validation".to_string(),
            description: "Validates private trades with commitments and nullifiers".to_string(),
            public_inputs: 3, // commitment, nullifier, merkle_root
            private_inputs: 4, // amount, sender, recipient, secret
            constraint_count: 50000,
            parameters: self.generate_circuit_parameters("private_trade").await?,
            proving_key: Some(self.generate_proving_key("private_trade").await?),
            verification_key: self.generate_verification_key("private_trade").await?,
            compiled_at: chrono::Utc::now().timestamp() as u64,
            version: "1.0.0".to_string(),
        };

        let mut circuits = self.circuits.write().await;
        circuits.insert(circuit_id.clone(), circuit);

        self.update_compilation_stats(true).await;

        Ok(circuit_id)
    }

    /// Compile a range proof circuit
    pub async fn compile_range_proof_circuit(&self, range_bits: usize) -> PrivacyResult<String> {
        let circuit_id = format!("range_proof_{}bit_v1", range_bits);

        let constraint_count = match range_bits {
            8 => 1000,
            16 => 5000,
            32 => 20000,
            64 => 80000,
            _ => return Err(PrivacyError::InvalidCircuitParameters {
                parameter: "range_bits".to_string(),
            }),
        };

        let circuit = Circuit {
            id: circuit_id.clone(),
            name: format!("{}-bit Range Proof", range_bits),
            description: format!("Proves a value is in range [0, 2^{})", range_bits),
            public_inputs: 4, // commitment, min_value, max_value, range_bits
            private_inputs: 2, // value, randomness
            constraint_count,
            parameters: self.generate_circuit_parameters("range_proof").await?,
            proving_key: Some(self.generate_proving_key("range_proof").await?),
            verification_key: self.generate_verification_key("range_proof").await?,
            compiled_at: chrono::Utc::now().timestamp() as u64,
            version: "1.0.0".to_string(),
        };

        let mut circuits = self.circuits.write().await;
        circuits.insert(circuit_id.clone(), circuit);

        self.update_compilation_stats(true).await;

        Ok(circuit_id)
    }

    /// Compile a nullifier derivation circuit
    pub async fn compile_nullifier_circuit(&self) -> PrivacyResult<String> {
        let circuit_id = "nullifier_derivation_v1".to_string();

        let circuit = Circuit {
            id: circuit_id.clone(),
            name: "Nullifier Derivation".to_string(),
            description: "Derives nullifiers from secret keys and trade data".to_string(),
            public_inputs: 2, // nullifier, public_key
            private_inputs: 4, // secret_key, trade_id, entropy, sequence
            constraint_count: 10000,
            parameters: self.generate_circuit_parameters("nullifier").await?,
            proving_key: Some(self.generate_proving_key("nullifier").await?),
            verification_key: self.generate_verification_key("nullifier").await?,
            compiled_at: chrono::Utc::now().timestamp() as u64,
            version: "1.0.0".to_string(),
        };

        let mut circuits = self.circuits.write().await;
        circuits.insert(circuit_id.clone(), circuit);

        self.update_compilation_stats(true).await;

        Ok(circuit_id)
    }

    /// Compile a mixing protocol circuit
    pub async fn compile_mixing_circuit(&self, max_inputs: usize) -> PrivacyResult<String> {
        let circuit_id = format!("mixing_{}inputs_v1", max_inputs);

        if max_inputs > 100 {
            return Err(PrivacyError::InvalidCircuitParameters {
                parameter: "max_inputs".to_string(),
            });
        }

        let constraint_count = max_inputs * 1000 + 10000; // Rough estimate

        let circuit = Circuit {
            id: circuit_id.clone(),
            name: format!("Mixing Protocol ({} inputs)", max_inputs),
            description: "Validates mixing of multiple transaction inputs".to_string(),
            public_inputs: max_inputs * 2 + 2, // output_commitments + mixing_fee + merkle_root
            private_inputs: max_inputs * 3, // input_commitments + amounts + randomness
            constraint_count,
            parameters: self.generate_circuit_parameters("mixing").await?,
            proving_key: Some(self.generate_proving_key("mixing").await?),
            verification_key: self.generate_verification_key("mixing").await?,
            compiled_at: chrono::Utc::now().timestamp() as u64,
            version: "1.0.0".to_string(),
        };

        let mut circuits = self.circuits.write().await;
        circuits.insert(circuit_id.clone(), circuit);

        self.update_compilation_stats(true).await;

        Ok(circuit_id)
    }

    /// Compile a privacy pool membership circuit
    pub async fn compile_pool_membership_circuit(&self, tree_depth: usize) -> PrivacyResult<String> {
        let circuit_id = format!("pool_membership_depth{}_v1", tree_depth);

        if tree_depth > 32 {
            return Err(PrivacyError::InvalidCircuitParameters {
                parameter: "tree_depth".to_string(),
            });
        }

        let constraint_count = tree_depth * 2000 + 5000; // Hash constraints for Merkle path

        let circuit = Circuit {
            id: circuit_id.clone(),
            name: format!("Pool Membership (depth {})", tree_depth),
            description: "Proves membership in privacy pool without revealing identity".to_string(),
            public_inputs: 3, // commitment, merkle_root, nullifier
            private_inputs: tree_depth + 4, // merkle_path + amount + secret + depositor + leaf_index
            constraint_count,
            parameters: self.generate_circuit_parameters("pool_membership").await?,
            proving_key: Some(self.generate_proving_key("pool_membership").await?),
            verification_key: self.generate_verification_key("pool_membership").await?,
            compiled_at: chrono::Utc::now().timestamp() as u64,
            version: "1.0.0".to_string(),
        };

        let mut circuits = self.circuits.write().await;
        circuits.insert(circuit_id.clone(), circuit);

        self.update_compilation_stats(true).await;

        Ok(circuit_id)
    }

    /// Compile a compliance disclosure circuit
    pub async fn compile_compliance_circuit(&self) -> PrivacyResult<String> {
        let circuit_id = "compliance_disclosure_v1".to_string();

        let circuit = Circuit {
            id: circuit_id.clone(),
            name: "Compliance Disclosure".to_string(),
            description: "Enables selective disclosure for regulatory compliance".to_string(),
            public_inputs: 2, // disclosed_info, disclosure_commitment
            private_inputs: 3, // transaction_data, disclosure_key, officer_auth
            constraint_count: 30000,
            parameters: self.generate_circuit_parameters("compliance").await?,
            proving_key: Some(self.generate_proving_key("compliance").await?),
            verification_key: self.generate_verification_key("compliance").await?,
            compiled_at: chrono::Utc::now().timestamp() as u64,
            version: "1.0.0".to_string(),
        };

        let mut circuits = self.circuits.write().await;
        circuits.insert(circuit_id.clone(), circuit);

        self.update_compilation_stats(true).await;

        Ok(circuit_id)
    }

    /// Get a compiled circuit by ID
    pub async fn get_circuit(&self, circuit_id: &str) -> Option<Circuit> {
        let circuits = self.circuits.read().await;
        circuits.get(circuit_id).cloned()
    }

    /// List all compiled circuits
    pub async fn list_circuits(&self) -> Vec<String> {
        let circuits = self.circuits.read().await;
        circuits.keys().cloned().collect()
    }

    /// Get circuit statistics
    pub async fn get_stats(&self) -> CircuitStats {
        self.stats.read().await.clone()
    }

    /// Validate circuit constraints
    pub async fn validate_circuit(&self, circuit_id: &str) -> PrivacyResult<bool> {
        let circuits = self.circuits.read().await;
        let circuit = circuits.get(circuit_id)
            .ok_or_else(|| PrivacyError::CircuitSetupFailed {
                reason: "Circuit not found".to_string(),
            })?;

        // Check constraint count is within limits
        if circuit.constraint_count > self.config.max_constraints {
            return Ok(false);
        }

        // Validate circuit parameters
        if circuit.parameters.is_empty() {
            return Ok(false);
        }

        // Validate verification key
        if circuit.verification_key.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Optimize circuit for better performance
    pub async fn optimize_circuit(&self, circuit_id: &str) -> PrivacyResult<String> {
        if !self.config.enable_optimizations {
            return Ok(circuit_id.to_string());
        }

        let circuits = self.circuits.read().await;
        let circuit = circuits.get(circuit_id)
            .ok_or_else(|| PrivacyError::CircuitSetupFailed {
                reason: "Circuit not found".to_string(),
            })?
            .clone();

        drop(circuits);

        // Create optimized version
        let optimized_id = format!("{}_optimized", circuit_id);
        let mut optimized_circuit = circuit;
        optimized_circuit.id = optimized_id.clone();
        optimized_circuit.name = format!("{} (Optimized)", optimized_circuit.name);

        // Simulate optimization by reducing constraint count
        optimized_circuit.constraint_count = (optimized_circuit.constraint_count as f64 * 0.8) as usize;

        let mut circuits = self.circuits.write().await;
        circuits.insert(optimized_id.clone(), optimized_circuit);

        Ok(optimized_id)
    }

    /// Export circuit for external use
    pub async fn export_circuit(&self, circuit_id: &str) -> PrivacyResult<Vec<u8>> {
        let circuits = self.circuits.read().await;
        let circuit = circuits.get(circuit_id)
            .ok_or_else(|| PrivacyError::CircuitSetupFailed {
                reason: "Circuit not found".to_string(),
            })?;

        serde_json::to_vec(circuit)
            .map_err(|e| PrivacyError::SerializationFailed { reason: e.to_string() })
    }

    /// Import circuit from external source
    pub async fn import_circuit(&self, circuit_data: &[u8]) -> PrivacyResult<String> {
        let circuit: Circuit = serde_json::from_slice(circuit_data)
            .map_err(|e| PrivacyError::DeserializationFailed { reason: e.to_string() })?;

        let circuit_id = circuit.id.clone();

        let mut circuits = self.circuits.write().await;
        circuits.insert(circuit_id.clone(), circuit);

        Ok(circuit_id)
    }

    /// Generate circuit parameters (mock implementation)
    async fn generate_circuit_parameters(&self, circuit_type: &str) -> PrivacyResult<Vec<u8>> {
        // Check cache first
        let cache_key = format!("{}_params", circuit_type);
        {
            let cache = self.parameter_cache.read().await;
            if let Some(params) = cache.get(&cache_key) {
                return Ok(params.clone());
            }
        }

        // Generate new parameters
        let params = match circuit_type {
            "private_trade" => vec![1u8; 1024],
            "range_proof" => vec![2u8; 512],
            "nullifier" => vec![3u8; 256],
            "mixing" => vec![4u8; 2048],
            "pool_membership" => vec![5u8; 1024],
            "compliance" => vec![6u8; 512],
            _ => vec![0u8; 256],
        };

        // Cache the parameters
        {
            let mut cache = self.parameter_cache.write().await;
            if cache.len() >= self.config.cache_size {
                // Remove oldest entry (simple FIFO)
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                }
            }
            cache.insert(cache_key, params.clone());
        }

        Ok(params)
    }

    /// Generate proving key (mock implementation)
    async fn generate_proving_key(&self, circuit_type: &str) -> PrivacyResult<Vec<u8>> {
        match circuit_type {
            "private_trade" => Ok(vec![11u8; 2048]),
            "range_proof" => Ok(vec![12u8; 1024]),
            "nullifier" => Ok(vec![13u8; 512]),
            "mixing" => Ok(vec![14u8; 4096]),
            "pool_membership" => Ok(vec![15u8; 2048]),
            "compliance" => Ok(vec![16u8; 1024]),
            _ => Ok(vec![10u8; 512]),
        }
    }

    /// Generate verification key (mock implementation)
    async fn generate_verification_key(&self, circuit_type: &str) -> PrivacyResult<Vec<u8>> {
        match circuit_type {
            "private_trade" => Ok(vec![21u8; 128]),
            "range_proof" => Ok(vec![22u8; 64]),
            "nullifier" => Ok(vec![23u8; 32]),
            "mixing" => Ok(vec![24u8; 256]),
            "pool_membership" => Ok(vec![25u8; 128]),
            "compliance" => Ok(vec![26u8; 64]),
            _ => Ok(vec![20u8; 32]),
        }
    }

    /// Update compilation statistics
    async fn update_compilation_stats(&self, success: bool) {
        let mut stats = self.stats.write().await;

        stats.circuits_compiled += 1;

        // Update success rate
        let total_compilations = stats.circuits_compiled as f64;
        let successes = if success {
            (stats.compilation_success_rate * (total_compilations - 1.0) / 100.0) + 1.0
        } else {
            stats.compilation_success_rate * (total_compilations - 1.0) / 100.0
        };
        stats.compilation_success_rate = (successes / total_compilations) * 100.0;
    }

    /// Update proof generation statistics
    pub async fn update_proof_stats(&self, proof_time: f64) {
        let mut stats = self.stats.write().await;

        stats.proofs_generated += 1;

        // Update rolling average
        let total_proofs = stats.proofs_generated as f64;
        stats.avg_proof_time =
            (stats.avg_proof_time * (total_proofs - 1.0) + proof_time) / total_proofs;
    }

    /// Update verification statistics
    pub async fn update_verification_stats(&self, verification_time: f64) {
        let mut stats = self.stats.write().await;

        stats.verifications_performed += 1;

        // Update rolling average
        let total_verifications = stats.verifications_performed as f64;
        stats.avg_verification_time =
            (stats.avg_verification_time * (total_verifications - 1.0) + verification_time) / total_verifications;
    }
}

/// Circuit builder for constructing complex circuits
pub struct CircuitBuilder {
    constraints: Vec<String>,
    public_inputs: Vec<String>,
    private_inputs: Vec<String>,
    circuit_type: String,
}

impl CircuitBuilder {
    /// Create a new circuit builder
    pub fn new(circuit_type: impl Into<String>) -> Self {
        Self {
            constraints: Vec::new(),
            public_inputs: Vec::new(),
            private_inputs: Vec::new(),
            circuit_type: circuit_type.into(),
        }
    }

    /// Add a public input
    pub fn add_public_input(mut self, name: impl Into<String>) -> Self {
        self.public_inputs.push(name.into());
        self
    }

    /// Add a private input
    pub fn add_private_input(mut self, name: impl Into<String>) -> Self {
        self.private_inputs.push(name.into());
        self
    }

    /// Add a constraint
    pub fn add_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    /// Build the circuit
    pub fn build(self) -> PrivacyResult<Circuit> {
        Ok(Circuit {
            id: format!("{}_{}", self.circuit_type, uuid::Uuid::new_v4()),
            name: self.circuit_type.clone(),
            description: format!("Custom {} circuit", self.circuit_type),
            public_inputs: self.public_inputs.len(),
            private_inputs: self.private_inputs.len(),
            constraint_count: self.constraints.len(),
            parameters: vec![0u8; 256], // Placeholder
            proving_key: None,
            verification_key: vec![0u8; 64], // Placeholder
            compiled_at: chrono::Utc::now().timestamp() as u64,
            version: "1.0.0".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> CircuitManager {
        let config = CircuitConfig {
            backend: CircuitBackend::Arkworks,
            max_constraints: 100000,
            enable_optimizations: true,
            proof_system: CircuitProofSystem::Groth16,
            cache_size: 10,
            universal_setup: false,
        };

        CircuitManager::new(config)
    }

    #[tokio::test]
    async fn test_circuit_compilation() {
        let manager = create_test_manager();

        let circuit_id = manager.compile_private_trade_circuit().await.unwrap();
        assert_eq!(circuit_id, "private_trade_v1");

        let circuit = manager.get_circuit(&circuit_id).await.unwrap();
        assert_eq!(circuit.name, "Private Trade Validation");
        assert_eq!(circuit.public_inputs, 3);
        assert_eq!(circuit.private_inputs, 4);
    }

    #[tokio::test]
    async fn test_range_proof_circuit() {
        let manager = create_test_manager();

        let circuit_id = manager.compile_range_proof_circuit(32).await.unwrap();
        assert_eq!(circuit_id, "range_proof_32bit_v1");

        let circuit = manager.get_circuit(&circuit_id).await.unwrap();
        assert_eq!(circuit.constraint_count, 20000);
    }

    #[tokio::test]
    async fn test_mixing_circuit() {
        let manager = create_test_manager();

        let circuit_id = manager.compile_mixing_circuit(5).await.unwrap();
        assert_eq!(circuit_id, "mixing_5inputs_v1");

        let circuit = manager.get_circuit(&circuit_id).await.unwrap();
        assert_eq!(circuit.public_inputs, 12); // 5*2 + 2
        assert_eq!(circuit.private_inputs, 15); // 5*3
    }

    #[tokio::test]
    async fn test_circuit_validation() {
        let manager = create_test_manager();

        let circuit_id = manager.compile_nullifier_circuit().await.unwrap();
        let is_valid = manager.validate_circuit(&circuit_id).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_circuit_optimization() {
        let manager = create_test_manager();

        let circuit_id = manager.compile_pool_membership_circuit(20).await.unwrap();
        let original_circuit = manager.get_circuit(&circuit_id).await.unwrap();

        let optimized_id = manager.optimize_circuit(&circuit_id).await.unwrap();
        let optimized_circuit = manager.get_circuit(&optimized_id).await.unwrap();

        // Optimized circuit should have fewer constraints
        assert!(optimized_circuit.constraint_count < original_circuit.constraint_count);
    }

    #[tokio::test]
    async fn test_circuit_export_import() {
        let manager = create_test_manager();

        let circuit_id = manager.compile_compliance_circuit().await.unwrap();
        let exported_data = manager.export_circuit(&circuit_id).await.unwrap();

        let imported_id = manager.import_circuit(&exported_data).await.unwrap();
        assert_eq!(imported_id, circuit_id);

        let imported_circuit = manager.get_circuit(&imported_id).await.unwrap();
        assert_eq!(imported_circuit.name, "Compliance Disclosure");
    }

    #[tokio::test]
    async fn test_circuit_builder() {
        let circuit = CircuitBuilder::new("test_circuit")
            .add_public_input("public_value")
            .add_private_input("secret_value")
            .add_constraint("public_value == hash(secret_value)")
            .build()
            .unwrap();

        assert_eq!(circuit.public_inputs, 1);
        assert_eq!(circuit.private_inputs, 1);
        assert_eq!(circuit.constraint_count, 1);
    }

    #[tokio::test]
    async fn test_invalid_parameters() {
        let manager = create_test_manager();

        // Invalid range bits
        let result = manager.compile_range_proof_circuit(128).await;
        assert!(matches!(result, Err(PrivacyError::InvalidCircuitParameters { .. })));

        // Too many mixing inputs
        let result = manager.compile_mixing_circuit(200).await;
        assert!(matches!(result, Err(PrivacyError::InvalidCircuitParameters { .. })));

        // Invalid tree depth
        let result = manager.compile_pool_membership_circuit(50).await;
        assert!(matches!(result, Err(PrivacyError::InvalidCircuitParameters { .. })));
    }

    #[tokio::test]
    async fn test_circuit_statistics() {
        let manager = create_test_manager();

        let initial_stats = manager.get_stats().await;
        assert_eq!(initial_stats.circuits_compiled, 0);

        // Compile some circuits
        manager.compile_private_trade_circuit().await.unwrap();
        manager.compile_nullifier_circuit().await.unwrap();

        let final_stats = manager.get_stats().await;
        assert_eq!(final_stats.circuits_compiled, 2);
        assert_eq!(final_stats.compilation_success_rate, 100.0);
    }

    #[tokio::test]
    async fn test_parameter_caching() {
        let manager = create_test_manager();

        // Compile circuit to populate cache
        manager.compile_private_trade_circuit().await.unwrap();

        // Compile same type again - should use cached parameters
        let circuit_id2 = manager.compile_private_trade_circuit().await.unwrap();

        // Should still work (parameters cached)
        let circuit = manager.get_circuit(&circuit_id2).await.unwrap();
        assert!(!circuit.parameters.is_empty());
    }
}