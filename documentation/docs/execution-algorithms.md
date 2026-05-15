# Execution Algorithms

## Overview

Moby Market provides sophisticated execution algorithms designed to minimize market impact and achieve optimal execution for large orders. These algorithms split and time orders to reduce slippage and avoid detection.

## TWAP (Time-Weighted Average Price)

### Algorithm Description

TWAP spreads order execution across a specified time window, executing smaller portions at regular intervals to achieve the average price over time.

### Core Implementation

```rust
pub struct TWAPOrder {
    pub trader: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub total_amount: u64,
    pub executed_amount: u64,
    pub time_window: i64,        // Total execution window in seconds
    pub num_splits: u32,         // Number of execution intervals
    pub current_split: u32,      // Current execution interval
    pub randomness_factor: u8,   // 0-100, adds entropy to timing
    pub price_deviation_limit: u16, // Max acceptable price deviation (bps)
    pub min_output_per_interval: u64,
    pub start_time: i64,
    pub next_execution_time: i64,
}

impl TWAPOrder {
    pub fn calculate_next_execution(&self, current_time: i64) -> ExecutionParams {
        let base_interval = self.time_window / self.num_splits as i64;
        let randomness_offset = self.generate_randomness_offset();

        ExecutionParams {
            amount: self.calculate_interval_amount(),
            execution_time: current_time + base_interval + randomness_offset,
            max_slippage: self.calculate_dynamic_slippage(),
            venue_preferences: self.select_optimal_venues(),
        }
    }

    fn generate_randomness_offset(&self) -> i64 {
        // Generate random offset to avoid predictable patterns
        let max_offset = (self.time_window / self.num_splits as i64)
            * self.randomness_factor as i64 / 100;
        // Use on-chain randomness source
        random_offset(-max_offset, max_offset)
    }

    fn calculate_interval_amount(&self) -> u64 {
        let remaining_amount = self.total_amount - self.executed_amount;
        let remaining_intervals = self.num_splits - self.current_split;

        if remaining_intervals == 1 {
            remaining_amount // Execute all remaining on final interval
        } else {
            // Dynamic sizing based on market conditions
            let base_amount = remaining_amount / remaining_intervals as u64;
            self.adjust_for_market_conditions(base_amount)
        }
    }
}
```

### Configuration Options

```typescript
interface TWAPConfig {
  totalAmount: bigint;           // Total order size
  timeWindow: number;            // Execution window in seconds
  numSplits: number;             // Number of execution intervals
  randomnessFactor: number;      // 0-100, timing randomization
  priceDeviationLimit: number;   // Max price deviation in basis points
  minOutputPerInterval: bigint;  // Minimum acceptable output per execution
  adaptiveSlippage: boolean;     // Enable dynamic slippage adjustment
  venuePreferences: VenueWeight[]; // Preferred execution venues
}
```

### Usage Example

```typescript
// Create a TWAP order to sell 10,000 SOL over 4 hours
const twapConfig = {
  tokenIn: "So11111111111111111111111111111111111111112", // SOL
  tokenOut: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
  totalAmount: 10000n * BigInt(LAMPORTS_PER_SOL),
  timeWindow: 4 * 3600, // 4 hours
  numSplits: 24,        // Execute every 10 minutes
  randomnessFactor: 25, // 25% timing randomization
  priceDeviationLimit: 50, // 0.5% max deviation
  minOutputPerInterval: 1000n * 1000000n, // Min 1,000 USDC per execution
  adaptiveSlippage: true,
  venuePreferences: [
    { venue: "Raydium", weight: 40 },
    { venue: "Orca", weight: 35 },
    { venue: "Phoenix", weight: 25 }
  ]
};

const orderId = await submitTWAPOrder(twapConfig);
```

## VWAP (Volume-Weighted Average Price)

### Algorithm Description

VWAP adjusts execution based on historical volume patterns, executing larger portions during high-volume periods and smaller portions during low-volume periods.

### Core Implementation

```rust
pub struct VWAPOrder {
    pub trader: Pubkey,
    pub token_pair: TokenPair,
    pub total_amount: u64,
    pub executed_amount: u64,
    pub target_volume_participation: u16, // Basis points of market volume
    pub max_spread_tolerance: u16,
    pub volume_curve: VolumeCurve,
    pub adaptive_parameters: AdaptiveParams,
    pub historical_volume_data: Vec<VolumeDataPoint>,
}

pub struct VolumeCurve {
    pub curve_type: CurveType,
    pub aggressiveness: f64,     // 0.0 (passive) to 1.0 (aggressive)
    pub frontload_factor: f64,   // Bias toward earlier execution
    pub participation_limits: ParticipationLimits,
}

pub enum CurveType {
    Linear,
    Exponential,
    Sigmoid,
    CustomWeights(Vec<f64>),
}

impl VWAPOrder {
    pub fn calculate_execution_schedule(&self) -> Vec<ExecutionSlot> {
        let volume_forecast = self.forecast_volume_profile();
        let participation_schedule = self.calculate_participation_rates(&volume_forecast);

        participation_schedule
            .iter()
            .enumerate()
            .map(|(i, &participation_rate)| {
                ExecutionSlot {
                    time_slot: i as i64 * self.get_slot_duration(),
                    target_amount: self.calculate_slot_amount(participation_rate),
                    max_participation: participation_rate,
                    venue_allocation: self.optimize_venue_allocation(i),
                }
            })
            .collect()
    }

    fn forecast_volume_profile(&self) -> Vec<f64> {
        // Use historical data and market predictors
        let mut profile = Vec::new();
        for hour in 0..24 {
            let historical_avg = self.get_historical_volume_for_hour(hour);
            let market_adjustment = self.get_market_condition_multiplier();
            profile.push(historical_avg * market_adjustment);
        }
        profile
    }
}
```

### Advanced Features

#### Adaptive Participation
```rust
pub struct AdaptiveParams {
    pub market_impact_threshold: f64,  // Slow down if impact too high
    pub rebalance_frequency: i64,      // How often to reassess schedule
    pub liquidity_seeking_mode: bool,  // Prefer high-liquidity venues
    pub volatility_adjustment: f64,    // Adjust for price volatility
    pub news_impact_detection: bool,   // Pause during news events
}
```

#### Market Condition Detection
```rust
impl VWAPOrder {
    pub fn assess_market_conditions(&self) -> MarketCondition {
        let volatility = self.calculate_recent_volatility();
        let liquidity = self.assess_current_liquidity();
        let momentum = self.detect_price_momentum();

        match (volatility, liquidity, momentum) {
            (High, Low, _) => MarketCondition::Cautious,
            (Low, High, _) => MarketCondition::Aggressive,
            (_, _, Strong) => MarketCondition::Momentum,
            _ => MarketCondition::Normal,
        }
    }
}
```

## Intent-Based Execution

### Intent System Overview

Intent-based execution allows traders to specify desired outcomes rather than explicit execution paths, enabling solver competition for optimal execution.

### Intent Structure

```rust
pub struct TradingIntent {
    pub intent_id: [u8; 32],
    pub trader: Pubkey,
    pub intent_type: IntentType,
    pub constraints: Vec<Constraint>,
    pub solver_competition_period: i64,
    pub max_solver_reward: u64,
    pub privacy_requirements: PrivacyRequirements,
    pub deadline: i64,
}

pub enum IntentType {
    BestExecution {
        token_in: Pubkey,
        token_out: Pubkey,
        amount: u64,
        min_output: u64,
    },
    LiquidityProvision {
        pool: Pubkey,
        range: PriceRange,
        amount: u64,
    },
    Arbitrage {
        paths: Vec<TradePath>,
        min_profit: u64,
        max_capital: u64,
    },
    Portfolio {
        target_allocations: Vec<(Pubkey, f64)>,
        rebalance_threshold: f64,
    },
    Custom {
        serialized_logic: Vec<u8>,
        validation_program: Pubkey,
    },
}
```

### Constraint System

```rust
pub enum Constraint {
    MaxSlippage(u16),                    // Basis points
    PreferredVenues(Vec<Pubkey>),        // Venue preferences
    ExcludedVenues(Vec<Pubkey>),         // Venues to avoid
    TimeConstraint(i64, i64),            // Start and end times
    PriceLimit(u64),                     // Limit price
    MinFillSize(u64),                    // Minimum partial fill
    MaxMarketImpact(f64),               // Maximum market impact
    ComplianceRule(ComplianceType),      // Regulatory requirements
    PrivacyLevel(PrivacyRequirement),    // Privacy constraints
    GasLimit(u64),                       // Maximum gas/compute cost
}
```

### Solver Network

#### Solver Registration
```rust
pub struct Solver {
    pub solver_id: Pubkey,
    pub reputation_score: u32,
    pub specializations: Vec<IntentType>,
    pub performance_metrics: SolverMetrics,
    pub stake_amount: u64,
    pub slash_conditions: Vec<SlashCondition>,
}

pub struct SolverMetrics {
    pub success_rate: f64,
    pub average_execution_quality: f64,
    pub speed_percentile: u8,
    pub total_volume_solved: u64,
    pub uptime_percentage: f64,
}
```

#### Solution Submission
```rust
pub struct Solution {
    pub solution_id: [u8; 32],
    pub solver: Pubkey,
    pub intent_id: [u8; 32],
    pub execution_plan: ExecutionPlan,
    pub expected_output: u64,
    pub execution_cost: u64,
    pub confidence_score: f64,
    pub proof_of_execution: Option<Vec<u8>>,
}

pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub total_gas_estimate: u64,
    pub expected_completion_time: i64,
    pub fallback_plans: Vec<FallbackPlan>,
}
```

#### Solution Selection
```rust
impl IntentSolver {
    pub fn evaluate_solutions(&self, solutions: Vec<Solution>) -> Solution {
        let mut scored_solutions: Vec<(Solution, f64)> = solutions
            .into_iter()
            .map(|solution| {
                let score = self.calculate_solution_score(&solution);
                (solution, score)
            })
            .collect();

        scored_solutions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored_solutions.into_iter().next().unwrap().0
    }

    fn calculate_solution_score(&self, solution: &Solution) -> f64 {
        let output_score = solution.expected_output as f64;
        let cost_score = 1.0 / (solution.execution_cost as f64 + 1.0);
        let reputation_score = self.get_solver_reputation(solution.solver);
        let speed_score = 1.0 / (solution.expected_completion_time as f64 + 1.0);

        // Weighted combination
        0.4 * output_score + 0.2 * cost_score + 0.2 * reputation_score + 0.2 * speed_score
    }
}
```

## Smart Order Routing

### Multi-Venue Optimization

```rust
pub struct SmartRouter {
    pub available_venues: Vec<VenueInfo>,
    pub routing_algorithm: RoutingAlgorithm,
    pub route_optimization: OptimizationStrategy,
}

pub struct VenueInfo {
    pub venue_id: Pubkey,
    pub venue_type: VenueType,
    pub liquidity_depth: LiquidityDepth,
    pub fee_structure: FeeStructure,
    pub latency_profile: LatencyProfile,
    pub reliability_score: f64,
}

impl SmartRouter {
    pub fn find_optimal_route(
        &self,
        token_in: Pubkey,
        token_out: Pubkey,
        amount: u64
    ) -> OptimalRoute {
        let all_paths = self.enumerate_possible_paths(token_in, token_out);
        let viable_paths = self.filter_viable_paths(all_paths, amount);

        let mut best_route = None;
        let mut best_score = 0.0;

        for path in viable_paths {
            let route = self.calculate_route_execution(path, amount);
            let score = self.score_route(&route);

            if score > best_score {
                best_score = score;
                best_route = Some(route);
            }
        }

        best_route.expect("No viable route found")
    }

    pub fn split_order_across_venues(
        &self,
        amount: u64,
        venues: Vec<VenueInfo>
    ) -> Vec<VenueSplit> {
        // Implement intelligent order splitting algorithm
        venues
            .iter()
            .map(|venue| {
                let allocation = self.calculate_venue_allocation(venue, amount);
                VenueSplit {
                    venue_id: venue.venue_id,
                    amount: allocation,
                    expected_slippage: self.estimate_slippage(venue, allocation),
                    execution_priority: self.calculate_priority(venue),
                }
            })
            .collect()
    }
}
```

### Route Optimization Strategies

#### Liquidity-Based Routing
```rust
pub fn optimize_for_liquidity(venues: &[VenueInfo], amount: u64) -> Vec<VenueSplit> {
    let total_liquidity: u64 = venues.iter().map(|v| v.liquidity_depth.available).sum();

    venues
        .iter()
        .map(|venue| {
            let allocation_ratio = venue.liquidity_depth.available as f64 / total_liquidity as f64;
            let allocated_amount = (amount as f64 * allocation_ratio) as u64;

            VenueSplit {
                venue_id: venue.venue_id,
                amount: allocated_amount,
                expected_slippage: calculate_slippage(venue, allocated_amount),
                execution_priority: 1, // Execute simultaneously
            }
        })
        .collect()
}
```

#### Cost-Minimization Routing
```rust
pub fn optimize_for_cost(venues: &[VenueInfo], amount: u64) -> OptimalRoute {
    let mut dp = vec![f64::INFINITY; amount as usize + 1];
    dp[0] = 0.0;

    for i in 1..=amount as usize {
        for venue in venues {
            if venue.liquidity_depth.available >= i as u64 {
                let cost = calculate_execution_cost(venue, i as u64);
                dp[i] = dp[i].min(cost);
            }
        }
    }

    // Reconstruct optimal route
    reconstruct_route(&dp, venues, amount)
}
```

## Performance Monitoring

### Execution Quality Metrics

```rust
pub struct ExecutionReport {
    pub order_id: [u8; 32],
    pub algorithm_type: AlgorithmType,
    pub total_amount: u64,
    pub executed_amount: u64,
    pub average_execution_price: f64,
    pub benchmark_price: f64,
    pub total_slippage: f64,
    pub execution_time: i64,
    pub venue_breakdown: Vec<VenueExecution>,
    pub market_impact: f64,
    pub implementation_shortfall: f64,
}

pub struct VenueExecution {
    pub venue_id: Pubkey,
    pub amount_executed: u64,
    pub average_price: f64,
    pub fees_paid: u64,
    pub execution_latency: u64,
}
```

### Benchmark Comparisons

```rust
impl ExecutionReport {
    pub fn calculate_implementation_shortfall(&self) -> f64 {
        let arrival_price = self.benchmark_price;
        let execution_price = self.average_execution_price;
        let decision_price = arrival_price; // Simplified

        let timing_cost = execution_price - decision_price;
        let market_impact_cost = self.market_impact;
        let opportunity_cost = self.calculate_opportunity_cost();

        timing_cost + market_impact_cost + opportunity_cost
    }

    pub fn calculate_price_improvement(&self) -> f64 {
        let reference_price = self.get_reference_price(); // Market price at order time
        (self.average_execution_price - reference_price) / reference_price
    }
}
```

## API Reference

### TWAP Operations

```typescript
interface TWAPExecutor {
  submitOrder(config: TWAPConfig): Promise<OrderId>;
  updateParameters(orderId: OrderId, params: Partial<TWAPConfig>): Promise<void>;
  cancelOrder(orderId: OrderId): Promise<TxSignature>;
  getExecutionSchedule(orderId: OrderId): Promise<ExecutionSchedule>;
  getProgressReport(orderId: OrderId): Promise<TWAPReport>;
}
```

### VWAP Operations

```typescript
interface VWAPExecutor {
  submitOrder(config: VWAPConfig): Promise<OrderId>;
  getVolumeProfile(tokenPair: TokenPair): Promise<VolumeProfile>;
  updateParticipationRate(orderId: OrderId, rate: number): Promise<void>;
  getExecutionReport(orderId: OrderId): Promise<VWAPReport>;
}
```

### Intent System

```typescript
interface IntentSystem {
  submitIntent(intent: TradingIntent): Promise<IntentId>;
  registerSolver(solverInfo: SolverRegistration): Promise<SolverId>;
  submitSolution(solution: Solution): Promise<SolutionId>;
  getIntentStatus(intentId: IntentId): Promise<IntentStatus>;
  getSolverMetrics(solverId: SolverId): Promise<SolverMetrics>;
}
```

## Best Practices

### TWAP Execution
1. **Time Window Selection**: Consider market hours and liquidity patterns
2. **Split Configuration**: Balance between market impact and completion time
3. **Randomization**: Use appropriate randomness to avoid detection
4. **Monitoring**: Track execution progress and adjust if needed

### VWAP Execution
1. **Volume Forecasting**: Use accurate historical volume data
2. **Participation Limits**: Don't exceed comfortable market participation rates
3. **Adaptive Parameters**: Enable dynamic adjustment for changing conditions
4. **Venue Selection**: Prefer high-liquidity venues during peak times

### Intent-Based Trading
1. **Clear Constraints**: Specify realistic and achievable constraints
2. **Solver Selection**: Consider solver reputation and specialization
3. **Deadline Management**: Allow sufficient time for solver competition
4. **Privacy Trade-offs**: Balance privacy requirements with execution quality

### Smart Routing
1. **Real-time Data**: Ensure liquidity and price data is current
2. **Fallback Routes**: Always have backup execution paths
3. **Cost Analysis**: Consider all costs including fees and market impact
4. **Venue Diversification**: Don't over-rely on single venues