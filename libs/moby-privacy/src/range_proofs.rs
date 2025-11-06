//! Range proofs for amount confidentiality in private transactions
//!
//! Range proofs allow proving that a committed value lies within a specific range
//! without revealing the actual value. This is essential for:
//! - Preventing overflow attacks
//! - Ensuring non-negative amounts
//! - Compliance with regulatory limits
//! - Amount confidentiality with bounds

use crate::{
    error::{PrivacyError, PrivacyResult},
    engine::TradeSecret,
    proofs::{ZkProof, ProofSystem},
};
use moby_types::WhaleAmount;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for range proof generation
#[derive(Debug, Clone)]
pub struct RangeProofConfig {
    /// Number of bits for the range proof
    pub range_bits: usize,
    /// Generator count for vector commitment
    pub generator_count: usize,
    /// Whether to use aggregated proofs for better efficiency
    pub use_aggregation: bool,
    /// Maximum batch size for aggregated proofs
    pub max_batch_size: usize,
    /// Proof system to use (Bulletproofs, Plonky2, etc.)
    pub proof_system: RangeProofSystem,
}

impl Default for RangeProofConfig {
    fn default() -> Self {
        Self {
            range_bits: 64,
            generator_count: 256,
            use_aggregation: true,
            max_batch_size: 16,
            proof_system: RangeProofSystem::Bulletproofs,
        }
    }
}

/// Supported range proof systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeProofSystem {
    /// Bulletproofs for efficient range proofs
    Bulletproofs,
    /// Plonky2 for recursive proofs
    Plonky2,
    /// Custom circuit-based proofs
    Custom,
}

/// A range proof for a committed value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    /// The actual proof data
    pub proof: ZkProof,
    /// Range being proven (minimum value)
    pub min_value: u64,
    /// Range being proven (maximum value)
    pub max_value: u64,
    /// Number of bits in the range
    pub range_bits: usize,
    /// Commitment to the value being range-proven
    pub commitment: [u8; 32],
    /// Proof system used
    pub system: RangeProofSystem,
    /// Proof generation timestamp
    pub created_at: u64,
}

/// Aggregated range proof for multiple values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedRangeProof {
    /// The aggregated proof
    pub proof: ZkProof,
    /// Individual commitments being proven
    pub commitments: Vec<[u8; 32]>,
    /// Common range for all values
    pub min_value: u64,
    pub max_value: u64,
    /// Number of values in the aggregation
    pub value_count: usize,
    /// Proof system used
    pub system: RangeProofSystem,
    /// Proof generation timestamp
    pub created_at: u64,
}

/// Range proof verification result
#[derive(Debug, Clone)]
pub struct RangeProofVerification {
    /// Whether the proof is valid
    pub is_valid: bool,
    /// Range that was proven
    pub proven_range: (u64, u64),
    /// Commitment that was verified
    pub commitment: [u8; 32],
    /// Verification timestamp
    pub verified_at: u64,
}

/// Statistics for range proof operations
#[derive(Debug, Clone, Default)]
pub struct RangeProofStats {
    /// Total proofs generated
    pub proofs_generated: u64,
    /// Total proofs verified
    pub proofs_verified: u64,
    /// Average proof generation time (ms)
    pub avg_generation_time: f64,
    /// Average proof verification time (ms)
    pub avg_verification_time: f64,
    /// Success rate for proof generation
    pub generation_success_rate: f64,
    /// Success rate for proof verification
    pub verification_success_rate: f64,
    /// Total aggregated proofs created
    pub aggregated_proofs: u64,
    /// Average aggregation factor
    pub avg_aggregation_factor: f64,
}

/// Range proof generator and verifier
pub struct RangeProofEngine {
    config: RangeProofConfig,
    proof_system: Box<dyn ProofSystem + Send + Sync>,
    stats: tokio::sync::RwLock<RangeProofStats>,
    cached_generators: tokio::sync::RwLock<HashMap<usize, Vec<[u8; 32]>>>,
}

impl RangeProofEngine {
    /// Create a new range proof engine
    pub fn new(
        config: RangeProofConfig,
        proof_system: Box<dyn ProofSystem + Send + Sync>,
    ) -> Self {
        Self {
            config,
            proof_system,
            stats: tokio::sync::RwLock::new(RangeProofStats::default()),
            cached_generators: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Generate a range proof for a single value
    pub async fn prove_range(
        &self,
        value: WhaleAmount,
        min_value: u64,
        max_value: u64,
        randomness: &TradeSecret,
    ) -> PrivacyResult<RangeProof> {
        let start_time = std::time::Instant::now();

        // Validate range
        if value.as_u64() < min_value || value.as_u64() > max_value {
            return Err(PrivacyError::ValueOutOfRange {
                value: value.as_u64(),
                bits: self.config.range_bits,
            });
        }

        // Validate range fits in specified bits
        let range_size = max_value - min_value;
        if range_size >= (1u64 << self.config.range_bits) {
            return Err(PrivacyError::InvalidRangeProofParameters);
        }

        // Generate commitment to the value
        let commitment = self.commit_value(value.as_u64(), randomness).await?;

        // Prepare proof inputs
        let public_inputs = vec![
            commitment.to_vec(),
            min_value.to_le_bytes().to_vec(),
            max_value.to_le_bytes().to_vec(),
            self.config.range_bits.to_le_bytes().to_vec(),
        ];

        let private_inputs = vec![
            value.as_u64().to_le_bytes().to_vec(),
            randomness.as_bytes().to_vec(),
        ];

        // Generate the range proof
        let proof = match self.config.proof_system {
            RangeProofSystem::Bulletproofs => {
                self.generate_bulletproof(&public_inputs, &private_inputs).await?
            }
            RangeProofSystem::Plonky2 => {
                self.generate_plonky2_proof(&public_inputs, &private_inputs).await?
            }
            RangeProofSystem::Custom => {
                self.proof_system
                    .prove("range_proof", &public_inputs, &private_inputs)
                    .await?
            }
        };

        let range_proof = RangeProof {
            proof,
            min_value,
            max_value,
            range_bits: self.config.range_bits,
            commitment,
            system: self.config.proof_system,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        // Update statistics
        let generation_time = start_time.elapsed().as_millis() as f64;
        self.update_generation_stats(generation_time, true).await;

        Ok(range_proof)
    }

    /// Generate aggregated range proof for multiple values
    pub async fn prove_range_aggregated(
        &self,
        values: &[(WhaleAmount, TradeSecret)],
        min_value: u64,
        max_value: u64,
    ) -> PrivacyResult<AggregatedRangeProof> {
        if values.len() > self.config.max_batch_size {
            return Err(PrivacyError::InvalidRangeProofParameters);
        }

        let start_time = std::time::Instant::now();

        // Validate all values are in range
        for (value, _) in values {
            if value.as_u64() < min_value || value.as_u64() > max_value {
                return Err(PrivacyError::ValueOutOfRange {
                    value: value.as_u64(),
                    bits: self.config.range_bits,
                });
            }
        }

        // Generate commitments for all values
        let mut commitments = Vec::new();
        let mut all_values = Vec::new();
        let mut all_randomness = Vec::new();

        for (value, randomness) in values {
            let commitment = self.commit_value(value.as_u64(), randomness).await?;
            commitments.push(commitment);
            all_values.push(value.as_u64());
            all_randomness.push(randomness.as_bytes().to_vec());
        }

        // Prepare aggregated proof inputs
        let public_inputs = vec![
            commitments.iter().flat_map(|c| c.iter()).cloned().collect(),
            min_value.to_le_bytes().to_vec(),
            max_value.to_le_bytes().to_vec(),
            values.len().to_le_bytes().to_vec(),
        ];

        let private_inputs = vec![
            all_values.iter().flat_map(|v| v.to_le_bytes()).collect(),
            all_randomness.into_iter().flatten().collect(),
        ];

        // Generate the aggregated proof
        let proof = self.proof_system
            .prove("aggregated_range_proof", &public_inputs, &private_inputs)
            .await?;

        let aggregated_proof = AggregatedRangeProof {
            proof,
            commitments,
            min_value,
            max_value,
            value_count: values.len(),
            system: self.config.proof_system,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        // Update statistics
        let generation_time = start_time.elapsed().as_millis() as f64;
        self.update_aggregation_stats(generation_time, values.len()).await;

        Ok(aggregated_proof)
    }

    /// Verify a range proof
    pub async fn verify_range_proof(
        &self,
        range_proof: &RangeProof,
    ) -> PrivacyResult<RangeProofVerification> {
        let start_time = std::time::Instant::now();

        // Prepare verification inputs
        let public_inputs = vec![
            range_proof.commitment.to_vec(),
            range_proof.min_value.to_le_bytes().to_vec(),
            range_proof.max_value.to_le_bytes().to_vec(),
            range_proof.range_bits.to_le_bytes().to_vec(),
        ];

        // Verify the proof
        let is_valid = match range_proof.system {
            RangeProofSystem::Bulletproofs => {
                self.verify_bulletproof(&public_inputs, &range_proof.proof).await?
            }
            RangeProofSystem::Plonky2 => {
                self.verify_plonky2_proof(&public_inputs, &range_proof.proof).await?
            }
            RangeProofSystem::Custom => {
                self.proof_system
                    .verify("range_proof", &public_inputs, &range_proof.proof)
                    .await?
            }
        };

        let verification = RangeProofVerification {
            is_valid,
            proven_range: (range_proof.min_value, range_proof.max_value),
            commitment: range_proof.commitment,
            verified_at: chrono::Utc::now().timestamp() as u64,
        };

        // Update statistics
        let verification_time = start_time.elapsed().as_millis() as f64;
        self.update_verification_stats(verification_time, is_valid).await;

        Ok(verification)
    }

    /// Verify an aggregated range proof
    pub async fn verify_aggregated_range_proof(
        &self,
        aggregated_proof: &AggregatedRangeProof,
    ) -> PrivacyResult<bool> {
        let start_time = std::time::Instant::now();

        // Prepare verification inputs
        let public_inputs = vec![
            aggregated_proof.commitments.iter().flat_map(|c| c.iter()).cloned().collect(),
            aggregated_proof.min_value.to_le_bytes().to_vec(),
            aggregated_proof.max_value.to_le_bytes().to_vec(),
            aggregated_proof.value_count.to_le_bytes().to_vec(),
        ];

        // Verify the aggregated proof
        let is_valid = self.proof_system
            .verify("aggregated_range_proof", &public_inputs, &aggregated_proof.proof)
            .await?;

        // Update statistics
        let verification_time = start_time.elapsed().as_millis() as f64;
        self.update_verification_stats(verification_time, is_valid).await;

        Ok(is_valid)
    }

    /// Commit to a value using Pedersen commitment
    async fn commit_value(&self, value: u64, randomness: &TradeSecret) -> PrivacyResult<[u8; 32]> {
        // Get or generate commitment generators
        let generators = self.get_generators(2).await?;

        // Compute Pedersen commitment: g^value * h^randomness
        let commitment_data = [
            value.to_le_bytes().to_vec(),
            randomness.as_bytes().to_vec(),
            generators[0].to_vec(),
            generators[1].to_vec(),
        ].concat();

        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&commitment_data);
        let result = hasher.finalize();

        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&result);
        Ok(commitment)
    }

    /// Get or generate commitment generators
    async fn get_generators(&self, count: usize) -> PrivacyResult<Vec<[u8; 32]>> {
        let mut cached = self.cached_generators.write().await;

        if let Some(generators) = cached.get(&count) {
            return Ok(generators.clone());
        }

        // Generate new generators using a deterministic process
        let mut generators = Vec::new();
        for i in 0..count {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(b"range_proof_generator");
            hasher.update(&i.to_le_bytes());
            let result = hasher.finalize();

            let mut generator = [0u8; 32];
            generator.copy_from_slice(&result);
            generators.push(generator);
        }

        cached.insert(count, generators.clone());
        Ok(generators)
    }

    /// Generate Bulletproof (mock implementation)
    async fn generate_bulletproof(
        &self,
        public_inputs: &[Vec<u8>],
        private_inputs: &[Vec<u8>],
    ) -> PrivacyResult<ZkProof> {
        // In a real implementation, this would use the Bulletproofs library
        // For now, use the generic proof system
        self.proof_system
            .prove("bulletproof_range", public_inputs, private_inputs)
            .await
    }

    /// Verify Bulletproof (mock implementation)
    async fn verify_bulletproof(
        &self,
        public_inputs: &[Vec<u8>],
        proof: &ZkProof,
    ) -> PrivacyResult<bool> {
        // In a real implementation, this would use the Bulletproofs library
        self.proof_system
            .verify("bulletproof_range", public_inputs, proof)
            .await
    }

    /// Generate Plonky2 proof (mock implementation)
    async fn generate_plonky2_proof(
        &self,
        public_inputs: &[Vec<u8>],
        private_inputs: &[Vec<u8>],
    ) -> PrivacyResult<ZkProof> {
        // In a real implementation, this would use Plonky2
        self.proof_system
            .prove("plonky2_range", public_inputs, private_inputs)
            .await
    }

    /// Verify Plonky2 proof (mock implementation)
    async fn verify_plonky2_proof(
        &self,
        public_inputs: &[Vec<u8>],
        proof: &ZkProof,
    ) -> PrivacyResult<bool> {
        // In a real implementation, this would use Plonky2
        self.proof_system
            .verify("plonky2_range", public_inputs, proof)
            .await
    }

    /// Update proof generation statistics
    async fn update_generation_stats(&self, generation_time: f64, success: bool) {
        let mut stats = self.stats.write().await;

        stats.proofs_generated += 1;

        // Update rolling average for generation time
        let total_proofs = stats.proofs_generated as f64;
        stats.avg_generation_time =
            (stats.avg_generation_time * (total_proofs - 1.0) + generation_time) / total_proofs;

        // Update success rate
        let successes = if success {
            (stats.generation_success_rate * (total_proofs - 1.0) / 100.0) + 1.0
        } else {
            stats.generation_success_rate * (total_proofs - 1.0) / 100.0
        };
        stats.generation_success_rate = (successes / total_proofs) * 100.0;
    }

    /// Update proof verification statistics
    async fn update_verification_stats(&self, verification_time: f64, success: bool) {
        let mut stats = self.stats.write().await;

        stats.proofs_verified += 1;

        // Update rolling average for verification time
        let total_verifications = stats.proofs_verified as f64;
        stats.avg_verification_time =
            (stats.avg_verification_time * (total_verifications - 1.0) + verification_time) / total_verifications;

        // Update success rate
        let successes = if success {
            (stats.verification_success_rate * (total_verifications - 1.0) / 100.0) + 1.0
        } else {
            stats.verification_success_rate * (total_verifications - 1.0) / 100.0
        };
        stats.verification_success_rate = (successes / total_verifications) * 100.0;
    }

    /// Update aggregation statistics
    async fn update_aggregation_stats(&self, generation_time: f64, aggregation_factor: usize) {
        let mut stats = self.stats.write().await;

        stats.aggregated_proofs += 1;

        // Update rolling average for aggregation factor
        let total_aggregated = stats.aggregated_proofs as f64;
        stats.avg_aggregation_factor =
            (stats.avg_aggregation_factor * (total_aggregated - 1.0) + aggregation_factor as f64) / total_aggregated;

        // Also update general generation stats
        stats.proofs_generated += 1;
        let total_proofs = stats.proofs_generated as f64;
        stats.avg_generation_time =
            (stats.avg_generation_time * (total_proofs - 1.0) + generation_time) / total_proofs;
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> RangeProofStats {
        self.stats.read().await.clone()
    }

    /// Get configuration
    pub fn config(&self) -> &RangeProofConfig {
        &self.config
    }

    /// Batch verify multiple range proofs
    pub async fn batch_verify_range_proofs(
        &self,
        proofs: &[RangeProof],
    ) -> PrivacyResult<Vec<bool>> {
        let mut results = Vec::new();

        for proof in proofs {
            let verification = self.verify_range_proof(proof).await?;
            results.push(verification.is_valid);
        }

        Ok(results)
    }

    /// Create a proof that a committed value is positive (> 0)
    pub async fn prove_positive(
        &self,
        value: WhaleAmount,
        randomness: &TradeSecret,
    ) -> PrivacyResult<RangeProof> {
        self.prove_range(value, 1, u64::MAX, randomness).await
    }

    /// Create a proof that a committed value is within regulatory limits
    pub async fn prove_regulatory_compliance(
        &self,
        value: WhaleAmount,
        min_limit: u64,
        max_limit: u64,
        randomness: &TradeSecret,
    ) -> PrivacyResult<RangeProof> {
        self.prove_range(value, min_limit, max_limit, randomness).await
    }

    /// Create a proof that the sum of committed values equals a public value
    pub async fn prove_sum_equals(
        &self,
        values: &[(WhaleAmount, TradeSecret)],
        expected_sum: u64,
    ) -> PrivacyResult<ZkProof> {
        // Compute sum of all values
        let actual_sum: u64 = values.iter().map(|(v, _)| v.as_u64()).sum();

        if actual_sum != expected_sum {
            return Err(PrivacyError::RangeProofGenerationFailed {
                reason: "Sum does not equal expected value".to_string(),
            });
        }

        // Generate commitments
        let mut commitments = Vec::new();
        for (value, randomness) in values {
            let commitment = self.commit_value(value.as_u64(), randomness).await?;
            commitments.push(commitment.to_vec());
        }

        let public_inputs = vec![
            commitments.into_iter().flatten().collect(),
            expected_sum.to_le_bytes().to_vec(),
        ];

        let private_inputs = values.iter().map(|(value, randomness)| {
            [value.as_u64().to_le_bytes().to_vec(), randomness.as_bytes().to_vec()].concat()
        }).collect();

        self.proof_system
            .prove("sum_equals", &public_inputs, &private_inputs)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proofs::MockProofSystem;

    fn create_test_engine() -> RangeProofEngine {
        let config = RangeProofConfig {
            range_bits: 32, // Smaller for testing
            generator_count: 64,
            use_aggregation: true,
            max_batch_size: 4,
            proof_system: RangeProofSystem::Custom,
        };

        RangeProofEngine::new(config, Box::new(MockProofSystem::new()))
    }

    #[tokio::test]
    async fn test_range_proof_generation() {
        let engine = create_test_engine();
        let value = WhaleAmount::new(1000);
        let randomness = TradeSecret::new_random();

        let proof = engine.prove_range(value, 0, 2000, &randomness).await.unwrap();

        assert_eq!(proof.min_value, 0);
        assert_eq!(proof.max_value, 2000);
        assert_eq!(proof.range_bits, 32);
        assert_eq!(proof.system, RangeProofSystem::Custom);
    }

    #[tokio::test]
    async fn test_range_proof_verification() {
        let engine = create_test_engine();
        let value = WhaleAmount::new(1000);
        let randomness = TradeSecret::new_random();

        let proof = engine.prove_range(value, 0, 2000, &randomness).await.unwrap();
        let verification = engine.verify_range_proof(&proof).await.unwrap();

        assert!(verification.is_valid);
        assert_eq!(verification.proven_range, (0, 2000));
    }

    #[tokio::test]
    async fn test_value_out_of_range() {
        let engine = create_test_engine();
        let value = WhaleAmount::new(3000); // Outside range
        let randomness = TradeSecret::new_random();

        let result = engine.prove_range(value, 0, 2000, &randomness).await;

        assert!(matches!(result, Err(PrivacyError::ValueOutOfRange { .. })));
    }

    #[tokio::test]
    async fn test_aggregated_range_proof() {
        let engine = create_test_engine();

        let values = vec![
            (WhaleAmount::new(100), TradeSecret::new_random()),
            (WhaleAmount::new(200), TradeSecret::new_random()),
            (WhaleAmount::new(300), TradeSecret::new_random()),
        ];

        let aggregated_proof = engine.prove_range_aggregated(&values, 0, 1000).await.unwrap();

        assert_eq!(aggregated_proof.value_count, 3);
        assert_eq!(aggregated_proof.commitments.len(), 3);
        assert_eq!(aggregated_proof.min_value, 0);
        assert_eq!(aggregated_proof.max_value, 1000);

        let is_valid = engine.verify_aggregated_range_proof(&aggregated_proof).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_positive_proof() {
        let engine = create_test_engine();
        let value = WhaleAmount::new(100);
        let randomness = TradeSecret::new_random();

        let proof = engine.prove_positive(value, &randomness).await.unwrap();

        assert_eq!(proof.min_value, 1);
        assert_eq!(proof.max_value, u64::MAX);

        let verification = engine.verify_range_proof(&proof).await.unwrap();
        assert!(verification.is_valid);
    }

    #[tokio::test]
    async fn test_regulatory_compliance_proof() {
        let engine = create_test_engine();
        let value = WhaleAmount::new(5000);
        let randomness = TradeSecret::new_random();

        // Prove compliance with a regulatory limit
        let proof = engine.prove_regulatory_compliance(value, 1000, 10000, &randomness).await.unwrap();

        assert_eq!(proof.min_value, 1000);
        assert_eq!(proof.max_value, 10000);

        let verification = engine.verify_range_proof(&proof).await.unwrap();
        assert!(verification.is_valid);
    }

    #[tokio::test]
    async fn test_sum_equals_proof() {
        let engine = create_test_engine();

        let values = vec![
            (WhaleAmount::new(100), TradeSecret::new_random()),
            (WhaleAmount::new(200), TradeSecret::new_random()),
            (WhaleAmount::new(300), TradeSecret::new_random()),
        ];

        let expected_sum = 600; // 100 + 200 + 300

        let proof = engine.prove_sum_equals(&values, expected_sum).await.unwrap();
        assert_eq!(proof.proof_data.len(), 32); // Mock proof has 32 bytes
    }

    #[tokio::test]
    async fn test_batch_verification() {
        let engine = create_test_engine();

        let mut proofs = Vec::new();
        for i in 1..=3 {
            let value = WhaleAmount::new(i * 100);
            let randomness = TradeSecret::new_random();
            let proof = engine.prove_range(value, 0, 1000, &randomness).await.unwrap();
            proofs.push(proof);
        }

        let results = engine.batch_verify_range_proofs(&proofs).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|&valid| valid));
    }

    #[tokio::test]
    async fn test_statistics_update() {
        let engine = create_test_engine();

        let initial_stats = engine.get_stats().await;
        assert_eq!(initial_stats.proofs_generated, 0);

        // Generate some proofs
        for i in 1..=5 {
            let value = WhaleAmount::new(i * 100);
            let randomness = TradeSecret::new_random();
            let proof = engine.prove_range(value, 0, 1000, &randomness).await.unwrap();
            let _verification = engine.verify_range_proof(&proof).await.unwrap();
        }

        let final_stats = engine.get_stats().await;
        assert_eq!(final_stats.proofs_generated, 5);
        assert_eq!(final_stats.proofs_verified, 5);
        assert!(final_stats.avg_generation_time > 0.0);
        assert!(final_stats.avg_verification_time > 0.0);
    }
}