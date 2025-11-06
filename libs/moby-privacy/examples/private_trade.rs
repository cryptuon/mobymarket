use moby_privacy::*;
use moby_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Private Trade Example");

    // Initialize privacy engine
    let config = engine::PrivacyEngineConfig::default();
    let engine = PrivacyEngine::new(config);

    // Create trader accounts
    let trader_a = AccountKey::generate_random();
    let trader_b = AccountKey::generate_random();

    // Define trade parameters
    let trade_amount = WhaleAmount::new(5_000_000); // $5M trade
    let trade_price = moby_math::Price::new(100, 0);

    println!("Initiating private trade: {} tokens at ${}",
             trade_amount.as_u64(), trade_price.value());

    // Initialize private trade
    let trade_init = engine.initialize_private_trade(
        trader_a,
        trade_amount,
        Some(trader_b),
        PrivacyLevel::Enhanced,
    ).await?;

    println!("Created trade commitment: {}", hex::encode(trade_init.commitment.hash()));

    // Generate zero-knowledge proof for the trade
    let trade_proof = engine.generate_trade_proof(
        &trade_init.commitment,
        trade_amount,
        &trade_init.secret,
    ).await?;

    println!("Generated ZK proof of valid trade");

    // Verify the proof
    let verification = engine.verify_trade_proof(
        &trade_proof,
        &trade_init.commitment,
    ).await?;

    println!("Trade proof verification: {}", verification.is_valid);

    if verification.is_valid {
        println!("✅ Private trade successfully validated!");
    } else {
        println!("❌ Trade verification failed");
    }

    Ok(())
}