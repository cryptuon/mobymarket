# Testing Framework

> **Note:** This page may be out of date relative to the current source. The filename is `core-libraries.md` but its contents describe the testing framework — the title and filename do not match. PRs welcome.

> **Comprehensive testing strategy for reliable whale trading infrastructure**

## Testing Philosophy

### Test-Driven Development
- **Write tests first** - Define expected behavior before implementation
- **Comprehensive coverage** - Unit, integration, and end-to-end testing
- **Performance testing** - Ensure whale-scale performance under load
- **Security testing** - Protect against financial attacks
- **Property-based testing** - Verify mathematical properties hold

### Testing Pyramid
```
    /\
   /E2E\      End-to-End Tests (5%)
  /______\     - Full system integration
 /Integration\ Integration Tests (15%)
/______________\ - Cross-component testing
/   Unit Tests   \ Unit Tests (80%)
/________________\ - Individual function testing
```

---

## 1. Unit Testing Setup

### Test Structure
```
tests/
├── unit/
│   ├── math/
│   │   ├── price_tests.rs
│   │   ├── slippage_tests.rs
│   │   └── fee_tests.rs
│   ├── state/
│   │   ├── order_state_tests.rs
│   │   ├── account_tests.rs
│   │   └── serialization_tests.rs
│   ├── oracle/
│   │   ├── aggregator_tests.rs
│   │   ├── pyth_adapter_tests.rs
│   │   └── price_validation_tests.rs
│   └── trading/
│       ├── otc_logic_tests.rs
│       ├── algorithm_tests.rs
│       └── routing_tests.rs
├── integration/
│   ├── program_interactions/
│   ├── cross_program_calls/
│   └── end_to_end_flows/
├── performance/
│   ├── load_tests/
│   ├── stress_tests/
│   └── benchmark_tests/
└── property/
    ├── mathematical_properties/
    ├── invariant_tests/
    └── fuzzing_tests/
```

### Unit Test Framework (`tests/unit/math/price_tests.rs`)
```rust
use anchor_lang::prelude::*;
use moby_math::price::*;
use moby_math::slippage::*;
use moby_math::error::MathError;

#[cfg(test)]
mod price_tests {
    use super::*;

    #[test]
    fn test_price_creation_from_float() {
        let price = Price::from_float(150.25, 6).unwrap();
        assert_eq!(price.value, 150_250_000);
        assert_eq!(price.decimals, 6);
        assert_eq!(price.to_float(), 150.25);
    }

    #[test]
    fn test_price_precision_limits() {
        // Test maximum safe values
        let max_safe_price = (u64::MAX / Price::PRECISION) as f64 / Price::PRECISION as f64;
        let price = Price::from_float(max_safe_price, 6);
        assert!(price.is_ok());

        // Test overflow protection
        let too_large_price = u64::MAX as f64;
        let price = Price::from_float(too_large_price, 6);
        assert!(price.is_err());
    }

    #[test]
    fn test_whale_sized_calculations() {
        let btc_price = Price::from_float(43_250.75, 6).unwrap();

        // Test $100M trade
        let whale_amount = 100_000_000 * Price::PRECISION; // $100M
        let btc_amount = btc_price.divide_amount(whale_amount).unwrap();

        // Should be around 2,312 BTC
        let expected_btc = 2_312 * Price::PRECISION;
        let tolerance = expected_btc / 100; // 1% tolerance
        assert!((btc_amount as i64 - expected_btc as i64).abs() < tolerance as i64);

        // Test reverse calculation
        let usd_back = btc_price.multiply_amount(btc_amount).unwrap();
        let usd_tolerance = whale_amount / 1000; // 0.1% tolerance
        assert!((usd_back as i64 - whale_amount as i64).abs() < usd_tolerance as i64);
    }

    #[test]
    fn test_price_edge_cases() {
        // Zero price handling
        let zero_price = Price::new(0, 6);
        assert!(zero_price.multiply_amount(1000).is_ok()); // Should be 0
        assert!(zero_price.divide_amount(1000).is_err());  // Should error

        // Minimal price
        let min_price = Price::new(1, 6); // 0.000001
        assert!(min_price.multiply_amount(1_000_000).unwrap() == 1);

        // Maximum safe calculations
        let max_price = Price::new(u64::MAX / 2, 6);
        assert!(max_price.multiply_amount(1).is_ok());
        assert!(max_price.multiply_amount(u64::MAX).is_err()); // Should overflow
    }
}

#[cfg(test)]
mod slippage_tests {
    use super::*;

    #[test]
    fn test_slippage_calculation_precision() {
        // Test precise slippage calculations
        let expected = 1_000_000_000; // $1000 with 6 decimals
        let actual = 995_000_000;     // $995 with 6 decimals

        let slippage = SlippageCalculator::calculate_slippage(expected, actual).unwrap();
        assert_eq!(slippage, 500); // 5% = 500 basis points
    }

    #[test]
    fn test_whale_sized_slippage() {
        // Test slippage on $100M trade
        let expected = 100_000_000 * Price::PRECISION;
        let actual = 99_900_000 * Price::PRECISION; // 0.1% slippage

        let slippage = SlippageCalculator::calculate_slippage(expected, actual).unwrap();
        assert_eq!(slippage, 10); // 0.1% = 10 basis points
    }

    #[test]
    fn test_amm_price_impact() {
        // Uniswap-style constant product test
        // Pool: 1M USDC, 1K ETH (price = $1000 per ETH)
        let reserve_usdc = 1_000_000 * Price::PRECISION;
        let reserve_eth = 1_000 * Price::PRECISION;

        // Swap $50K USDC for ETH
        let usdc_in = 50_000 * Price::PRECISION;

        let impact = SlippageCalculator::calculate_price_impact(
            usdc_in,
            reserve_usdc,
            reserve_eth
        ).unwrap();

        // Should be around 2.5% impact
        assert!(impact >= 240 && impact <= 260); // 2.4% - 2.6%
    }

    #[test]
    fn test_slippage_tolerance_application() {
        let amount = 1_000_000 * Price::PRECISION; // $1M

        // 0.5% tolerance
        let min_amount = SlippageCalculator::apply_slippage_tolerance(amount, 50).unwrap();
        let expected_min = 995_000 * Price::PRECISION; // $995K
        assert_eq!(min_amount, expected_min);

        // 0.01% tolerance (1 basis point)
        let min_amount_tight = SlippageCalculator::apply_slippage_tolerance(amount, 1).unwrap();
        let expected_min_tight = 999_900 * Price::PRECISION; // $999.9K
        assert_eq!(min_amount_tight, expected_min_tight);
    }
}

// Property-based testing with proptest
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_price_multiplication_division_inverse(
            amount in 1u64..1_000_000_000_000u64,
            price_float in 0.01f64..1_000_000.0f64
        ) {
            let price = Price::from_float(price_float, 6)?;

            if price.value > 0 {
                let usd_value = price.multiply_amount(amount)?;
                let back_to_tokens = price.divide_amount(usd_value)?;

                // Should be approximately equal (within rounding)
                let diff = (amount as i64 - back_to_tokens as i64).abs();
                let tolerance = std::cmp::max(1, amount / 1_000_000); // 0.0001% tolerance
                prop_assert!(diff <= tolerance as i64);
            }
        }

        #[test]
        fn test_slippage_properties(
            expected in 1u64..u64::MAX/2,
            slippage_bps in 0u16..10000u16
        ) {
            let slippage_amount = expected * slippage_bps as u64 / 10_000;
            let actual = expected - slippage_amount;

            let calculated_slippage = SlippageCalculator::calculate_slippage(expected, actual)?;

            // Calculated slippage should be very close to input slippage
            let diff = (calculated_slippage as i32 - slippage_bps as i32).abs();
            prop_assert!(diff <= 1); // Allow 1 basis point rounding error
        }

        #[test]
        fn test_price_impact_monotonicity(
            amount1 in 1u64..1_000_000u64,
            amount2 in 1_000_001u64..10_000_000u64,
            reserve_in in 10_000_000u64..1_000_000_000u64,
            reserve_out in 10_000_000u64..1_000_000_000u64
        ) {
            let impact1 = SlippageCalculator::calculate_price_impact(amount1, reserve_in, reserve_out)?;
            let impact2 = SlippageCalculator::calculate_price_impact(amount2, reserve_in, reserve_out)?;

            // Larger trade should always have higher impact
            prop_assert!(impact2 >= impact1);
        }
    }
}
```

---

## 2. Integration Testing

### Cross-Program Testing (`tests/integration/program_interactions.rs`)
```rust
use anchor_lang::prelude::*;
use anchor_client::{Client, Cluster};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    system_program,
};

#[tokio::test]
async fn test_otc_order_full_flow() {
    let client = setup_test_client().await;

    // Setup test accounts
    let maker = Keypair::new();
    let taker = Keypair::new();
    let (maker_usdc, maker_sol) = setup_token_accounts(&client, &maker).await;
    let (taker_usdc, taker_sol) = setup_token_accounts(&client, &taker).await;

    // Fund accounts with test tokens
    fund_account(&client, &maker_usdc, 1_000_000 * 1_000_000).await; // 1M USDC
    fund_account(&client, &taker_sol, 10_000 * 1_000_000_000).await; // 10K SOL

    // Create OTC order: Sell 1000 SOL for 150K USDC
    let order_params = CreateOrderParams {
        order_id: [1u8; 32],
        amount_a: 1000 * 1_000_000_000, // 1000 SOL
        amount_b: 150_000 * 1_000_000,  // 150K USDC
        expiry: get_current_timestamp() + 3600, // 1 hour
        min_fill_amount: 100 * 1_000_000_000, // Min 100 SOL
        slippage_tolerance: 50, // 0.5%
        privacy_level: PrivacyLevel::Public,
    };

    let order_pubkey = create_otc_order(&client, &maker, order_params).await?;

    // Verify order created correctly
    let order_account: OTCOrder = client.account(order_pubkey).await?;
    assert_eq!(order_account.maker, maker.pubkey());
    assert_eq!(order_account.status, OrderStatus::Created);
    assert_eq!(order_account.amount_a, 1000 * 1_000_000_000);

    // Partially fill order: Take 500 SOL
    let fill_amount = 500 * 1_000_000_000;
    accept_otc_order(&client, &taker, order_pubkey, fill_amount).await?;

    // Verify partial fill
    let order_account: OTCOrder = client.account(order_pubkey).await?;
    assert_eq!(order_account.status, OrderStatus::Partial);
    assert_eq!(order_account.filled_amount_a, fill_amount);

    // Verify token transfers
    let maker_usdc_balance = get_token_balance(&client, &maker_usdc).await;
    assert_eq!(maker_usdc_balance, 1_075_000 * 1_000_000); // Received 75K USDC

    let taker_sol_balance = get_token_balance(&client, &taker_sol).await;
    assert_eq!(taker_sol_balance, 10_500 * 1_000_000_000); // Received 500 SOL

    // Complete the fill
    accept_otc_order(&client, &taker, order_pubkey, fill_amount).await?;

    // Verify complete fill
    let order_account: OTCOrder = client.account(order_pubkey).await?;
    assert_eq!(order_account.status, OrderStatus::Filled);
    assert_eq!(order_account.filled_amount_a, order_account.amount_a);
}

async fn setup_test_client() -> Client {
    let payer = Keypair::new();
    let client = Client::new_with_options(
        Cluster::Localnet,
        payer,
        CommitmentConfig::processed(),
    );

    // Airdrop SOL for fees
    client.request_airdrop(&client.payer().pubkey(), 10_000_000_000).await.unwrap();

    client
}

#[tokio::test]
async fn test_twap_execution() {
    let client = setup_test_client().await;
    let trader = Keypair::new();

    // Setup TWAP order: Sell 10K SOL over 1 hour in 12 intervals
    let twap_params = CreateTWAPParams {
        order_id: [2u8; 32],
        token_in: get_sol_mint(),
        token_out: get_usdc_mint(),
        total_amount: 10_000 * 1_000_000_000,
        min_amount_out: 1_400_000 * 1_000_000, // Min $1.4M USDC
        time_window: 3600, // 1 hour
        num_intervals: 12, // Every 5 minutes
        randomness_factor: 20, // 20% timing randomness
        max_slippage_bps: 50, // 0.5% max slippage
        venue_preferences: vec![
            VenueWeight { venue: get_raydium_program(), weight: 50 },
            VenueWeight { venue: get_orca_program(), weight: 50 },
        ],
        privacy_enabled: false,
    };

    let twap_order_pubkey = create_twap_order(&client, &trader, twap_params).await?;

    // Simulate time passage and execute intervals
    for interval in 0..12 {
        // Fast-forward time (in real test, would use clock manipulation)
        advance_clock(&client, 300).await; // 5 minutes

        // Execute interval
        execute_twap_interval(&client, twap_order_pubkey).await?;

        // Verify progress
        let twap_order: TWAPOrder = client.account(twap_order_pubkey).await?;
        assert_eq!(twap_order.current_interval, interval + 1);
        assert!(twap_order.executed_amount_in > 0);

        if interval == 11 {
            assert_eq!(twap_order.status, AlgorithmStatus::Completed);
            assert_eq!(twap_order.executed_amount_in, twap_order.total_amount_in);
        }
    }
}

#[tokio::test]
async fn test_oracle_price_aggregation() {
    let client = setup_test_client().await;

    // Setup multiple oracle feeds
    let pyth_feed = setup_mock_pyth_feed(&client, 43_250_750_000).await; // $43,250.75
    let switchboard_feed = setup_mock_switchboard_feed(&client, 43_245_500_000).await; // $43,245.50
    let chainlink_feed = setup_mock_chainlink_feed(&client, 43_260_000_000).await; // $43,260.00

    // Create oracle aggregator
    let aggregator_params = InitializeOracleParams {
        token_mint: get_btc_mint(),
        oracle_sources: vec![
            OracleSource {
                oracle_type: OracleType::Pyth,
                feed_account: pyth_feed,
                weight: 40,
                max_confidence: 1_000_000, // 1%
                enabled: true,
            },
            OracleSource {
                oracle_type: OracleType::Switchboard,
                feed_account: switchboard_feed,
                weight: 35,
                max_confidence: 1_000_000,
                enabled: true,
            },
            OracleSource {
                oracle_type: OracleType::Chainlink,
                feed_account: chainlink_feed,
                weight: 25,
                max_confidence: 1_000_000,
                enabled: true,
            },
        ],
        aggregation_method: AggregationMethod::Median,
        deviation_threshold_bps: 200, // 2%
        staleness_threshold: 300, // 5 minutes
        min_sources_required: 2,
    };

    let aggregator_pubkey = initialize_oracle_aggregator(&client, aggregator_params).await?;

    // Update price feed
    update_price_feed(&client, aggregator_pubkey).await?;

    // Verify aggregated price
    let oracle_aggregator: OracleAggregator = client.account(aggregator_pubkey).await?;

    // Should be median of the three prices: $43,250.75
    assert_eq!(oracle_aggregator.current_price.price.value, 43_250_750_000);
    assert_eq!(oracle_aggregator.valid_sources_count, 3);
    assert!(oracle_aggregator.manipulation_score < 50); // Low manipulation risk
}
```

---

## 3. Performance Testing

### Load Testing (`tests/performance/load_tests.rs`)
```rust
use anchor_client::Client;
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[tokio::test]
async fn test_concurrent_order_creation() {
    let client = Arc::new(setup_test_client().await);
    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let start_time = Instant::now();
    let num_orders = 1000;
    let concurrent_tasks = 50;

    // Create semaphore to limit concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent_tasks));

    let mut handles = Vec::new();

    for i in 0..num_orders {
        let client = Arc::clone(&client);
        let success_count = Arc::clone(&success_count);
        let error_count = Arc::clone(&error_count);
        let semaphore = Arc::clone(&semaphore);

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let maker = Keypair::new();
            let order_params = CreateOrderParams {
                order_id: [i as u8; 32],
                amount_a: 1000 * 1_000_000_000, // 1000 SOL
                amount_b: 150_000 * 1_000_000,  // 150K USDC
                expiry: get_current_timestamp() + 3600,
                min_fill_amount: 100 * 1_000_000_000,
                slippage_tolerance: 50,
                privacy_level: PrivacyLevel::Public,
            };

            match create_otc_order(&client, &maker, order_params).await {
                Ok(_) => success_count.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_count.fetch_add(1, Ordering::Relaxed),
            };
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start_time.elapsed();
    let success = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);

    println!("Load Test Results:");
    println!("  Duration: {:?}", duration);
    println!("  Success: {}", success);
    println!("  Errors: {}", errors);
    println!("  Orders/sec: {:.2}", success as f64 / duration.as_secs_f64());

    // Assertions
    assert!(success >= num_orders * 95 / 100); // 95% success rate
    assert!(duration < Duration::from_secs(60)); // Complete within 1 minute
    assert!(success as f64 / duration.as_secs_f64() >= 16.0); // >16 orders/sec
}

#[tokio::test]
async fn test_twap_performance_under_load() {
    let client = Arc::new(setup_test_client().await);
    let num_twaps = 100;
    let intervals_per_twap = 10;

    let start_time = Instant::now();

    // Create multiple TWAP orders
    let mut twap_orders = Vec::new();
    for i in 0..num_twaps {
        let trader = Keypair::new();
        let twap_params = CreateTWAPParams {
            order_id: [(i as u8); 32],
            token_in: get_sol_mint(),
            token_out: get_usdc_mint(),
            total_amount: 1000 * 1_000_000_000, // 1000 SOL
            min_amount_out: 140_000 * 1_000_000, // $140K USDC
            time_window: 600, // 10 minutes
            num_intervals: intervals_per_twap,
            randomness_factor: 10,
            max_slippage_bps: 100,
            venue_preferences: vec![],
            privacy_enabled: false,
        };

        let twap_pubkey = create_twap_order(&client, &trader, twap_params).await.unwrap();
        twap_orders.push(twap_pubkey);
    }

    // Execute all intervals for all TWAPs
    let mut total_executions = 0;
    for interval in 0..intervals_per_twap {
        advance_clock(&client, 60).await; // 1 minute

        let interval_start = Instant::now();
        let mut handles = Vec::new();

        for &twap_order in &twap_orders {
            let client = Arc::clone(&client);
            let handle = tokio::spawn(async move {
                execute_twap_interval(&client, twap_order).await
            });
            handles.push(handle);
        }

        // Wait for all interval executions
        let mut success_count = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success_count += 1;
            }
        }

        total_executions += success_count;
        let interval_duration = interval_start.elapsed();

        println!("Interval {} - Executed {} TWAPs in {:?}",
                interval, success_count, interval_duration);

        // Each interval should complete within reasonable time
        assert!(interval_duration < Duration::from_secs(30));
        assert!(success_count >= num_twaps * 95 / 100); // 95% success rate
    }

    let total_duration = start_time.elapsed();
    println!("Total TWAP Performance:");
    println!("  Total executions: {}", total_executions);
    println!("  Total duration: {:?}", total_duration);
    println!("  Executions/sec: {:.2}", total_executions as f64 / total_duration.as_secs_f64());

    // Should handle >100 TWAP executions per second
    assert!(total_executions as f64 / total_duration.as_secs_f64() >= 100.0);
}

#[tokio::test]
async fn test_oracle_update_performance() {
    let client = setup_test_client().await;
    let num_tokens = 50; // Test 50 different token price feeds
    let updates_per_token = 20;

    // Setup oracle aggregators for multiple tokens
    let mut aggregators = Vec::new();
    for i in 0..num_tokens {
        let token_mint = create_test_token_mint(&client, i).await;
        let aggregator = setup_oracle_aggregator(&client, token_mint).await;
        aggregators.push(aggregator);
    }

    let start_time = Instant::now();
    let mut total_updates = 0;

    for update_round in 0..updates_per_token {
        let round_start = Instant::now();

        // Update all oracle feeds simultaneously
        let mut handles = Vec::new();
        for &aggregator in &aggregators {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                update_price_feed(&client_clone, aggregator).await
            });
            handles.push(handle);
        }

        // Wait for all updates to complete
        let mut success_count = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success_count += 1;
            }
        }

        total_updates += success_count;
        let round_duration = round_start.elapsed();

        println!("Update round {} - {} oracles updated in {:?}",
                update_round, success_count, round_duration);

        // Each round should complete quickly
        assert!(round_duration < Duration::from_secs(5));
        assert!(success_count >= num_tokens * 98 / 100); // 98% success rate

        // Wait before next round
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let total_duration = start_time.elapsed();
    println!("Oracle Performance Summary:");
    println!("  Total updates: {}", total_updates);
    println!("  Duration: {:?}", total_duration);
    println!("  Updates/sec: {:.2}", total_updates as f64 / total_duration.as_secs_f64());

    // Should handle >50 oracle updates per second
    assert!(total_updates as f64 / total_duration.as_secs_f64() >= 50.0);
}
```

---

## 4. Security Testing

### Security Test Suite (`tests/security/attack_vectors.rs`)
```rust
use anchor_lang::prelude::*;

#[tokio::test]
async fn test_reentrancy_protection() {
    let client = setup_test_client().await;

    // Attempt to create a reentrancy attack on OTC order acceptance
    // This should fail due to reentrancy guards

    let attacker = Keypair::new();
    let victim = Keypair::new();

    // Setup malicious contract that attempts reentrancy
    let malicious_contract = deploy_malicious_contract(&client).await;

    // Create victim's order
    let order_params = CreateOrderParams {
        order_id: [1u8; 32],
        amount_a: 1000 * 1_000_000_000,
        amount_b: 150_000 * 1_000_000,
        expiry: get_current_timestamp() + 3600,
        min_fill_amount: 100 * 1_000_000_000,
        slippage_tolerance: 50,
        privacy_level: PrivacyLevel::Public,
    };

    let order_pubkey = create_otc_order(&client, &victim, order_params).await.unwrap();

    // Attempt reentrancy attack
    let attack_result = malicious_contract.attempt_reentrancy(&client, order_pubkey).await;

    // Should fail with reentrancy error
    assert!(attack_result.is_err());

    // Order should still be in valid state
    let order_account: OTCOrder = client.account(order_pubkey).await.unwrap();
    assert_eq!(order_account.status, OrderStatus::Created);
}

#[tokio::test]
async fn test_integer_overflow_protection() {
    let client = setup_test_client().await;
    let attacker = Keypair::new();

    // Attempt to create order with values that would cause overflow
    let malicious_params = CreateOrderParams {
        order_id: [2u8; 32],
        amount_a: u64::MAX,
        amount_b: u64::MAX,
        expiry: get_current_timestamp() + 3600,
        min_fill_amount: u64::MAX,
        slippage_tolerance: u16::MAX,
        privacy_level: PrivacyLevel::Public,
    };

    // Should fail with overflow protection
    let result = create_otc_order(&client, &attacker, malicious_params).await;
    assert!(result.is_err());

    // Test TWAP with overflow values
    let malicious_twap = CreateTWAPParams {
        order_id: [3u8; 32],
        token_in: get_sol_mint(),
        token_out: get_usdc_mint(),
        total_amount: u64::MAX,
        min_amount_out: u64::MAX,
        time_window: i64::MAX,
        num_intervals: u32::MAX,
        randomness_factor: u8::MAX,
        max_slippage_bps: u16::MAX,
        venue_preferences: vec![],
        privacy_enabled: false,
    };

    let result = create_twap_order(&client, &attacker, malicious_twap).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unauthorized_access_protection() {
    let client = setup_test_client().await;
    let authorized_user = Keypair::new();
    let unauthorized_user = Keypair::new();

    // Create order with authorized user
    let order_params = CreateOrderParams {
        order_id: [4u8; 32],
        amount_a: 1000 * 1_000_000_000,
        amount_b: 150_000 * 1_000_000,
        expiry: get_current_timestamp() + 3600,
        min_fill_amount: 100 * 1_000_000_000,
        slippage_tolerance: 50,
        privacy_level: PrivacyLevel::Public,
    };

    let order_pubkey = create_otc_order(&client, &authorized_user, order_params).await.unwrap();

    // Attempt to cancel order with unauthorized user
    let cancel_result = cancel_otc_order(&client, &unauthorized_user, order_pubkey).await;
    assert!(cancel_result.is_err());

    // Attempt to modify order with unauthorized user
    let modify_result = modify_otc_order(&client, &unauthorized_user, order_pubkey, 500).await;
    assert!(modify_result.is_err());

    // Order should still be owned by authorized user
    let order_account: OTCOrder = client.account(order_pubkey).await.unwrap();
    assert_eq!(order_account.maker, authorized_user.pubkey());
    assert_eq!(order_account.status, OrderStatus::Created);
}

#[tokio::test]
async fn test_oracle_manipulation_detection() {
    let client = setup_test_client().await;

    // Setup oracle aggregator
    let aggregator = setup_oracle_aggregator(&client, get_btc_mint()).await;

    // Submit normal prices
    update_mock_oracle_price(&client, OracleType::Pyth, 43_250_000_000).await; // $43,250
    update_mock_oracle_price(&client, OracleType::Switchboard, 43_245_000_000).await; // $43,245
    update_mock_oracle_price(&client, OracleType::Chainlink, 43_260_000_000).await; // $43,260

    update_price_feed(&client, aggregator).await.unwrap();

    let initial_state: OracleAggregator = client.account(aggregator).await.unwrap();
    assert!(initial_state.manipulation_score < 30); // Low manipulation risk

    // Inject manipulated price
    update_mock_oracle_price(&client, OracleType::Pyth, 50_000_000_000).await; // $50,000 (15% spike)

    update_price_feed(&client, aggregator).await.unwrap();

    let manipulated_state: OracleAggregator = client.account(aggregator).await.unwrap();

    // Should detect manipulation
    assert!(manipulated_state.manipulation_score > 70); // High manipulation risk

    // Price should either be filtered out or marked as suspicious
    let price_diff = (manipulated_state.current_price.price.value as i64 -
                     initial_state.current_price.price.value as i64).abs();
    let max_expected_change = initial_state.current_price.price.value / 10; // 10% max

    assert!(price_diff < max_expected_change as i64); // Price should be filtered
}

#[tokio::test]
async fn test_slippage_protection() {
    let client = setup_test_client().await;
    let trader = Keypair::new();

    // Create order with tight slippage tolerance
    let order_params = CreateOrderParams {
        order_id: [5u8; 32],
        amount_a: 1000 * 1_000_000_000, // 1000 SOL
        amount_b: 150_000 * 1_000_000,  // 150K USDC
        expiry: get_current_timestamp() + 3600,
        min_fill_amount: 100 * 1_000_000_000,
        slippage_tolerance: 10, // 0.1% tolerance
        privacy_level: PrivacyLevel::Public,
    };

    let order_pubkey = create_otc_order(&client, &trader, order_params).await.unwrap();

    // Attempt to fill with amount that exceeds slippage tolerance
    let taker = Keypair::new();
    let excessive_slippage_fill = 149_000 * 1_000_000; // Only 149K USDC (>0.1% slippage)

    let fill_result = accept_otc_order_with_amount(&client, &taker, order_pubkey,
                                                  1000 * 1_000_000_000, excessive_slippage_fill).await;

    // Should fail due to slippage protection
    assert!(fill_result.is_err());

    // Order should remain unfilled
    let order_account: OTCOrder = client.account(order_pubkey).await.unwrap();
    assert_eq!(order_account.status, OrderStatus::Created);
    assert_eq!(order_account.filled_amount_a, 0);
}

#[tokio::test]
async fn test_access_control_bypass_attempts() {
    let client = setup_test_client().await;
    let attacker = Keypair::new();

    // Attempt to initialize system with unauthorized account
    let malicious_init = InitializeSystemParams {
        authority: attacker.pubkey(),
        initial_config: InitialConfig::default(),
    };

    let result = initialize_system(&client, &attacker, malicious_init).await;
    assert!(result.is_err());

    // Attempt to pause system without authority
    let result = emergency_pause(&client, &attacker).await;
    assert!(result.is_err());

    // Attempt to upgrade program without authority
    let result = upgrade_program(&client, &attacker, Pubkey::default()).await;
    assert!(result.is_err());

    // Attempt to modify fee collector without authority
    let result = update_fee_collector(&client, &attacker, Pubkey::default()).await;
    assert!(result.is_err());
}
```

### Property-Based Security Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_no_fund_loss_property(
        initial_balance_a in 1u64..1_000_000_000_000u64,
        initial_balance_b in 1u64..1_000_000_000_000u64,
        trade_amount in 1u64..1_000_000_000u64,
    ) {
        // Property: Total funds should never decrease in any valid trade
        let total_before = initial_balance_a + initial_balance_b;

        // Simulate trade execution
        let (final_balance_a, final_balance_b) = execute_simulated_trade(
            initial_balance_a,
            initial_balance_b,
            trade_amount
        );

        let total_after = final_balance_a + final_balance_b;

        // Funds should never be lost (only fees should be deducted)
        prop_assert!(total_after >= total_before * 99 / 100); // Allow up to 1% fees
    }

    #[test]
    fn test_price_monotonicity_property(
        base_price in 1u64..1_000_000_000u64,
        amount in 1u64..1_000_000u64,
        liquidity in 1_000_000u64..1_000_000_000_000u64,
    ) {
        // Property: Larger trades should never result in better prices (more slippage)
        let small_amount = amount;
        let large_amount = amount * 2;

        let price_small = calculate_trade_price(base_price, small_amount, liquidity);
        let price_large = calculate_trade_price(base_price, large_amount, liquidity);

        // Larger trade should have worse or equal price
        prop_assert!(price_large <= price_small);
    }

    #[test]
    fn test_order_matching_fairness(
        orders in prop::collection::vec(
            (1u64..1_000_000u64, 1u64..1_000_000u64), // (price, amount) pairs
            1..100
        )
    ) {
        // Property: Order matching should follow price-time priority
        let mut sorted_orders = orders.clone();
        sorted_orders.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by price descending

        let matching_result = simulate_order_matching(orders);

        // First matched order should be highest price
        if let Some(first_match) = matching_result.first() {
            prop_assert_eq!(first_match.price, sorted_orders[0].0);
        }
    }
}
```

This comprehensive testing framework ensures that the core libraries are robust, secure, and performant enough for whale-scale trading operations. Each library can be developed and tested independently, then integrated into the full system.

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[{"content": "Create core library specifications and interfaces", "status": "completed", "activeForm": "Creating core library specifications and interfaces"}, {"content": "Set up comprehensive testing framework", "status": "completed", "activeForm": "Setting up comprehensive testing framework"}, {"content": "Create development environment setup guide", "status": "in_progress", "activeForm": "Creating development environment setup guide"}, {"content": "Document implementation order and dependencies", "status": "pending", "activeForm": "Documenting implementation order and dependencies"}, {"content": "Set up CI/CD and code quality tools", "status": "pending", "activeForm": "Setting up CI/CD and code quality tools"}]