// Copyright (c) 2024 Moby Market
//
// Licensed under the MIT License. See LICENSE file in the project root for license information.

//! Stealth address system for private payments

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{PrivacyError, PrivacyResult};
use moby_types::AccountKey;

/// Stealth address for private payments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StealthAddress {
    /// Public stealth address
    pub address: Vec<u8>,

    /// View tag for efficient scanning
    pub view_tag: u8,

    /// Additional metadata
    pub metadata: StealthMetadata,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl StealthAddress {
    /// Create a new stealth address
    pub fn new(address: Vec<u8>, view_tag: u8) -> Self {
        Self {
            address,
            view_tag,
            metadata: StealthMetadata::default(),
            created_at: Utc::now(),
        }
    }

    /// Generate a random stealth address (mock implementation)
    pub fn generate() -> PrivacyResult<Self> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();

        let mut address = vec![0u8; 33]; // Compressed public key format
        rng.fill_bytes(&mut address);
        address[0] = 0x02; // Set compressed key prefix

        let view_tag = (rng.next_u32() % 256) as u8;

        Ok(Self::new(address, view_tag))
    }

    /// Parse stealth address from string representation
    pub fn from_string(address_str: &str) -> PrivacyResult<Self> {
        // Expected format: "stealth:address_hex:view_tag"
        let parts: Vec<&str> = address_str.split(':').collect();
        if parts.len() != 3 || parts[0] != "stealth" {
            return Err(PrivacyError::InvalidStealthAddress {
                reason: "Invalid address format".to_string(),
            });
        }

        let address = hex::decode(parts[1])
            .map_err(|_| PrivacyError::InvalidStealthAddress {
                reason: "Invalid hex encoding".to_string(),
            })?;

        let view_tag = parts[2].parse::<u8>()
            .map_err(|_| PrivacyError::InvalidStealthAddress {
                reason: "Invalid view tag".to_string(),
            })?;

        let mut stealth_addr = Self::new(address, view_tag);
        stealth_addr.validate()?;
        Ok(stealth_addr)
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("stealth:{}:{}", hex::encode(&self.address), self.view_tag)
    }

    /// Validate stealth address structure
    pub fn validate(&self) -> PrivacyResult<()> {
        if self.address.is_empty() {
            return Err(PrivacyError::InvalidStealthAddress {
                reason: "Empty address".to_string(),
            });
        }

        if self.address.len() != 33 && self.address.len() != 65 {
            return Err(PrivacyError::InvalidStealthAddress {
                reason: "Invalid address length".to_string(),
            });
        }

        // Check compressed key format
        if self.address.len() == 33 && (self.address[0] != 0x02 && self.address[0] != 0x03) {
            return Err(PrivacyError::InvalidStealthAddress {
                reason: "Invalid compressed key prefix".to_string(),
            });
        }

        // Check uncompressed key format
        if self.address.len() == 65 && self.address[0] != 0x04 {
            return Err(PrivacyError::InvalidStealthAddress {
                reason: "Invalid uncompressed key prefix".to_string(),
            });
        }

        Ok(())
    }

    /// Check if this address matches a view tag for efficient scanning
    pub fn matches_view_tag(&self, tag: u8) -> bool {
        self.view_tag == tag
    }

    /// Get address as hex string
    pub fn as_hex(&self) -> String {
        hex::encode(&self.address)
    }
}

impl std::fmt::Display for StealthAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Stealth address metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthMetadata {
    /// Key derivation method used
    pub derivation_method: String,

    /// Whether address supports view key scanning
    pub supports_view_key: bool,

    /// Network identifier
    pub network: String,

    /// Additional properties
    pub properties: HashMap<String, String>,
}

impl Default for StealthMetadata {
    fn default() -> Self {
        Self {
            derivation_method: "ecdh".to_string(),
            supports_view_key: true,
            network: "solana".to_string(),
            properties: HashMap::new(),
        }
    }
}

/// Stealth key pair for generating and detecting stealth addresses
#[derive(Debug, Clone)]
pub struct StealthKeyPair {
    /// Private spend key
    pub spend_key: PrivateKey,

    /// Private view key
    pub view_key: PrivateKey,

    /// Public spend key
    pub public_spend_key: PublicKey,

    /// Public view key
    pub public_view_key: PublicKey,

    /// Key pair metadata
    pub metadata: KeyPairMetadata,
}

impl StealthKeyPair {
    /// Generate a new stealth key pair
    pub fn generate() -> PrivacyResult<Self> {
        let spend_key = PrivateKey::generate()?;
        let view_key = PrivateKey::generate()?;

        let public_spend_key = spend_key.to_public_key()?;
        let public_view_key = view_key.to_public_key()?;

        Ok(Self {
            spend_key,
            view_key,
            public_spend_key,
            public_view_key,
            metadata: KeyPairMetadata::default(),
        })
    }

    /// Derive a stealth address for a recipient
    pub fn derive_stealth_address(
        &self,
        recipient_public_spend: &PublicKey,
        recipient_public_view: &PublicKey,
    ) -> PrivacyResult<(StealthAddress, EphemeralKey)> {
        // Generate ephemeral key pair
        let ephemeral_private = PrivateKey::generate()?;
        let ephemeral_public = ephemeral_private.to_public_key()?;

        // Derive shared secret using ECDH
        let shared_secret = self.compute_shared_secret(&ephemeral_private, recipient_public_view)?;

        // Derive stealth public key
        let stealth_public = self.derive_stealth_public_key(recipient_public_spend, &shared_secret)?;

        // Generate view tag for efficient scanning
        let view_tag = self.compute_view_tag(&shared_secret)?;

        let stealth_address = StealthAddress::new(stealth_public.to_bytes(), view_tag);

        let ephemeral_key = EphemeralKey {
            public_key: ephemeral_public,
            shared_secret: shared_secret.clone(),
            created_at: Utc::now(),
        };

        Ok((stealth_address, ephemeral_key))
    }

    /// Detect if a stealth address belongs to this key pair
    pub fn detect_stealth_payment(
        &self,
        stealth_address: &StealthAddress,
        ephemeral_public: &PublicKey,
    ) -> PrivacyResult<Option<PrivateKey>> {
        // Quick view tag check
        let shared_secret = self.compute_shared_secret_from_public(&self.view_key, ephemeral_public)?;
        let expected_view_tag = self.compute_view_tag(&shared_secret)?;

        if !stealth_address.matches_view_tag(expected_view_tag) {
            return Ok(None); // Not for us
        }

        // Derive expected stealth public key
        let expected_stealth_public = self.derive_stealth_public_key(&self.public_spend_key, &shared_secret)?;

        // Check if it matches
        if expected_stealth_public.to_bytes() == stealth_address.address {
            // Derive the private key for spending
            let stealth_private = self.derive_stealth_private_key(&shared_secret)?;
            Ok(Some(stealth_private))
        } else {
            Ok(None)
        }
    }

    /// Scan a batch of transactions for stealth payments
    pub fn scan_transactions(&self, transactions: &[StealthTransaction]) -> PrivacyResult<Vec<DetectedPayment>> {
        let mut detected_payments = Vec::new();

        for (tx_index, transaction) in transactions.iter().enumerate() {
            for (output_index, output) in transaction.stealth_outputs.iter().enumerate() {
                if let Some(private_key) = self.detect_stealth_payment(&output.stealth_address, &output.ephemeral_key)? {
                    detected_payments.push(DetectedPayment {
                        transaction_index: tx_index,
                        output_index,
                        stealth_address: output.stealth_address.clone(),
                        amount: output.amount,
                        private_key,
                        detected_at: Utc::now(),
                    });
                }
            }
        }

        Ok(detected_payments)
    }

    // Private helper methods

    fn compute_shared_secret(&self, private_key: &PrivateKey, public_key: &PublicKey) -> PrivacyResult<SharedSecret> {
        // Mock ECDH computation
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(b"ecdh_shared_secret");
        hasher.update(&private_key.bytes);
        hasher.update(&public_key.bytes);

        Ok(SharedSecret {
            bytes: hasher.finalize().to_vec(),
        })
    }

    fn compute_shared_secret_from_public(&self, private_key: &PrivateKey, public_key: &PublicKey) -> PrivacyResult<SharedSecret> {
        self.compute_shared_secret(private_key, public_key)
    }

    fn derive_stealth_public_key(&self, recipient_public: &PublicKey, shared_secret: &SharedSecret) -> PrivacyResult<PublicKey> {
        // Mock stealth public key derivation: P_stealth = P_recipient + H(shared_secret) * G
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(b"stealth_public_derivation");
        hasher.update(&recipient_public.bytes);
        hasher.update(&shared_secret.bytes);

        let derived_bytes = hasher.finalize().to_vec();

        // Mock point addition - in reality would use elliptic curve operations
        let mut result_bytes = recipient_public.bytes.clone();
        for (i, &byte) in derived_bytes.iter().take(result_bytes.len()).enumerate() {
            result_bytes[i] = result_bytes[i].wrapping_add(byte);
        }

        Ok(PublicKey { bytes: result_bytes })
    }

    fn derive_stealth_private_key(&self, shared_secret: &SharedSecret) -> PrivacyResult<PrivateKey> {
        // Mock stealth private key derivation: k_stealth = k_spend + H(shared_secret)
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(b"stealth_private_derivation");
        hasher.update(&self.spend_key.bytes);
        hasher.update(&shared_secret.bytes);

        let derived_bytes = hasher.finalize().to_vec();

        // Mock scalar addition - in reality would use proper field arithmetic
        let mut result_bytes = self.spend_key.bytes.clone();
        for (i, &byte) in derived_bytes.iter().take(result_bytes.len()).enumerate() {
            result_bytes[i] = result_bytes[i].wrapping_add(byte);
        }

        Ok(PrivateKey { bytes: result_bytes })
    }

    fn compute_view_tag(&self, shared_secret: &SharedSecret) -> PrivacyResult<u8> {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(b"view_tag");
        hasher.update(&shared_secret.bytes);

        let hash = hasher.finalize();
        Ok(hash[0]) // Use first byte as view tag
    }
}

/// Private key structure
#[derive(Debug, Clone)]
pub struct PrivateKey {
    bytes: Vec<u8>,
}

impl PrivateKey {
    /// Generate a random private key
    pub fn generate() -> PrivacyResult<Self> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; 32];
        rng.fill_bytes(&mut bytes);

        Ok(Self { bytes })
    }

    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> PrivacyResult<Self> {
        if bytes.len() != 32 {
            return Err(PrivacyError::StealthKeyDerivationFailed);
        }
        Ok(Self { bytes })
    }

    /// Convert to public key
    pub fn to_public_key(&self) -> PrivacyResult<PublicKey> {
        // Mock public key derivation - in reality would use curve operations
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(b"private_to_public");
        hasher.update(&self.bytes);

        let hash = hasher.finalize().to_vec();
        let mut public_bytes = vec![0x02]; // Compressed key prefix
        public_bytes.extend_from_slice(&hash[..32]);

        Ok(PublicKey { bytes: public_bytes })
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }
}

/// Public key structure
#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey {
    bytes: Vec<u8>,
}

impl PublicKey {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> PrivacyResult<Self> {
        if bytes.len() != 33 && bytes.len() != 65 {
            return Err(PrivacyError::InvalidStealthAddress {
                reason: "Invalid public key length".to_string(),
            });
        }
        Ok(Self { bytes })
    }

    /// Get bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }
}

/// Shared secret from ECDH
#[derive(Debug, Clone)]
pub struct SharedSecret {
    bytes: Vec<u8>,
}

/// Ephemeral key for stealth address derivation
#[derive(Debug, Clone)]
pub struct EphemeralKey {
    pub public_key: PublicKey,
    pub shared_secret: SharedSecret,
    pub created_at: DateTime<Utc>,
}

/// Key pair metadata
#[derive(Debug, Clone)]
pub struct KeyPairMetadata {
    pub derivation_path: String,
    pub created_at: DateTime<Utc>,
    pub network: String,
}

impl Default for KeyPairMetadata {
    fn default() -> Self {
        Self {
            derivation_path: "m/44'/501'/0'/0'".to_string(), // Solana derivation path
            created_at: Utc::now(),
            network: "solana".to_string(),
        }
    }
}

/// Stealth transaction output
#[derive(Debug, Clone)]
pub struct StealthOutput {
    pub stealth_address: StealthAddress,
    pub ephemeral_key: PublicKey,
    pub amount: u64, // Encrypted amount in practice
}

/// Stealth transaction structure
#[derive(Debug, Clone)]
pub struct StealthTransaction {
    pub transaction_id: String,
    pub stealth_outputs: Vec<StealthOutput>,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
}

/// Detected payment information
#[derive(Debug, Clone)]
pub struct DetectedPayment {
    pub transaction_index: usize,
    pub output_index: usize,
    pub stealth_address: StealthAddress,
    pub amount: u64,
    pub private_key: PrivateKey,
    pub detected_at: DateTime<Utc>,
}

/// Address generator utility
pub struct AddressGenerator {
    key_pair: StealthKeyPair,
}

impl AddressGenerator {
    /// Create a new address generator
    pub fn new() -> PrivacyResult<Self> {
        Ok(Self {
            key_pair: StealthKeyPair::generate()?,
        })
    }

    /// Create from existing key pair
    pub fn from_key_pair(key_pair: StealthKeyPair) -> Self {
        Self { key_pair }
    }

    /// Generate a stealth address for a given recipient
    pub fn generate_for_recipient(&self, recipient: Option<AccountKey>) -> PrivacyResult<StealthAddress> {
        // In practice, would use recipient's public keys
        // For now, generate a random address
        StealthAddress::generate()
    }

    /// Get the viewing key for scanning
    pub fn get_view_key(&self) -> &PrivateKey {
        &self.key_pair.view_key
    }

    /// Get the spending key
    pub fn get_spend_key(&self) -> &PrivateKey {
        &self.key_pair.spend_key
    }

    /// Get public keys for sharing
    pub fn get_public_keys(&self) -> (PublicKey, PublicKey) {
        (self.key_pair.public_spend_key.clone(), self.key_pair.public_view_key.clone())
    }
}

impl Default for AddressGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default address generator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_address_creation() {
        let address = StealthAddress::generate().unwrap();
        assert_eq!(address.address.len(), 33);
        assert_eq!(address.address[0], 0x02);
        assert!(address.validate().is_ok());
    }

    #[test]
    fn test_stealth_address_string_conversion() {
        let address = StealthAddress::generate().unwrap();
        let address_str = address.to_string();
        assert!(address_str.starts_with("stealth:"));

        let parsed = StealthAddress::from_string(&address_str).unwrap();
        assert_eq!(parsed.address, address.address);
        assert_eq!(parsed.view_tag, address.view_tag);
    }

    #[test]
    fn test_stealth_address_validation() {
        // Valid compressed key
        let valid_address = StealthAddress::new(vec![0x02; 33], 42);
        assert!(valid_address.validate().is_ok());

        // Invalid length
        let invalid_address = StealthAddress::new(vec![0x02; 32], 42);
        assert!(invalid_address.validate().is_err());

        // Invalid prefix
        let mut invalid_prefix = vec![0u8; 33];
        invalid_prefix[0] = 0x01; // Invalid prefix
        let invalid_address = StealthAddress::new(invalid_prefix, 42);
        assert!(invalid_address.validate().is_err());
    }

    #[test]
    fn test_key_pair_generation() {
        let key_pair = StealthKeyPair::generate().unwrap();
        assert_eq!(key_pair.spend_key.bytes.len(), 32);
        assert_eq!(key_pair.view_key.bytes.len(), 32);
        assert_eq!(key_pair.public_spend_key.bytes.len(), 33);
        assert_eq!(key_pair.public_view_key.bytes.len(), 33);
    }

    #[test]
    fn test_private_to_public_key() {
        let private_key = PrivateKey::generate().unwrap();
        let public_key = private_key.to_public_key().unwrap();
        assert_eq!(public_key.bytes.len(), 33);
        assert_eq!(public_key.bytes[0], 0x02);
    }

    #[test]
    fn test_stealth_address_derivation() {
        let sender_keys = StealthKeyPair::generate().unwrap();
        let recipient_keys = StealthKeyPair::generate().unwrap();

        let (stealth_address, ephemeral_key) = sender_keys.derive_stealth_address(
            &recipient_keys.public_spend_key,
            &recipient_keys.public_view_key,
        ).unwrap();

        assert!(stealth_address.validate().is_ok());
        assert_eq!(ephemeral_key.public_key.bytes.len(), 33);
    }

    #[test]
    fn test_stealth_payment_detection() {
        let sender_keys = StealthKeyPair::generate().unwrap();
        let recipient_keys = StealthKeyPair::generate().unwrap();

        // Sender creates stealth address for recipient
        let (stealth_address, ephemeral_key) = sender_keys.derive_stealth_address(
            &recipient_keys.public_spend_key,
            &recipient_keys.public_view_key,
        ).unwrap();

        // Recipient detects the payment
        let detected_key = recipient_keys.detect_stealth_payment(
            &stealth_address,
            &ephemeral_key.public_key,
        ).unwrap();

        assert!(detected_key.is_some());
    }

    #[test]
    fn test_view_tag_matching() {
        let address = StealthAddress::new(vec![0x02; 33], 42);
        assert!(address.matches_view_tag(42));
        assert!(!address.matches_view_tag(43));
    }

    #[test]
    fn test_transaction_scanning() {
        let recipient_keys = StealthKeyPair::generate().unwrap();

        // Create mock transactions
        let transactions = vec![
            StealthTransaction {
                transaction_id: "tx1".to_string(),
                stealth_outputs: vec![
                    StealthOutput {
                        stealth_address: StealthAddress::generate().unwrap(),
                        ephemeral_key: PublicKey::from_bytes(vec![0x02; 33]).unwrap(),
                        amount: 1000,
                    },
                ],
                block_number: 1,
                timestamp: Utc::now(),
            },
        ];

        let detected = recipient_keys.scan_transactions(&transactions).unwrap();
        // Should be empty since we're using random addresses
        assert_eq!(detected.len(), 0);
    }

    #[test]
    fn test_address_generator() {
        let generator = AddressGenerator::new().unwrap();

        let stealth_address = generator.generate_for_recipient(None).unwrap();
        assert!(stealth_address.validate().is_ok());

        let (spend_pub, view_pub) = generator.get_public_keys();
        assert_eq!(spend_pub.bytes.len(), 33);
        assert_eq!(view_pub.bytes.len(), 33);
    }

    #[test]
    fn test_hex_conversions() {
        let private_key = PrivateKey::generate().unwrap();
        let hex_str = private_key.to_hex();
        assert_eq!(hex_str.len(), 64); // 32 bytes * 2 hex chars

        let public_key = private_key.to_public_key().unwrap();
        let pub_hex = public_key.to_hex();
        assert_eq!(pub_hex.len(), 66); // 33 bytes * 2 hex chars
    }
}