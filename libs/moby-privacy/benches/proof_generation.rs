use criterion::{black_box, criterion_group, criterion_main, Criterion};
use moby_privacy::*;
use moby_types::*;

fn benchmark_proof_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("zk_proof_generation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let proof_system = proofs::MockProofSystem::new();
                let public_inputs = vec![vec![1, 2, 3, 4]];
                let private_inputs = vec![vec![5, 6, 7, 8]];

                let proof = proof_system.prove("test_circuit", &public_inputs, &private_inputs).await.unwrap();
                black_box(proof);
            });
        });
    });

    c.bench_function("commitment_generation", |b| {
        b.iter(|| {
            let commitment_scheme = commitments::MockCommitmentScheme::new();
            let data = b"test data for commitment";
            let randomness = engine::TradeSecret::new_random();

            let commitment = commitment_scheme.commit(data, &randomness).unwrap();
            black_box(commitment);
        });
    });

    c.bench_function("nullifier_derivation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = nullifiers::NullifierConfig::default();
                let proof_system = Box::new(proofs::MockProofSystem::new());
                let system = nullifiers::NullifierSystem::new(config, proof_system);

                let derivation = nullifiers::NullifierDerivation {
                    account: AccountKey::generate_random(),
                    secret: engine::TradeSecret::new_random(),
                    trade_id: TradeId::new(),
                    entropy: Some([42u8; 32]),
                    sequence: None,
                };

                let nullifier = system.derive_nullifier(&derivation).await.unwrap();
                black_box(nullifier);
            });
        });
    });

    c.bench_function("range_proof_generation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = range_proofs::RangeProofConfig::default();
                let proof_system = Box::new(proofs::MockProofSystem::new());
                let engine = range_proofs::RangeProofEngine::new(config, proof_system);

                let amount = WhaleAmount::new(1_000_000);
                let randomness = engine::TradeSecret::new_random();

                let proof = engine.prove_range(amount, 0, 10_000_000, &randomness).await.unwrap();
                black_box(proof);
            });
        });
    });

    c.bench_function("stealth_address_generation", |b| {
        b.iter(|| {
            let keypair = stealth::StealthKeyPair::generate();
            let result = keypair.derive_stealth_address(
                &keypair.public_spend,
                &keypair.public_view,
            ).unwrap();
            black_box(result);
        });
    });
}

criterion_group!(benches, benchmark_proof_generation);
criterion_main!(benches);