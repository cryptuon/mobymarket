# OTC Trading Guide

## Overview

The OTC (Over-The-Counter) trading module enables direct peer-to-peer trades between large traders without impacting public market prices. Built with trustless escrow mechanisms and flexible settlement options.

## Core Features

### Trustless Escrow System

All OTC trades are secured through on-chain escrows that eliminate counterparty risk:

- **Atomic Settlement**: Single transaction execution
- **Time-locked Protection**: Automatic expiry and fund return
- **Partial Fill Support**: Configurable minimum fill sizes
- **Multi-asset Swaps**: Support for complex trade structures

### Order Types

#### 1. Fixed Price Orders
```rust
pub struct FixedPriceOrder {
    pub seller: Pubkey,
    pub token_sell: Pubkey,
    pub token_buy: Pubkey,
    pub amount_sell: u64,
    pub amount_buy: u64,
    pub expiry: i64,
    pub partial_fills: bool,
}
```

#### 2. Request for Quote (RFQ)
```rust
pub struct RFQOrder {
    pub requester: Pubkey,
    pub token_desired: Pubkey,
    pub amount_desired: u64,
    pub token_offered: Pubkey,
    pub amount_offered: u64,
    pub quote_deadline: i64,
    pub settlement_deadline: i64,
    pub min_responder_reputation: u32,
}
```

#### 3. Dark Pool Orders
```rust
pub struct DarkPoolOrder {
    pub trader: Pubkey,
    pub commitment_hash: [u8; 32], // Hidden order details
    pub size_range: (u64, u64),    // Min/max size hints
    pub asset_type_hash: [u8; 32], // Hidden asset type
    pub expiry: i64,
    pub reveal_conditions: RevealConditions,
}
```

## Trading Workflow

### 1. Order Creation

#### Fixed Price Order
```typescript
// Create a fixed price OTC order
const orderParams = {
  tokenSell: "So11111111111111111111111111111111111111112", // SOL
  tokenBuy: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",   // USDC
  amountSell: 1000 * LAMPORTS_PER_SOL,
  amountBuy: 150000 * 1e6, // 150,000 USDC
  expiry: Date.now() / 1000 + 86400, // 24 hours
  partialFills: true,
  minFillSize: 100 * LAMPORTS_PER_SOL,
};

const orderId = await createOTCOrder(orderParams);
```

#### RFQ Creation
```typescript
// Request quotes from market makers
const rfqParams = {
  tokenDesired: "So11111111111111111111111111111111111111112",
  amountDesired: 5000 * LAMPORTS_PER_SOL,
  tokenOffered: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  amountOffered: 750000 * 1e6,
  quoteDeadline: Date.now() / 1000 + 1800, // 30 minutes
  settlementDeadline: Date.now() / 1000 + 3600, // 1 hour
  minResponderReputation: 75,
};

const rfqId = await createRFQ(rfqParams);
```

### 2. Order Discovery

#### Public Order Book
- View available fixed price orders
- Filter by asset pair, size, expiry
- Sort by price, reputation, fill history

#### RFQ Marketplace
- Browse active RFQs
- Submit competitive quotes
- Track quote acceptance rates

#### Dark Pool Interface
- Submit hidden orders with size hints
- Match based on compatible parameters
- Progressive reveal mechanism

### 3. Order Matching

#### Manual Matching
Traders can directly accept published orders:

```typescript
// Accept an OTC order
const acceptParams = {
  orderId: "order_123",
  fillAmount: 500 * LAMPORTS_PER_SOL, // Partial fill
};

const txSignature = await acceptOTCOrder(acceptParams);
```

#### Automated Matching
System automatically matches compatible orders:

```rust
pub struct MatchingEngine {
    pub price_tolerance: u16,      // Basis points
    pub size_compatibility: bool,
    pub reputation_weighting: f64,
    pub privacy_requirements: PrivacyLevel,
}

impl MatchingEngine {
    pub fn find_matches(&self, order: &OTCOrder) -> Vec<MatchCandidate> {
        // Intelligent matching algorithm
    }
}
```

## Settlement Mechanisms

### 1. Atomic Swaps
Direct token exchange in single transaction:

```rust
pub fn execute_atomic_swap(
    ctx: Context<ExecuteSwap>,
    order_id: [u8; 32],
) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;

    // Transfer tokens from escrow to respective parties
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_a.to_account_info(),
                to: ctx.accounts.buyer_token_a.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
        ),
        escrow.amount_a,
    )?;

    // Mark as completed
    escrow.escrow_state = EscrowState::Completed;
    Ok(())
}
```

### 2. Progressive Settlement
Multi-stage settlement for large orders:

```rust
pub struct ProgressiveSettlement {
    pub total_stages: u8,
    pub current_stage: u8,
    pub stage_amounts: Vec<u64>,
    pub stage_intervals: Vec<i64>,
    pub completion_requirements: Vec<CompletionCondition>,
}
```

### 3. Cross-Chain Settlement
Settlement across different blockchains:

```rust
pub struct CrossChainSettlement {
    pub source_chain_tx: [u8; 32],
    pub destination_chain: ChainId,
    pub bridge_provider: BridgeProvider,
    pub wormhole_sequence: u64,
    pub timeout_height: u64,
    pub relay_fee: u64,
}
```

## Privacy Features

### Confidential Order Amounts
Hide exact trade sizes using range proofs:

```rust
pub struct ConfidentialOrder {
    pub amount_commitment: [u8; 32],   // Pedersen commitment
    pub range_proof: Vec<u8>,          // Bulletproof for amount range
    pub blinding_factor: [u8; 32],     // Private randomness
}
```

### Identity Shielding
Protect trader identities using stealth addresses:

```rust
pub struct StealthAddress {
    pub view_key: [u8; 32],
    pub spend_key: [u8; 32],
    pub address_commitment: [u8; 32],
}
```

### Order Mixing
Combine multiple orders to obfuscate trading patterns:

```rust
pub struct OrderMixer {
    pub mix_set_size: u32,
    pub anonymity_threshold: u32,
    pub mixing_delay: i64,
    pub participant_commitments: Vec<[u8; 32]>,
}
```

## Risk Management

### Counterparty Risk
- **Escrow Protection**: Funds locked until settlement
- **Reputation System**: Track trader performance history
- **Stake Requirements**: Minimum stake for large orders

### Market Risk
- **Price Oracles**: Real-time price validation
- **Slippage Limits**: Maximum acceptable price deviation
- **Time Limits**: Automatic order expiry

### Operational Risk
- **Multi-sig Controls**: Emergency pause mechanisms
- **Audit Trail**: Complete transaction history
- **Recovery Procedures**: Fund recovery for failed settlements

## Fee Structure

### Base Trading Fees
- **Maker Fee**: 0.05% of trade value
- **Taker Fee**: 0.08% of trade value
- **RFQ Fee**: 0.02% for quote requests
- **Dark Pool Premium**: Additional 0.03%

### Volume Discounts
Progressive fee reduction based on 30-day volume:

| Volume Tier | Maker Fee | Taker Fee |
|-------------|-----------|-----------|
| $0 - $1M    | 0.05%     | 0.08%     |
| $1M - $10M  | 0.04%     | 0.07%     |
| $10M - $50M | 0.03%     | 0.06%     |
| $50M+       | 0.02%     | 0.05%     |

### Privacy Fees
- **ZK Proof Generation**: 0.1 SOL per proof
- **Privacy Pool Entry**: 0.01% of deposit amount
- **Stealth Address**: 0.05 SOL setup fee

## API Reference

### Core Functions

```typescript
interface OTCTrading {
  // Order Management
  createFixedPriceOrder(params: FixedPriceParams): Promise<OrderId>;
  createRFQ(params: RFQParams): Promise<RFQId>;
  createDarkPoolOrder(params: DarkPoolParams): Promise<OrderId>;

  // Quote Management
  submitQuote(rfqId: RFQId, quote: Quote): Promise<QuoteId>;
  acceptQuote(quoteId: QuoteId): Promise<TxSignature>;

  // Order Execution
  acceptOrder(orderId: OrderId, amount?: bigint): Promise<TxSignature>;
  cancelOrder(orderId: OrderId): Promise<TxSignature>;

  // Information
  getOrder(orderId: OrderId): Promise<OrderInfo>;
  getOrderBook(pair: TokenPair): Promise<OrderBook>;
  getTradingHistory(trader: PublicKey): Promise<TradeHistory>;
}
```

### Event Types

```typescript
type OTCEvent =
  | { type: 'OrderCreated'; order: OrderInfo }
  | { type: 'OrderMatched'; match: MatchInfo }
  | { type: 'OrderFilled'; fill: FillInfo }
  | { type: 'OrderCancelled'; orderId: OrderId }
  | { type: 'QuoteSubmitted'; quote: QuoteInfo }
  | { type: 'SettlementCompleted'; settlement: SettlementInfo };
```

## Best Practices

### For Makers (Order Creators)
1. **Competitive Pricing**: Monitor market rates regularly
2. **Appropriate Sizing**: Consider market liquidity when sizing orders
3. **Time Management**: Set realistic expiry times
4. **Privacy Considerations**: Use confidential orders for sensitive trades

### For Takers (Order Acceptors)
1. **Due Diligence**: Verify counterparty reputation
2. **Price Validation**: Cross-check with market prices
3. **Timing**: Execute during optimal market conditions
4. **Risk Management**: Don't exceed position limits

### For Market Makers
1. **Liquidity Provision**: Maintain two-sided markets
2. **Risk Management**: Implement proper hedging strategies
3. **Response Time**: Quick RFQ response for better reputation
4. **Technology**: Use automated systems for efficiency

## Troubleshooting

### Common Issues

#### Order Not Filling
- Check price competitiveness
- Verify sufficient counterparty interest
- Consider adjusting expiry time

#### High Slippage
- Use smaller order sizes
- Implement TWAP/VWAP strategies
- Consider dark pool execution

#### Settlement Delays
- Monitor cross-chain bridge status
- Verify gas/compute availability
- Check oracle feed health

#### Privacy Concerns
- Ensure proper ZK proof generation
- Verify mixer participation
- Use fresh stealth addresses