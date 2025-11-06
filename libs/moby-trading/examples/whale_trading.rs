use moby_trading::*;
use moby_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Whale Trading Example");

    // Create a trading engine
    let engine = TradingEngine::new(TradingConfig::default());

    // Create a whale trader account
    let whale_trader = AccountKey::generate_random();

    // Create a large buy order
    let buy_order = OrderBuilder::new()
        .trader(whale_trader)
        .order_type(OrderType::Limit)
        .side(OrderSide::Buy)
        .amount(WhaleAmount::new(10_000_000)) // $10M order
        .price(moby_math::Price::new(100, 0))
        .tier(TradingTier::Whale)
        .privacy_level(PrivacyLevel::Enhanced)
        .time_in_force(TimeInForce::GoodTillCancelled)
        .build()?;

    println!("Created whale buy order: {} tokens at $100", buy_order.amount.as_u64());

    // Submit the order
    let order_id = engine.submit_order(buy_order).await?;
    println!("Submitted order with ID: {}", order_id);

    // Check order status
    if let Some(order) = engine.get_order(order_id).await {
        println!("Order status: {:?}", order.status);
        println!("Filled amount: {}", order.filled_amount.as_u64());
    }

    // Get trading statistics
    let stats = engine.get_stats().await;
    println!("Total orders processed: {}", stats.total_orders);
    println!("Total volume: ${}", stats.total_volume.as_u64());

    Ok(())
}