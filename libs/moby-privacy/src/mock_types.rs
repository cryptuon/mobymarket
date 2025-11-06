//! Mock types for testing privacy functionality independently

use serde::{Deserialize, Serialize};
use std::fmt;

/// Mock account key for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountKey([u8; 32]);

impl AccountKey {
    pub fn generate_random() -> Self {
        Self(rand::random())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for AccountKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// Mock whale amount for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WhaleAmount(u64);

impl WhaleAmount {
    pub fn new(amount: u64) -> Self {
        Self(amount)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn to_le_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub fn from_dollars(dollars: u64) -> Self {
        Self(dollars * 1_000_000) // Assume 6 decimals
    }
}

impl Default for WhaleAmount {
    fn default() -> Self {
        Self(0)
    }
}

/// Mock trade ID for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeId(uuid::Uuid);

impl TradeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl Default for TradeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock trading tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradingTier {
    Whale,
    Standard,
    Basic,
}

/// Mock price type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    value: u64,
    decimals: u8,
}

impl Price {
    pub fn new(value: u64, decimals: u8) -> Self {
        Self { value, decimals }
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

/// Mock amount type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount(u64);

impl Amount {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Mock percentage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Percentage(u32); // Basis points

impl Percentage {
    pub fn new(basis_points: u32) -> Self {
        Self(basis_points)
    }

    pub fn as_basis_points(&self) -> u32 {
        self.0
    }
}