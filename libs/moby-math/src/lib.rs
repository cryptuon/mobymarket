pub mod price;
pub mod slippage;
pub mod error;
pub mod utils;

pub use price::{Price, PriceError};
pub use slippage::{SlippageCalculator, SlippageParams, SlippageResult};
pub use error::MathError;
pub use utils::{validate_whale_amount, calculate_basis_points, normalize_decimals, calculate_price_impact_savings};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_integration() {
        let btc_price = Price::from_float(43_250.75, 6).unwrap();
        let whale_amount = 100_000_000u64 * Price::PRECISION; // $100M

        let btc_amount = btc_price.divide_amount(whale_amount).unwrap();
        let slippage_calc = SlippageCalculator::new(
            SlippageParams {
                depth_coefficient: 0.15,
                impact_exponent: 1.2,
                base_slippage: 0.001,
                max_slippage: 0.05,
            }
        );

        let slippage = slippage_calc.calculate_slippage(
            btc_amount,
            1_000_000 * Price::PRECISION
        ).unwrap();

        assert!(slippage.total_slippage > 0.0);
        assert!(slippage.total_slippage < 0.05);
    }
}