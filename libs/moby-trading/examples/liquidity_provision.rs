use moby_trading::*;
use moby_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Liquidity Provision Example");

    // Create a liquidity manager
    let manager = liquidity::LiquidityManager::new();

    // Create a liquidity provider
    let provider = AccountKey::generate_random();

    // Add liquidity
    let liquidity_id = manager.add_liquidity(
        provider,
        WhaleAmount::new(5_000_000), // $5M liquidity
        moby_math::Price::new(95, 0),  // Lower bound
        moby_math::Price::new(105, 0), // Upper bound
    ).await?;

    println!("Added liquidity with ID: {}", liquidity_id);

    // Check liquidity stats
    let stats = manager.get_liquidity_stats().await;
    println!("Total liquidity: ${}", stats.total_liquidity.as_u64());
    println!("Active providers: {}", stats.active_providers);

    // Simulate some trading activity
    let engine = TradingEngine::new(TradingConfig::default());

    // Create a market order that will consume liquidity
    let market_order = OrderBuilder::new()
        .trader(AccountKey::generate_random())
        .order_type(OrderType::Market)
        .side(OrderSide::Buy)
        .amount(WhaleAmount::new(100_000)) // $100K market buy
        .tier(TradingTier::Standard)
        .privacy_level(PrivacyLevel::Basic)
        .build()?;

    let order_id = engine.submit_order(market_order).await?;
    println!("Submitted market order: {}", order_id);

    Ok(())
}