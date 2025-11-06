use moby_privacy::*;
use moby_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of using privacy pools
    println!("Privacy Pool Example");

    let config = pools::PoolConfig::default();
    let proof_system = Box::new(proofs::MockProofSystem::new());
    let commitment_scheme = Box::new(commitments::MockCommitmentScheme::new());

    let pool = pools::PrivacyPool::new(config, proof_system, commitment_scheme)?;

    // Make a deposit
    let depositor = AccountKey::generate_random();
    let amount = WhaleAmount::new(1_000_000);
    let secret = engine::TradeSecret::new_random();

    let deposit_id = pool.deposit(
        depositor,
        amount,
        secret,
        TradeId::new(),
    ).await?;

    println!("Made deposit with ID: {}", deposit_id);

    // Check pool state
    let state = pool.get_state().await;
    println!("Pool state: {} deposits, anonymity set size: {}",
             state.total_deposits,
             state.anonymity_set_size);

    Ok(())
}