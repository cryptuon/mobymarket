use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use moby_oracle::{
    PriceAggregator, AggregationMethod, PriceFeed, PriceData,
    sources::OracleSource, ConfidenceCalculator
};
use moby_math::Price;
use chrono::Utc;

fn bench_aggregation_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregation_methods");

    let feeds = create_benchmark_feeds();

    let methods = vec![
        ("weighted_average", AggregationMethod::WeightedAverage),
        ("median_trimmed", AggregationMethod::MedianWithTrimming { trim_percent: 10.0 }),
        ("volume_weighted", AggregationMethod::VolumeWeighted),
        ("confidence_weighted", AggregationMethod::ConfidenceWeighted),
        ("hybrid", AggregationMethod::Hybrid),
    ];

    for (name, method) in methods {
        let aggregator = PriceAggregator::with_method(method);

        group.bench_with_input(
            BenchmarkId::new("aggregate", name),
            &feeds,
            |b, feeds| {
                b.iter(|| aggregator.aggregate(black_box(feeds)).unwrap())
            },
        );
    }

    group.finish();
}

fn bench_source_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("source_scaling");

    let base_feeds = create_benchmark_feeds();
    let aggregator = PriceAggregator::new();

    for source_count in [3, 5, 10, 20, 50].iter() {
        let feeds = extend_feeds(&base_feeds, *source_count);

        group.bench_with_input(
            BenchmarkId::new("sources", source_count),
            &feeds,
            |b, feeds| {
                b.iter(|| aggregator.aggregate(black_box(feeds)).unwrap())
            },
        );
    }

    group.finish();
}

fn bench_confidence_calculation(c: &mut Criterion) {
    let feeds = create_benchmark_feeds();
    let aggregated_price = Price::from_float(43_255.0, 8).unwrap();
    let calculator = ConfidenceCalculator::new();

    c.bench_function("confidence_calculation", |b| {
        b.iter(|| {
            calculator.calculate(black_box(&feeds), black_box(&aggregated_price)).unwrap()
        })
    });
}

fn bench_outlier_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("outlier_detection");

    let normal_feeds = create_benchmark_feeds();
    let outlier_feeds = create_outlier_feeds();

    let aggregator = PriceAggregator::new().with_outlier_threshold(0.05);

    group.bench_with_input(
        BenchmarkId::new("filter_outliers", "normal"),
        &normal_feeds,
        |b, feeds| {
            b.iter(|| aggregator.filter_outliers(black_box(feeds)).unwrap())
        },
    );

    group.bench_with_input(
        BenchmarkId::new("filter_outliers", "with_outliers"),
        &outlier_feeds,
        |b, feeds| {
            b.iter(|| aggregator.filter_outliers(black_box(feeds)).unwrap())
        },
    );

    group.finish();
}

fn bench_price_statistics(c: &mut Criterion) {
    let feeds = create_benchmark_feeds();
    let aggregator = PriceAggregator::new();

    c.bench_function("price_statistics", |b| {
        b.iter(|| aggregator.get_price_statistics(black_box(&feeds)).unwrap())
    });
}

fn create_benchmark_feeds() -> Vec<PriceFeed> {
    let base_price = 43_250.0;
    let sources = vec![
        ("chainlink", 0.3, 0.95),
        ("pyth", 0.25, 0.92),
        ("switchboard", 0.15, 0.90),
        ("binance", 0.2, 0.88),
        ("coingecko", 0.1, 0.85),
    ];

    sources.into_iter().map(|(id, weight, confidence)| {
        use rand::random;
        let variance = 0.999 + (0.002 * random::<f64>());
        let price = base_price * variance;

        PriceFeed {
            source_id: id.to_string(),
            symbol: "BTC".to_string(),
            data: PriceData {
                price: Price::from_float(price, 8).unwrap(),
                volume_24h: (100_000_000.0 * (0.5 + random::<f64>())) as u64 * Price::PRECISION,
                timestamp: Utc::now(),
                confidence,
            },
            weight,
        }
    }).collect()
}

fn extend_feeds(base_feeds: &[PriceFeed], target_count: usize) -> Vec<PriceFeed> {
    let mut feeds = base_feeds.to_vec();

    while feeds.len() < target_count {
        let base_feed = &feeds[feeds.len() % base_feeds.len()];
        let mut new_feed = base_feed.clone();

        // Slightly modify the feed
        let variance = 0.998 + (0.004 * random::<f64>());
        let original_price = base_feed.data.price.to_f64().unwrap();
        new_feed.data.price = Price::from_float(original_price * variance, 8).unwrap();
        new_feed.source_id = format!("source_{}", feeds.len());

        feeds.push(new_feed);
    }

    feeds
}

fn create_outlier_feeds() -> Vec<PriceFeed> {
    let mut feeds = create_benchmark_feeds();

    // Add some outliers
    feeds.push(PriceFeed {
        source_id: "outlier_1".to_string(),
        symbol: "BTC".to_string(),
        data: PriceData {
            price: Price::from_float(50_000.0, 8).unwrap(), // Major outlier
            volume_24h: 10_000_000 * Price::PRECISION,
            timestamp: Utc::now(),
            confidence: 0.5,
        },
        weight: 0.1,
    });

    feeds.push(PriceFeed {
        source_id: "outlier_2".to_string(),
        symbol: "BTC".to_string(),
        data: PriceData {
            price: Price::from_float(38_000.0, 8).unwrap(), // Major outlier
            volume_24h: 5_000_000 * Price::PRECISION,
            timestamp: Utc::now(),
            confidence: 0.4,
        },
        weight: 0.05,
    });

    feeds
}

criterion_group!(
    benches,
    bench_aggregation_methods,
    bench_source_scaling,
    bench_confidence_calculation,
    bench_outlier_detection,
    bench_price_statistics
);
criterion_main!(benches);