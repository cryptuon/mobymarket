use criterion::{black_box, criterion_group, criterion_main, Criterion};
use moby_trading::*;
use moby_types::*;

fn benchmark_order_matching(c: &mut Criterion) {
    let mut order_book = OrderBook::new();

    // Create test orders
    let buy_order = Order {
        id: OrderId::new(),
        trader: AccountKey::generate_random(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        amount: WhaleAmount::new(1_000_000),
        price: Some(moby_math::Price::new(100, 0)),
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

    let sell_order = Order {
        id: OrderId::new(),
        trader: AccountKey::generate_random(),
        order_type: OrderType::Limit,
        side: OrderSide::Sell,
        amount: WhaleAmount::new(800_000),
        price: Some(moby_math::Price::new(100, 0)),
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

    c.bench_function("add_order", |b| {
        b.iter(|| {
            let mut book = order_book.clone();
            let result = book.add_order(buy_order.clone());
            black_box(result);
        })
    });

    c.bench_function("match_orders", |b| {
        let mut book = order_book.clone();
        book.add_order(buy_order.clone()).unwrap();

        b.iter(|| {
            let mut book_copy = book.clone();
            let matches = book_copy.find_matches(&sell_order);
            black_box(matches);
        })
    });

    c.bench_function("remove_order", |b| {
        b.iter(|| {
            let mut book = order_book.clone();
            book.add_order(buy_order.clone()).unwrap();
            let result = book.remove_order(buy_order.id);
            black_box(result);
        })
    });
}

criterion_group!(benches, benchmark_order_matching);
criterion_main!(benches);