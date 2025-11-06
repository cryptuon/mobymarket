use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use moby_math::{Price, SlippageCalculator, SlippageParams};

fn bench_price_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("price_calculations");

    let prices = vec![
        ("BTC", 43_250.75),
        ("ETH", 2_500.0),
        ("Small Altcoin", 0.00000123),
        ("Stablecoin", 1.0),
    ];

    let whale_amounts = vec![
        ("$1M", 1_000_000),
        ("$10M", 10_000_000),
        ("$50M", 50_000_000),
        ("$100M", 100_000_000),
        ("$500M", 500_000_000),
    ];

    for (price_name, price_value) in &prices {
        for (amount_name, amount_value) in &whale_amounts {
            let price = Price::from_float(*price_value, 6).unwrap();
            let amount = *amount_value * Price::PRECISION;

            group.bench_with_input(
                BenchmarkId::new("divide_amount", format!("{}-{}", price_name, amount_name)),
                &(price, amount),
                |b, (p, a)| {
                    b.iter(|| p.divide_amount(black_box(*a)).unwrap())
                },
            );

            group.bench_with_input(
                BenchmarkId::new("multiply_amount", format!("{}-{}", price_name, amount_name)),
                &(price, amount),
                |b, (p, a)| {
                    let token_amount = p.divide_amount(*a).unwrap();
                    b.iter(|| p.multiply_amount(black_box(token_amount)).unwrap())
                },
            );
        }
    }

    group.finish();
}

fn bench_slippage_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("slippage_calculations");

    let calculator = SlippageCalculator::new(SlippageParams::default());

    let scenarios = vec![
        ("Small Whale", 1_000_000, 50_000_000),    // $1M in $50M pool
        ("Medium Whale", 10_000_000, 100_000_000), // $10M in $100M pool
        ("Large Whale", 50_000_000, 200_000_000),  // $50M in $200M pool
        ("Mega Whale", 100_000_000, 500_000_000),  // $100M in $500M pool
        ("Sovereign Fund", 500_000_000, 1_000_000_000), // $500M in $1B pool
    ];

    for (scenario_name, trade_amount, liquidity) in scenarios {
        let trade_amount_scaled = trade_amount * Price::PRECISION;
        let liquidity_scaled = liquidity * Price::PRECISION;

        group.bench_with_input(
            BenchmarkId::new("calculate_slippage", scenario_name),
            &(trade_amount_scaled, liquidity_scaled),
            |b, (trade, liq)| {
                b.iter(|| {
                    calculator.calculate_slippage(black_box(*trade), black_box(*liq)).unwrap()
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("optimal_chunks", scenario_name),
            &(trade_amount_scaled, liquidity_scaled),
            |b, (trade, liq)| {
                b.iter(|| {
                    calculator.calculate_whale_optimal_chunks(
                        black_box(*trade),
                        black_box(*liq),
                        black_box(0.02)
                    ).unwrap()
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("twap_cost", scenario_name),
            &(trade_amount_scaled, liquidity_scaled),
            |b, (trade, liq)| {
                b.iter(|| {
                    calculator.estimate_twap_execution_cost(
                        black_box(*trade),
                        black_box(*liq),
                        black_box(24)
                    ).unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_roundtrip_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_calculations");

    let btc_price = Price::from_float(43_250.0, 6).unwrap();
    let eth_price = Price::from_float(2_500.0, 6).unwrap();

    let amounts = vec![
        ("$1M", 1_000_000),
        ("$50M", 50_000_000),
        ("$100M", 100_000_000),
    ];

    for (amount_name, amount_value) in amounts {
        let amount = amount_value * Price::PRECISION;

        group.bench_with_input(
            BenchmarkId::new("btc_roundtrip", amount_name),
            &amount,
            |b, a| {
                b.iter(|| {
                    let token_amount = btc_price.divide_amount(black_box(*a)).unwrap();
                    btc_price.multiply_amount(black_box(token_amount)).unwrap()
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("eth_roundtrip", amount_name),
            &amount,
            |b, a| {
                b.iter(|| {
                    let token_amount = eth_price.divide_amount(black_box(*a)).unwrap();
                    eth_price.multiply_amount(black_box(token_amount)).unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_whale_tier_classification(c: &mut Criterion) {
    let calculator = SlippageCalculator::new(SlippageParams::default());

    let amounts = vec![
        500_000,     // Retail
        5_000_000,   // Small whale
        25_000_000,  // Medium whale
        75_000_000,  // Large whale
        150_000_000, // Mega whale
    ];

    c.bench_function("whale_tier_classification", |b| {
        b.iter(|| {
            for &amount in &amounts {
                let amount_scaled = amount * Price::PRECISION;
                calculator.get_whale_tier_slippage(black_box(amount_scaled));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_price_calculations,
    bench_slippage_calculations,
    bench_roundtrip_calculations,
    bench_whale_tier_classification
);
criterion_main!(benches);