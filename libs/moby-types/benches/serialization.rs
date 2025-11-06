use criterion::{black_box, criterion_group, criterion_main, Criterion};
use moby_types::*;

fn benchmark_serialization(c: &mut Criterion) {
    let amount = WhaleAmount::new(1_000_000);
    let account = AccountKey::generate_random();

    c.bench_function("whale_amount_serialize", |b| {
        b.iter(|| {
            let serialized = borsh::to_vec(&amount).unwrap();
            black_box(serialized);
        })
    });

    c.bench_function("whale_amount_deserialize", |b| {
        let serialized = borsh::to_vec(&amount).unwrap();
        b.iter(|| {
            let deserialized: WhaleAmount = borsh::from_slice(&serialized).unwrap();
            black_box(deserialized);
        })
    });

    c.bench_function("account_key_serialize", |b| {
        b.iter(|| {
            let serialized = borsh::to_vec(&account).unwrap();
            black_box(serialized);
        })
    });
}

criterion_group!(benches, benchmark_serialization);
criterion_main!(benches);