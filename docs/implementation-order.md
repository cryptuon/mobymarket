# Implementation Order & Dependencies

> **Step-by-step guide for building whale trading infrastructure from the ground up**

## Implementation Philosophy

### Build Order Strategy
1. **Foundation First** - Core mathematical and state management libraries
2. **Programs in Dependency Order** - Controller → Oracle → OTC → Executor → Privacy
3. **Test-Driven Development** - Each component fully tested before building dependents
4. **Incremental Integration** - Add one component at a time with full integration testing
5. **Parallel Development** - Independent teams can work on different layers simultaneously

### Dependency Graph
```
┌─ Core Libraries ────────────────────────────────┐
│  ├─ moby-math (price, slippage calculations)   │
│  ├─ moby-state (account management)            │
│  ├─ moby-utils (common utilities)              │
│  └─ moby-errors (error definitions)            │
└─────────────────────────────────────────────────┘
                    ↓
┌─ Controller Program ─────────────────────────────┐
│  ├─ Global state management                     │
│  ├─ Emergency controls                          │
│  ├─ Program coordination                        │
│  └─ Multi-sig governance                        │
└─────────────────────────────────────────────────┘
                    ↓
┌─ Oracle Program ─────────────────────────────────┐
│  ├─ Price feed aggregation                      │
│  ├─ Manipulation detection                      │
│  ├─ Multi-source validation                     │
│  └─ Emergency price fallbacks                   │
└─────────────────────────────────────────────────┘
                    ↓
┌─ OTC Trading Program ────────────────────────────┐
│  ├─ Fixed price orders                          │
│  ├─ RFQ system                                  │
│  ├─ Escrow management                           │
│  └─ Settlement logic                            │
└─────────────────────────────────────────────────┘
                    ↓
┌─ Algorithm Execution Program ────────────────────┐
│  ├─ TWAP implementation                         │
│  ├─ VWAP implementation                         │
│  ├─ Smart routing                               │
│  └─ Intent processing                           │
└─────────────────────────────────────────────────┘
                    ↓
┌─ Privacy Program ────────────────────────────────┐
│  ├─ ZK proof verification                       │
│  ├─ Privacy pools                               │
│  ├─ Commitment schemes                          │
│  └─ Selective disclosure                        │
└─────────────────────────────────────────────────┘
```

---

## Phase 1: Core Libraries (Weeks 1-3)

### Week 1: Mathematical Foundation Libraries

#### 1.1 Price Library (`libs/moby-math/src/price.rs`)
```rust
// Implementation priority: CRITICAL - All pricing depends on this

use anchor_lang::prelude::*;

/// High-precision price representation for whale-scale trading
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct Price {
    pub value: u64,        // Price in micro-units (6 decimals)
    pub decimals: u8,      // Number of decimal places
    pub timestamp: i64,    // When price was set
}

impl Price {
    pub const PRECISION: u64 = 1_000_000; // 6 decimal places

    /// Create price from floating point value
    pub fn from_float(price: f64, decimals: u8) -> Result<Self> {
        // Implementation details in core-libraries.md
    }

    /// Multiply amount by price (amount * price)
    pub fn multiply_amount(&self, amount: u64) -> Result<u64> {
        // Checked math to prevent overflow
    }

    /// Divide amount by price (amount / price)
    pub fn divide_amount(&self, amount: u64) -> Result<u64> {
        // Safe division with zero-check
    }
}

// TESTS REQUIRED:
// ✅ Price creation from various float values
// ✅ Whale-sized calculation (>$100M trades)
// ✅ Overflow protection
// ✅ Precision limits
// ✅ Edge cases (zero, max values)
```

#### 1.2 Slippage Library (`libs/moby-math/src/slippage.rs`)
```rust
// Implementation priority: CRITICAL - All trade validation depends on this

/// Calculate slippage between expected and actual execution
pub struct SlippageCalculator;

impl SlippageCalculator {
    /// Calculate slippage in basis points
    pub fn calculate_slippage(expected: u64, actual: u64) -> Result<u16> {
        // Returns basis points (100 = 1%)
    }

    /// Calculate price impact for AMM pools
    pub fn calculate_price_impact(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> Result<u16> {
        // Constant product formula: x * y = k
    }

    /// Apply slippage tolerance to amount
    pub fn apply_slippage_tolerance(amount: u64, tolerance_bps: u16) -> Result<u64> {
        // Returns minimum acceptable amount
    }
}

// TESTS REQUIRED:
// ✅ Precise slippage calculations
// ✅ Whale-sized slippage scenarios
// ✅ AMM price impact calculations
// ✅ Slippage tolerance application
// ✅ Property-based testing for mathematical invariants
```

#### 1.3 State Management Library (`libs/moby-state/src/lib.rs`)
```rust
// Implementation priority: HIGH - All programs use this

use anchor_lang::prelude::*;

/// Account validation utilities
pub trait AccountValidation {
    fn validate_signature(&self, signer: &Pubkey) -> Result<()>;
    fn validate_ownership(&self, owner: &Pubkey) -> Result<()>;
    fn validate_not_expired(&self, current_time: i64) -> Result<()>;
}

/// Safe account initialization
pub trait SafeInit {
    fn safe_init(&mut self, params: Self::InitParams) -> Result<()>;
    type InitParams;
}

/// Account size calculation helpers
pub trait InitSpace {
    const INIT_SPACE: usize;
    fn calculate_space(params: &Self::InitParams) -> usize;
    type InitParams;
}

// IMPLEMENTATION ORDER:
// 1. Basic validation traits
// 2. Common account patterns
// 3. Integration with Anchor macros
// 4. Comprehensive test suite
```

### Week 2: Error Handling & Utilities

#### 2.1 Error System (`libs/moby-errors/src/lib.rs`)
```rust
// Implementation priority: HIGH - All error handling depends on this

use anchor_lang::prelude::*;

#[error_code]
pub enum MobyError {
    // Math errors (codes 1000-1099)
    #[msg("Mathematical overflow detected")]
    MathOverflow = 1000,
    #[msg("Division by zero")]
    DivisionByZero = 1001,
    #[msg("Price calculation failed")]
    PriceCalculationFailed = 1002,

    // Trading errors (codes 1100-1199)
    #[msg("Order not found")]
    OrderNotFound = 1100,
    #[msg("Insufficient balance")]
    InsufficientBalance = 1101,
    #[msg("Order expired")]
    OrderExpired = 1102,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded = 1103,

    // Oracle errors (codes 1200-1299)
    #[msg("Price feed stale")]
    StalePriceFeed = 1200,
    #[msg("Oracle manipulation detected")]
    OracleManipulation = 1201,
    #[msg("Insufficient oracle sources")]
    InsufficientOracles = 1202,

    // Privacy errors (codes 1300-1399)
    #[msg("Invalid ZK proof")]
    InvalidZKProof = 1300,
    #[msg("Nullifier already used")]
    NullifierUsed = 1301,
    #[msg("Privacy pool full")]
    PoolFull = 1302,

    // System errors (codes 1400-1499)
    #[msg("System paused")]
    SystemPaused = 1400,
    #[msg("Unauthorized access")]
    Unauthorized = 1401,
    #[msg("Emergency mode active")]
    EmergencyMode = 1402,
}
```

#### 2.2 Utility Functions (`libs/moby-utils/src/lib.rs`)
```rust
// Implementation priority: MEDIUM - Nice to have early

/// Time utilities for trading
pub mod time {
    use anchor_lang::prelude::*;

    pub fn get_current_timestamp() -> i64 {
        Clock::get().unwrap().unix_timestamp
    }

    pub fn is_expired(expiry: i64) -> bool {
        get_current_timestamp() > expiry
    }

    pub fn seconds_until_expiry(expiry: i64) -> i64 {
        expiry - get_current_timestamp()
    }
}

/// Validation helpers
pub mod validation {
    use anchor_lang::prelude::*;

    pub fn validate_amount(amount: u64) -> Result<()> {
        require!(amount > 0, crate::MobyError::InvalidAmount);
        Ok(())
    }

    pub fn validate_slippage_tolerance(tolerance_bps: u16) -> Result<()> {
        require!(tolerance_bps <= 10000, crate::MobyError::InvalidSlippage);
        Ok(())
    }
}
```

### Week 3: Testing Infrastructure

#### 3.1 Test Utilities (`libs/moby-test-utils/src/lib.rs`)
```rust
// Implementation priority: CRITICAL - All subsequent testing depends on this

use anchor_client::{Client, Cluster};
use solana_sdk::signature::Keypair;

/// Test environment setup
pub struct TestEnvironment {
    pub client: Client,
    pub payer: Keypair,
    pub test_accounts: Vec<Keypair>,
}

impl TestEnvironment {
    pub async fn new() -> Self {
        // Setup test validator and accounts
    }

    pub async fn create_test_token(&self, decimals: u8) -> Pubkey {
        // Create SPL token for testing
    }

    pub async fn fund_account(&self, account: &Pubkey, amount: u64) {
        // Fund account with test tokens
    }
}

/// Property-based testing helpers
pub mod property_testing {
    use proptest::prelude::*;

    pub fn price_strategy() -> impl Strategy<Value = f64> {
        (0.01f64..1_000_000.0f64)
    }

    pub fn whale_amount_strategy() -> impl Strategy<Value = u64> {
        (1_000_000u64..1_000_000_000_000u64) // $1M to $1T
    }

    pub fn slippage_strategy() -> impl Strategy<Value = u16> {
        (0u16..1000u16) // 0% to 10%
    }
}
```

### Success Criteria for Phase 1:
- [ ] All core math functions implemented and tested
- [ ] Property-based tests pass for mathematical invariants
- [ ] Whale-scale calculations (>$100M) work correctly
- [ ] Error handling system covers all major error cases
- [ ] Test utilities enable comprehensive program testing
- [ ] Code coverage >95% for all core libraries
- [ ] Performance benchmarks meet targets (<1ms for math operations)

---

## Phase 2: Controller Program (Weeks 4-5)

### Week 4: Core Controller Implementation

#### 4.1 Program Structure (`programs/moby-controller/src/lib.rs`)
```rust
// Implementation priority: CRITICAL - Foundation for all other programs

use anchor_lang::prelude::*;
use moby_state::*;
use moby_errors::*;

declare_id!("MobyCtrl1111111111111111111111111111111111");

#[program]
pub mod moby_controller {
    use super::*;

    /// Initialize the entire Moby Market system
    pub fn initialize_system(
        ctx: Context<InitializeSystem>,
        params: InitializeParams,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;

        global_state.authority = ctx.accounts.authority.key();
        global_state.paused = false;
        global_state.emergency_mode = false;
        global_state.supported_tokens = params.supported_tokens;
        global_state.fee_collector = params.fee_collector;
        global_state.created_at = get_current_timestamp();

        emit!(SystemInitialized {
            authority: global_state.authority,
            timestamp: global_state.created_at,
        });

        Ok(())
    }

    /// Emergency pause all operations
    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;

        require!(!global_state.paused, MobyError::SystemPaused);

        global_state.paused = true;
        global_state.emergency_mode = true;

        emit!(EmergencyPauseActivated {
            triggered_by: ctx.accounts.authority.key(),
            timestamp: get_current_timestamp(),
        });

        Ok(())
    }
}

// IMPLEMENTATION ORDER:
// 1. Basic initialization (Day 1)
// 2. Emergency controls (Day 2)
// 3. Multi-sig governance (Day 3)
// 4. Program coordination (Day 4)
// 5. Comprehensive testing (Day 5)
```

#### 4.2 Account Structures (`programs/moby-controller/src/state.rs`)
```rust
use anchor_lang::prelude::*;
use moby_state::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub authority: Pubkey,
    pub paused: bool,
    pub emergency_mode: bool,
    pub total_volume_traded: u64,
    pub total_fees_collected: u64,
    #[max_len(50)]
    pub supported_tokens: Vec<Pubkey>,
    pub fee_collector: Pubkey,
    pub upgrade_authority: Pubkey,
    pub created_at: i64,
    pub last_updated: i64,
}

impl SafeInit for GlobalState {
    type InitParams = InitializeParams;

    fn safe_init(&mut self, params: Self::InitParams) -> Result<()> {
        self.authority = params.authority;
        self.paused = false;
        self.emergency_mode = false;
        self.supported_tokens = params.supported_tokens;
        self.fee_collector = params.fee_collector;
        self.upgrade_authority = params.upgrade_authority;
        self.created_at = get_current_timestamp();
        self.last_updated = self.created_at;
        Ok(())
    }
}
```

### Week 5: Multi-Sig Governance & Testing

#### 5.1 Multi-Sig Implementation
```rust
#[account]
#[derive(InitSpace)]
pub struct MultiSigProposal {
    pub proposal_id: [u8; 32],
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub votes_for: u8,
    pub votes_against: u8,
    pub required_votes: u8,
    pub expiry: i64,
    pub executed: bool,
    #[max_len(9)]
    pub signers: Vec<Pubkey>,
    #[max_len(9)]
    pub voted: Vec<bool>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ProposalType {
    UpgradeProgram { new_program_id: Pubkey },
    ChangeFeeCollector { new_collector: Pubkey },
    AddSupportedToken { token_mint: Pubkey },
    EmergencyPause,
    EmergencyUnpause,
}
```

#### 5.2 Integration Tests
```rust
#[tokio::test]
async fn test_system_initialization() {
    let env = TestEnvironment::new().await;

    let params = InitializeParams {
        authority: env.payer.pubkey(),
        supported_tokens: vec![get_sol_mint(), get_usdc_mint()],
        fee_collector: env.payer.pubkey(),
        upgrade_authority: env.payer.pubkey(),
    };

    let result = initialize_system(&env.client, &env.payer, params).await;
    assert!(result.is_ok());

    let global_state: GlobalState = env.client.account(get_global_state_pda()).await.unwrap();
    assert_eq!(global_state.authority, env.payer.pubkey());
    assert!(!global_state.paused);
}

#[tokio::test]
async fn test_emergency_pause_access_control() {
    let env = TestEnvironment::new().await;
    let unauthorized = Keypair::new();

    // Should fail with unauthorized user
    let result = emergency_pause(&env.client, &unauthorized).await;
    assert!(result.is_err());

    // Should succeed with authorized user
    let result = emergency_pause(&env.client, &env.payer).await;
    assert!(result.is_ok());
}
```

### Success Criteria for Phase 2:
- [ ] Controller program deploys successfully
- [ ] All governance functions work correctly
- [ ] Emergency controls function properly
- [ ] Multi-sig voting mechanism operational
- [ ] Access control prevents unauthorized actions
- [ ] Program can coordinate with other programs
- [ ] All integration tests pass
- [ ] Gas usage optimized (<100k CU per instruction)

---

## Phase 3: Oracle Program (Weeks 6-7)

### Week 6: Oracle Implementation

#### 6.1 Basic Oracle Structure
```rust
// Implementation priority: HIGH - Required for all price-dependent operations

use anchor_lang::prelude::*;
use moby_math::Price;

declare_id!("MobyOrcl1111111111111111111111111111111111");

#[program]
pub mod moby_oracle {
    use super::*;

    pub fn initialize_oracle_aggregator(
        ctx: Context<InitializeOracle>,
        params: InitializeOracleParams,
    ) -> Result<()> {
        let oracle = &mut ctx.accounts.oracle_aggregator;

        require!(params.oracle_sources.len() >= 2, MobyError::InsufficientOracles);

        oracle.token_mint = params.token_mint;
        oracle.oracle_sources = params.oracle_sources;
        oracle.aggregation_method = params.aggregation_method;
        oracle.deviation_threshold_bps = params.deviation_threshold_bps;
        oracle.min_sources_required = params.min_sources_required;

        Ok(())
    }

    pub fn update_price_feed(
        ctx: Context<UpdatePriceFeed>,
    ) -> Result<()> {
        let oracle = &mut ctx.accounts.oracle_aggregator;

        // Collect prices from all sources
        let prices = collect_oracle_prices(&ctx.accounts)?;

        // Filter outliers
        let filtered_prices = filter_outliers(&prices, oracle.deviation_threshold_bps)?;

        // Aggregate price
        let aggregated_price = aggregate_prices(&filtered_prices, oracle.aggregation_method)?;

        // Update oracle state
        oracle.current_price = aggregated_price;
        oracle.last_update_timestamp = get_current_timestamp();

        Ok(())
    }
}

// IMPLEMENTATION ORDER:
// 1. Basic oracle aggregation (Day 1-2)
// 2. Pyth integration (Day 3)
// 3. Switchboard integration (Day 4)
// 4. Manipulation detection (Day 5)
```

### Week 7: Advanced Oracle Features

#### 7.1 Manipulation Detection
```rust
pub fn detect_price_manipulation(
    price_history: &[AggregatedPrice],
    new_price: &AggregatedPrice,
) -> Result<u8> {
    let mut risk_score = 0u8;

    if let Some(last_price) = price_history.last() {
        // Check for sudden price spikes
        let price_change = calculate_price_change_percentage(last_price, new_price);
        if price_change > 10.0 { // 10% spike
            risk_score += 30;
        }

        // Check volume correlation
        if new_price.volume < last_price.volume / 2 { // Low volume with price spike
            risk_score += 20;
        }

        // Check time-based patterns
        if detect_suspicious_timing_pattern(price_history) {
            risk_score += 25;
        }
    }

    Ok(risk_score)
}
```

#### 7.2 Integration Testing
```rust
#[tokio::test]
async fn test_oracle_price_aggregation() {
    let env = TestEnvironment::new().await;

    // Setup mock oracle feeds
    let pyth_feed = setup_mock_pyth_feed(&env, 43_250_000_000).await;
    let switchboard_feed = setup_mock_switchboard_feed(&env, 43_245_000_000).await;

    let params = InitializeOracleParams {
        token_mint: get_btc_mint(),
        oracle_sources: vec![
            OracleSource {
                oracle_type: OracleType::Pyth,
                feed_account: pyth_feed,
                weight: 50,
                enabled: true,
            },
            OracleSource {
                oracle_type: OracleType::Switchboard,
                feed_account: switchboard_feed,
                weight: 50,
                enabled: true,
            },
        ],
        aggregation_method: AggregationMethod::Median,
        deviation_threshold_bps: 200, // 2%
        min_sources_required: 2,
    };

    let oracle_pubkey = initialize_oracle_aggregator(&env.client, params).await.unwrap();
    update_price_feed(&env.client, oracle_pubkey).await.unwrap();

    let oracle: OracleAggregator = env.client.account(oracle_pubkey).await.unwrap();
    assert!(oracle.current_price.price.value > 0);
    assert_eq!(oracle.valid_sources_count, 2);
}
```

### Success Criteria for Phase 3:
- [ ] Oracle aggregates prices from multiple sources correctly
- [ ] Manipulation detection catches suspicious price movements
- [ ] Staleness checks prevent using old price data
- [ ] Integration with Pyth and Switchboard works
- [ ] Price updates complete in <5 seconds
- [ ] All edge cases handled properly
- [ ] Performance meets requirements (<50k CU per update)

---

## Phase 4: OTC Trading Program (Weeks 8-10)

### Week 8: Core OTC Functionality

#### 8.1 Fixed Price Orders
```rust
// Implementation priority: CRITICAL - Core whale trading functionality

pub fn create_fixed_price_order(
    ctx: Context<CreateFixedPriceOrder>,
    params: CreateOrderParams,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let escrow = &mut ctx.accounts.escrow;

    // Validate order parameters
    validate_order_params(&params)?;

    // Get current price for validation
    let oracle_price = get_oracle_price(&ctx.accounts.oracle_aggregator, params.token_a)?;
    validate_order_pricing(&params, &oracle_price)?;

    // Initialize order
    order.safe_init(params.clone())?;

    // Setup escrow
    escrow.initialize_escrow(&order, &ctx.accounts.escrow_token_account)?;

    // Transfer tokens to escrow
    transfer_to_escrow(&ctx.accounts, params.amount_a)?;

    emit!(OTCOrderCreated {
        order_id: params.order_id,
        maker: order.maker,
        token_a: params.token_a,
        token_b: params.token_b,
        amount_a: params.amount_a,
        amount_b: params.amount_b,
        timestamp: get_current_timestamp(),
    });

    Ok(())
}

// IMPLEMENTATION ORDER:
// 1. Basic order creation (Day 1-2)
// 2. Escrow system (Day 3)
// 3. Order matching/acceptance (Day 4)
// 4. Settlement logic (Day 5)
```

### Week 9: RFQ System

#### 9.1 Request for Quote Implementation
```rust
pub fn create_rfq(
    ctx: Context<CreateRFQ>,
    params: CreateRFQParams,
) -> Result<()> {
    let rfq = &mut ctx.accounts.rfq;

    // Validate RFQ parameters
    require!(params.amount_desired > 0, MobyError::InvalidAmount);
    require!(params.quote_deadline > get_current_timestamp(), MobyError::InvalidDeadline);

    rfq.safe_init(params)?;

    emit!(RFQCreated {
        rfq_id: rfq.rfq_id,
        requester: rfq.requester,
        token_desired: rfq.token_desired,
        amount_desired: rfq.amount_desired,
        quote_deadline: rfq.quote_deadline,
        timestamp: get_current_timestamp(),
    });

    Ok(())
}

pub fn submit_quote(
    ctx: Context<SubmitQuote>,
    quote: Quote,
) -> Result<()> {
    let rfq = &mut ctx.accounts.rfq;

    // Validate quote
    require!(rfq.quotes.len() < rfq.max_responders as usize, MobyError::TooManyQuotes);
    require!(quote.valid_until > get_current_timestamp(), MobyError::QuoteExpired);

    // Check responder reputation if required
    if rfq.min_responder_reputation > 0 {
        let reputation = get_responder_reputation(&ctx.accounts.responder_profile)?;
        require!(reputation >= rfq.min_responder_reputation, MobyError::InsufficientReputation);
    }

    rfq.quotes.push(quote);

    Ok(())
}
```

### Week 10: Advanced OTC Features & Testing

#### 10.1 Dark Pool Implementation
```rust
pub fn create_dark_pool_order(
    ctx: Context<CreateDarkPoolOrder>,
    params: DarkPoolParams,
) -> Result<()> {
    let order = &mut ctx.accounts.order;

    // Dark pool orders hide exact amounts
    let commitment = create_amount_commitment(params.actual_amount, params.randomness)?;

    order.commitment_hash = commitment;
    order.size_range = params.size_range;
    order.asset_type_hash = hash_asset_type(&params.token_mint);
    order.reveal_conditions = params.reveal_conditions;

    // Only reveal order details when matched
    emit!(DarkPoolOrderCreated {
        order_id: order.order_id,
        size_range: order.size_range,
        asset_hash: order.asset_type_hash,
        timestamp: get_current_timestamp(),
    });

    Ok(())
}
```

#### 10.2 Comprehensive Testing
```rust
#[tokio::test]
async fn test_otc_full_trading_flow() {
    let env = TestEnvironment::new().await;

    // Setup accounts and tokens
    let maker = Keypair::new();
    let taker = Keypair::new();
    setup_trading_accounts(&env, &maker, &taker).await;

    // Create order: Sell 1000 SOL for 150K USDC
    let order_params = CreateOrderParams {
        order_id: [1u8; 32],
        token_a: get_sol_mint(),
        token_b: get_usdc_mint(),
        amount_a: 1000 * LAMPORTS_PER_SOL,
        amount_b: 150_000 * 1_000_000,
        expiry: get_current_timestamp() + 3600,
        min_fill_amount: 100 * LAMPORTS_PER_SOL,
        slippage_tolerance: 50, // 0.5%
        privacy_level: PrivacyLevel::Public,
    };

    let order_pubkey = create_otc_order(&env, &maker, order_params).await.unwrap();

    // Partially fill order
    let fill_amount = 500 * LAMPORTS_PER_SOL;
    accept_otc_order(&env, &taker, order_pubkey, fill_amount).await.unwrap();

    // Verify state
    let order: OTCOrder = env.client.account(order_pubkey).await.unwrap();
    assert_eq!(order.status, OrderStatus::Partial);
    assert_eq!(order.filled_amount_a, fill_amount);

    // Complete the order
    let remaining_amount = order.amount_a - order.filled_amount_a;
    accept_otc_order(&env, &taker, order_pubkey, remaining_amount).await.unwrap();

    // Verify completion
    let order: OTCOrder = env.client.account(order_pubkey).await.unwrap();
    assert_eq!(order.status, OrderStatus::Filled);
}

#[tokio::test]
async fn test_rfq_competitive_bidding() {
    let env = TestEnvironment::new().await;

    // Create RFQ for large trade
    let rfq_params = CreateRFQParams {
        rfq_id: [2u8; 32],
        token_desired: get_sol_mint(),
        token_offered: get_usdc_mint(),
        amount_desired: 10_000 * LAMPORTS_PER_SOL,
        amount_offered: 1_400_000 * 1_000_000,
        quote_deadline: get_current_timestamp() + 1800,
        settlement_deadline: get_current_timestamp() + 3600,
        max_responders: 5,
        min_responder_reputation: 50,
    };

    let rfq_pubkey = create_rfq(&env, &env.payer, rfq_params).await.unwrap();

    // Submit multiple competitive quotes
    let market_makers = create_market_makers(&env, 3).await;
    let quotes = vec![
        Quote { price: 140_000_000, amount: 10_000 * LAMPORTS_PER_SOL, /* ... */ },
        Quote { price: 140_500_000, amount: 10_000 * LAMPORTS_PER_SOL, /* ... */ },
        Quote { price: 139_800_000, amount: 8_000 * LAMPORTS_PER_SOL, /* ... */ },
    ];

    for (mm, quote) in market_makers.iter().zip(quotes.iter()) {
        submit_quote(&env, mm, rfq_pubkey, quote.clone()).await.unwrap();
    }

    // Accept best quote
    let best_quote_index = 2; // Highest price quote
    accept_rfq_quote(&env, &env.payer, rfq_pubkey, best_quote_index).await.unwrap();

    // Verify settlement
    let rfq: RFQOrder = env.client.account(rfq_pubkey).await.unwrap();
    assert_eq!(rfq.status, RFQStatus::Accepted);
    assert_eq!(rfq.selected_quote, Some(best_quote_index));
}
```

### Success Criteria for Phase 4:
- [ ] Fixed price orders work end-to-end
- [ ] RFQ system enables competitive market making
- [ ] Dark pool orders maintain privacy until matched
- [ ] Escrow system prevents double-spending
- [ ] Settlement happens atomically
- [ ] All order types handle partial fills correctly
- [ ] Performance supports whale-scale trading
- [ ] Integration with oracle pricing works

---

## Phase 5: Algorithm Execution Program (Weeks 11-14)

### Implementation order continues with TWAP, VWAP, smart routing, and finally privacy features...

## Parallel Development Opportunities

### Team Structure for Parallel Development
```
Team A (3 developers): Core Libraries + Controller Program
├─ Lead: Senior Rust developer
├─ Dev 1: Math library specialist
└─ Dev 2: State management expert

Team B (3 developers): Oracle + OTC Programs
├─ Lead: DeFi protocol expert
├─ Dev 1: Oracle integration specialist
└─ Dev 2: Trading logic developer

Team C (2 developers): Algorithm Execution
├─ Lead: Algorithm expert
└─ Dev 1: Smart routing specialist

Team D (2 developers): Privacy Features
├─ Lead: Cryptography expert
└─ Dev 1: ZK circuit developer

Team E (2 developers): Frontend + SDK
├─ Lead: Frontend architect
└─ Dev 1: SDK developer

Team F (2 developers): Testing + DevOps
├─ Lead: Testing framework architect
└─ Dev 1: CI/CD specialist
```

### Milestone Dependencies
```
Week 1-3: Team A (Foundation)
Week 4-5: Team A (Controller) || Team F (Testing Infrastructure)
Week 6-7: Team B (Oracle) || Team E (Basic Frontend)
Week 8-10: Team B (OTC) || Team E (Trading Interface) || Team F (Integration Tests)
Week 11-14: Team C (Algorithms) || Team D (Privacy) || Team E (Advanced Features)
```

This implementation order ensures that:
1. **Foundation is solid** before building complex features
2. **Dependencies are respected** - no component depends on something not yet built
3. **Testing happens continuously** - each component is fully tested before moving on
4. **Teams can work in parallel** where dependencies allow
5. **Integration points are well-defined** and tested
6. **Whale traders get value early** with basic OTC functionality

The key is to build incrementally, test thoroughly, and always have a working system that provides value to whale traders.

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[{"content": "Create core library specifications and interfaces", "status": "completed", "activeForm": "Creating core library specifications and interfaces"}, {"content": "Set up comprehensive testing framework", "status": "completed", "activeForm": "Setting up comprehensive testing framework"}, {"content": "Create development environment setup guide", "status": "completed", "activeForm": "Creating development environment setup guide"}, {"content": "Document implementation order and dependencies", "status": "completed", "activeForm": "Documenting implementation order and dependencies"}, {"content": "Set up CI/CD and code quality tools", "status": "in_progress", "activeForm": "Setting up CI/CD and code quality tools"}]