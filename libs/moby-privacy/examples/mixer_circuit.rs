use moby_privacy::*;
use moby_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of using the mixer circuit
    println!("Mixer Circuit Example");

    let config = MixerConfig::default();
    let proof_system = Box::new(proofs::MockProofSystem::new());
    let mixer = PrivacyMixer::new(config, proof_system);

    // Submit a test transaction
    let tx_id = mixer.submit_transaction(
        TradeId::new(),
        AccountKey::generate_random(),
        AccountKey::generate_random(),
        WhaleAmount::new(1_000_000),
        commitments::TradeCommitment::mock_commitment(),
        proofs::ZkProof::mock_proof(),
        vec![
            AccountKey::generate_random(),
            AccountKey::generate_random(),
            AccountKey::generate_random(),
        ],
    ).await?;

    println!("Submitted transaction with ID: {}", tx_id);
    println!("Queue size: {}", mixer.queue_size().await);

    Ok(())
}