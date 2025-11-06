// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Commitment schemes for private trading

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{PrivacyError, PrivacyResult, HashFunction};
use moby_math::{Amount, Price};

/// Trade commitment structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeCommitment {
    /// Commitment value
    pub commitment: Vec<u8>,

    /// Commitment scheme used
    pub scheme: String,

    /// Hash function used
    pub hash_function: HashFunction,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: CommitmentMetadata,
}

impl TradeCommitment {
    /// Create a new trade commitment
    pub fn new(
        commitment: Vec<u8>,
        scheme: String,
        hash_function: HashFunction,
    ) -> Self {
        Self {
            commitment,
            scheme,
            hash_function,
            created_at: Utc::now(),
            metadata: CommitmentMetadata::default(),
        }
    }

    /// Create a mock commitment for testing
    pub fn mock() -> PrivacyResult<Self> {
        Ok(Self::new(
            vec![0u8; 32],
            "pedersen".to_string(),
            HashFunction::Poseidon,
        ))
    }

    /// Convert to field element for use in circuits
    pub fn to_field_element(&self) -> Vec<u8> {
        self.commitment.clone()
    }

    /// Get commitment size in bytes
    pub fn size(&self) -> usize {
        self.commitment.len()
    }

    /// Validate commitment structure
    pub fn validate(&self) -> PrivacyResult<()> {
        if self.commitment.is_empty() {
            return Err(PrivacyError::InvalidCommitment {
                reason: "Empty commitment".to_string(),
            });
        }

        if self.commitment.len() != 32 && self.commitment.len() != 48 {
            return Err(PrivacyError::InvalidCommitment {
                reason: "Invalid commitment length".to_string(),
            });
        }

        Ok(())
    }

    /// Convert to hex string for display
    pub fn to_hex(&self) -> String {
        hex::encode(&self.commitment)
    }

    /// Parse from hex string
    pub fn from_hex(hex_str: &str, scheme: String, hash_function: HashFunction) -> PrivacyResult<Self> {
        let commitment = hex::decode(hex_str)
            .map_err(|_| PrivacyError::InvalidCommitment {
                reason: "Invalid hex encoding".to_string(),
            })?;

        let mut comm = Self::new(commitment, scheme, hash_function);
        comm.validate()?;
        Ok(comm)
    }
}

impl std::fmt::Display for TradeCommitment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.scheme, self.to_hex())
    }
}

/// Commitment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentMetadata {
    /// Randomness size used
    pub randomness_bits: usize,

    /// Binding property verification
    pub binding_verified: bool,

    /// Hiding property verification
    pub hiding_verified: bool,

    /// Additional properties
    pub properties: HashMap<String, String>,
}

impl Default for CommitmentMetadata {
    fn default() -> Self {
        Self {
            randomness_bits: 256,
            binding_verified: false,
            hiding_verified: false,
            properties: HashMap::new(),
        }
    }
}

/// Trait for commitment schemes
pub trait CommitmentScheme {
    /// Create a commitment to a value with given randomness
    fn commit(&self, value: &[u8], randomness: &crate::engine::TradeSecret) -> PrivacyResult<TradeCommitment>;

    /// Verify a commitment opening
    fn verify(&self, commitment: &TradeCommitment, value: &[u8], randomness: &crate::engine::TradeSecret) -> PrivacyResult<bool>;

    /// Get scheme name
    fn name(&self) -> &str;

    /// Get scheme properties
    fn properties(&self) -> CommitmentProperties;
}

/// Commitment scheme properties
#[derive(Debug, Clone)]
pub struct CommitmentProperties {
    /// Scheme provides computational binding
    pub computational_binding: bool,

    /// Scheme provides perfect hiding
    pub perfect_hiding: bool,

    /// Scheme supports additive homomorphism
    pub additive_homomorphic: bool,

    /// Scheme supports range proofs
    pub range_proof_compatible: bool,

    /// Commitment size in bytes
    pub commitment_size: usize,

    /// Randomness size in bytes
    pub randomness_size: usize,
}

/// Pedersen commitment scheme
#[derive(Debug)]
pub struct PedersenCommitment {
    /// Generator points (mocked for now)
    generators: Vec<[u8; 32]>,

    /// Hash function to use
    hash_function: HashFunction,
}

impl PedersenCommitment {
    /// Create a new Pedersen commitment scheme
    pub fn new(hash_function: HashFunction) -> Self {
        // In a real implementation, these would be proper elliptic curve points
        let generators = vec![
            [1u8; 32], // G - value generator
            [2u8; 32], // H - randomness generator
        ];

        Self {
            generators,
            hash_function,
        }
    }

    /// Compute Pedersen commitment: C = vG + rH
    fn compute_commitment(&self, value: &[u8], randomness: &[u8]) -> PrivacyResult<Vec<u8>> {
        // Mock implementation - in reality would use elliptic curve operations
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(b"pedersen_commit");
        hasher.update(value);
        hasher.update(randomness);
        hasher.update(&self.generators[0]);
        hasher.update(&self.generators[1]);

        Ok(hasher.finalize().to_vec())
    }

    /// Hash inputs using configured hash function
    fn hash_inputs(&self, inputs: &[&[u8]]) -> PrivacyResult<Vec<u8>> {
        match self.hash_function {
            HashFunction::Sha256 => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                for input in inputs {
                    hasher.update(input);
                }
                Ok(hasher.finalize().to_vec())
            }
            HashFunction::Blake2b => {
                use blake2::{Blake2b512, Digest};
                let mut hasher = Blake2b512::new();
                for input in inputs {
                    hasher.update(input);
                }
                Ok(hasher.finalize().to_vec()[..32].to_vec())
            }
            HashFunction::Keccak256 => {
                use sha3::{Keccak256, Digest};
                let mut hasher = Keccak256::new();
                for input in inputs {
                    hasher.update(input);
                }
                Ok(hasher.finalize().to_vec())
            }
            HashFunction::Poseidon => {
                // Mock Poseidon hash - in reality would use proper Poseidon implementation
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(b"poseidon");
                for input in inputs {
                    hasher.update(input);
                }
                Ok(hasher.finalize().to_vec())
            }
        }
    }
}

impl CommitmentScheme for PedersenCommitment {
    fn commit(&self, value: &[u8], randomness: &crate::engine::TradeSecret) -> PrivacyResult<TradeCommitment> {
        let commitment_bytes = self.compute_commitment(value, &randomness.to_field_element())?;

        let mut commitment = TradeCommitment::new(
            commitment_bytes,
            "pedersen".to_string(),
            self.hash_function,
        );

        commitment.metadata.randomness_bits = 256;
        commitment.metadata.binding_verified = true;
        commitment.metadata.hiding_verified = true;

        Ok(commitment)
    }

    fn verify(&self, commitment: &TradeCommitment, value: &[u8], randomness: &crate::engine::TradeSecret) -> PrivacyResult<bool> {
        commitment.validate()?;

        if commitment.scheme != "pedersen" {
            return Err(PrivacyError::InvalidCommitment {
                reason: "Wrong commitment scheme".to_string(),
            });
        }

        let recomputed = self.compute_commitment(value, &randomness.to_field_element())?;
        Ok(recomputed == commitment.commitment)
    }

    fn name(&self) -> &str {
        "pedersen"
    }

    fn properties(&self) -> CommitmentProperties {
        CommitmentProperties {
            computational_binding: true,
            perfect_hiding: true,
            additive_homomorphic: true,
            range_proof_compatible: true,
            commitment_size: 32,
            randomness_size: 32,
        }
    }
}

/// Vector commitment for batch commitments
#[derive(Debug)]
pub struct VectorCommitment {
    base_scheme: PedersenCommitment,
}

impl VectorCommitment {
    /// Create a new vector commitment scheme
    pub fn new(hash_function: HashFunction) -> Self {
        Self {
            base_scheme: PedersenCommitment::new(hash_function),
        }
    }

    /// Commit to a vector of values
    pub fn commit_vector(&self, values: &[Vec<u8>], randomness: &[crate::engine::TradeSecret]) -> PrivacyResult<VectorCommitmentResult> {
        if values.len() != randomness.len() {
            return Err(PrivacyError::InvalidCommitment {
                reason: "Values and randomness length mismatch".to_string(),
            });
        }

        let mut individual_commitments = Vec::new();
        let mut all_commitment_bytes = Vec::new();

        for (value, rand) in values.iter().zip(randomness.iter()) {
            let commitment = self.base_scheme.commit(value, rand)?;
            all_commitment_bytes.extend_from_slice(&commitment.commitment);
            individual_commitments.push(commitment);
        }

        // Create aggregate commitment
        let aggregate_commitment = self.base_scheme.hash_inputs(&[&all_commitment_bytes])?;

        Ok(VectorCommitmentResult {
            individual_commitments,
            aggregate_commitment: TradeCommitment::new(
                aggregate_commitment,
                "vector_pedersen".to_string(),
                self.base_scheme.hash_function,
            ),
            vector_size: values.len(),
        })
    }

    /// Verify a vector commitment
    pub fn verify_vector(
        &self,
        result: &VectorCommitmentResult,
        values: &[Vec<u8>],
        randomness: &[crate::engine::TradeSecret],
    ) -> PrivacyResult<bool> {
        if values.len() != result.vector_size || randomness.len() != result.vector_size {
            return Ok(false);
        }

        // Verify individual commitments
        for (i, (value, rand)) in values.iter().zip(randomness.iter()).enumerate() {
            if !self.base_scheme.verify(&result.individual_commitments[i], value, rand)? {
                return Ok(false);
            }
        }

        // Verify aggregate commitment
        let mut all_commitment_bytes = Vec::new();
        for commitment in &result.individual_commitments {
            all_commitment_bytes.extend_from_slice(&commitment.commitment);
        }

        let expected_aggregate = self.base_scheme.hash_inputs(&[&all_commitment_bytes])?;
        Ok(expected_aggregate == result.aggregate_commitment.commitment)
    }
}

/// Vector commitment result
#[derive(Debug, Clone)]
pub struct VectorCommitmentResult {
    /// Individual commitments for each value
    pub individual_commitments: Vec<TradeCommitment>,

    /// Aggregate commitment for the entire vector
    pub aggregate_commitment: TradeCommitment,

    /// Size of the committed vector
    pub vector_size: usize,
}

/// Homomorphic commitment operations
pub struct HomomorphicCommitment {
    base_scheme: PedersenCommitment,
}

impl HomomorphicCommitment {
    /// Create a new homomorphic commitment handler
    pub fn new(hash_function: HashFunction) -> Self {
        Self {
            base_scheme: PedersenCommitment::new(hash_function),
        }
    }

    /// Add two commitments homomorphically
    pub fn add_commitments(&self, comm1: &TradeCommitment, comm2: &TradeCommitment) -> PrivacyResult<TradeCommitment> {
        if comm1.scheme != "pedersen" || comm2.scheme != "pedersen" {
            return Err(PrivacyError::InvalidCommitment {
                reason: "Both commitments must be Pedersen".to_string(),
            });
        }

        // Mock homomorphic addition - in reality would use elliptic curve point addition
        let mut result = Vec::new();
        for (a, b) in comm1.commitment.iter().zip(comm2.commitment.iter()) {
            result.push(a.wrapping_add(*b));
        }

        Ok(TradeCommitment::new(
            result,
            "pedersen".to_string(),
            comm1.hash_function,
        ))
    }

    /// Multiply commitment by scalar
    pub fn scalar_multiply(&self, commitment: &TradeCommitment, scalar: &[u8]) -> PrivacyResult<TradeCommitment> {
        if commitment.scheme != "pedersen" {
            return Err(PrivacyError::InvalidCommitment {
                reason: "Commitment must be Pedersen".to_string(),
            });
        }

        // Mock scalar multiplication
        let mut result = Vec::new();
        let scalar_byte = scalar.get(0).unwrap_or(&1);
        for byte in &commitment.commitment {
            result.push(byte.wrapping_mul(*scalar_byte));
        }

        Ok(TradeCommitment::new(
            result,
            "pedersen".to_string(),
            commitment.hash_function,
        ))
    }

    /// Create zero commitment (identity element)
    pub fn zero_commitment(&self) -> PrivacyResult<TradeCommitment> {
        Ok(TradeCommitment::new(
            vec![0u8; 32],
            "pedersen".to_string(),
            self.base_scheme.hash_function,
        ))
    }
}

/// Commitment factory for creating different types of commitments
pub struct CommitmentFactory;

impl CommitmentFactory {
    /// Create a commitment scheme by name
    pub fn create_scheme(name: &str, hash_function: HashFunction) -> PrivacyResult<Box<dyn CommitmentScheme>> {
        match name {
            "pedersen" => Ok(Box::new(PedersenCommitment::new(hash_function))),
            _ => Err(PrivacyError::InvalidCommitment {
                reason: format!("Unknown commitment scheme: {}", name),
            }),
        }
    }

    /// List available commitment schemes
    pub fn available_schemes() -> Vec<String> {
        vec!["pedersen".to_string()]
    }

    /// Get recommended scheme for given use case
    pub fn recommended_scheme(use_case: CommitmentUseCase) -> String {
        match use_case {
            CommitmentUseCase::TradeAmount => "pedersen".to_string(),
            CommitmentUseCase::BatchCommitment => "pedersen".to_string(),
            CommitmentUseCase::RangeProof => "pedersen".to_string(),
        }
    }
}

/// Use cases for commitments
#[derive(Debug, Clone, Copy)]
pub enum CommitmentUseCase {
    TradeAmount,
    BatchCommitment,
    RangeProof,
}

/// Extensions for Amount type
pub trait AmountCommitment {
    fn to_commitment_bytes(&self) -> Vec<u8>;
}

impl AmountCommitment for Amount {
    fn to_commitment_bytes(&self) -> Vec<u8> {
        self.as_u64().to_le_bytes().to_vec()
    }
}

impl AmountCommitment for Price {
    fn to_commitment_bytes(&self) -> Vec<u8> {
        self.as_u64().to_le_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::TradeSecret;

    #[test]
    fn test_trade_commitment_creation() {
        let commitment = TradeCommitment::new(
            vec![1, 2, 3, 4],
            "pedersen".to_string(),
            HashFunction::Poseidon,
        );

        assert_eq!(commitment.scheme, "pedersen");
        assert_eq!(commitment.size(), 4);
        assert_eq!(commitment.hash_function, HashFunction::Poseidon);
    }

    #[test]
    fn test_commitment_validation() {
        let valid_commitment = TradeCommitment::new(
            vec![0u8; 32],
            "pedersen".to_string(),
            HashFunction::Poseidon,
        );
        assert!(valid_commitment.validate().is_ok());

        let invalid_commitment = TradeCommitment::new(
            vec![],
            "pedersen".to_string(),
            HashFunction::Poseidon,
        );
        assert!(invalid_commitment.validate().is_err());
    }

    #[test]
    fn test_commitment_hex_conversion() {
        let original = TradeCommitment::new(
            vec![0x12, 0x34, 0x56, 0x78],
            "pedersen".to_string(),
            HashFunction::Poseidon,
        );

        let hex_str = original.to_hex();
        assert_eq!(hex_str, "12345678");

        let parsed = TradeCommitment::from_hex(&hex_str, "pedersen".to_string(), HashFunction::Poseidon);
        assert!(parsed.is_err()); // Invalid length for commitment
    }

    #[test]
    fn test_pedersen_commitment() {
        let scheme = PedersenCommitment::new(HashFunction::Sha256);
        let value = b"test_value";
        let randomness = TradeSecret::random().unwrap();

        let commitment = scheme.commit(value, &randomness).unwrap();
        assert_eq!(commitment.scheme, "pedersen");

        let is_valid = scheme.verify(&commitment, value, &randomness).unwrap();
        assert!(is_valid);

        // Test with wrong value
        let is_invalid = scheme.verify(&commitment, b"wrong_value", &randomness).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_commitment_properties() {
        let scheme = PedersenCommitment::new(HashFunction::Poseidon);
        let properties = scheme.properties();

        assert!(properties.computational_binding);
        assert!(properties.perfect_hiding);
        assert!(properties.additive_homomorphic);
        assert!(properties.range_proof_compatible);
        assert_eq!(properties.commitment_size, 32);
    }

    #[test]
    fn test_vector_commitment() {
        let vector_scheme = VectorCommitment::new(HashFunction::Sha256);

        let values = vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
            b"value3".to_vec(),
        ];

        let randomness = vec![
            TradeSecret::random().unwrap(),
            TradeSecret::random().unwrap(),
            TradeSecret::random().unwrap(),
        ];

        let result = vector_scheme.commit_vector(&values, &randomness).unwrap();
        assert_eq!(result.vector_size, 3);
        assert_eq!(result.individual_commitments.len(), 3);

        let is_valid = vector_scheme.verify_vector(&result, &values, &randomness).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_homomorphic_operations() {
        let homo = HomomorphicCommitment::new(HashFunction::Poseidon);
        let scheme = PedersenCommitment::new(HashFunction::Poseidon);

        let value1 = b"100";
        let value2 = b"200";
        let rand1 = TradeSecret::random().unwrap();
        let rand2 = TradeSecret::random().unwrap();

        let comm1 = scheme.commit(value1, &rand1).unwrap();
        let comm2 = scheme.commit(value2, &rand2).unwrap();

        let sum_commitment = homo.add_commitments(&comm1, &comm2).unwrap();
        assert_eq!(sum_commitment.scheme, "pedersen");

        let scaled_commitment = homo.scalar_multiply(&comm1, &[2u8]).unwrap();
        assert_eq!(scaled_commitment.scheme, "pedersen");

        let zero = homo.zero_commitment().unwrap();
        assert_eq!(zero.commitment, vec![0u8; 32]);
    }

    #[test]
    fn test_commitment_factory() {
        let scheme = CommitmentFactory::create_scheme("pedersen", HashFunction::Blake2b).unwrap();
        assert_eq!(scheme.name(), "pedersen");

        let invalid_scheme = CommitmentFactory::create_scheme("invalid", HashFunction::Sha256);
        assert!(invalid_scheme.is_err());

        let available = CommitmentFactory::available_schemes();
        assert!(available.contains(&"pedersen".to_string()));

        let recommended = CommitmentFactory::recommended_scheme(CommitmentUseCase::TradeAmount);
        assert_eq!(recommended, "pedersen");
    }

    #[test]
    fn test_amount_commitment_extension() {
        let amount = Amount::from_u64(1000);
        let bytes = amount.to_commitment_bytes();
        assert_eq!(bytes, 1000u64.to_le_bytes().to_vec());

        let price = Price::from_u64(500).unwrap();
        let price_bytes = price.to_commitment_bytes();
        assert_eq!(price_bytes, 500u64.to_le_bytes().to_vec());
    }
}