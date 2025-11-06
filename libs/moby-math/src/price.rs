use rust_decimal::{Decimal, prelude::*};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PriceError {
    #[error("Invalid price: {0}")]
    InvalidPrice(String),
    #[error("Overflow in calculation")]
    Overflow,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Precision loss: {0}")]
    PrecisionLoss(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    value: Decimal,
    decimals: u8,
}

impl Price {
    pub const PRECISION: u64 = 1_000_000_000; // 9 decimal places for whale precision
    pub const MAX_WHALE_AMOUNT: u64 = 1_000_000_000_000_000_000; // $1B max

    pub fn new(value: u64, decimals: u8) -> Result<Self, PriceError> {
        if decimals > 18 {
            return Err(PriceError::InvalidPrice("Decimals cannot exceed 18".to_string()));
        }

        let decimal_value = Decimal::new(value as i64, decimals as u32);
        if decimal_value < Decimal::ZERO {
            return Err(PriceError::InvalidPrice("Price cannot be negative".to_string()));
        }

        Ok(Price {
            value: decimal_value,
            decimals,
        })
    }

    pub fn from_float(price: f64, decimals: u8) -> Result<Self, PriceError> {
        if price < 0.0 || !price.is_finite() {
            return Err(PriceError::InvalidPrice(format!("Invalid price: {}", price)));
        }

        let decimal_value = Decimal::from_f64(price)
            .ok_or_else(|| PriceError::InvalidPrice("Cannot convert float to decimal".to_string()))?;

        Ok(Price {
            value: decimal_value,
            decimals,
        })
    }

    pub fn multiply_amount(&self, token_amount: u64) -> Result<u64, PriceError> {
        if token_amount > Self::MAX_WHALE_AMOUNT {
            return Err(PriceError::Overflow);
        }

        // token_amount is already scaled by PRECISION
        // Convert to Decimal and multiply by price
        let token_decimal = Decimal::from(token_amount) / Decimal::from(Self::PRECISION);
        let result = self.value.checked_mul(token_decimal)
            .ok_or(PriceError::Overflow)?;

        // Scale back up and convert to u64
        let scaled_result = result * Decimal::from(Self::PRECISION);
        scaled_result.to_u64()
            .ok_or(PriceError::Overflow)
    }

    pub fn divide_amount(&self, usd_amount: u64) -> Result<u64, PriceError> {
        if usd_amount > Self::MAX_WHALE_AMOUNT {
            return Err(PriceError::Overflow);
        }

        if self.value == Decimal::ZERO {
            return Err(PriceError::DivisionByZero);
        }

        // usd_amount is scaled by PRECISION
        // Convert to Decimal and divide by price
        let usd_decimal = Decimal::from(usd_amount) / Decimal::from(Self::PRECISION);
        let result = usd_decimal.checked_div(self.value)
            .ok_or(PriceError::DivisionByZero)?;

        // Scale back up and convert to u64
        let scaled_result = result * Decimal::from(Self::PRECISION);
        scaled_result.to_u64()
            .ok_or(PriceError::Overflow)
    }

    pub fn add(&self, other: &Price) -> Result<Price, PriceError> {
        let result = self.value.checked_add(other.value)
            .ok_or(PriceError::Overflow)?;

        Ok(Price {
            value: result,
            decimals: self.decimals.max(other.decimals),
        })
    }

    pub fn subtract(&self, other: &Price) -> Result<Price, PriceError> {
        let result = self.value.checked_sub(other.value)
            .ok_or(PriceError::InvalidPrice("Result would be negative".to_string()))?;

        Ok(Price {
            value: result,
            decimals: self.decimals.max(other.decimals),
        })
    }

    pub fn percentage_change(&self, new_price: &Price) -> Result<f64, PriceError> {
        if self.value == Decimal::ZERO {
            return Err(PriceError::DivisionByZero);
        }

        let diff = new_price.value - self.value;
        let percentage = (diff / self.value) * Decimal::from(100);

        percentage.to_f64()
            .ok_or(PriceError::PrecisionLoss("Cannot convert to f64".to_string()))
    }

    pub fn to_f64(&self) -> Result<f64, PriceError> {
        self.value.to_f64()
            .ok_or(PriceError::PrecisionLoss("Cannot convert to f64".to_string()))
    }

    pub fn to_u64_scaled(&self) -> Result<u64, PriceError> {
        let scaled = self.value * Decimal::new(Self::PRECISION as i64, 9);
        scaled.to_u64()
            .ok_or(PriceError::Overflow)
    }

    pub fn raw_value(&self) -> Decimal {
        self.value
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_whale_sized_calculations() {
        let btc_price = Price::from_float(43_250.75, 6).unwrap();
        let whale_amount = 100_000_000 * Price::PRECISION; // $100M

        let btc_amount = btc_price.divide_amount(whale_amount).unwrap();
        // Expected: $100M / $43,250.75 = ~2,312 BTC, but scaled by PRECISION
        let expected_btc_float = 100_000_000.0 / 43_250.75;
        let expected_btc = (expected_btc_float * Price::PRECISION as f64) as u64;
        let tolerance = expected_btc / 100; // 1% tolerance

        assert!((btc_amount as i64 - expected_btc as i64).abs() < tolerance as i64);
    }

    #[test]
    fn test_precision_preservation() {
        let price = Price::from_float(0.1, 8).unwrap(); // Small altcoin price
        let large_amount = 1_000_000 * Price::PRECISION; // $1M

        let token_amount = price.divide_amount(large_amount).unwrap();
        let back_to_usd = price.multiply_amount(token_amount).unwrap();

        let difference = (back_to_usd as i64 - large_amount as i64).abs();
        let tolerance = (large_amount / 100) as i64; // 1% tolerance

        assert!(difference < tolerance);
    }

    #[test]
    fn test_extreme_whale_amounts() {
        let eth_price = Price::from_float(2_500.0, 6).unwrap();
        let sovereign_fund_amount = 500_000_000 * Price::PRECISION; // $500M

        let eth_amount = eth_price.divide_amount(sovereign_fund_amount).unwrap();
        assert!(eth_amount > 0);

        let back_to_usd = eth_price.multiply_amount(eth_amount).unwrap();
        let slippage = ((back_to_usd as f64 - sovereign_fund_amount as f64) / sovereign_fund_amount as f64).abs();

        assert!(slippage < 0.01); // Less than 1% calculation error for extreme amounts
    }

    #[test]
    fn test_overflow_protection() {
        let price = Price::from_float(1.0, 6).unwrap();
        let result = price.multiply_amount(Price::MAX_WHALE_AMOUNT + 1);
        assert!(matches!(result, Err(PriceError::Overflow)));
    }

    #[test]
    fn test_zero_division_protection() {
        let zero_price = Price::new(0, 6).unwrap();
        let result = zero_price.divide_amount(1000);
        assert!(matches!(result, Err(PriceError::DivisionByZero)));
    }

    proptest! {
        #[test]
        fn test_price_roundtrip(
            price in 0.1f64..100_000.0,
            amount in 1000u64..1_000_000_000
        ) {
            let p = Price::from_float(price, 6).unwrap();
            let scaled_amount = amount * Price::PRECISION;
            let token_amount = p.divide_amount(scaled_amount).unwrap();
            let back_to_usd = p.multiply_amount(token_amount).unwrap();

            let error_ratio = (back_to_usd as f64 - scaled_amount as f64).abs() / scaled_amount as f64;
            prop_assert!(error_ratio < 0.1); // Less than 10% error for random values
        }
    }
}