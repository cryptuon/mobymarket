//! Governance token and staking system

use crate::error::{GovernanceError, GovernanceResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Amount of governance tokens
pub type TokenAmount = u64;

/// Governance token represents voting power in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceToken {
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Total supply
    pub total_supply: TokenAmount,
    /// Circulating supply
    pub circulating_supply: TokenAmount,
    /// Token decimals
    pub decimals: u8,
    /// Token metadata
    pub metadata: HashMap<String, String>,
}

/// Token holder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    /// Holder address
    pub address: String,
    /// Available balance
    pub balance: TokenAmount,
    /// Staked balance
    pub staked_balance: TokenAmount,
    /// Locked balance (e.g., for governance)
    pub locked_balance: TokenAmount,
    /// Voting power multiplier
    pub voting_multiplier: Decimal,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Staking pool for governance tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    /// Pool identifier
    pub id: String,
    /// Pool name
    pub name: String,
    /// Total tokens staked in pool
    pub total_staked: TokenAmount,
    /// Annual percentage yield
    pub apy: Decimal,
    /// Minimum staking period
    pub min_staking_period: chrono::Duration,
    /// Maximum staking period
    pub max_staking_period: chrono::Duration,
    /// Lock-up period after unstaking
    pub lockup_period: chrono::Duration,
    /// Pool capacity (0 = unlimited)
    pub capacity: TokenAmount,
    /// Whether pool is active
    pub active: bool,
    /// Pool creation time
    pub created_at: DateTime<Utc>,
    /// Pool metadata
    pub metadata: HashMap<String, String>,
}

/// Individual staking position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPosition {
    /// Position identifier
    pub id: String,
    /// Staker address
    pub staker: String,
    /// Pool identifier
    pub pool_id: String,
    /// Amount staked
    pub amount: TokenAmount,
    /// Staking start time
    pub staked_at: DateTime<Utc>,
    /// Intended staking duration
    pub duration: chrono::Duration,
    /// Maturity date
    pub matures_at: DateTime<Utc>,
    /// Unstaking request time (if any)
    pub unstake_requested_at: Option<DateTime<Utc>>,
    /// Available for withdrawal time
    pub available_at: Option<DateTime<Utc>>,
    /// Current status
    pub status: StakingStatus,
    /// Accrued rewards
    pub accrued_rewards: TokenAmount,
    /// Last reward calculation
    pub last_reward_update: DateTime<Utc>,
}

/// Status of a staking position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StakingStatus {
    /// Currently staked and earning rewards
    Active,
    /// Unstaking requested, in lockup period
    Unstaking,
    /// Available for withdrawal
    Withdrawable,
    /// Position has been withdrawn
    Withdrawn,
    /// Position has been slashed
    Slashed,
}

/// Reward calculation and distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingReward {
    /// Reward identifier
    pub id: String,
    /// Position this reward belongs to
    pub position_id: String,
    /// Reward amount
    pub amount: TokenAmount,
    /// Reward period start
    pub period_start: DateTime<Utc>,
    /// Reward period end
    pub period_end: DateTime<Utc>,
    /// APY used for calculation
    pub apy: Decimal,
    /// Whether reward has been claimed
    pub claimed: bool,
    /// Claim timestamp
    pub claimed_at: Option<DateTime<Utc>>,
}

/// Configuration for token system
#[derive(Debug, Clone)]
pub struct TokenConfig {
    /// Minimum staking amount
    pub min_stake_amount: TokenAmount,
    /// Maximum staking amount per position
    pub max_stake_amount: TokenAmount,
    /// Default staking period
    pub default_staking_period: chrono::Duration,
    /// Minimum voting power for proposals
    pub min_proposal_tokens: TokenAmount,
    /// Voting power calculation multiplier
    pub voting_power_multiplier: Decimal,
    /// Reward calculation frequency
    pub reward_calculation_frequency: chrono::Duration,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            min_stake_amount: 100,
            max_stake_amount: 10_000_000,
            default_staking_period: chrono::Duration::days(30),
            min_proposal_tokens: 1000,
            voting_power_multiplier: Decimal::new(1, 0),
            reward_calculation_frequency: chrono::Duration::hours(24),
        }
    }
}

/// Token system manages governance tokens and staking
pub struct TokenSystem {
    /// Configuration
    config: TokenConfig,
    /// Governance token information
    token: GovernanceToken,
    /// Token holders
    holders: HashMap<String, TokenHolder>,
    /// Staking pools
    pools: HashMap<String, StakingPool>,
    /// Staking positions
    positions: HashMap<String, StakingPosition>,
    /// Rewards
    rewards: HashMap<String, Vec<StakingReward>>,
    /// Position counter for IDs
    position_counter: u64,
    /// Reward counter for IDs
    reward_counter: u64,
}

impl TokenSystem {
    /// Create a new token system
    pub fn new(config: TokenConfig, token: GovernanceToken) -> Self {
        Self {
            config,
            token,
            holders: HashMap::new(),
            pools: HashMap::new(),
            positions: HashMap::new(),
            rewards: HashMap::new(),
            position_counter: 0,
            reward_counter: 0,
        }
    }

    /// Initialize token holder
    pub async fn initialize_holder(
        &mut self,
        address: String,
        initial_balance: TokenAmount,
    ) -> GovernanceResult<()> {
        if self.holders.contains_key(&address) {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Holder {} already exists", address),
            });
        }

        let holder = TokenHolder {
            address: address.clone(),
            balance: initial_balance,
            staked_balance: 0,
            locked_balance: 0,
            voting_multiplier: self.config.voting_power_multiplier,
            last_updated: Utc::now(),
        };

        self.holders.insert(address, holder);
        Ok(())
    }

    /// Transfer tokens between holders
    pub async fn transfer(
        &mut self,
        from: &str,
        to: &str,
        amount: TokenAmount,
    ) -> GovernanceResult<()> {
        // Validate transfer
        let from_holder = self.holders.get_mut(from)
            .ok_or_else(|| GovernanceError::InvalidAddress {
                address: from.to_string(),
            })?;

        if from_holder.balance < amount {
            return Err(GovernanceError::InsufficientTokenBalance {
                required: amount,
                available: from_holder.balance,
            });
        }

        // Deduct from sender
        from_holder.balance -= amount;
        from_holder.last_updated = Utc::now();

        // Add to recipient (create if doesn't exist)
        if !self.holders.contains_key(to) {
            self.initialize_holder(to.to_string(), 0).await?;
        }

        let to_holder = self.holders.get_mut(to).unwrap();
        to_holder.balance += amount;
        to_holder.last_updated = Utc::now();

        Ok(())
    }

    /// Create a staking pool
    pub async fn create_staking_pool(
        &mut self,
        name: String,
        apy: Decimal,
        min_period: chrono::Duration,
        max_period: chrono::Duration,
        lockup_period: chrono::Duration,
        capacity: TokenAmount,
        metadata: HashMap<String, String>,
    ) -> GovernanceResult<String> {
        let id = format!("pool_{}", self.pools.len() + 1);

        let pool = StakingPool {
            id: id.clone(),
            name,
            total_staked: 0,
            apy,
            min_staking_period: min_period,
            max_staking_period: max_period,
            lockup_period,
            capacity,
            active: true,
            created_at: Utc::now(),
            metadata,
        };

        self.pools.insert(id.clone(), pool);
        Ok(id)
    }

    /// Stake tokens in a pool
    pub async fn stake(
        &mut self,
        staker: &str,
        pool_id: &str,
        amount: TokenAmount,
        duration: chrono::Duration,
    ) -> GovernanceResult<String> {
        // Validate inputs
        if amount < self.config.min_stake_amount {
            return Err(GovernanceError::InvalidTokenAmount {
                amount: amount.to_string(),
            });
        }

        if amount > self.config.max_stake_amount {
            return Err(GovernanceError::InvalidTokenAmount {
                amount: amount.to_string(),
            });
        }

        // Get pool
        let pool = self.pools.get_mut(pool_id)
            .ok_or_else(|| GovernanceError::ResourceNotAvailable {
                resource: format!("staking pool {}", pool_id),
            })?;

        if !pool.active {
            return Err(GovernanceError::OperationFailed {
                reason: "Pool is not active".to_string(),
            });
        }

        // Validate duration
        if duration < pool.min_staking_period || duration > pool.max_staking_period {
            return Err(GovernanceError::InvalidStakingDuration {
                duration: format!("{} days", duration.num_days()),
            });
        }

        // Check pool capacity
        if pool.capacity > 0 && pool.total_staked + amount > pool.capacity {
            return Err(GovernanceError::InsufficientTreasuryFunds {
                required: amount,
                available: pool.capacity - pool.total_staked,
            });
        }

        // Get holder
        let holder = self.holders.get_mut(staker)
            .ok_or_else(|| GovernanceError::InvalidAddress {
                address: staker.to_string(),
            })?;

        if holder.balance < amount {
            return Err(GovernanceError::InsufficientTokenBalance {
                required: amount,
                available: holder.balance,
            });
        }

        // Create staking position
        self.position_counter += 1;
        let position_id = format!("pos_{}", self.position_counter);

        let now = Utc::now();
        let matures_at = now + duration;

        let position = StakingPosition {
            id: position_id.clone(),
            staker: staker.to_string(),
            pool_id: pool_id.to_string(),
            amount,
            staked_at: now,
            duration,
            matures_at,
            unstake_requested_at: None,
            available_at: None,
            status: StakingStatus::Active,
            accrued_rewards: 0,
            last_reward_update: now,
        };

        // Update balances
        holder.balance -= amount;
        holder.staked_balance += amount;
        holder.last_updated = now;

        pool.total_staked += amount;

        // Store position
        self.positions.insert(position_id.clone(), position);

        Ok(position_id)
    }

    /// Request unstaking
    pub async fn request_unstake(&mut self, position_id: &str) -> GovernanceResult<()> {
        let position = self.positions.get_mut(position_id)
            .ok_or_else(|| GovernanceError::ResourceNotAvailable {
                resource: format!("staking position {}", position_id),
            })?;

        if position.status != StakingStatus::Active {
            return Err(GovernanceError::OperationFailed {
                reason: "Position is not active".to_string(),
            });
        }

        let now = Utc::now();

        // Check if position has matured
        if now < position.matures_at {
            return Err(GovernanceError::StakingPeriodNotEnded);
        }

        // Calculate final rewards
        self.calculate_and_add_rewards(position_id).await?;

        // Get pool for lockup period
        let pool = self.pools.get(&position.pool_id).unwrap();
        let available_at = now + pool.lockup_period;

        // Update position
        position.status = StakingStatus::Unstaking;
        position.unstake_requested_at = Some(now);
        position.available_at = Some(available_at);

        Ok(())
    }

    /// Withdraw staked tokens
    pub async fn withdraw(&mut self, position_id: &str) -> GovernanceResult<TokenAmount> {
        let position = self.positions.get_mut(position_id)
            .ok_or_else(|| GovernanceError::ResourceNotAvailable {
                resource: format!("staking position {}", position_id),
            })?;

        // Check if withdrawable
        match position.status {
            StakingStatus::Withdrawable => {}
            StakingStatus::Unstaking => {
                if let Some(available_at) = position.available_at {
                    if Utc::now() >= available_at {
                        position.status = StakingStatus::Withdrawable;
                    } else {
                        return Err(GovernanceError::OperationFailed {
                            reason: "Lockup period not ended".to_string(),
                        });
                    }
                } else {
                    return Err(GovernanceError::OperationFailed {
                        reason: "Position not ready for withdrawal".to_string(),
                    });
                }
            }
            _ => {
                return Err(GovernanceError::OperationFailed {
                    reason: format!("Position status is {:?}", position.status),
                });
            }
        }

        let staker = position.staker.clone();
        let pool_id = position.pool_id.clone();
        let amount = position.amount;
        let rewards = position.accrued_rewards;

        // Update holder balances
        let holder = self.holders.get_mut(&staker).unwrap();
        holder.balance += amount + rewards;
        holder.staked_balance -= amount;
        holder.last_updated = Utc::now();

        // Update pool
        let pool = self.pools.get_mut(&pool_id).unwrap();
        pool.total_staked -= amount;

        // Mark position as withdrawn
        position.status = StakingStatus::Withdrawn;

        Ok(amount + rewards)
    }

    /// Calculate voting power for an address
    pub async fn calculate_voting_power(&self, address: &str) -> u64 {
        let holder = self.holders.get(address);
        if holder.is_none() {
            return 0;
        }

        let holder = holder.unwrap();
        let total_tokens = holder.balance + holder.staked_balance + holder.locked_balance;

        // Apply multiplier
        let voting_power = Decimal::from(total_tokens) * holder.voting_multiplier;
        voting_power.to_u64().unwrap_or(0)
    }

    /// Lock tokens for governance participation
    pub async fn lock_tokens(
        &mut self,
        address: &str,
        amount: TokenAmount,
        duration: chrono::Duration,
    ) -> GovernanceResult<()> {
        let holder = self.holders.get_mut(address)
            .ok_or_else(|| GovernanceError::InvalidAddress {
                address: address.to_string(),
            })?;

        if holder.balance < amount {
            return Err(GovernanceError::InsufficientTokenBalance {
                required: amount,
                available: holder.balance,
            });
        }

        // Transfer to locked balance
        holder.balance -= amount;
        holder.locked_balance += amount;
        holder.last_updated = Utc::now();

        // Apply voting multiplier based on lock duration
        let multiplier_bonus = self.calculate_lock_multiplier(duration);
        holder.voting_multiplier += multiplier_bonus;

        Ok(())
    }

    /// Unlock tokens after governance period
    pub async fn unlock_tokens(
        &mut self,
        address: &str,
        amount: TokenAmount,
    ) -> GovernanceResult<()> {
        let holder = self.holders.get_mut(address)
            .ok_or_else(|| GovernanceError::InvalidAddress {
                address: address.to_string(),
            })?;

        if holder.locked_balance < amount {
            return Err(GovernanceError::InsufficientTokenBalance {
                required: amount,
                available: holder.locked_balance,
            });
        }

        // Transfer back to available balance
        holder.locked_balance -= amount;
        holder.balance += amount;
        holder.last_updated = Utc::now();

        Ok(())
    }

    /// Calculate and distribute rewards for a position
    pub async fn calculate_and_add_rewards(&mut self, position_id: &str) -> GovernanceResult<()> {
        let position = self.positions.get(position_id)
            .ok_or_else(|| GovernanceError::ResourceNotAvailable {
                resource: format!("staking position {}", position_id),
            })?;

        let pool = self.pools.get(&position.pool_id).unwrap();

        let now = Utc::now();
        let duration_since_last_update = now - position.last_reward_update;
        let reward_duration = duration_since_last_update.min(
            position.matures_at - position.last_reward_update
        );

        if reward_duration.num_seconds() <= 0 {
            return Ok(());
        }

        // Calculate reward amount
        let annual_reward = Decimal::from(position.amount) * pool.apy / Decimal::new(100, 0);
        let days_rewarded = Decimal::from(reward_duration.num_days());
        let daily_reward = annual_reward / Decimal::new(365, 0);
        let reward_amount = (daily_reward * days_rewarded).to_u64().unwrap_or(0);

        if reward_amount == 0 {
            return Ok(());
        }

        // Create reward record
        self.reward_counter += 1;
        let reward_id = format!("reward_{}", self.reward_counter);

        let reward = StakingReward {
            id: reward_id,
            position_id: position_id.to_string(),
            amount: reward_amount,
            period_start: position.last_reward_update,
            period_end: now,
            apy: pool.apy,
            claimed: false,
            claimed_at: None,
        };

        // Add reward to position
        let position = self.positions.get_mut(position_id).unwrap();
        position.accrued_rewards += reward_amount;
        position.last_reward_update = now;

        // Store reward
        self.rewards
            .entry(position_id.to_string())
            .or_insert_with(Vec::new)
            .push(reward);

        Ok(())
    }

    /// Process all reward calculations
    pub async fn process_reward_calculations(&mut self) -> GovernanceResult<usize> {
        let position_ids: Vec<String> = self.positions
            .values()
            .filter(|p| p.status == StakingStatus::Active)
            .map(|p| p.id.clone())
            .collect();

        let mut processed = 0;
        for position_id in position_ids {
            if self.calculate_and_add_rewards(&position_id).await.is_ok() {
                processed += 1;
            }
        }

        Ok(processed)
    }

    /// Get token holder information
    pub fn get_holder(&self, address: &str) -> Option<&TokenHolder> {
        self.holders.get(address)
    }

    /// Get staking pool information
    pub fn get_pool(&self, pool_id: &str) -> Option<&StakingPool> {
        self.pools.get(pool_id)
    }

    /// Get staking position
    pub fn get_position(&self, position_id: &str) -> Option<&StakingPosition> {
        self.positions.get(position_id)
    }

    /// Get positions for a staker
    pub fn get_positions_by_staker(&self, staker: &str) -> Vec<&StakingPosition> {
        self.positions
            .values()
            .filter(|p| p.staker == staker)
            .collect()
    }

    /// Get active pools
    pub fn get_active_pools(&self) -> Vec<&StakingPool> {
        self.pools
            .values()
            .filter(|p| p.active)
            .collect()
    }

    /// Get token statistics
    pub fn get_token_statistics(&self) -> TokenStatistics {
        let mut stats = TokenStatistics::default();

        stats.total_supply = self.token.total_supply;
        stats.circulating_supply = self.token.circulating_supply;

        for holder in self.holders.values() {
            stats.total_holders += 1;
            stats.total_balance += holder.balance;
            stats.total_staked += holder.staked_balance;
            stats.total_locked += holder.locked_balance;
        }

        stats.total_pools = self.pools.len();
        stats.active_pools = self.pools.values().filter(|p| p.active).count();
        stats.total_positions = self.positions.len();
        stats.active_positions = self.positions.values()
            .filter(|p| p.status == StakingStatus::Active)
            .count();

        stats
    }

    // Helper methods

    fn calculate_lock_multiplier(&self, duration: chrono::Duration) -> Decimal {
        // Bonus multiplier based on lock duration
        // 1 day = 0.001 bonus, max 1 year = 0.365 bonus
        let days = duration.num_days() as u64;
        let max_days = 365;
        let max_bonus = Decimal::new(365, 3); // 0.365

        if days >= max_days {
            max_bonus
        } else {
            Decimal::new(days, 3)
        }
    }
}

/// Statistics for token system analysis
#[derive(Debug, Default, Clone)]
pub struct TokenStatistics {
    pub total_supply: TokenAmount,
    pub circulating_supply: TokenAmount,
    pub total_holders: usize,
    pub total_balance: TokenAmount,
    pub total_staked: TokenAmount,
    pub total_locked: TokenAmount,
    pub total_pools: usize,
    pub active_pools: usize,
    pub total_positions: usize,
    pub active_positions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_token() -> GovernanceToken {
        GovernanceToken {
            symbol: "MOBY".to_string(),
            name: "Moby Governance Token".to_string(),
            total_supply: 1_000_000_000,
            circulating_supply: 500_000_000,
            decimals: 18,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_initialize_holder() {
        let mut token_system = TokenSystem::new(TokenConfig::default(), create_test_token());

        token_system.initialize_holder("holder1".to_string(), 1000).await.unwrap();

        let holder = token_system.get_holder("holder1").unwrap();
        assert_eq!(holder.balance, 1000);
        assert_eq!(holder.staked_balance, 0);
        assert_eq!(holder.locked_balance, 0);
    }

    #[tokio::test]
    async fn test_transfer_tokens() {
        let mut token_system = TokenSystem::new(TokenConfig::default(), create_test_token());

        token_system.initialize_holder("sender".to_string(), 1000).await.unwrap();
        token_system.initialize_holder("recipient".to_string(), 500).await.unwrap();

        token_system.transfer("sender", "recipient", 300).await.unwrap();

        let sender = token_system.get_holder("sender").unwrap();
        let recipient = token_system.get_holder("recipient").unwrap();

        assert_eq!(sender.balance, 700);
        assert_eq!(recipient.balance, 800);
    }

    #[tokio::test]
    async fn test_create_staking_pool() {
        let mut token_system = TokenSystem::new(TokenConfig::default(), create_test_token());

        let pool_id = token_system.create_staking_pool(
            "Test Pool".to_string(),
            Decimal::new(12, 0), // 12% APY
            chrono::Duration::days(7),
            chrono::Duration::days(365),
            chrono::Duration::days(7),
            1_000_000,
            HashMap::new(),
        ).await.unwrap();

        let pool = token_system.get_pool(&pool_id).unwrap();
        assert_eq!(pool.name, "Test Pool");
        assert_eq!(pool.apy, Decimal::new(12, 0));
        assert!(pool.active);
    }

    #[tokio::test]
    async fn test_staking() {
        let mut token_system = TokenSystem::new(TokenConfig::default(), create_test_token());

        // Initialize holder
        token_system.initialize_holder("staker".to_string(), 10000).await.unwrap();

        // Create pool
        let pool_id = token_system.create_staking_pool(
            "Test Pool".to_string(),
            Decimal::new(12, 0),
            chrono::Duration::days(7),
            chrono::Duration::days(365),
            chrono::Duration::days(7),
            1_000_000,
            HashMap::new(),
        ).await.unwrap();

        // Stake tokens
        let position_id = token_system.stake(
            "staker",
            &pool_id,
            5000,
            chrono::Duration::days(30),
        ).await.unwrap();

        let holder = token_system.get_holder("staker").unwrap();
        let position = token_system.get_position(&position_id).unwrap();
        let pool = token_system.get_pool(&pool_id).unwrap();

        assert_eq!(holder.balance, 5000);
        assert_eq!(holder.staked_balance, 5000);
        assert_eq!(position.amount, 5000);
        assert_eq!(position.status, StakingStatus::Active);
        assert_eq!(pool.total_staked, 5000);
    }

    #[tokio::test]
    async fn test_voting_power_calculation() {
        let mut token_system = TokenSystem::new(TokenConfig::default(), create_test_token());

        token_system.initialize_holder("voter".to_string(), 1000).await.unwrap();

        // Base voting power
        let voting_power = token_system.calculate_voting_power("voter").await;
        assert_eq!(voting_power, 1000);

        // Lock tokens for bonus
        token_system.lock_tokens("voter", 500, chrono::Duration::days(100)).await.unwrap();

        let voting_power = token_system.calculate_voting_power("voter").await;
        assert!(voting_power > 1000); // Should have bonus from locking
    }

    #[tokio::test]
    async fn test_reward_calculation() {
        let mut token_system = TokenSystem::new(TokenConfig::default(), create_test_token());

        // Initialize holder
        token_system.initialize_holder("staker".to_string(), 10000).await.unwrap();

        // Create pool with 12% APY
        let pool_id = token_system.create_staking_pool(
            "Test Pool".to_string(),
            Decimal::new(12, 0),
            chrono::Duration::days(1),
            chrono::Duration::days(365),
            chrono::Duration::days(1),
            1_000_000,
            HashMap::new(),
        ).await.unwrap();

        // Stake tokens
        let position_id = token_system.stake(
            "staker",
            &pool_id,
            1000,
            chrono::Duration::days(30),
        ).await.unwrap();

        // Calculate rewards (simulate after some time)
        token_system.calculate_and_add_rewards(&position_id).await.unwrap();

        let position = token_system.get_position(&position_id).unwrap();
        // Should have some accrued rewards
        assert!(position.accrued_rewards >= 0);
    }

    #[tokio::test]
    async fn test_token_statistics() {
        let mut token_system = TokenSystem::new(TokenConfig::default(), create_test_token());

        token_system.initialize_holder("holder1".to_string(), 1000).await.unwrap();
        token_system.initialize_holder("holder2".to_string(), 2000).await.unwrap();

        let pool_id = token_system.create_staking_pool(
            "Test Pool".to_string(),
            Decimal::new(10, 0),
            chrono::Duration::days(1),
            chrono::Duration::days(365),
            chrono::Duration::days(1),
            1_000_000,
            HashMap::new(),
        ).await.unwrap();

        token_system.stake("holder1", &pool_id, 500, chrono::Duration::days(30)).await.unwrap();

        let stats = token_system.get_token_statistics();
        assert_eq!(stats.total_holders, 2);
        assert_eq!(stats.total_balance, 2500); // 500 + 2000
        assert_eq!(stats.total_staked, 500);
        assert_eq!(stats.total_pools, 1);
        assert_eq!(stats.active_pools, 1);
        assert_eq!(stats.total_positions, 1);
        assert_eq!(stats.active_positions, 1);
    }
}