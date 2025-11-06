use criterion::{black_box, criterion_group, criterion_main, Criterion};
use moby_privacy::*;
use moby_types::*;

fn benchmark_proof_verification(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Pre-generate proofs for verification benchmarks
    let proof_system = proofs::MockProofSystem::new();
    let public_inputs = vec![vec![1, 2, 3, 4]];
    let private_inputs = vec![vec![5, 6, 7, 8]];

    let zk_proof = rt.block_on(async {
        proof_system.prove("test_circuit", &public_inputs, &private_inputs).await.unwrap()
    });

    let range_proof = rt.block_on(async {
        let config = range_proofs::RangeProofConfig::default();
        let proof_system = Box::new(proofs::MockProofSystem::new());
        let engine = range_proofs::RangeProofEngine::new(config, proof_system);

        let amount = WhaleAmount::new(1_000_000);
        let randomness = engine::TradeSecret::new_random();
        engine.prove_range(amount, 0, 10_000_000, &randomness).await.unwrap()
    });

    c.bench_function("zk_proof_verification", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = proof_system.verify("test_circuit", &public_inputs, &zk_proof).await.unwrap();
                black_box(result);
            });
        });
    });

    c.bench_function("commitment_verification", |b| {
        b.iter(|| {
            let commitment_scheme = commitments::MockCommitmentScheme::new();
            let data = b"test data for commitment";
            let randomness = engine::TradeSecret::new_random();

            let commitment = commitment_scheme.commit(data, &randomness).unwrap();
            let result = commitment_scheme.verify(&commitment, data, &randomness).unwrap();
            black_box(result);
        });
    });

    c.bench_function("range_proof_verification", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = range_proofs::RangeProofConfig::default();
                let proof_system = Box::new(proofs::MockProofSystem::new());
                let engine = range_proofs::RangeProofEngine::new(config, proof_system);

                let verification = engine.verify_range_proof(&range_proof).await.unwrap();
                black_box(verification);
            });
        });
    });

    c.bench_function("nullifier_verification", |b| {
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
                let result = system.verify_nullifier_derivation(&nullifier, &derivation).await.unwrap();
                black_box(result);
            });
        });
    });

    c.bench_function("stealth_payment_detection", |b| {
        b.iter(|| {
            let keypair = stealth::StealthKeyPair::generate();
            let (stealth_address, ephemeral_key) = keypair.derive_stealth_address(
                &keypair.public_spend,
                &keypair.public_view,
            ).unwrap();

            let result = keypair.check_stealth_payment(&stealth_address, &ephemeral_key).unwrap();
            black_box(result);
        });
    });
}

criterion_group!(benches, benchmark_proof_verification);
criterion_main!(benches);