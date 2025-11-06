use moby_privacy::*;
use moby_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of using range proofs
    println!("Range Proof Example");

    let config = range_proofs::RangeProofConfig::default();
    let proof_system = Box::new(proofs::MockProofSystem::new());
    let engine = range_proofs::RangeProofEngine::new(config, proof_system);

    let amount = WhaleAmount::new(5_000_000);
    let randomness = engine::TradeSecret::new_random();

    // Prove the amount is in range [1M, 10M]
    let proof = engine.prove_range(
        amount,
        1_000_000,
        10_000_000,
        &randomness,
    ).await?;

    println!("Generated range proof for amount in [1M, 10M]");

    // Verify the proof
    let verification = engine.verify_range_proof(&proof).await?;

    println!("Proof verification result: {}", verification.is_valid);

    Ok(())
}