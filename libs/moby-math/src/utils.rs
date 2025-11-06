use crate::{Price, PriceError, MathError};

pub fn validate_whale_amount(amount: u64) -> Result<(), MathError> {
    if amount == 0 {
        return Err(MathError::invalid_param("amount", "0"));
    }

    if amount > Price::MAX_WHALE_AMOUNT {
        return Err(MathError::invalid_param(
            "amount",
            format!("exceeds maximum of {}", Price::MAX_WHALE_AMOUNT)
        ));
    }

    Ok(())
}

pub fn calculate_basis_points(amount: u64, total: u64) -> Result<u64, MathError> {
    if total == 0 {
        return Err(MathError::invalid_param("total", "0"));
    }

    let bp = (amount as f64 / total as f64 * 10_000.0) as u64;
    Ok(bp)
}

pub fn normalize_decimals(amount: u64, from_decimals: u8, to_decimals: u8) -> Result<u64, MathError> {
    if from_decimals > 18 || to_decimals > 18 {
        return Err(MathError::invalid_param("decimals", "exceeds 18"));
    }

    if from_decimals == to_decimals {
        return Ok(amount);
    }

    if from_decimals > to_decimals {
        let divisor = 10u64.pow((from_decimals - to_decimals) as u32);
        Ok(amount / divisor)
    } else {
        let multiplier = 10u64.pow((to_decimals - from_decimals) as u32);
        amount.checked_mul(multiplier)
            .ok_or(MathError::Overflow)
    }
}

pub fn calculate_volume_weighted_price(
    prices: &[Price],
    volumes: &[u64],
) -> Result<Price, MathError> {
    if prices.len() != volumes.len() {
        return Err(MathError::invalid_param(
            "arrays",
            "prices and volumes must have same length"
        ));
    }

    if prices.is_empty() {
        return Err(MathError::invalid_param("arrays", "cannot be empty"));
    }

    let mut total_volume = 0u64;
    let mut weighted_sum = 0.0;

    for (price, &volume) in prices.iter().zip(volumes.iter()) {
        validate_whale_amount(volume)?;

        let price_f64 = price.to_f64()?;
        weighted_sum += price_f64 * volume as f64;
        total_volume = total_volume.checked_add(volume)
            .ok_or(MathError::Overflow)?;
    }

    if total_volume == 0 {
        return Err(MathError::invalid_param("total_volume", "0"));
    }

    let vwap = weighted_sum / total_volume as f64;
    Price::from_float(vwap, 6).map_err(MathError::from)
}

pub fn calculate_price_impact_savings(
    direct_execution_slippage: f64,
    smart_execution_slippage: f64,
    trade_amount: u64,
) -> Result<u64, MathError> {
    if direct_execution_slippage < smart_execution_slippage {
        return Err(MathError::invalid_param(
            "slippage",
            "direct execution cannot be better than smart execution"
        ));
    }

    validate_whale_amount(trade_amount)?;

    let savings_rate = direct_execution_slippage - smart_execution_slippage;
    let savings_amount = (trade_amount as f64 * savings_rate) as u64;

    Ok(savings_amount)
}

pub fn estimate_gas_cost_whale_trade(
    trade_amount: u64,
    gas_price_gwei: u64,
    complexity_factor: f64,
) -> Result<u64, MathError> {
    validate_whale_amount(trade_amount)?;

    if gas_price_gwei == 0 {
        return Err(MathError::invalid_param("gas_price", "0"));
    }

    if complexity_factor <= 0.0 || complexity_factor > 10.0 {
        return Err(MathError::invalid_param(
            "complexity_factor",
            "must be between 0 and 10"
        ));
    }

    // Base gas cost for whale trade: 500k gas
    let base_gas = 500_000u64;

    // Scale by complexity (privacy features, multi-hop, etc.)
    let total_gas = (base_gas as f64 * complexity_factor) as u64;

    // Convert to wei (gwei * 1e9)
    let gas_cost_wei = total_gas.checked_mul(gas_price_gwei)
        .and_then(|v| v.checked_mul(1_000_000_000))
        .ok_or(MathError::Overflow)?;

    Ok(gas_cost_wei)
}

pub fn calculate_time_decay_factor(
    current_timestamp: u64,
    order_timestamp: u64,
    half_life_seconds: u64,
) -> Result<f64, MathError> {
    if half_life_seconds == 0 {
        return Err(MathError::invalid_param("half_life", "0"));
    }

    if current_timestamp < order_timestamp {
        return Err(MathError::invalid_param(
            "timestamp",
            "current cannot be before order"
        ));
    }

    let age_seconds = current_timestamp - order_timestamp;
    let decay_factor = 0.5f64.powf(age_seconds as f64 / half_life_seconds as f64);

    Ok(decay_factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_amount_validation() {
        assert!(validate_whale_amount(0).is_err());
        assert!(validate_whale_amount(1_000_000).is_ok());
        assert!(validate_whale_amount(Price::MAX_WHALE_AMOUNT).is_ok());
        assert!(validate_whale_amount(Price::MAX_WHALE_AMOUNT + 1).is_err());
    }

    #[test]
    fn test_basis_points_calculation() {
        let bp = calculate_basis_points(1_000_000, 100_000_000).unwrap();
        assert_eq!(bp, 100); // 1% = 100 basis points

        let bp = calculate_basis_points(5_000_000, 100_000_000).unwrap();
        assert_eq!(bp, 500); // 5% = 500 basis points

        assert!(calculate_basis_points(1000, 0).is_err());
    }

    #[test]
    fn test_decimal_normalization() {
        // Convert from 18 decimals to 6 decimals
        let amount = 1_000_000_000_000_000_000u64; // 1 token with 18 decimals
        let normalized = normalize_decimals(amount, 18, 6).unwrap();
        assert_eq!(normalized, 1_000_000); // 1 token with 6 decimals

        // Convert from 6 decimals to 18 decimals
        let amount = 1_000_000u64; // 1 token with 6 decimals
        let normalized = normalize_decimals(amount, 6, 18).unwrap();
        assert_eq!(normalized, 1_000_000_000_000_000_000); // 1 token with 18 decimals

        // Same decimals
        let normalized = normalize_decimals(1000, 6, 6).unwrap();
        assert_eq!(normalized, 1000);
    }

    #[test]
    fn test_volume_weighted_price() {
        let prices = vec![
            Price::from_float(100.0, 6).unwrap(),
            Price::from_float(101.0, 6).unwrap(),
            Price::from_float(99.0, 6).unwrap(),
        ];
        let volumes = vec![1000, 2000, 1000];

        let vwap = calculate_volume_weighted_price(&prices, &volumes).unwrap();
        let vwap_value = vwap.to_f64().unwrap();

        // Expected: (100*1000 + 101*2000 + 99*1000) / 4000 = 100.25
        assert!((vwap_value - 100.25).abs() < 0.01);
    }

    #[test]
    fn test_price_impact_savings() {
        let trade_amount = 10_000_000 * Price::PRECISION; // $10M
        let direct_slippage = 0.05; // 5%
        let smart_slippage = 0.02;  // 2%

        let savings = calculate_price_impact_savings(
            direct_slippage,
            smart_slippage,
            trade_amount
        ).unwrap();

        let expected_savings = (trade_amount as f64 * 0.03) as u64; // 3% savings
        assert_eq!(savings, expected_savings);
    }

    #[test]
    fn test_gas_cost_estimation() {
        let trade_amount = 50_000_000 * Price::PRECISION; // $50M
        let gas_price = 20; // 20 gwei
        let complexity = 2.5; // Privacy + multi-hop

        let gas_cost = estimate_gas_cost_whale_trade(
            trade_amount,
            gas_price,
            complexity
        ).unwrap();

        // Expected: 500k * 2.5 * 20 * 1e9 = 25e15 wei
        let expected = 25_000_000_000_000_000u64;
        assert_eq!(gas_cost, expected);
    }

    #[test]
    fn test_time_decay_factor() {
        let current = 1000;
        let order = 500;
        let half_life = 250; // 250 seconds half-life

        let decay = calculate_time_decay_factor(current, order, half_life).unwrap();

        // Age = 500 seconds = 2 half-lives, so decay = 0.5^2 = 0.25
        assert!((decay - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_error_conditions() {
        // Invalid timestamp order
        assert!(calculate_time_decay_factor(100, 200, 50).is_err());

        // Zero half-life
        assert!(calculate_time_decay_factor(200, 100, 0).is_err());

        // Invalid complexity factor
        assert!(estimate_gas_cost_whale_trade(1000, 20, 0.0).is_err());
        assert!(estimate_gas_cost_whale_trade(1000, 20, 11.0).is_err());
    }
}