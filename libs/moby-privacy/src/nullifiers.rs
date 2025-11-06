//! Nullifier system for double-spend prevention in private transactions
//!
//! Nullifiers are unique identifiers derived from private inputs that:
//! - Prevent double-spending without revealing transaction linkability
//! - Enable public verification of uniqueness
//! - Maintain privacy of the underlying secrets
//! - Support efficient nullifier set management

use crate::{
    error::{PrivacyError, PrivacyResult},
    engine::TradeSecret,
    proofs::{ZkProof, ProofSystem},
};
use moby_types::{AccountKey, TradeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for the nullifier system
#[derive(Debug, Clone)]
pub struct NullifierConfig {
    /// Hash function to use for nullifier derivation
    pub hash_function: NullifierHashFunction,
    /// Whether to use incremental nullifiers for sequential transactions
    pub use_incremental: bool,
    /// Maximum age of nullifiers before they can be pruned
    pub max_nullifier_age: u64,
    /// Whether to store nullifier derivation proofs
    pub store_derivation_proofs: bool,
    /// Nullifier commitment scheme
    pub commitment_scheme: NullifierCommitmentScheme,
}

impl Default for NullifierConfig {
    fn default() -> Self {
        Self {
            hash_function: NullifierHashFunction::Poseidon,
            use_incremental: false,
            max_nullifier_age: 86400 * 365, // 1 year
            store_derivation_proofs: true,
            commitment_scheme: NullifierCommitmentScheme::Pedersen,
        }
    }
}

/// Supported hash functions for nullifier derivation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullifierHashFunction {
    /// Poseidon hash (ZK-friendly)
    Poseidon,
    /// SHA-256 (standard)
    Sha256,
    /// Blake2b (fast and secure)
    Blake2b,
    /// Keccak-256 (Ethereum compatible)
    Keccak256,
}

/// Nullifier commitment schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullifierCommitmentScheme {
    /// Pedersen commitments
    Pedersen,
    /// ElGamal commitments
    ElGamal,
    /// Custom commitment scheme
    Custom,
}

/// A nullifier for preventing double-spending
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Nullifier {
    /// The nullifier value (hash)
    pub value: [u8; 32],
    /// Hash function used for derivation
    pub hash_function: NullifierHashFunction,
    /// Commitment to the nullifier derivation
    pub commitment: Option<[u8; 32]>,
    /// Zero-knowledge proof of correct derivation
    pub derivation_proof: Option<ZkProof>,
    /// Timestamp when nullifier was created
    pub created_at: u64,
}

/// Nullifier derivation parameters
#[derive(Debug, Clone)]
pub struct NullifierDerivation {
    /// Account key used in derivation
    pub account: AccountKey,
    /// Secret used in derivation
    pub secret: TradeSecret,
    /// Trade ID for context
    pub trade_id: TradeId,
    /// Optional additional entropy
    pub entropy: Option<[u8; 32]>,
    /// Sequence number for incremental nullifiers
    pub sequence: Option<u64>,
}

/// Nullifier spent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullifierRecord {
    /// The nullifier that was spent
    pub nullifier: Nullifier,
    /// Transaction ID where it was spent
    pub spent_in_tx: String,
    /// Timestamp when it was spent
    pub spent_at: u64,
    /// Block height where it was spent (if applicable)
    pub block_height: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Statistics for nullifier operations
#[derive(Debug, Clone, Default)]
pub struct NullifierStats {
    /// Total nullifiers generated
    pub nullifiers_generated: u64,
    /// Total nullifiers spent
    pub nullifiers_spent: u64,
    /// Total double-spend attempts detected
    pub double_spend_attempts: u64,
    /// Success rate for nullifier generation
    pub generation_success_rate: f64,
    /// Average nullifier derivation time (ms)
    pub avg_derivation_time: f64,
    /// Current size of nullifier set
    pub nullifier_set_size: u64,
    /// Number of nullifiers pruned
    pub nullifiers_pruned: u64,
}

/// Nullifier system for double-spend prevention
pub struct NullifierSystem {
    config: NullifierConfig,
    spent_nullifiers: RwLock<HashSet<[u8; 32]>>,
    nullifier_records: RwLock<HashMap<[u8; 32], NullifierRecord>>,
    derivation_cache: RwLock<HashMap<String, Nullifier>>,
    stats: RwLock<NullifierStats>,
    proof_system: Box<dyn ProofSystem + Send + Sync>,
}

impl NullifierSystem {
    /// Create a new nullifier system
    pub fn new(
        config: NullifierConfig,
        proof_system: Box<dyn ProofSystem + Send + Sync>,
    ) -> Self {
        Self {
            config,
            spent_nullifiers: RwLock::new(HashSet::new()),
            nullifier_records: RwLock::new(HashMap::new()),
            derivation_cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(NullifierStats::default()),
            proof_system,
        }
    }

    /// Derive a nullifier from the given parameters
    pub async fn derive_nullifier(
        &self,
        derivation: &NullifierDerivation,
    ) -> PrivacyResult<Nullifier> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = self.compute_cache_key(derivation);
        {
            let cache = self.derivation_cache.read().await;
            if let Some(nullifier) = cache.get(&cache_key) {
                return Ok(nullifier.clone());
            }
        }

        // Prepare derivation inputs
        let mut derivation_inputs = Vec::new();
        derivation_inputs.extend_from_slice(&derivation.account.to_bytes());
        derivation_inputs.extend_from_slice(derivation.secret.as_bytes());
        derivation_inputs.extend_from_slice(&derivation.trade_id.to_bytes());

        if let Some(entropy) = &derivation.entropy {
            derivation_inputs.extend_from_slice(entropy);
        }

        if let Some(sequence) = derivation.sequence {
            derivation_inputs.extend_from_slice(&sequence.to_le_bytes());
        }

        // Derive nullifier value
        let nullifier_value = self.hash_inputs(&derivation_inputs).await?;

        // Generate commitment if enabled
        let commitment = if self.config.store_derivation_proofs {
            Some(self.commit_nullifier_derivation(&derivation_inputs).await?)
        } else {
            None
        };

        // Generate zero-knowledge proof of correct derivation
        let derivation_proof = if self.config.store_derivation_proofs {
            Some(self.generate_derivation_proof(derivation, &nullifier_value).await?)
        } else {
            None
        };

        let nullifier = Nullifier {
            value: nullifier_value,
            hash_function: self.config.hash_function,
            commitment,
            derivation_proof,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        // Cache the result
        {
            let mut cache = self.derivation_cache.write().await;
            cache.insert(cache_key, nullifier.clone());
        }

        // Update statistics
        let derivation_time = start_time.elapsed().as_millis() as f64;
        self.update_generation_stats(derivation_time, true).await;

        Ok(nullifier)
    }

    /// Check if a nullifier has been spent
    pub async fn is_nullifier_spent(&self, nullifier: &[u8; 32]) -> bool {
        let spent = self.spent_nullifiers.read().await;
        spent.contains(nullifier)
    }

    /// Mark a nullifier as spent
    pub async fn spend_nullifier(
        &self,
        nullifier: Nullifier,
        transaction_id: String,
        block_height: Option<u64>,
        metadata: HashMap<String, String>,
    ) -> PrivacyResult<()> {
        let mut spent = self.spent_nullifiers.write().await;
        let mut records = self.nullifier_records.write().await;

        // Check if already spent
        if spent.contains(&nullifier.value) {
            self.update_double_spend_stats().await;
            return Err(PrivacyError::NullifierAlreadySpent {
                nullifier: hex::encode(nullifier.value),
            });
        }

        // Verify nullifier derivation proof if available
        if let Some(ref proof) = nullifier.derivation_proof {
            let is_valid = self.verify_derivation_proof(&nullifier, proof).await?;
            if !is_valid {
                return Err(PrivacyError::InvalidNullifier {
                    reason: "Derivation proof verification failed".to_string(),
                });
            }
        }

        // Mark as spent
        spent.insert(nullifier.value);

        // Create spend record
        let record = NullifierRecord {
            nullifier: nullifier.clone(),
            spent_in_tx: transaction_id,
            spent_at: chrono::Utc::now().timestamp() as u64,
            block_height,
            metadata,
        };

        records.insert(nullifier.value, record);

        // Update statistics
        self.update_spend_stats().await;

        Ok(())
    }

    /// Get nullifier spend record
    pub async fn get_nullifier_record(&self, nullifier: &[u8; 32]) -> Option<NullifierRecord> {
        let records = self.nullifier_records.read().await;
        records.get(nullifier).cloned()
    }

    /// Verify a nullifier derivation proof
    pub async fn verify_nullifier_derivation(
        &self,
        nullifier: &Nullifier,
        derivation: &NullifierDerivation,
    ) -> PrivacyResult<bool> {
        // Re-derive nullifier and compare
        let expected_nullifier = self.derive_nullifier(derivation).await?;

        if expected_nullifier.value != nullifier.value {
            return Ok(false);
        }

        // Verify proof if available
        if let Some(ref proof) = nullifier.derivation_proof {
            self.verify_derivation_proof(nullifier, proof).await
        } else {
            Ok(true)
        }
    }

    /// Batch verify multiple nullifiers
    pub async fn batch_verify_nullifiers(
        &self,
        nullifiers: &[(Nullifier, NullifierDerivation)],
    ) -> PrivacyResult<Vec<bool>> {
        let mut results = Vec::new();

        for (nullifier, derivation) in nullifiers {
            let is_valid = self.verify_nullifier_derivation(nullifier, derivation).await?;
            results.push(is_valid);
        }

        Ok(results)
    }

    /// Generate incremental nullifiers for a sequence of transactions
    pub async fn derive_incremental_nullifiers(
        &self,
        base_derivation: &NullifierDerivation,
        count: usize,
    ) -> PrivacyResult<Vec<Nullifier>> {
        let mut nullifiers = Vec::new();

        for i in 0..count {
            let mut derivation = base_derivation.clone();
            derivation.sequence = Some(i as u64);

            let nullifier = self.derive_nullifier(&derivation).await?;
            nullifiers.push(nullifier);
        }

        Ok(nullifiers)
    }

    /// Prune old nullifier records
    pub async fn prune_old_nullifiers(&self) -> PrivacyResult<usize> {
        let mut records = self.nullifier_records.write().await;
        let current_time = chrono::Utc::now().timestamp() as u64;
        let max_age = self.config.max_nullifier_age;

        let old_nullifiers: Vec<_> = records
            .iter()
            .filter(|(_, record)| current_time - record.spent_at > max_age)
            .map(|(nullifier, _)| *nullifier)
            .collect();

        for nullifier in &old_nullifiers {
            records.remove(nullifier);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.nullifiers_pruned += old_nullifiers.len() as u64;
            stats.nullifier_set_size = records.len() as u64;
        }

        Ok(old_nullifiers.len())
    }

    /// Get current nullifier statistics
    pub async fn get_stats(&self) -> NullifierStats {
        let mut stats = self.stats.read().await.clone();
        let records = self.nullifier_records.read().await;
        stats.nullifier_set_size = records.len() as u64;
        stats
    }

    /// List recent nullifier spends
    pub async fn list_recent_spends(&self, limit: usize) -> Vec<NullifierRecord> {
        let records = self.nullifier_records.read().await;
        let mut spend_list: Vec<_> = records.values().cloned().collect();
        spend_list.sort_by(|a, b| b.spent_at.cmp(&a.spent_at));
        spend_list.into_iter().take(limit).collect()
    }

    /// Export spent nullifiers for external verification
    pub async fn export_spent_nullifiers(&self) -> HashSet<[u8; 32]> {
        self.spent_nullifiers.read().await.clone()
    }

    /// Import spent nullifiers from external source
    pub async fn import_spent_nullifiers(&self, nullifiers: HashSet<[u8; 32]>) -> PrivacyResult<()> {
        let mut spent = self.spent_nullifiers.write().await;
        spent.extend(nullifiers);
        Ok(())
    }

    /// Check for nullifier collisions
    pub async fn check_nullifier_collision(&self, nullifier: &Nullifier) -> bool {
        let spent = self.spent_nullifiers.read().await;
        spent.contains(&nullifier.value)
    }

    /// Hash inputs using the configured hash function
    async fn hash_inputs(&self, inputs: &[u8]) -> PrivacyResult<[u8; 32]> {
        match self.config.hash_function {
            NullifierHashFunction::Poseidon => self.hash_poseidon(inputs).await,
            NullifierHashFunction::Sha256 => self.hash_sha256(inputs).await,
            NullifierHashFunction::Blake2b => self.hash_blake2b(inputs).await,
            NullifierHashFunction::Keccak256 => self.hash_keccak256(inputs).await,
        }
    }

    /// Hash using Poseidon (ZK-friendly)
    async fn hash_poseidon(&self, inputs: &[u8]) -> PrivacyResult<[u8; 32]> {
        // In a real implementation, this would use the Poseidon hash function
        // For now, use SHA-256 as a placeholder
        self.hash_sha256(inputs).await
    }

    /// Hash using SHA-256
    async fn hash_sha256(&self, inputs: &[u8]) -> PrivacyResult<[u8; 32]> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(inputs);
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    /// Hash using Blake2b
    async fn hash_blake2b(&self, inputs: &[u8]) -> PrivacyResult<[u8; 32]> {
        use blake2::{Blake2b512, Digest};
        let mut hasher = Blake2b512::new();
        hasher.update(inputs);
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result[..32]);
        Ok(hash)
    }

    /// Hash using Keccak-256
    async fn hash_keccak256(&self, inputs: &[u8]) -> PrivacyResult<[u8; 32]> {
        use sha3::{Keccak256, Digest};
        let mut hasher = Keccak256::new();
        hasher.update(inputs);
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    /// Commit to nullifier derivation
    async fn commit_nullifier_derivation(&self, inputs: &[u8]) -> PrivacyResult<[u8; 32]> {
        match self.config.commitment_scheme {
            NullifierCommitmentScheme::Pedersen => {
                // Simple commitment using hash for now
                self.hash_sha256(inputs).await
            }
            NullifierCommitmentScheme::ElGamal => {
                // ElGamal commitment (simplified)
                self.hash_sha256(inputs).await
            }
            NullifierCommitmentScheme::Custom => {
                // Custom commitment scheme
                self.hash_sha256(inputs).await
            }
        }
    }

    /// Generate zero-knowledge proof of correct nullifier derivation
    async fn generate_derivation_proof(
        &self,
        derivation: &NullifierDerivation,
        nullifier_value: &[u8; 32],
    ) -> PrivacyResult<ZkProof> {
        let public_inputs = vec![
            nullifier_value.to_vec(),
            derivation.account.to_bytes(),
            derivation.trade_id.to_bytes(),
        ];

        let mut private_inputs = vec![derivation.secret.as_bytes().to_vec()];

        if let Some(entropy) = &derivation.entropy {
            private_inputs.push(entropy.to_vec());
        }

        if let Some(sequence) = derivation.sequence {
            private_inputs.push(sequence.to_le_bytes().to_vec());
        }

        self.proof_system
            .prove("nullifier_derivation", &public_inputs, &private_inputs)
            .await
    }

    /// Verify zero-knowledge proof of nullifier derivation
    async fn verify_derivation_proof(
        &self,
        nullifier: &Nullifier,
        proof: &ZkProof,
    ) -> PrivacyResult<bool> {
        let public_inputs = vec![nullifier.value.to_vec()];

        self.proof_system
            .verify("nullifier_derivation", &public_inputs, proof)
            .await
    }

    /// Compute cache key for nullifier derivation
    fn compute_cache_key(&self, derivation: &NullifierDerivation) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&derivation.account.to_bytes());
        hasher.update(derivation.secret.as_bytes());
        hasher.update(&derivation.trade_id.to_bytes());

        if let Some(entropy) = &derivation.entropy {
            hasher.update(entropy);
        }

        if let Some(sequence) = derivation.sequence {
            hasher.update(&sequence.to_le_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Update generation statistics
    async fn update_generation_stats(&self, derivation_time: f64, success: bool) {
        let mut stats = self.stats.write().await;

        stats.nullifiers_generated += 1;

        // Update rolling average for derivation time
        let total_generated = stats.nullifiers_generated as f64;
        stats.avg_derivation_time =
            (stats.avg_derivation_time * (total_generated - 1.0) + derivation_time) / total_generated;

        // Update success rate
        let successes = if success {
            (stats.generation_success_rate * (total_generated - 1.0) / 100.0) + 1.0
        } else {
            stats.generation_success_rate * (total_generated - 1.0) / 100.0
        };
        stats.generation_success_rate = (successes / total_generated) * 100.0;
    }

    /// Update spend statistics
    async fn update_spend_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.nullifiers_spent += 1;
    }

    /// Update double-spend attempt statistics
    async fn update_double_spend_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.double_spend_attempts += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proofs::MockProofSystem;

    fn create_test_system() -> NullifierSystem {
        let config = NullifierConfig {
            hash_function: NullifierHashFunction::Sha256,
            use_incremental: true,
            max_nullifier_age: 86400,
            store_derivation_proofs: true,
            commitment_scheme: NullifierCommitmentScheme::Pedersen,
        };

        NullifierSystem::new(config, Box::new(MockProofSystem::new()))
    }

    fn create_test_derivation() -> NullifierDerivation {
        NullifierDerivation {
            account: AccountKey::generate_random(),
            secret: TradeSecret::new_random(),
            trade_id: TradeId::new(),
            entropy: Some([1u8; 32]),
            sequence: None,
        }
    }

    #[tokio::test]
    async fn test_nullifier_derivation() {
        let system = create_test_system();
        let derivation = create_test_derivation();

        let nullifier = system.derive_nullifier(&derivation).await.unwrap();

        assert_eq!(nullifier.hash_function, NullifierHashFunction::Sha256);
        assert!(nullifier.commitment.is_some());
        assert!(nullifier.derivation_proof.is_some());
        assert!(nullifier.created_at > 0);
    }

    #[tokio::test]
    async fn test_nullifier_spending() {
        let system = create_test_system();
        let derivation = create_test_derivation();

        let nullifier = system.derive_nullifier(&derivation).await.unwrap();

        // Initially not spent
        assert!(!system.is_nullifier_spent(&nullifier.value).await);

        // Spend the nullifier
        system.spend_nullifier(
            nullifier.clone(),
            "test_tx_123".to_string(),
            Some(12345),
            HashMap::new(),
        ).await.unwrap();

        // Now should be marked as spent
        assert!(system.is_nullifier_spent(&nullifier.value).await);

        // Should have a spend record
        let record = system.get_nullifier_record(&nullifier.value).await.unwrap();
        assert_eq!(record.spent_in_tx, "test_tx_123");
        assert_eq!(record.block_height, Some(12345));
    }

    #[tokio::test]
    async fn test_double_spend_prevention() {
        let system = create_test_system();
        let derivation = create_test_derivation();

        let nullifier = system.derive_nullifier(&derivation).await.unwrap();

        // First spend should succeed
        system.spend_nullifier(
            nullifier.clone(),
            "test_tx_1".to_string(),
            Some(12345),
            HashMap::new(),
        ).await.unwrap();

        // Second spend should fail
        let result = system.spend_nullifier(
            nullifier.clone(),
            "test_tx_2".to_string(),
            Some(12346),
            HashMap::new(),
        ).await;

        assert!(matches!(result, Err(PrivacyError::NullifierAlreadySpent { .. })));
    }

    #[tokio::test]
    async fn test_nullifier_verification() {
        let system = create_test_system();
        let derivation = create_test_derivation();

        let nullifier = system.derive_nullifier(&derivation).await.unwrap();

        let is_valid = system.verify_nullifier_derivation(&nullifier, &derivation).await.unwrap();
        assert!(is_valid);

        // Test with wrong derivation
        let mut wrong_derivation = derivation.clone();
        wrong_derivation.secret = TradeSecret::new_random();

        let is_valid = system.verify_nullifier_derivation(&nullifier, &wrong_derivation).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_incremental_nullifiers() {
        let system = create_test_system();
        let base_derivation = create_test_derivation();

        let nullifiers = system.derive_incremental_nullifiers(&base_derivation, 3).await.unwrap();

        assert_eq!(nullifiers.len(), 3);

        // Each nullifier should be different
        for i in 0..nullifiers.len() {
            for j in i + 1..nullifiers.len() {
                assert_ne!(nullifiers[i].value, nullifiers[j].value);
            }
        }
    }

    #[tokio::test]
    async fn test_batch_verification() {
        let system = create_test_system();

        let mut nullifiers_and_derivations = Vec::new();
        for _ in 0..3 {
            let derivation = create_test_derivation();
            let nullifier = system.derive_nullifier(&derivation).await.unwrap();
            nullifiers_and_derivations.push((nullifier, derivation));
        }

        let results = system.batch_verify_nullifiers(&nullifiers_and_derivations).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|&valid| valid));
    }

    #[tokio::test]
    async fn test_nullifier_caching() {
        let system = create_test_system();
        let derivation = create_test_derivation();

        // First derivation
        let nullifier1 = system.derive_nullifier(&derivation).await.unwrap();

        // Second derivation with same parameters should return cached result
        let nullifier2 = system.derive_nullifier(&derivation).await.unwrap();

        assert_eq!(nullifier1.value, nullifier2.value);
        assert_eq!(nullifier1.created_at, nullifier2.created_at);
    }

    #[tokio::test]
    async fn test_nullifier_pruning() {
        let system = create_test_system();

        // Create and spend some nullifiers
        for i in 0..5 {
            let derivation = create_test_derivation();
            let nullifier = system.derive_nullifier(&derivation).await.unwrap();

            system.spend_nullifier(
                nullifier,
                format!("tx_{}", i),
                Some(i),
                HashMap::new(),
            ).await.unwrap();
        }

        let initial_stats = system.get_stats().await;
        assert_eq!(initial_stats.nullifier_set_size, 5);

        // Pruning with current max age shouldn't remove anything
        let pruned = system.prune_old_nullifiers().await.unwrap();
        assert_eq!(pruned, 0);
    }

    #[tokio::test]
    async fn test_different_hash_functions() {
        for hash_function in &[
            NullifierHashFunction::Sha256,
            NullifierHashFunction::Blake2b,
            NullifierHashFunction::Keccak256,
        ] {
            let config = NullifierConfig {
                hash_function: *hash_function,
                use_incremental: false,
                max_nullifier_age: 86400,
                store_derivation_proofs: false,
                commitment_scheme: NullifierCommitmentScheme::Pedersen,
            };

            let system = NullifierSystem::new(config, Box::new(MockProofSystem::new()));
            let derivation = create_test_derivation();

            let nullifier = system.derive_nullifier(&derivation).await.unwrap();
            assert_eq!(nullifier.hash_function, *hash_function);
        }
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let system = create_test_system();

        let initial_stats = system.get_stats().await;
        assert_eq!(initial_stats.nullifiers_generated, 0);
        assert_eq!(initial_stats.nullifiers_spent, 0);

        // Generate and spend some nullifiers
        for _ in 0..3 {
            let derivation = create_test_derivation();
            let nullifier = system.derive_nullifier(&derivation).await.unwrap();
            system.spend_nullifier(
                nullifier,
                "test_tx".to_string(),
                None,
                HashMap::new(),
            ).await.unwrap();
        }

        let final_stats = system.get_stats().await;
        assert_eq!(final_stats.nullifiers_generated, 3);
        assert_eq!(final_stats.nullifiers_spent, 3);
        assert!(final_stats.avg_derivation_time > 0.0);
    }
}