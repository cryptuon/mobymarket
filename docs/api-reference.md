# API Reference

## Overview

Moby Market provides comprehensive APIs for institutional and programmatic trading. The API supports REST endpoints, WebSocket connections, and direct Solana program interfaces.

## Authentication

### API Key Authentication
```typescript
interface APICredentials {
  apiKey: string;
  secretKey: string;
  passphrase?: string;
}

// Initialize client
const client = new MobyMarketClient({
  endpoint: 'https://api.moby-market.com',
  credentials: {
    apiKey: 'your-api-key',
    secretKey: 'your-secret-key',
    passphrase: 'your-passphrase'
  }
});
```

### Wallet Authentication
```typescript
// Using wallet adapter
import { useWallet } from '@solana/wallet-adapter-react';

const { publicKey, signMessage } = useWallet();
const client = new MobyMarketClient({
  endpoint: 'https://api.moby-market.com',
  wallet: { publicKey, signMessage }
});
```

## OTC Trading API

### Order Management

#### Create OTC Order
```typescript
interface OTCOrderParams {
  tokenSell: string;           // Token mint address to sell
  tokenBuy: string;            // Token mint address to buy
  amountSell: bigint;          // Amount to sell (in token units)
  amountBuy: bigint;           // Amount to buy (in token units)
  expiry: number;              // Unix timestamp
  partialFills: boolean;       // Allow partial fills
  minFillSize?: bigint;        // Minimum fill size
  privacyLevel?: 'public' | 'confidential' | 'anonymous';
}

async function createOTCOrder(params: OTCOrderParams): Promise<OrderResponse> {
  const response = await client.post('/otc/orders', params);
  return response.data;
}
```

#### Accept OTC Order
```typescript
interface AcceptOrderParams {
  orderId: string;
  fillAmount?: bigint;         // For partial fills
  slippageTolerance?: number;  // Basis points
}

async function acceptOTCOrder(params: AcceptOrderParams): Promise<TransactionResponse> {
  const response = await client.post(`/otc/orders/${params.orderId}/accept`, params);
  return response.data;
}
```

#### Get Order Book
```typescript
interface OrderBookParams {
  tokenA: string;
  tokenB: string;
  limit?: number;              // Default 100, max 1000
  includePrivate?: boolean;    // Include dark pool orders (if authorized)
}

async function getOrderBook(params: OrderBookParams): Promise<OrderBook> {
  const response = await client.get('/otc/orderbook', { params });
  return response.data;
}
```

### RFQ (Request for Quote)

#### Create RFQ
```typescript
interface RFQParams {
  tokenDesired: string;
  amountDesired: bigint;
  tokenOffered: string;
  amountOffered: bigint;
  quoteDeadline: number;       // Unix timestamp
  settlementDeadline: number;  // Unix timestamp
  minResponderReputation?: number; // 0-100
  maxResponders?: number;      // Limit number of quotes
}

async function createRFQ(params: RFQParams): Promise<RFQResponse> {
  const response = await client.post('/rfq/create', params);
  return response.data;
}
```

#### Submit Quote
```typescript
interface QuoteParams {
  rfqId: string;
  price: number;               // Price per unit
  amount: bigint;              // Quote amount
  validUntil: number;          // Unix timestamp
  partialFill: boolean;
}

async function submitQuote(params: QuoteParams): Promise<QuoteResponse> {
  const response = await client.post('/rfq/quote', params);
  return response.data;
}
```

## Execution Algorithms API

### TWAP Orders

#### Submit TWAP Order
```typescript
interface TWAPParams {
  tokenIn: string;
  tokenOut: string;
  totalAmount: bigint;
  timeWindow: number;          // Seconds
  numSplits: number;           // Number of execution intervals
  randomnessFactor: number;    // 0-100, timing randomization
  priceDeviationLimit: number; // Basis points
  minOutputPerInterval: bigint;
  venuePreferences?: VenuePreference[];
  adaptiveSlippage?: boolean;
}

interface VenuePreference {
  venue: string;               // Venue identifier
  weight: number;              // 0-100, allocation weight
}

async function submitTWAPOrder(params: TWAPParams): Promise<AlgorithmOrderResponse> {
  const response = await client.post('/execution/twap', params);
  return response.data;
}
```

#### Get TWAP Status
```typescript
async function getTWAPStatus(orderId: string): Promise<TWAPStatus> {
  const response = await client.get(`/execution/twap/${orderId}`);
  return response.data;
}

interface TWAPStatus {
  orderId: string;
  status: 'pending' | 'executing' | 'completed' | 'cancelled';
  totalAmount: bigint;
  executedAmount: bigint;
  averageExecutionPrice: number;
  remainingSplits: number;
  nextExecutionTime: number;
  venueBreakdown: VenueExecution[];
}
```

### VWAP Orders

#### Submit VWAP Order
```typescript
interface VWAPParams {
  tokenPair: TokenPair;
  totalAmount: bigint;
  targetVolumeParticipation: number; // Basis points of market volume
  maxSpreadTolerance: number;        // Basis points
  adaptiveParameters: AdaptiveParams;
  volumeCurve?: 'linear' | 'exponential' | 'sigmoid';
}

interface AdaptiveParams {
  marketImpactThreshold: number;     // Max acceptable market impact
  rebalanceFrequency: number;        // Seconds between rebalancing
  liquiditySeekingMode: boolean;     // Prefer high-liquidity venues
}

async function submitVWAPOrder(params: VWAPParams): Promise<AlgorithmOrderResponse> {
  const response = await client.post('/execution/vwap', params);
  return response.data;
}
```

### Intent-Based Trading

#### Submit Trading Intent
```typescript
interface TradingIntent {
  intentType: IntentType;
  constraints: Constraint[];
  solverCompetitionPeriod: number;   // Seconds
  maxSolverReward: bigint;
  privacyRequirements?: PrivacyRequirements;
  deadline: number;                  // Unix timestamp
}

type IntentType =
  | { type: 'BestExecution'; tokenIn: string; tokenOut: string; amount: bigint; minOutput: bigint }
  | { type: 'LiquidityProvision'; pool: string; range: PriceRange; amount: bigint }
  | { type: 'Arbitrage'; paths: TradePath[]; minProfit: bigint; maxCapital: bigint }
  | { type: 'Portfolio'; targetAllocations: [string, number][]; rebalanceThreshold: number };

async function submitIntent(intent: TradingIntent): Promise<IntentResponse> {
  const response = await client.post('/intent/submit', intent);
  return response.data;
}
```

## Privacy API

### Zero-Knowledge Proofs

#### Generate Proof
```typescript
interface ProofRequest {
  circuitType: 'transaction' | 'range' | 'compliance' | 'custom';
  publicInputs: any[];
  privateWitnesses: any[];
  provingKey?: string;         // Optional custom proving key
}

async function generateProof(request: ProofRequest): Promise<ProofResponse> {
  const response = await client.post('/privacy/proof/generate', request);
  return response.data;
}
```

#### Verify Proof
```typescript
interface VerificationRequest {
  proof: string;               // Base64 encoded proof
  verificationKey: string;     // Verification key identifier
  publicInputs: any[];
}

async function verifyProof(request: VerificationRequest): Promise<VerificationResponse> {
  const response = await client.post('/privacy/proof/verify', request);
  return response.data;
}
```

### Confidential Transfers

#### Create Confidential Transfer
```typescript
interface ConfidentialTransferParams {
  amount: bigint;
  recipient: string;           // Recipient public key
  asset: string;               // Token mint
  privacyLevel: 'amount' | 'asset' | 'both';
}

async function createConfidentialTransfer(
  params: ConfidentialTransferParams
): Promise<ConfidentialTransferResponse> {
  const response = await client.post('/privacy/confidential-transfer', params);
  return response.data;
}
```

### Privacy Pools

#### Deposit to Pool
```typescript
interface PoolDepositParams {
  poolId: string;
  amount: bigint;
  anonymitySetTarget?: number; // Wait for this many participants
}

async function depositToPool(params: PoolDepositParams): Promise<DepositResponse> {
  const response = await client.post('/privacy/pool/deposit', params);
  return response.data;
}
```

#### Withdraw from Pool
```typescript
interface PoolWithdrawalParams {
  poolId: string;
  depositReceipt: DepositReceipt;
  recipient: string;
  amount?: bigint;             // For partial withdrawals
}

async function withdrawFromPool(params: PoolWithdrawalParams): Promise<WithdrawalResponse> {
  const response = await client.post('/privacy/pool/withdraw', params);
  return response.data;
}
```

## Market Data API

### Price Feeds

#### Get Current Prices
```typescript
interface PriceRequest {
  tokens: string[];            // Array of token mints
  quoteCurrency?: string;      // Default: USDC
  includeTWAP?: boolean;       // Include time-weighted prices
}

async function getCurrentPrices(request: PriceRequest): Promise<PriceResponse> {
  const response = await client.get('/market-data/prices', { params: request });
  return response.data;
}
```

#### Get Historical Prices
```typescript
interface HistoricalPriceRequest {
  token: string;
  quoteCurrency?: string;
  interval: '1m' | '5m' | '15m' | '1h' | '4h' | '1d';
  startTime: number;           // Unix timestamp
  endTime: number;             // Unix timestamp
  limit?: number;              // Max 1000
}

async function getHistoricalPrices(
  request: HistoricalPriceRequest
): Promise<HistoricalPriceResponse> {
  const response = await client.get('/market-data/historical', { params: request });
  return response.data;
}
```

### Liquidity Data

#### Get Liquidity Depth
```typescript
interface LiquidityRequest {
  tokenPair: TokenPair;
  venues?: string[];           // Specific venues to query
  depth?: number;              // Price levels to include
}

async function getLiquidityDepth(request: LiquidityRequest): Promise<LiquidityResponse> {
  const response = await client.get('/market-data/liquidity', { params: request });
  return response.data;
}
```

## WebSocket API

### Connection
```typescript
import WebSocket from 'ws';

const ws = new WebSocket('wss://api.moby-market.com/ws');

// Authentication message
ws.send(JSON.stringify({
  type: 'auth',
  apiKey: 'your-api-key',
  signature: 'signed-timestamp',
  timestamp: Date.now()
}));
```

### Subscriptions

#### Order Updates
```typescript
// Subscribe to order updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'orders',
  params: {
    userId: 'your-user-id'
  }
}));

// Message format
interface OrderUpdateMessage {
  type: 'order_update';
  data: {
    orderId: string;
    status: 'created' | 'partial' | 'filled' | 'cancelled';
    executedAmount: bigint;
    averagePrice: number;
    remainingAmount: bigint;
    timestamp: number;
  };
}
```

#### Market Data
```typescript
// Subscribe to price updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'prices',
  params: {
    tokens: ['SOL', 'ETH', 'BTC'],
    quoteCurrency: 'USDC'
  }
}));

// Message format
interface PriceUpdateMessage {
  type: 'price_update';
  data: {
    token: string;
    price: number;
    change24h: number;
    volume24h: bigint;
    timestamp: number;
  };
}
```

#### Execution Algorithm Updates
```typescript
// Subscribe to TWAP/VWAP execution updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'algorithm_execution',
  params: {
    orderId: 'your-order-id'
  }
}));

// Message format
interface AlgorithmUpdateMessage {
  type: 'algorithm_update';
  data: {
    orderId: string;
    algorithmType: 'TWAP' | 'VWAP';
    currentSplit: number;
    totalSplits: number;
    executedAmount: bigint;
    averagePrice: number;
    nextExecutionTime: number;
  };
}
```

## Direct Program Interface

### Solana Program Instructions

#### OTC Order Creation
```rust
pub fn create_otc_order(
    ctx: Context<CreateOTCOrder>,
    token_sell: Pubkey,
    token_buy: Pubkey,
    amount_sell: u64,
    amount_buy: u64,
    expiry: i64,
    partial_fills: bool,
) -> Result<()> {
    // Implementation
}
```

#### TWAP Order Execution
```rust
pub fn execute_twap_interval(
    ctx: Context<ExecuteTWAP>,
    order_id: [u8; 32],
    execution_params: ExecutionParams,
) -> Result<()> {
    // Implementation
}
```

#### Privacy Pool Deposit
```rust
pub fn deposit_to_privacy_pool(
    ctx: Context<PrivacyPoolDeposit>,
    pool_id: Pubkey,
    commitment: [u8; 32],
    amount: u64,
) -> Result<()> {
    // Implementation
}
```

## Error Handling

### Error Response Format
```typescript
interface APIError {
  code: number;
  message: string;
  details?: any;
  timestamp: number;
}

// Example error codes
enum ErrorCode {
  INVALID_SIGNATURE = 1001,
  INSUFFICIENT_BALANCE = 1002,
  ORDER_NOT_FOUND = 1003,
  MARKET_CLOSED = 1004,
  RATE_LIMITED = 1005,
  PRIVACY_PROOF_INVALID = 2001,
  COMPLIANCE_CHECK_FAILED = 2002,
  ZK_CIRCUIT_ERROR = 2003,
}
```

### Error Handling Example
```typescript
try {
  const order = await createOTCOrder(params);
} catch (error) {
  if (error.response?.data?.code === ErrorCode.INSUFFICIENT_BALANCE) {
    console.error('Insufficient balance for order');
  } else if (error.response?.data?.code === ErrorCode.RATE_LIMITED) {
    console.error('Rate limited, try again later');
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

## Rate Limits

### API Rate Limits
- **Public endpoints**: 1000 requests/minute
- **Private endpoints**: 5000 requests/minute
- **WebSocket connections**: 10 connections per API key
- **Order placement**: 100 orders/minute

### Headers
```typescript
// Rate limit headers in responses
{
  'X-RateLimit-Limit': '1000',
  'X-RateLimit-Remaining': '995',
  'X-RateLimit-Reset': '1640995200'
}
```

## SDK Examples

### TypeScript SDK
```typescript
import { MobyMarketSDK } from '@moby-market/sdk';

const sdk = new MobyMarketSDK({
  apiKey: process.env.MOBY_API_KEY,
  secretKey: process.env.MOBY_SECRET_KEY,
  environment: 'mainnet' // or 'devnet'
});

// Create OTC order
const order = await sdk.otc.createOrder({
  tokenSell: 'SOL',
  tokenBuy: 'USDC',
  amountSell: 1000,
  amountBuy: 150000,
  expiry: Date.now() + 86400000 // 24 hours
});

// Submit TWAP order
const twapOrder = await sdk.execution.twap({
  tokenIn: 'SOL',
  tokenOut: 'USDC',
  totalAmount: 10000,
  timeWindow: 3600, // 1 hour
  numSplits: 12     // Execute every 5 minutes
});
```

### Python SDK
```python
from moby_market import MobyMarketClient

client = MobyMarketClient(
    api_key=os.environ['MOBY_API_KEY'],
    secret_key=os.environ['MOBY_SECRET_KEY'],
    environment='mainnet'
)

# Create OTC order
order = client.otc.create_order(
    token_sell='SOL',
    token_buy='USDC',
    amount_sell=1000,
    amount_buy=150000,
    expiry=int(time.time()) + 86400
)

# Submit VWAP order
vwap_order = client.execution.vwap(
    token_pair=('SOL', 'USDC'),
    total_amount=5000,
    target_volume_participation=500  # 5% of market volume
)
```

## Testing

### Testnet Endpoints
- **REST API**: `https://api-devnet.moby-market.com`
- **WebSocket**: `wss://api-devnet.moby-market.com/ws`
- **Solana RPC**: `https://api.devnet.solana.com`

### Mock Data
```typescript
// Enable mock mode for testing
const client = new MobyMarketClient({
  endpoint: 'https://api-devnet.moby-market.com',
  mockMode: true,
  credentials: testCredentials
});
```