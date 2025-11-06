use proptest::prelude::*;
use moby_math::{Price, SlippageCalculator, SlippageParams, validate_whale_amount, calculate_basis_points};

proptest! {
    #[test]
    fn test_price_arithmetic_properties(
        price1 in 0.01f64..100_000.0,
        price2 in 0.01f64..100_000.0,
        amount in 1000u64..10_000_000
    ) {
        let p1 = Price::from_float(price1, 6).unwrap();
        let p2 = Price::from_float(price2, 6).unwrap();
        let amount_scaled = amount * Price::PRECISION;

        // Test commutative property of addition
        let sum1 = p1.add(&p2).unwrap();
        let sum2 = p2.add(&p1).unwrap();

        let diff = (sum1.to_f64().unwrap() - sum2.to_f64().unwrap()).abs();
        prop_assert!(diff < 0.000001);

        // Test that price conversions are positive
        let token_amount = p1.divide_amount(amount_scaled).unwrap();
        prop_assert!(token_amount > 0);

        let usd_amount = p1.multiply_amount(token_amount).unwrap();
        prop_assert!(usd_amount > 0);
    }

    #[test]
    fn test_slippage_monotonicity(
        trade_size_base in 1_000_000u64..50_000_000,
        liquidity_base in 10_000_000u64..500_000_000,
        multiplier in 1.1f64..3.0
    ) {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let liquidity = liquidity_base * Price::PRECISION;
        let trade1 = trade_size_base * Price::PRECISION;
        let trade2 = (trade_size_base as f64 * multiplier) as u64 * Price::PRECISION;

        if trade2 <= Price::MAX_WHALE_AMOUNT {
            let slippage1 = calc.calculate_slippage(trade1, liquidity).unwrap();
            let slippage2 = calc.calculate_slippage(trade2, liquidity).unwrap();

            // Larger trades should have higher slippage
            prop_assert!(slippage2.total_slippage >= slippage1.total_slippage);
        }
    }

    #[test]
    fn test_basis_points_properties(
        amount in 1u64..1_000_000_000,
        total in 1_000_000u64..10_000_000_000
    ) {
        if amount <= total {
            let bp = calculate_basis_points(amount, total).unwrap();

            // Basis points should be between 0 and 10,000 (100%)
            prop_assert!(bp <= 10_000);

            // If amount equals total, should be 10,000 bp (100%)
            if amount == total {
                prop_assert_eq!(bp, 10_000);
            }

            // If amount is half of total, should be 5,000 bp (50%)
            if amount * 2 == total {
                prop_assert!((bp as i64 - 5000).abs() <= 1); // Allow for rounding
            }
        }
    }

    #[test]
    fn test_whale_amount_validation_properties(
        amount in 0u64..Price::MAX_WHALE_AMOUNT + 1000
    ) {
        let validation_result = validate_whale_amount(amount);

        if amount == 0 {
            prop_assert!(validation_result.is_err());
        } else if amount <= Price::MAX_WHALE_AMOUNT {
            prop_assert!(validation_result.is_ok());
        } else {
            prop_assert!(validation_result.is_err());
        }
    }

    #[test]
    fn test_slippage_bounds(
        trade_amount in 1_000_000u64..100_000_000,
        liquidity in 10_000_000u64..1_000_000_000
    ) {
        let params = SlippageParams {
            depth_coefficient: 0.1,
            impact_exponent: 1.5,
            base_slippage: 0.001,
            max_slippage: 0.05,
        };
        let calc = SlippageCalculator::new(params);

        let trade_scaled = trade_amount * Price::PRECISION;
        let liquidity_scaled = liquidity * Price::PRECISION;

        let slippage = calc.calculate_slippage(trade_scaled, liquidity_scaled).unwrap();

        // Slippage should always be within bounds
        prop_assert!(slippage.total_slippage >= 0.0);
        prop_assert!(slippage.total_slippage <= 0.05); // Max slippage cap

        // Linear component should be positive
        prop_assert!(slippage.linear_slippage >= 0.001); // At least base slippage

        // Quadratic component should be non-negative
        prop_assert!(slippage.quadratic_slippage >= 0.0);

        // Total should be sum of components (before capping)
        let uncapped_total = slippage.linear_slippage + slippage.quadratic_slippage;
        if uncapped_total <= 0.05 {
            prop_assert!((slippage.total_slippage - uncapped_total).abs() < 0.000001);
        }
    }

    #[test]
    fn test_chunking_completeness(
        total_amount in 10_000_000u64..100_000_000,
        liquidity in 50_000_000u64..500_000_000,
        max_impact in 0.01f64..0.05
    ) {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let total_scaled = total_amount * Price::PRECISION;
        let liquidity_scaled = liquidity * Price::PRECISION;

        let chunks = calc.calculate_whale_optimal_chunks(
            total_scaled,
            liquidity_scaled,
            max_impact
        ).unwrap();

        // Chunks should not be empty
        prop_assert!(!chunks.is_empty());

        // Sum of chunks should equal total
        let total_chunked: u64 = chunks.iter().sum();
        prop_assert_eq!(total_chunked, total_scaled);

        // Each chunk should respect max impact constraint
        for chunk in &chunks {
            let impact = *chunk as f64 / liquidity_scaled as f64;
            prop_assert!(impact <= max_impact + f64::EPSILON);
        }

        // Should not have excessive fragmentation
        prop_assert!(chunks.len() <= 1000);
    }

    #[test]
    fn test_twap_cost_reasonableness(
        amount in 5_000_000u64..50_000_000,
        liquidity in 25_000_000u64..250_000_000,
        time_buckets in 6u32..48
    ) {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let amount_scaled = amount * Price::PRECISION;
        let liquidity_scaled = liquidity * Price::PRECISION;

        let twap_cost = calc.estimate_twap_execution_cost(
            amount_scaled,
            liquidity_scaled,
            time_buckets
        ).unwrap();

        // TWAP cost should be reasonable
        prop_assert!(twap_cost >= 0.0);
        prop_assert!(twap_cost <= 0.1); // Should not exceed 10%

        // More time buckets should generally lead to lower costs
        if time_buckets >= 24 {
            let fewer_buckets_cost = calc.estimate_twap_execution_cost(
                amount_scaled,
                liquidity_scaled,
                12
            ).unwrap();

            // More buckets should be better or equal
            prop_assert!(twap_cost <= fewer_buckets_cost + 0.001);
        }
    }

    #[test]
    fn test_price_percentage_change_properties(
        old_price in 1.0f64..10_000.0,
        new_price in 1.0f64..10_000.0
    ) {
        let p1 = Price::from_float(old_price, 6).unwrap();
        let p2 = Price::from_float(new_price, 6).unwrap();

        let change = p1.percentage_change(&p2).unwrap();

        // Change should be finite
        prop_assert!(change.is_finite());

        // If prices are equal, change should be zero
        if (old_price - new_price).abs() < 0.000001 {
            prop_assert!((change).abs() < 0.001);
        }

        // If new price is double, change should be approximately 100%
        if (new_price / old_price - 2.0).abs() < 0.01 {
            prop_assert!((change - 100.0).abs() < 1.0);
        }

        // If new price is half, change should be approximately -50%
        if (new_price / old_price - 0.5).abs() < 0.01 {
            prop_assert!((change + 50.0).abs() < 1.0);
        }
    }

    #[test]
    fn test_volume_bucket_consistency(
        trade_amount in 10_000_000u64..100_000_000,
        liquidity in 50_000_000u64..500_000_000
    ) {
        let calc = SlippageCalculator::new(SlippageParams::default());

        let trade_scaled = trade_amount * Price::PRECISION;
        let liquidity_scaled = liquidity * Price::PRECISION;

        let slippage = calc.calculate_slippage(trade_scaled, liquidity_scaled).unwrap();

        // Should have exactly 10 volume buckets
        prop_assert_eq!(slippage.volume_buckets.len(), 10);

        // Sum of bucket amounts should equal trade amount
        let total_bucket_amount: u64 = slippage.volume_buckets.iter()
            .map(|b| b.amount)
            .sum();
        prop_assert_eq!(total_bucket_amount, trade_scaled);

        // Sum of bucket percentages should equal 100%
        let total_percentage: f64 = slippage.volume_buckets.iter()
            .map(|b| b.percentage)
            .sum();
        prop_assert!((total_percentage - 100.0).abs() < 0.001);

        // Buckets should show monotonic slippage increase
        for i in 1..slippage.volume_buckets.len() {
            prop_assert!(slippage.volume_buckets[i].slippage >=
                        slippage.volume_buckets[i-1].slippage);
        }
    }
}