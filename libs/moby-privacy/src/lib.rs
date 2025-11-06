// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! # Moby Privacy
//!
//! Zero-knowledge proof system and privacy infrastructure for whale trading.
//!
//! This library provides:
//! - Zero-knowledge proof generation and verification for private trades
//! - Privacy pools for anonymous liquidity provision
//! - Commitment schemes for trade confidentiality
//! - Mixing protocols for transaction privacy
//! - Stealth addresses for recipient privacy
//! - Range proofs for amount confidentiality
//!
//! ## Features
//!
//! - **Private Trading**: Execute trades without revealing amounts or identities
//! - **Privacy Pools**: Anonymous liquidity pools with ZK membership proofs
//! - **Stealth Addresses**: Generate unlinkable addresses for enhanced privacy
//! - **Range Proofs**: Prove amounts are within valid ranges without revealing values
//! - **Mixing Protocol**: Break transaction linkability through cryptographic mixing
//! - **Regulatory Compliance**: Selective disclosure capabilities for compliance
//!
//! ## Quick Start
//!
//! ```rust
//! use moby_privacy::{PrivacyEngine, TradeCommitment, ZkProof};
//! use moby_types::WhaleAmount;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize privacy engine
//! let privacy_engine = PrivacyEngine::new();
//!
//! // Create a private trade commitment
//! let trade_amount = WhaleAmount::from_dollars(10_000_000); // $10M trade
//! let commitment = privacy_engine.create_trade_commitment(
//!     trade_amount,
//!     "trade_secret".as_bytes(),
//! )?;
//!
//! // Generate zero-knowledge proof
//! let proof = privacy_engine.generate_trade_proof(
//!     &commitment,
//!     trade_amount,
//!     "trade_secret".as_bytes(),
//! ).await?;
//!
//! // Verify the proof
//! let is_valid = privacy_engine.verify_trade_proof(&proof, &commitment)?;
//! assert!(is_valid);
//! # Ok(())
//! # }
//! ```

pub mod circuits;
pub mod commitments;
pub mod engine;
pub mod mixer;
pub mod pools;
pub mod proofs;
pub mod stealth;
pub mod range_proofs;
pub mod nullifiers;
pub mod compliance;
pub mod error;
pub mod mock_types;

// Re-export core types
pub use engine::{
    PrivacyEngine, PrivacyEngineConfig, PrivacyLevel, TradeSecret,
    PrivateTradeInitialization, TradeProofVerification, PrivacyMetrics
};
pub use proofs::{ZkProof, ProofSystem};
pub use commitments::{TradeCommitment, CommitmentScheme};
pub use mixer::{PrivacyMixer, MixTransaction, MixResult, MixerConfig};
pub use pools::{PrivacyPool, PoolManager, PoolConfig};
pub use stealth::{StealthAddress, StealthKeyPair};
pub use range_proofs::{RangeProofEngine, RangeProof, RangeProofConfig};
pub use nullifiers::{NullifierSystem, Nullifier, NullifierConfig};
pub use compliance::{ComplianceSystem, ComplianceOfficer, ComplianceConfig};
pub use circuits::{CircuitManager, Circuit, CircuitConfig};
pub use error::{PrivacyError, PrivacyResult};

// Re-export from dependencies (using mock types for testing)
pub use mock_types::{Amount, Price, Percentage, WhaleAmount, AccountKey, TradingTier, TradeId};

/// Privacy engine configuration
#[derive(Debug, Clone)]
pub struct PrivacyConfig {
    /// Default proof system to use
    pub default_proof_system: ProofSystemType,

    /// Commitment scheme configuration
    pub commitment_config: CommitmentConfig,

    /// Privacy pool configuration
    pub pool_config: PoolConfig,

    /// Mixing protocol configuration
    pub mixer_config: MixerConfig,

    /// Compliance configuration
    pub compliance_config: ComplianceConfig,

    /// Enable performance optimizations
    pub performance_optimizations: bool,

    /// Enable hardware acceleration if available
    pub hardware_acceleration: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            default_proof_system: ProofSystemType::Groth16,
            commitment_config: CommitmentConfig::default(),
            pool_config: PoolConfig::default(),
            mixer_config: MixerConfig::default(),
            compliance_config: ComplianceConfig::default(),
            performance_optimizations: true,
            hardware_acceleration: false,
        }
    }
}

/// Supported proof systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofSystemType {
    /// Groth16 - Efficient for production use
    Groth16,
    /// PLONK - Universal setup, larger proofs
    Plonk,
    /// Bulletproofs - No trusted setup, larger verification time
    Bulletproofs,
}

/// Commitment scheme configuration
#[derive(Debug, Clone)]
pub struct CommitmentConfig {
    /// Use Pedersen commitments
    pub use_pedersen: bool,

    /// Hash function for commitments
    pub hash_function: HashFunction,

    /// Commitment randomness size in bits
    pub randomness_bits: usize,
}

impl Default for CommitmentConfig {
    fn default() -> Self {
        Self {
            use_pedersen: true,
            hash_function: HashFunction::Poseidon,
            randomness_bits: 256,
        }
    }
}

/// Privacy pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum pool size for anonymity
    pub max_pool_size: u32,

    /// Minimum anonymity set size
    pub min_anonymity_set: u32,

    /// Pool refresh interval in seconds
    pub refresh_interval: u64,

    /// Enable dynamic pool sizing
    pub dynamic_sizing: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 1000,
            min_anonymity_set: 8,
            refresh_interval: 3600, // 1 hour
            dynamic_sizing: true,
        }
    }
}

/// Mixer configuration
#[derive(Debug, Clone)]
pub struct MixerConfig {
    /// Number of mixing rounds
    pub mixing_rounds: u32,

    /// Minimum mix delay in seconds
    pub min_mix_delay: u64,

    /// Maximum mix delay in seconds
    pub max_mix_delay: u64,

    /// Mix fee in basis points
    pub mix_fee_bps: u16,
}

impl Default for MixerConfig {
    fn default() -> Self {
        Self {
            mixing_rounds: 3,
            min_mix_delay: 300,  // 5 minutes
            max_mix_delay: 1800, // 30 minutes
            mix_fee_bps: 10,     // 0.1%
        }
    }
}

/// Compliance configuration
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    /// Enable regulatory reporting
    pub regulatory_reporting: bool,

    /// Authorized compliance officers
    pub compliance_officers: Vec<AccountKey>,

    /// Enable selective disclosure
    pub selective_disclosure: bool,

    /// Compliance proof expiry in seconds
    pub proof_expiry: u64,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            regulatory_reporting: false,
            compliance_officers: Vec::new(),
            selective_disclosure: true,
            proof_expiry: 86400, // 24 hours
        }
    }
}

/// Hash functions supported for commitments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashFunction {
    /// Poseidon hash - ZK-friendly
    Poseidon,
    /// SHA-256
    Sha256,
    /// Blake2b
    Blake2b,
    /// Keccak-256
    Keccak256,
}

/// Privacy levels for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivacyLevel {
    /// Basic privacy - hide amounts only
    Basic,
    /// Standard privacy - hide amounts and timing
    Standard,
    /// Enhanced privacy - full transaction privacy
    Enhanced,
    /// Maximum privacy - full anonymity with mixing
    Maximum,
}

impl PrivacyLevel {
    /// Get the anonymity set size required for this privacy level
    pub fn required_anonymity_set_size(&self) -> u32 {
        match self {
            PrivacyLevel::Basic => 4,
            PrivacyLevel::Standard => 8,
            PrivacyLevel::Enhanced => 16,
            PrivacyLevel::Maximum => 64,
        }
    }

    /// Get the number of mixing rounds required
    pub fn required_mixing_rounds(&self) -> u32 {
        match self {
            PrivacyLevel::Basic => 0,
            PrivacyLevel::Standard => 1,
            PrivacyLevel::Enhanced => 2,
            PrivacyLevel::Maximum => 4,
        }
    }

    /// Check if stealth addresses are required
    pub fn requires_stealth_addresses(&self) -> bool {
        matches!(self, PrivacyLevel::Enhanced | PrivacyLevel::Maximum)
    }

    /// Check if range proofs are required
    pub fn requires_range_proofs(&self) -> bool {
        !matches!(self, PrivacyLevel::Basic)
    }
}

/// Circuit parameters for different privacy operations
#[derive(Debug, Clone)]
pub struct CircuitParameters {
    /// Circuit name/identifier
    pub name: String,

    /// Number of public inputs
    pub public_inputs: usize,

    /// Number of private inputs
    pub private_inputs: usize,

    /// Number of constraints
    pub constraints: usize,

    /// Proving key size in bytes
    pub proving_key_size: usize,

    /// Verification key size in bytes
    pub verification_key_size: usize,

    /// Expected proof generation time in milliseconds
    pub proof_time_ms: u64,

    /// Expected verification time in milliseconds
    pub verification_time_ms: u64,
}

impl CircuitParameters {
    /// Parameters for private trade circuit
    pub fn private_trade() -> Self {
        Self {
            name: "private_trade".to_string(),
            public_inputs: 3, // commitment, nullifier, new_commitment
            private_inputs: 4, // amount, secret, recipient, fee
            constraints: 1000,
            proving_key_size: 1024 * 1024, // 1MB
            verification_key_size: 1024,   // 1KB
            proof_time_ms: 500,
            verification_time_ms: 10,
        }
    }

    /// Parameters for mixer circuit
    pub fn mixer() -> Self {
        Self {
            name: "mixer".to_string(),
            public_inputs: 5, // root, nullifiers, commitments
            private_inputs: 10, // paths, secrets, amounts
            constraints: 5000,
            proving_key_size: 5 * 1024 * 1024, // 5MB
            verification_key_size: 2 * 1024,   // 2KB
            proof_time_ms: 2000,
            verification_time_ms: 25,
        }
    }

    /// Parameters for range proof circuit
    pub fn range_proof(bit_length: usize) -> Self {
        let constraints = bit_length * 10; // Approximate
        Self {
            name: format!("range_proof_{}", bit_length),
            public_inputs: 1, // commitment
            private_inputs: 2, // value, randomness
            constraints,
            proving_key_size: constraints * 100, // Approximate
            verification_key_size: 1024,
            proof_time_ms: (bit_length as u64) * 5,
            verification_time_ms: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_config_defaults() {
        let config = PrivacyConfig::default();
        assert_eq!(config.default_proof_system, ProofSystemType::Groth16);
        assert!(config.performance_optimizations);
        assert!(!config.hardware_acceleration);
    }

    #[test]
    fn test_privacy_levels() {
        assert!(PrivacyLevel::Maximum > PrivacyLevel::Enhanced);
        assert!(PrivacyLevel::Enhanced > PrivacyLevel::Standard);
        assert!(PrivacyLevel::Standard > PrivacyLevel::Basic);

        assert_eq!(PrivacyLevel::Basic.required_anonymity_set_size(), 4);
        assert_eq!(PrivacyLevel::Maximum.required_anonymity_set_size(), 64);

        assert!(!PrivacyLevel::Basic.requires_stealth_addresses());
        assert!(PrivacyLevel::Maximum.requires_stealth_addresses());
    }

    #[test]
    fn test_circuit_parameters() {
        let trade_params = CircuitParameters::private_trade();
        assert_eq!(trade_params.name, "private_trade");
        assert_eq!(trade_params.public_inputs, 3);
        assert!(trade_params.proof_time_ms > 0);

        let mixer_params = CircuitParameters::mixer();
        assert_eq!(mixer_params.name, "mixer");
        assert!(mixer_params.constraints > trade_params.constraints);

        let range_params = CircuitParameters::range_proof(64);
        assert!(range_params.name.contains("range_proof"));
        assert_eq!(range_params.public_inputs, 1);
    }

    #[test]
    fn test_pool_config() {
        let config = PoolConfig::default();
        assert_eq!(config.max_pool_size, 1000);
        assert_eq!(config.min_anonymity_set, 8);
        assert!(config.dynamic_sizing);
    }

    #[test]
    fn test_mixer_config() {
        let config = MixerConfig::default();
        assert_eq!(config.mixing_rounds, 3);
        assert!(config.min_mix_delay < config.max_mix_delay);
        assert_eq!(config.mix_fee_bps, 10);
    }
}