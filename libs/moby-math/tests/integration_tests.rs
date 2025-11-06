use moby_math::{Price, SlippageCalculator, SlippageParams, calculate_price_impact_savings};

#[test]
fn test_whale_trading_scenario_btc() {
    // Scenario: Whale wants to buy $100M worth of BTC at $43,250
    let btc_price = Price::from_float(43_250.0, 6).unwrap();
    let whale_budget = 100_000_000 * Price::PRECISION; // $100M

    // Calculate BTC amount
    let btc_amount = btc_price.divide_amount(whale_budget).unwrap();

    // Should get approximately 2,312 BTC
    let expected_btc = (100_000_000.0 / 43_250.0 * Price::PRECISION as f64) as u64;
    let tolerance = expected_btc / 1000; // 0.1% tolerance

    assert!((btc_amount as i64 - expected_btc as i64).abs() < tolerance as i64);

    // Calculate slippage for this trade size
    let slippage_calc = SlippageCalculator::new(SlippageParams::default());
    let market_liquidity = 500_000_000 * Price::PRECISION; // $500M BTC liquidity

    let slippage_result = slippage_calc.calculate_slippage(btc_amount, market_liquidity).unwrap();

    // For whale trade in deep liquidity, slippage should be reasonable
    assert!(slippage_result.total_slippage > 0.0);  // Positive slippage
    assert!(slippage_result.total_slippage < 0.1);  // Less than 10%

    // Verify volume buckets show progressive slippage
    assert_eq!(slippage_result.volume_buckets.len(), 10);
    for i in 1..slippage_result.volume_buckets.len() {
        assert!(slippage_result.volume_buckets[i].slippage >=
                slippage_result.volume_buckets[i-1].slippage);
    }
}

#[test]
fn test_sovereign_wealth_fund_scenario() {
    // Scenario: $500M sovereign wealth fund wants to diversify into ETH
    let eth_price = Price::from_float(2_500.0, 6).unwrap();
    let fund_allocation = 500_000_000 * Price::PRECISION; // $500M

    let eth_amount = eth_price.divide_amount(fund_allocation).unwrap();

    // Should get 200,000 ETH
    let expected_eth = (500_000_000.0 / 2_500.0 * Price::PRECISION as f64) as u64;
    let tolerance = expected_eth / 1000; // 0.1% tolerance

    assert!((eth_amount as i64 - expected_eth as i64).abs() < tolerance as i64);

    // Calculate optimal chunking for minimal slippage
    let slippage_calc = SlippageCalculator::new(SlippageParams {
        depth_coefficient: 0.2,
        impact_exponent: 1.8,
        base_slippage: 0.001,
        max_slippage: 0.05,
    });

    let eth_liquidity = 1_000_000_000 * Price::PRECISION; // $1B ETH liquidity
    let max_chunk_impact = 0.02; // 2% max impact per chunk

    let chunks = slippage_calc.calculate_whale_optimal_chunks(
        eth_amount,
        eth_liquidity,
        max_chunk_impact
    ).unwrap();

    // Should require multiple chunks for this size or be reasonable for large liquidity
    assert!(chunks.len() >= 1);
    assert!(chunks.len() < 100); // But not too many

    // Verify total equals original amount
    let total_chunked: u64 = chunks.iter().sum();
    assert_eq!(total_chunked, eth_amount);

    // Each chunk should respect impact limit
    for chunk in &chunks {
        let impact = *chunk as f64 / eth_liquidity as f64;
        assert!(impact <= max_chunk_impact + f64::EPSILON);
    }
}

#[test]
fn test_twap_vs_market_order_comparison() {
    // Compare TWAP execution vs immediate market order for large trade
    let token_price = Price::from_float(1_000.0, 6).unwrap();
    let trade_size = 25_000_000 * Price::PRECISION; // $25M trade
    let market_liquidity = 100_000_000 * Price::PRECISION; // $100M liquidity

    let slippage_calc = SlippageCalculator::new(SlippageParams::default());

    // Immediate market order slippage
    let token_amount = token_price.divide_amount(trade_size).unwrap();
    let market_order_slippage = slippage_calc.calculate_slippage(
        token_amount,
        market_liquidity
    ).unwrap();

    // 24-hour TWAP execution cost
    let twap_cost = slippage_calc.estimate_twap_execution_cost(
        token_amount,
        market_liquidity,
        24
    ).unwrap();

    // TWAP should be significantly better for large trades
    assert!(twap_cost < market_order_slippage.effective_price_impact);

    // Calculate savings from using TWAP
    let savings = calculate_price_impact_savings(
        market_order_slippage.total_slippage,
        twap_cost,
        trade_size
    ).unwrap();

    // Should have some savings from TWAP vs market order
    assert!(savings > 0); // Positive savings
}

#[test]
fn test_multi_asset_whale_portfolio() {
    // Test whale diversifying across multiple assets
    let assets = vec![
        ("BTC", 43_250.0, 40.0), // 40% allocation
        ("ETH", 2_500.0, 30.0),  // 30% allocation
        ("SOL", 100.0, 20.0),    // 20% allocation
        ("AVAX", 35.0, 10.0),    // 10% allocation
    ];

    let total_portfolio = 200_000_000 * Price::PRECISION; // $200M portfolio
    let slippage_calc = SlippageCalculator::new(SlippageParams::default());

    let mut total_allocated = 0u64;
    let mut total_slippage_cost = 0.0;

    for (symbol, price_value, allocation_percent) in assets {
        let price = Price::from_float(price_value, 6).unwrap();
        let allocation_amount = (total_portfolio as f64 * allocation_percent / 100.0) as u64;

        let token_amount = price.divide_amount(allocation_amount).unwrap();

        // Assume different liquidity levels for different assets
        let liquidity_multiplier = match symbol {
            "BTC" => 10.0, // Highest liquidity
            "ETH" => 8.0,
            "SOL" => 3.0,
            "AVAX" => 1.5, // Lowest liquidity
            _ => 1.0,
        };

        let asset_liquidity = (allocation_amount as f64 * liquidity_multiplier) as u64;

        let slippage = slippage_calc.calculate_slippage(token_amount, asset_liquidity).unwrap();

        total_allocated += allocation_amount;
        total_slippage_cost += slippage.effective_price_impact * allocation_amount as f64;

        // Verify reasonable slippage for each asset
        assert!(slippage.total_slippage < 0.1); // Less than 10%

        // Higher cap assets should have lower slippage
        if symbol == "BTC" || symbol == "ETH" {
            assert!(slippage.total_slippage < 0.05); // Less than 5% for majors
        }
    }

    // Verify full allocation
    assert_eq!(total_allocated, total_portfolio);

    // Total slippage cost should be reasonable for diversified portfolio
    let average_slippage = total_slippage_cost / total_portfolio as f64;
    assert!(average_slippage < 0.03); // Less than 3% average slippage
}

#[test]
fn test_whale_tier_classification_accuracy() {
    let slippage_calc = SlippageCalculator::new(SlippageParams::default());

    // Test various whale tiers
    let test_cases = vec![
        (500_000, 0.001),     // $500K - retail tier
        (2_000_000, 0.005),   // $2M - small whale
        (15_000_000, 0.015),  // $15M - medium whale
        (75_000_000, 0.025),  // $75M - large whale
        (200_000_000, 0.05),  // $200M - mega whale
    ];

    for (amount_usd, expected_slippage) in test_cases {
        let amount_scaled = amount_usd * Price::PRECISION;
        let tier_slippage = slippage_calc.get_whale_tier_slippage(amount_scaled);

        assert_eq!(tier_slippage, expected_slippage);
    }
}

#[test]
fn test_extreme_market_conditions() {
    // Test behavior during extreme market conditions
    let slippage_calc = SlippageCalculator::new(SlippageParams {
        depth_coefficient: 0.5,  // Very thin markets
        impact_exponent: 2.0,    // Quadratic impact
        base_slippage: 0.005,    // Higher base slippage
        max_slippage: 0.15,      // 15% max slippage cap
    });

    // Large trade in thin market
    let trade_amount = 50_000_000 * Price::PRECISION; // $50M
    let thin_liquidity = 60_000_000 * Price::PRECISION; // Only $60M liquidity

    let slippage = slippage_calc.calculate_slippage(trade_amount, thin_liquidity).unwrap();

    // Should hit the max slippage cap
    assert_eq!(slippage.total_slippage, 0.15);

    // Chunking should help significantly
    let chunks = slippage_calc.calculate_whale_optimal_chunks(
        trade_amount,
        thin_liquidity,
        0.01 // 1% max impact per chunk
    ).unwrap();

    // Should require many chunks for such tight constraints
    assert!(chunks.len() > 10);

    // TWAP should provide much better execution
    let twap_cost = slippage_calc.estimate_twap_execution_cost(
        trade_amount,
        thin_liquidity,
        48 // 48-hour execution
    ).unwrap();

    // TWAP should be significantly better than market order
    assert!(twap_cost < slippage.effective_price_impact * 0.5);
}

#[test]
fn test_roundtrip_precision_whale_amounts() {
    // Test precision for various whale amounts and token prices
    let test_scenarios = vec![
        (43_250.0, 100_000_000), // BTC: $100M
        (2_500.0, 50_000_000),   // ETH: $50M
        (100.0, 25_000_000),     // SOL: $25M
        (1.0, 10_000_000),       // USDC: $10M
        (0.5, 5_000_000),        // Low-cap: $5M
    ];

    for (price_value, amount_usd) in test_scenarios {
        let price = Price::from_float(price_value, 6).unwrap();
        let amount_scaled = amount_usd * Price::PRECISION;

        // Convert USD to tokens and back
        let token_amount = price.divide_amount(amount_scaled).unwrap();
        let back_to_usd = price.multiply_amount(token_amount).unwrap();

        // Calculate precision loss
        let precision_loss = (back_to_usd as f64 - amount_scaled as f64).abs() / amount_scaled as f64;

        // Should maintain high precision even for large amounts
        assert!(precision_loss < 0.001,
               "Precision loss {} too high for price {} amount {}",
               precision_loss, price_value, amount_usd);
    }
}