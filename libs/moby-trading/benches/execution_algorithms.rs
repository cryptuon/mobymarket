use criterion::{black_box, criterion_group, criterion_main, Criterion};
use moby_trading::*;
use moby_types::*;

fn benchmark_execution_algorithms(c: &mut Criterion) {
    let executor = execution::MockExecutionEngine::new();
    let amount = WhaleAmount::new(1_000_000);
    let price = moby_math::Price::new(100, 0);

    let order = Order {
        id: OrderId::new(),
        trader: AccountKey::generate_random(),
        order_type: OrderType::Market,
        side: OrderSide::Buy,
        amount,
        price: Some(price),
        tier: TradingTier::Whale,
        privacy_level: PrivacyLevel::Standard,
        time_in_force: TimeInForce::GoodTillCancelled,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: OrderStatus::Pending,
        filled_amount: WhaleAmount::new(0),
        fees_paid: WhaleAmount::new(0),
        metadata: std::collections::HashMap::new(),
    };

    c.bench_function("order_validation", |b| {
        b.iter(|| {
            let result = executor.validate_order(&order);
            black_box(result);
        })
    });

    c.bench_function("order_matching", |b| {
        b.iter(|| {
            let result = executor.execute_order(order.clone());
            black_box(result);
        })
    });
}

criterion_group!(benches, benchmark_execution_algorithms);
criterion_main!(benches);