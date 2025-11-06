use thiserror::Error;
use crate::price::PriceError;

#[derive(Error, Debug, PartialEq)]
pub enum MathError {
    #[error("Price calculation error: {0}")]
    Price(#[from] PriceError),

    #[error("Invalid parameter: {field} = {value}")]
    InvalidParameter { field: String, value: String },

    #[error("Calculation overflow")]
    Overflow,

    #[error("Insufficient liquidity for trade size")]
    InsufficientLiquidity,

    #[error("Market data stale: last update {minutes} minutes ago")]
    StaleMarketData { minutes: u64 },

    #[error("Unsupported token pair: {pair}")]
    UnsupportedPair { pair: String },

    #[error("Risk limit exceeded: {limit_type}")]
    RiskLimitExceeded { limit_type: String },
}

impl MathError {
    pub fn invalid_param(field: &str, value: impl std::fmt::Display) -> Self {
        Self::InvalidParameter {
            field: field.to_string(),
            value: value.to_string(),
        }
    }

    pub fn stale_data(minutes: u64) -> Self {
        Self::StaleMarketData { minutes }
    }

    pub fn unsupported_pair(base: &str, quote: &str) -> Self {
        Self::UnsupportedPair {
            pair: format!("{}/{}", base, quote),
        }
    }

    pub fn risk_limit(limit_type: &str) -> Self {
        Self::RiskLimitExceeded {
            limit_type: limit_type.to_string(),
        }
    }
}