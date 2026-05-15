# Solana Programs Specification

> **Detailed implementation specifications for all on-chain programs**

## Program Overview

### Core Programs
1. **Controller Program** - System orchestration and governance
2. **OTC Trading Program** - Peer-to-peer trading and escrow
3. **Algorithm Execution Program** - TWAP/VWAP/Intent execution
4. **Privacy Program** - Zero-knowledge proofs and privacy pools
5. **Oracle Program** - Price feed aggregation and validation

### Program Addresses (Mainnet)
```
moby-controller:  MobyCtrl1111111111111111111111111111111111
moby-otc:         MobyOTC11111111111111111111111111111111111
moby-executor:    MobyExec1111111111111111111111111111111111
moby-privacy:     MobyPriv1111111111111111111111111111111111
moby-oracle:      MobyOrcl1111111111111111111111111111111111
```

---

## 1. Controller Program (moby-controller)

### Purpose
Central coordination and governance for the entire Moby Market ecosystem.

### Instructions

#### Initialize System
```rust
#[derive(Accounts)]
pub struct InitializeSystem<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_system(
    ctx: Context<InitializeSystem>,
    initial_config: InitialConfig,
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;

    global_state.authority = ctx.accounts.authority.key();
    global_state.paused = false;
    global_state.emergency_mode = false;
    global_state.total_volume_traded = 0;
    global_state.total_fees_collected = 0;
    global_state.supported_tokens = initial_config.supported_tokens;
    global_state.fee_collector = initial_config.fee_collector;
    global_state.upgrade_authority = initial_config.upgrade_authority;
    global_state.created_at = Clock::get()?.unix_timestamp;
    global_state.last_updated = Clock::get()?.unix_timestamp;

    emit!(SystemInitialized {
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Update System Configuration
```rust
#[derive(Accounts)]
pub struct UpdateSystemConfig<'info> {
    #[account(
        mut,
        seeds = [b"global-state"],
        bump,
        has_one = authority
    )]
    pub global_state: Account<'info, GlobalState>,

    pub authority: Signer<'info>,
}

pub fn update_system_config(
    ctx: Context<UpdateSystemConfig>,
    updates: SystemConfigUpdates,
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;

    // Apply updates with validation
    if let Some(new_tokens) = updates.supported_tokens {
        global_state.supported_tokens = new_tokens;
    }

    if let Some(new_fee_collector) = updates.fee_collector {
        global_state.fee_collector = new_fee_collector;
    }

    global_state.last_updated = Clock::get()?.unix_timestamp;

    emit!(SystemConfigUpdated {
        authority: ctx.accounts.authority.key(),
        updates,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Emergency Pause
```rust
pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;

    require!(!global_state.paused, ErrorCode::AlreadyPaused);

    global_state.paused = true;
    global_state.emergency_mode = true;
    global_state.last_updated = Clock::get()?.unix_timestamp;

    emit!(EmergencyPauseActivated {
        triggered_by: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

### State Management

#### Global State Structure
```rust
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
    pub program_versions: ProgramVersions,
    pub risk_parameters: GlobalRiskParameters,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub struct GlobalRiskParameters {
    pub max_order_size: u64,
    pub max_daily_volume: u64,
    pub emergency_stop_threshold: u64,
    pub min_collateral_ratio: u16,
    pub liquidation_threshold: u16,
}
```

---

## 2. OTC Trading Program (moby-otc)

### Purpose
Handles all over-the-counter trading operations including fixed-price orders, RFQ system, and dark pools.

### Instructions

#### Create Fixed Price Order
```rust
#[derive(Accounts)]
#[instruction(order_params: CreateOrderParams)]
pub struct CreateFixedPriceOrder<'info> {
    #[account(
        init,
        payer = maker,
        space = 8 + OTCOrder::INIT_SPACE,
        seeds = [
            b"otc-order",
            maker.key().as_ref(),
            &order_params.order_id
        ],
        bump
    )]
    pub order: Account<'info, OTCOrder>,

    #[account(
        init,
        payer = maker,
        space = 8 + OTCEscrow::INIT_SPACE,
        seeds = [
            b"otc-escrow",
            order.key().as_ref()
        ],
        bump
    )]
    pub escrow: Account<'info, OTCEscrow>,

    /// CHECK: Will be validated by token program
    #[account(
        init,
        payer = maker,
        token::mint = token_mint_a,
        token::authority = escrow,
        seeds = [
            b"escrow-token-account",
            escrow.key().as_ref(),
            token_mint_a.key().as_ref()
        ],
        bump
    )]
    pub escrow_token_account_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_mint_a,
        token::authority = maker
    )]
    pub maker_token_account_a: Account<'info, TokenAccount>,

    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    #[account(mut)]
    pub maker: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_fixed_price_order(
    ctx: Context<CreateFixedPriceOrder>,
    params: CreateOrderParams,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let escrow = &mut ctx.accounts.escrow;

    // Validate order parameters
    require!(params.amount_a > 0, ErrorCode::InvalidAmount);
    require!(params.amount_b > 0, ErrorCode::InvalidAmount);
    require!(params.expiry > Clock::get()?.unix_timestamp, ErrorCode::InvalidExpiry);
    require!(
        ctx.accounts.token_mint_a.key() != ctx.accounts.token_mint_b.key(),
        ErrorCode::SameTokenSwap
    );

    // Initialize order
    order.order_id = params.order_id;
    order.maker = ctx.accounts.maker.key();
    order.token_mint_a = ctx.accounts.token_mint_a.key();
    order.token_mint_b = ctx.accounts.token_mint_b.key();
    order.amount_a = params.amount_a;
    order.amount_b = params.amount_b;
    order.min_fill_amount = params.min_fill_amount.unwrap_or(params.amount_a);
    order.expiry = params.expiry;
    order.order_type = OTCOrderType::FixedPrice;
    order.privacy_level = params.privacy_level;
    order.status = OrderStatus::Created;
    order.created_at = Clock::get()?.unix_timestamp;
    order.slippage_tolerance = params.slippage_tolerance;

    // Initialize escrow
    escrow.order_id = params.order_id;
    escrow.escrow_authority = escrow.key();
    escrow.token_account_a = ctx.accounts.escrow_token_account_a.key();

    // Transfer tokens to escrow
    let transfer_instruction = Transfer {
        from: ctx.accounts.maker_token_account_a.to_account_info(),
        to: ctx.accounts.escrow_token_account_a.to_account_info(),
        authority: ctx.accounts.maker.to_account_info(),
    };

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        ),
        params.amount_a,
    )?;

    escrow.deposited_amount_a = params.amount_a;

    emit!(OTCOrderCreated {
        order_id: params.order_id,
        maker: ctx.accounts.maker.key(),
        token_a: ctx.accounts.token_mint_a.key(),
        token_b: ctx.accounts.token_mint_b.key(),
        amount_a: params.amount_a,
        amount_b: params.amount_b,
        order_type: OTCOrderType::FixedPrice,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Accept OTC Order
```rust
#[derive(Accounts)]
#[instruction(fill_amount: u64)]
pub struct AcceptOTCOrder<'info> {
    #[account(
        mut,
        seeds = [
            b"otc-order",
            order.maker.as_ref(),
            &order.order_id
        ],
        bump,
        constraint = order.status == OrderStatus::Created || order.status == OrderStatus::Partial
    )]
    pub order: Account<'info, OTCOrder>,

    #[account(
        mut,
        seeds = [
            b"otc-escrow",
            order.key().as_ref()
        ],
        bump
    )]
    pub escrow: Account<'info, OTCEscrow>,

    #[account(
        mut,
        token::mint = order.token_mint_a,
        token::authority = escrow
    )]
    pub escrow_token_account_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = order.token_mint_b,
        token::authority = taker
    )]
    pub taker_token_account_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = order.token_mint_a,
        token::authority = taker
    )]
    pub taker_token_account_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = order.token_mint_b,
        token::authority = order.maker
    )]
    pub maker_token_account_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub taker: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn accept_otc_order(
    ctx: Context<AcceptOTCOrder>,
    fill_amount: u64,
) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let escrow = &mut ctx.accounts.escrow;

    // Validate fill amount
    let remaining_amount = order.amount_a - order.filled_amount_a;
    require!(fill_amount <= remaining_amount, ErrorCode::ExcessiveFillAmount);
    require!(fill_amount >= order.min_fill_amount, ErrorCode::BelowMinFillAmount);

    // Calculate proportional amount B
    let fill_amount_b = (fill_amount as u128 * order.amount_b as u128 / order.amount_a as u128) as u64;

    // Transfer token B from taker to maker
    let transfer_b_to_maker = Transfer {
        from: ctx.accounts.taker_token_account_b.to_account_info(),
        to: ctx.accounts.maker_token_account_b.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_b_to_maker,
        ),
        fill_amount_b,
    )?;

    // Transfer token A from escrow to taker
    let escrow_key = escrow.key();
    let seeds = &[
        b"otc-escrow",
        order.to_account_info().key.as_ref(),
        &[ctx.bumps.escrow],
    ];
    let signer = &[&seeds[..]];

    let transfer_a_to_taker = Transfer {
        from: ctx.accounts.escrow_token_account_a.to_account_info(),
        to: ctx.accounts.taker_token_account_a.to_account_info(),
        authority: escrow.to_account_info(),
    };

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_a_to_taker,
            signer,
        ),
        fill_amount,
    )?;

    // Update order state
    order.filled_amount_a += fill_amount;
    order.filled_amount_b += fill_amount_b;
    order.taker = Some(ctx.accounts.taker.key());
    order.updated_at = Clock::get()?.unix_timestamp;

    if order.filled_amount_a == order.amount_a {
        order.status = OrderStatus::Filled;
    } else {
        order.status = OrderStatus::Partial;
    }

    emit!(OTCOrderFilled {
        order_id: order.order_id,
        taker: ctx.accounts.taker.key(),
        fill_amount_a: fill_amount,
        fill_amount_b: fill_amount_b,
        total_filled_a: order.filled_amount_a,
        remaining_amount_a: order.amount_a - order.filled_amount_a,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Create RFQ (Request for Quote)
```rust
pub fn create_rfq(
    ctx: Context<CreateRFQ>,
    params: CreateRFQParams,
) -> Result<()> {
    let rfq = &mut ctx.accounts.rfq;

    // Validate RFQ parameters
    require!(params.amount_desired > 0, ErrorCode::InvalidAmount);
    require!(params.amount_offered > 0, ErrorCode::InvalidAmount);
    require!(
        params.quote_deadline > Clock::get()?.unix_timestamp,
        ErrorCode::InvalidDeadline
    );
    require!(
        params.settlement_deadline > params.quote_deadline,
        ErrorCode::InvalidSettlementDeadline
    );

    rfq.rfq_id = params.rfq_id;
    rfq.requester = ctx.accounts.requester.key();
    rfq.token_desired = params.token_desired;
    rfq.token_offered = params.token_offered;
    rfq.amount_desired = params.amount_desired;
    rfq.amount_offered = params.amount_offered;
    rfq.quote_deadline = params.quote_deadline;
    rfq.settlement_deadline = params.settlement_deadline;
    rfq.min_responder_reputation = params.min_responder_reputation;
    rfq.max_responders = params.max_responders;
    rfq.status = RFQStatus::Active;
    rfq.created_at = Clock::get()?.unix_timestamp;

    emit!(RFQCreated {
        rfq_id: params.rfq_id,
        requester: ctx.accounts.requester.key(),
        token_desired: params.token_desired,
        token_offered: params.token_offered,
        amount_desired: params.amount_desired,
        amount_offered: params.amount_offered,
        quote_deadline: params.quote_deadline,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

### Account Structures

```rust
#[account]
#[derive(InitSpace)]
pub struct OTCOrder {
    pub order_id: [u8; 32],
    pub maker: Pubkey,
    pub taker: Option<Pubkey>,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub filled_amount_a: u64,
    pub filled_amount_b: u64,
    pub min_fill_amount: u64,
    pub expiry: i64,
    pub order_type: OTCOrderType,
    pub privacy_level: PrivacyLevel,
    pub status: OrderStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub fees_paid: u64,
    pub slippage_tolerance: u16,
    pub maker_fee_bps: u16,
    pub taker_fee_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum OTCOrderType {
    FixedPrice,
    RFQ,
    DarkPool,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum OrderStatus {
    Created,
    Partial,
    Filled,
    Cancelled,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum PrivacyLevel {
    Public,
    Stealth,
    Anonymous,
}

#[account]
#[derive(InitSpace)]
pub struct RFQOrder {
    pub rfq_id: [u8; 32],
    pub requester: Pubkey,
    pub token_desired: Pubkey,
    pub token_offered: Pubkey,
    pub amount_desired: u64,
    pub amount_offered: u64,
    pub quote_deadline: i64,
    pub settlement_deadline: i64,
    pub min_responder_reputation: u32,
    pub max_responders: u8,
    pub status: RFQStatus,
    pub created_at: i64,
    #[max_len(10)]
    pub quotes: Vec<Quote>,
    pub selected_quote: Option<u8>, // Index into quotes array
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub struct Quote {
    pub responder: Pubkey,
    pub price: u64,              // Price per unit in quote token
    pub amount: u64,             // Amount willing to trade
    pub valid_until: i64,
    pub partial_fill_allowed: bool,
    pub submitted_at: i64,
}
```

---

## 3. Algorithm Execution Program (moby-executor)

### Purpose
Implements sophisticated execution algorithms including TWAP, VWAP, and intent-based execution.

### Instructions

#### Create TWAP Order
```rust
pub fn create_twap_order(
    ctx: Context<CreateTWAPOrder>,
    params: CreateTWAPParams,
) -> Result<()> {
    let twap_order = &mut ctx.accounts.twap_order;

    // Validate TWAP parameters
    require!(params.total_amount > 0, ErrorCode::InvalidAmount);
    require!(params.time_window > 0, ErrorCode::InvalidTimeWindow);
    require!(params.num_intervals > 0, ErrorCode::InvalidIntervals);
    require!(params.num_intervals <= 1000, ErrorCode::TooManyIntervals);

    // Calculate interval amounts with randomization
    let base_amount = params.total_amount / params.num_intervals as u64;
    let mut interval_amounts = Vec::new();
    let mut remaining_amount = params.total_amount;

    for i in 0..params.num_intervals {
        let is_last = i == params.num_intervals - 1;
        let amount = if is_last {
            remaining_amount
        } else {
            // Apply randomization factor
            let randomness = generate_randomness(params.randomness_factor);
            let adjusted_amount = apply_randomness(base_amount, randomness);
            std::cmp::min(adjusted_amount, remaining_amount)
        };

        interval_amounts.push(amount);
        remaining_amount -= amount;
    }

    twap_order.order_id = params.order_id;
    twap_order.trader = ctx.accounts.trader.key();
    twap_order.token_in = params.token_in;
    twap_order.token_out = params.token_out;
    twap_order.total_amount_in = params.total_amount;
    twap_order.min_amount_out = params.min_amount_out;
    twap_order.time_window_seconds = params.time_window;
    twap_order.num_intervals = params.num_intervals;
    twap_order.randomness_factor = params.randomness_factor;
    twap_order.max_slippage_bps = params.max_slippage_bps;
    twap_order.interval_amounts = interval_amounts;
    twap_order.venue_preferences = params.venue_preferences;
    twap_order.status = AlgorithmStatus::Active;
    twap_order.privacy_enabled = params.privacy_enabled;
    twap_order.created_at = Clock::get()?.unix_timestamp;

    // Calculate next execution time
    let interval_duration = params.time_window / params.num_intervals as i64;
    twap_order.next_execution_time = Clock::get()?.unix_timestamp + interval_duration;

    emit!(TWAPOrderCreated {
        order_id: params.order_id,
        trader: ctx.accounts.trader.key(),
        total_amount: params.total_amount,
        num_intervals: params.num_intervals,
        time_window: params.time_window,
        next_execution_time: twap_order.next_execution_time,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Execute TWAP Interval
```rust
pub fn execute_twap_interval(
    ctx: Context<ExecuteTWAPInterval>,
) -> Result<()> {
    let twap_order = &mut ctx.accounts.twap_order;

    // Verify execution timing
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time >= twap_order.next_execution_time,
        ErrorCode::ExecutionTooEarly
    );
    require!(
        twap_order.status == AlgorithmStatus::Active,
        ErrorCode::OrderNotActive
    );
    require!(
        twap_order.current_interval < twap_order.num_intervals,
        ErrorCode::AllIntervalsExecuted
    );

    // Get current interval amount
    let interval_index = twap_order.current_interval as usize;
    let execution_amount = twap_order.interval_amounts[interval_index];

    // Execute trade through optimal routing
    let route_result = execute_optimal_route(
        &ctx.accounts,
        execution_amount,
        twap_order.max_slippage_bps,
        &twap_order.venue_preferences,
    )?;

    // Record execution
    let execution_record = ExecutionRecord {
        interval_index: twap_order.current_interval,
        amount_in: execution_amount,
        amount_out: route_result.amount_out,
        execution_price: route_result.execution_price,
        slippage_bps: route_result.slippage_bps,
        venue_used: route_result.venue_used,
        gas_used: route_result.gas_used,
        timestamp: current_time,
    };

    twap_order.execution_history.push(execution_record);
    twap_order.executed_amount_in += execution_amount;
    twap_order.received_amount_out += route_result.amount_out;
    twap_order.current_interval += 1;

    // Update next execution time with randomization
    if twap_order.current_interval < twap_order.num_intervals {
        let base_interval = twap_order.time_window_seconds / twap_order.num_intervals as i64;
        let randomness_offset = generate_execution_delay(twap_order.randomness_factor);
        twap_order.next_execution_time = current_time + base_interval + randomness_offset;
    } else {
        twap_order.status = AlgorithmStatus::Completed;
        twap_order.next_execution_time = 0;
    }

    twap_order.updated_at = current_time;

    emit!(TWAPIntervalExecuted {
        order_id: twap_order.order_id,
        interval_index: interval_index as u32,
        amount_in: execution_amount,
        amount_out: route_result.amount_out,
        execution_price: route_result.execution_price,
        slippage_bps: route_result.slippage_bps,
        next_execution_time: twap_order.next_execution_time,
        timestamp: current_time,
    });

    Ok(())
}
```

#### Create VWAP Order
```rust
pub fn create_vwap_order(
    ctx: Context<CreateVWAPOrder>,
    params: CreateVWAPParams,
) -> Result<()> {
    let vwap_order = &mut ctx.accounts.vwap_order;

    // Validate VWAP parameters
    require!(params.total_amount > 0, ErrorCode::InvalidAmount);
    require!(params.target_participation_bps <= 2000, ErrorCode::ExcessiveParticipation); // Max 20%
    require!(params.time_horizon_hours > 0, ErrorCode::InvalidTimeHorizon);
    require!(params.time_horizon_hours <= 168, ErrorCode::TimeHorizonTooLong); // Max 1 week

    // Generate volume forecast based on historical data
    let volume_forecast = generate_volume_forecast(
        &params.token_pair,
        params.time_horizon_hours,
        &ctx.accounts.oracle_data,
    )?;

    // Create execution schedule based on volume forecast
    let execution_schedule = create_vwap_schedule(
        params.total_amount,
        params.target_participation_bps,
        &volume_forecast,
        params.rebalance_frequency_minutes,
    )?;

    vwap_order.order_id = params.order_id;
    vwap_order.trader = ctx.accounts.trader.key();
    vwap_order.token_pair = params.token_pair;
    vwap_order.total_amount = params.total_amount;
    vwap_order.target_participation_bps = params.target_participation_bps;
    vwap_order.volume_curve_type = params.volume_curve_type;
    vwap_order.time_horizon_hours = params.time_horizon_hours;
    vwap_order.rebalance_frequency_minutes = params.rebalance_frequency_minutes;
    vwap_order.volume_forecast = volume_forecast;
    vwap_order.execution_schedule = execution_schedule;
    vwap_order.adaptive_parameters = params.adaptive_parameters;

    emit!(VWAPOrderCreated {
        order_id: params.order_id,
        trader: ctx.accounts.trader.key(),
        total_amount: params.total_amount,
        target_participation_bps: params.target_participation_bps,
        time_horizon_hours: params.time_horizon_hours,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

### Helper Functions

```rust
fn execute_optimal_route(
    accounts: &ExecuteTWAPInterval,
    amount_in: u64,
    max_slippage_bps: u16,
    venue_preferences: &[VenueWeight],
) -> Result<RouteResult> {
    // Implementation of smart routing logic
    // 1. Query liquidity across all venues
    // 2. Calculate optimal route considering fees and slippage
    // 3. Execute through best venue or split across multiple venues
    // 4. Return execution results

    // Placeholder implementation
    Ok(RouteResult {
        amount_out: amount_in * 150, // Assuming SOL -> USDC conversion
        execution_price: 150_000_000, // Price in micro-units
        slippage_bps: 5, // 0.05%
        venue_used: Pubkey::default(),
        gas_used: 50_000,
    })
}

fn generate_randomness(factor: u8) -> i64 {
    // Generate cryptographically secure randomness
    // Uses Solana's built-in randomness when available
    // Falls back to deterministic pseudo-randomness based on slot/timestamp

    if factor == 0 {
        return 0;
    }

    let slot = Clock::get().unwrap().slot;
    let timestamp = Clock::get().unwrap().unix_timestamp;

    // Simple PRNG based on slot and timestamp
    let seed = (slot as i64).wrapping_mul(timestamp);
    let rand_val = seed % (factor as i64 * 2) - (factor as i64);

    rand_val
}

fn generate_volume_forecast(
    token_pair: &TokenPair,
    time_horizon_hours: u8,
    oracle_data: &Account<OracleData>,
) -> Result<Vec<VolumeDataPoint>> {
    // Generate volume forecast based on:
    // 1. Historical volume patterns
    // 2. Time-of-day effects
    // 3. Day-of-week effects
    // 4. Market volatility
    // 5. External events

    let mut forecast = Vec::new();
    let intervals = time_horizon_hours as usize * 4; // 15-minute intervals

    for i in 0..intervals {
        let hour_of_day = (i / 4) % 24;
        let base_volume = get_historical_average_volume(token_pair, hour_of_day)?;
        let volatility_adjustment = get_volatility_adjustment(oracle_data, i)?;
        let predicted_volume = (base_volume as f64 * volatility_adjustment) as u64;

        forecast.push(VolumeDataPoint {
            timestamp: Clock::get()?.unix_timestamp + (i as i64 * 900), // 15 min intervals
            predicted_volume,
            confidence: 0.8, // 80% confidence
        });
    }

    Ok(forecast)
}
```

---

## 4. Privacy Program (moby-privacy)

### Purpose
Implements zero-knowledge privacy features including privacy pools, commitment schemes, and proof verification.

### Instructions

#### Initialize Privacy Pool
```rust
pub fn initialize_privacy_pool(
    ctx: Context<InitializePrivacyPool>,
    params: InitializePoolParams,
) -> Result<()> {
    let privacy_pool = &mut ctx.accounts.privacy_pool;
    let commitment_tree = &mut ctx.accounts.commitment_tree;

    // Validate pool parameters
    require!(params.min_deposit_amount > 0, ErrorCode::InvalidMinDeposit);
    require!(params.merkle_tree_height >= 10, ErrorCode::TreeTooShallow);
    require!(params.merkle_tree_height <= 32, ErrorCode::TreeTooDeep);

    privacy_pool.pool_id = params.pool_id;
    privacy_pool.merkle_tree_height = params.merkle_tree_height;
    privacy_pool.commitment_scheme = params.commitment_scheme;
    privacy_pool.zk_circuit_id = params.zk_circuit_id;
    privacy_pool.verification_key_hash = params.verification_key_hash;
    privacy_pool.min_deposit_amount = params.min_deposit_amount;
    privacy_pool.deposit_fee_bps = params.deposit_fee_bps;
    privacy_pool.withdrawal_delay_seconds = params.withdrawal_delay_seconds;

    // Initialize commitment tree
    commitment_tree.tree_height = params.merkle_tree_height;
    commitment_tree.next_index = 0;
    commitment_tree.current_root_index = 0;

    // Initialize zero hashes for each level
    let mut current_zero = [0u8; 32];
    commitment_tree.zeros[0] = current_zero;

    for i in 1..params.merkle_tree_height as usize {
        current_zero = hash_pair(&current_zero, &current_zero);
        commitment_tree.zeros[i] = current_zero;
    }

    // Set initial root
    commitment_tree.roots[0] = commitment_tree.zeros[(params.merkle_tree_height - 1) as usize];

    emit!(PrivacyPoolInitialized {
        pool_id: params.pool_id,
        tree_height: params.merkle_tree_height,
        min_deposit: params.min_deposit_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Deposit to Privacy Pool
```rust
pub fn deposit_to_privacy_pool(
    ctx: Context<DepositToPrivacyPool>,
    commitment: [u8; 32],
    amount: u64,
) -> Result<()> {
    let privacy_pool = &mut ctx.accounts.privacy_pool;
    let commitment_tree = &mut ctx.accounts.commitment_tree;

    // Validate deposit
    require!(amount >= privacy_pool.min_deposit_amount, ErrorCode::DepositTooSmall);
    require!(
        commitment_tree.next_index < (1u32 << privacy_pool.merkle_tree_height),
        ErrorCode::PoolFull
    );

    // Transfer tokens to pool
    let transfer_instruction = Transfer {
        from: ctx.accounts.depositor_token_account.to_account_info(),
        to: ctx.accounts.pool_token_account.to_account_info(),
        authority: ctx.accounts.depositor.to_account_info(),
    };

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        ),
        amount,
    )?;

    // Add commitment to tree
    let leaf_index = commitment_tree.next_index;
    insert_commitment(commitment_tree, commitment, leaf_index)?;

    // Update pool statistics
    privacy_pool.total_deposited += amount;
    privacy_pool.anonymity_set_size += 1;
    privacy_pool.next_leaf_index = leaf_index + 1;

    // Update merkle tree root
    let new_root = commitment_tree.roots[commitment_tree.current_root_index as usize];
    privacy_pool.merkle_tree_root = new_root;

    emit!(PrivacyPoolDeposit {
        pool_id: privacy_pool.pool_id,
        commitment,
        leaf_index,
        amount,
        new_root,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Withdraw from Privacy Pool
```rust
pub fn withdraw_from_privacy_pool(
    ctx: Context<WithdrawFromPrivacyPool>,
    proof_data: WithdrawalProofData,
) -> Result<()> {
    let privacy_pool = &mut ctx.accounts.privacy_pool;

    // Verify the proof hasn't been used (check nullifier)
    require!(
        !privacy_pool.nullifier_hashes.contains(&proof_data.nullifier),
        ErrorCode::NullifierAlreadyUsed
    );

    // Verify the merkle root is in our recent roots
    let root_index = find_root_index(&privacy_pool, proof_data.root)?;
    require!(root_index.is_some(), ErrorCode::InvalidMerkleRoot);

    // Verify ZK proof
    verify_withdrawal_proof(
        &proof_data,
        &ctx.accounts.verification_key,
        privacy_pool.zk_circuit_id,
    )?;

    // Add nullifier to prevent double spending
    privacy_pool.nullifier_hashes.push(proof_data.nullifier);

    // Transfer tokens to recipient
    let pool_seed = &privacy_pool.pool_id;
    let seeds = &[
        b"privacy-pool-authority",
        pool_seed.as_ref(),
        &[ctx.bumps.pool_authority],
    ];
    let signer = &[&seeds[..]];

    let transfer_instruction = Transfer {
        from: ctx.accounts.pool_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.pool_authority.to_account_info(),
    };

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        ),
        proof_data.amount,
    )?;

    // Update pool statistics
    privacy_pool.total_withdrawn += proof_data.amount;
    privacy_pool.anonymity_set_size = privacy_pool.anonymity_set_size.saturating_sub(1);

    emit!(PrivacyPoolWithdrawal {
        pool_id: privacy_pool.pool_id,
        nullifier: proof_data.nullifier,
        amount: proof_data.amount,
        recipient: proof_data.recipient,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

### ZK Proof Verification

```rust
fn verify_withdrawal_proof(
    proof_data: &WithdrawalProofData,
    verification_key: &Account<VerificationKey>,
    circuit_id: [u8; 32],
) -> Result<()> {
    // Verify the circuit ID matches
    require!(
        verification_key.circuit_id == circuit_id,
        ErrorCode::CircuitMismatch
    );

    // Prepare public inputs for proof verification
    let public_inputs = vec![
        proof_data.nullifier.to_vec(),
        proof_data.root.to_vec(),
        proof_data.recipient.to_bytes().to_vec(),
        proof_data.amount.to_le_bytes().to_vec(),
    ];

    // Verify the proof using the appropriate proof system
    match verification_key.proof_system {
        ProofSystem::Groth16 => {
            verify_groth16_proof(&proof_data.proof, &verification_key.vk_data, &public_inputs)?;
        }
        ProofSystem::PLONK => {
            verify_plonk_proof(&proof_data.proof, &verification_key.vk_data, &public_inputs)?;
        }
        ProofSystem::STARKs => {
            verify_stark_proof(&proof_data.proof, &verification_key.vk_data, &public_inputs)?;
        }
    }

    Ok(())
}

fn insert_commitment(
    commitment_tree: &mut Account<CommitmentTree>,
    commitment: [u8; 32],
    leaf_index: u32,
) -> Result<()> {
    let tree_height = commitment_tree.tree_height as usize;
    let mut current_index = leaf_index;
    let mut current_hash = commitment;

    // Update path from leaf to root
    for level in 0..tree_height {
        let is_right_node = current_index % 2 == 1;

        if is_right_node {
            let left_hash = commitment_tree.filled_subtrees[level];
            current_hash = hash_pair(&left_hash, &current_hash);
            current_index /= 2;
        } else {
            commitment_tree.filled_subtrees[level] = current_hash;
            let right_hash = commitment_tree.zeros[level];
            current_hash = hash_pair(&current_hash, &right_hash);
            current_index /= 2;
        }
    }

    // Update root
    commitment_tree.current_root_index = (commitment_tree.current_root_index + 1) % 100;
    commitment_tree.roots[commitment_tree.current_root_index as usize] = current_hash;

    Ok(())
}

fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    use solana_program::keccak;

    let mut input = [0u8; 64];
    input[..32].copy_from_slice(left);
    input[32..].copy_from_slice(right);

    keccak::hash(&input).to_bytes()
}
```

---

## 5. Oracle Program (moby-oracle)

### Purpose
Aggregates price feeds from multiple oracle sources with manipulation detection and emergency fallbacks.

### Instructions

#### Initialize Oracle Aggregator
```rust
pub fn initialize_oracle_aggregator(
    ctx: Context<InitializeOracleAggregator>,
    params: InitializeOracleParams,
) -> Result<()> {
    let oracle_aggregator = &mut ctx.accounts.oracle_aggregator;

    // Validate oracle configuration
    require!(params.oracle_sources.len() >= 2, ErrorCode::InsufficientOracles);
    require!(params.deviation_threshold_bps <= 1000, ErrorCode::DeviationTooHigh); // Max 10%
    require!(params.staleness_threshold > 0, ErrorCode::InvalidStaleness);

    oracle_aggregator.authority = ctx.accounts.authority.key();
    oracle_aggregator.token_mint = params.token_mint;
    oracle_aggregator.oracle_sources = params.oracle_sources;
    oracle_aggregator.aggregation_method = params.aggregation_method;
    oracle_aggregator.deviation_threshold_bps = params.deviation_threshold_bps;
    oracle_aggregator.staleness_threshold = params.staleness_threshold;
    oracle_aggregator.min_sources_required = params.min_sources_required;
    oracle_aggregator.created_at = Clock::get()?.unix_timestamp;

    emit!(OracleAggregatorInitialized {
        token_mint: params.token_mint,
        num_sources: params.oracle_sources.len() as u8,
        aggregation_method: params.aggregation_method,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
```

#### Update Price Feed
```rust
pub fn update_price_feed(
    ctx: Context<UpdatePriceFeed>,
) -> Result<()> {
    let oracle_aggregator = &mut ctx.accounts.oracle_aggregator;
    let current_time = Clock::get()?.unix_timestamp;

    let mut prices = Vec::new();
    let mut total_valid_sources = 0;

    // Collect prices from all oracle sources
    for oracle_source in &oracle_aggregator.oracle_sources {
        match oracle_source.oracle_type {
            OracleType::Pyth => {
                if let Ok(price_data) = get_pyth_price(&ctx.accounts.pyth_price_account) {
                    if is_price_fresh(&price_data, current_time, oracle_aggregator.staleness_threshold) {
                        prices.push(PricePoint {
                            price: price_data.price,
                            confidence: price_data.confidence,
                            timestamp: price_data.timestamp,
                            source: oracle_source.clone(),
                        });
                        total_valid_sources += 1;
                    }
                }
            }
            OracleType::Switchboard => {
                if let Ok(price_data) = get_switchboard_price(&ctx.accounts.switchboard_feed) {
                    if is_price_fresh(&price_data, current_time, oracle_aggregator.staleness_threshold) {
                        prices.push(PricePoint {
                            price: price_data.price,
                            confidence: price_data.confidence,
                            timestamp: price_data.timestamp,
                            source: oracle_source.clone(),
                        });
                        total_valid_sources += 1;
                    }
                }
            }
            OracleType::Chainlink => {
                // Handle Chainlink feeds via bridge
                if let Ok(price_data) = get_chainlink_price(&ctx.accounts.chainlink_feed) {
                    if is_price_fresh(&price_data, current_time, oracle_aggregator.staleness_threshold) {
                        prices.push(PricePoint {
                            price: price_data.price,
                            confidence: price_data.confidence,
                            timestamp: price_data.timestamp,
                            source: oracle_source.clone(),
                        });
                        total_valid_sources += 1;
                    }
                }
            }
        }
    }

    // Verify we have enough valid sources
    require!(
        total_valid_sources >= oracle_aggregator.min_sources_required,
        ErrorCode::InsufficientValidSources
    );

    // Filter outliers
    let filtered_prices = filter_price_outliers(&prices, oracle_aggregator.deviation_threshold_bps)?;
    require!(!filtered_prices.is_empty(), ErrorCode::AllPricesFiltered);

    // Calculate aggregated price
    let aggregated_price = match oracle_aggregator.aggregation_method {
        AggregationMethod::Median => calculate_median_price(&filtered_prices)?,
        AggregationMethod::Mean => calculate_mean_price(&filtered_prices)?,
        AggregationMethod::VolumeWeighted => calculate_vwap(&filtered_prices)?,
        AggregationMethod::ConfidenceWeighted => calculate_confidence_weighted_price(&filtered_prices)?,
    };

    // Detect potential manipulation
    let manipulation_score = detect_price_manipulation(
        &oracle_aggregator.price_history,
        &aggregated_price,
    )?;

    // Update aggregator state
    oracle_aggregator.current_price = aggregated_price.clone();
    oracle_aggregator.last_update_slot = Clock::get()?.slot;
    oracle_aggregator.last_update_timestamp = current_time;
    oracle_aggregator.valid_sources_count = total_valid_sources;
    oracle_aggregator.manipulation_score = manipulation_score;

    // Add to price history (keep last 100 prices)
    oracle_aggregator.price_history.push(aggregated_price.clone());
    if oracle_aggregator.price_history.len() > 100 {
        oracle_aggregator.price_history.remove(0);
    }

    // Emit price update event
    emit!(PriceUpdated {
        token_mint: oracle_aggregator.token_mint,
        price: aggregated_price.price,
        confidence: aggregated_price.confidence,
        sources_used: total_valid_sources,
        manipulation_score,
        timestamp: current_time,
    });

    // Trigger alerts if manipulation detected
    if manipulation_score > 80 {
        emit!(ManipulationAlert {
            token_mint: oracle_aggregator.token_mint,
            manipulation_score,
            current_price: aggregated_price.price,
            timestamp: current_time,
        });
    }

    Ok(())
}
```

### Account Structures

```rust
#[account]
#[derive(InitSpace)]
pub struct OracleAggregator {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    #[max_len(10)]
    pub oracle_sources: Vec<OracleSource>,
    pub aggregation_method: AggregationMethod,
    pub deviation_threshold_bps: u16,
    pub staleness_threshold: i64,
    pub min_sources_required: u8,
    pub current_price: AggregatedPrice,
    pub last_update_slot: u64,
    pub last_update_timestamp: i64,
    pub valid_sources_count: u8,
    pub manipulation_score: u8,
    #[max_len(100)]
    pub price_history: Vec<AggregatedPrice>,
    pub created_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub struct OracleSource {
    pub oracle_type: OracleType,
    pub feed_account: Pubkey,
    pub weight: u8,              // 1-100, weight in aggregation
    pub max_confidence: u64,     // Maximum acceptable confidence interval
    pub enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum OracleType {
    Pyth,
    Switchboard,
    Chainlink,
    Custom,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum AggregationMethod {
    Median,
    Mean,
    VolumeWeighted,
    ConfidenceWeighted,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub struct AggregatedPrice {
    pub price: i64,
    pub confidence: u64,
    pub timestamp: i64,
    pub sources_used: u8,
    pub price_status: PriceStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum PriceStatus {
    Trading,    // Normal trading price
    Unknown,    // Price unknown or unavailable
    Halted,     // Trading halted due to manipulation
    Auction,    // In auction mode
}
```

### Error Codes

```rust
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount provided")]
    InvalidAmount,

    #[msg("Invalid expiry time")]
    InvalidExpiry,

    #[msg("Same token swap not allowed")]
    SameTokenSwap,

    #[msg("Order already filled")]
    OrderAlreadyFilled,

    #[msg("Excessive fill amount")]
    ExcessiveFillAmount,

    #[msg("Below minimum fill amount")]
    BelowMinFillAmount,

    #[msg("System is currently paused")]
    SystemPaused,

    #[msg("Already paused")]
    AlreadyPaused,

    #[msg("Execution too early")]
    ExecutionTooEarly,

    #[msg("Order not active")]
    OrderNotActive,

    #[msg("All intervals executed")]
    AllIntervalsExecuted,

    #[msg("Invalid time window")]
    InvalidTimeWindow,

    #[msg("Invalid number of intervals")]
    InvalidIntervals,

    #[msg("Too many intervals")]
    TooManyIntervals,

    #[msg("Deposit amount too small")]
    DepositTooSmall,

    #[msg("Privacy pool is full")]
    PoolFull,

    #[msg("Nullifier already used")]
    NullifierAlreadyUsed,

    #[msg("Invalid merkle root")]
    InvalidMerkleRoot,

    #[msg("Invalid ZK proof")]
    InvalidZKProof,

    #[msg("Circuit ID mismatch")]
    CircuitMismatch,

    #[msg("Insufficient oracle sources")]
    InsufficientOracles,

    #[msg("Deviation threshold too high")]
    DeviationTooHigh,

    #[msg("Invalid staleness threshold")]
    InvalidStaleness,

    #[msg("Insufficient valid price sources")]
    InsufficientValidSources,

    #[msg("All prices were filtered as outliers")]
    AllPricesFiltered,
}
```

This comprehensive Solana program specification provides the foundation for implementing the whale trading infrastructure. Each program is designed with security, efficiency, and composability in mind, following Solana best practices and Anchor framework conventions.

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[{"content": "Create comprehensive technical architecture documentation", "status": "completed", "activeForm": "Creating comprehensive technical architecture documentation"}, {"content": "Document Solana program specifications", "status": "completed", "activeForm": "Documenting Solana program specifications"}, {"content": "Create implementation roadmap and milestones", "status": "in_progress", "activeForm": "Creating implementation roadmap and milestones"}, {"content": "Document ZK circuit specifications", "status": "pending", "activeForm": "Documenting ZK circuit specifications"}, {"content": "Create deployment and infrastructure guides", "status": "pending", "activeForm": "Creating deployment and infrastructure guides"}]