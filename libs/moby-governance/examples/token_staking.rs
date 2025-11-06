//! Example: Token staking and governance power management

use moby_governance::{
    system::{GovernanceSystem, ParticipationLevel, PrivacyLevel},
    tokens::{TokenAmount, StakingStatus},
    proposals::{ProposalType, ProposalPriority},
    voting::VoteType,
    error::GovernanceResult,
};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> GovernanceResult<()> {
    println!("🪙 Moby Governance - Token Staking Example");
    println!("==========================================\n");

    // Initialize governance system
    let mut governance = GovernanceSystem::new();
    governance.initialize(Default::default()).await?;

    println!("✅ Governance system initialized\n");

    // Register participants with different token strategies
    println!("👥 Registering participants with diverse token strategies...");

    let participants = vec![
        ("long_term_whale", 50_000_000, ParticipationLevel::Governor, "Long-term whale investor"),
        ("active_trader", 20_000_000, ParticipationLevel::Proposer, "Active whale trader"),
        ("institutional_staker", 30_000_000, ParticipationLevel::Proposer, "Institutional staker"),
        ("yield_farmer", 15_000_000, ParticipationLevel::Voter, "Yield farming specialist"),
        ("governance_enthusiast", 5_000_000, ParticipationLevel::Reviewer, "Governance participant"),
        ("casual_holder", 2_000_000, ParticipationLevel::Voter, "Casual token holder"),
    ];

    for (address, tokens, level, description) in &participants {
        governance.register_participant(
            address.to_string(),
            *tokens,
            level.clone(),
            PrivacyLevel::Public,
        ).await?;
        println!("   ✅ {} - {} tokens ({})", description, tokens, address);
    }

    println!("\n📊 Initial token distribution:");
    let token_stats = governance.token_system.get_token_statistics();
    println!("   Total balance: {} tokens", token_stats.total_balance);
    println!("   Total holders: {}", token_stats.total_holders);

    // Scenario 1: Create Staking Pools
    println!("\n🏊 Scenario 1: Creating Staking Pools");
    println!("====================================");

    // Create different staking pools with varying terms
    println!("🏗️ Creating staking pools...");

    // Short-term pool for active traders
    let short_term_pool = governance.token_system.create_staking_pool(
        "Active Trader Pool".to_string(),
        Decimal::new(8, 0), // 8% APY
        chrono::Duration::days(30),   // Min 30 days
        chrono::Duration::days(90),   // Max 90 days
        chrono::Duration::days(7),    // 7-day unlock period
        100_000_000, // 100M capacity
        HashMap::from([
            ("pool_type".to_string(), "short_term".to_string()),
            ("target_audience".to_string(), "active_traders".to_string()),
        ]),
    ).await?;

    // Medium-term pool for institutions
    let medium_term_pool = governance.token_system.create_staking_pool(
        "Institutional Pool".to_string(),
        Decimal::new(12, 0), // 12% APY
        chrono::Duration::days(90),   // Min 90 days
        chrono::Duration::days(365),  // Max 1 year
        chrono::Duration::days(14),   // 14-day unlock period
        200_000_000, // 200M capacity
        HashMap::from([
            ("pool_type".to_string(), "medium_term".to_string()),
            ("target_audience".to_string(), "institutions".to_string()),
        ]),
    ).await?;

    // Long-term pool for governance participation
    let long_term_pool = governance.token_system.create_staking_pool(
        "Governance Participation Pool".to_string(),
        Decimal::new(18, 0), // 18% APY
        chrono::Duration::days(365),  // Min 1 year
        chrono::Duration::days(1095), // Max 3 years
        chrono::Duration::days(30),   // 30-day unlock period
        500_000_000, // 500M capacity
        HashMap::from([
            ("pool_type".to_string(), "long_term".to_string()),
            ("governance_bonus".to_string(), "enabled".to_string()),
        ]),
    ).await?;

    println!("   ✅ Short-term pool: {} (8% APY)", short_term_pool);
    println!("   ✅ Medium-term pool: {} (12% APY)", medium_term_pool);
    println!("   ✅ Long-term pool: {} (18% APY)", long_term_pool);

    // Scenario 2: Participants Stake Tokens
    println!("\n🔒 Scenario 2: Token Staking Strategy");
    println!("=====================================");

    // Long-term whale stakes for maximum governance power
    println!("🐋 Long-term whale staking strategy...");

    let whale_stake_1 = governance.token_system.stake(
        "long_term_whale",
        &long_term_pool,
        40_000_000, // 40M tokens
        chrono::Duration::days(730), // 2 years
    ).await?;
    println!("   ✅ Staked 40M tokens for 2 years (position: {})", whale_stake_1);

    // Lock additional tokens for extra governance power
    governance.token_system.lock_tokens(
        "long_term_whale",
        5_000_000, // 5M tokens
        chrono::Duration::days(365), // 1 year lock
    ).await?;
    println!("   ✅ Locked 5M tokens for governance bonus");

    // Active trader uses short-term pool
    println!("\n🔄 Active trader staking strategy...");

    let trader_stake_1 = governance.token_system.stake(
        "active_trader",
        &short_term_pool,
        10_000_000, // 10M tokens
        chrono::Duration::days(60), // 60 days
    ).await?;

    let trader_stake_2 = governance.token_system.stake(
        "active_trader",
        &medium_term_pool,
        8_000_000, // 8M tokens
        chrono::Duration::days(180), // 180 days
    ).await?;
    println!("   ✅ Split staking: 10M short-term, 8M medium-term");

    // Institutional staker goes for balanced approach
    println!("\n🏛️ Institutional staking strategy...");

    let institutional_stake = governance.token_system.stake(
        "institutional_staker",
        &medium_term_pool,
        25_000_000, // 25M tokens
        chrono::Duration::days(365), // 1 year
    ).await?;
    println!("   ✅ Institutional stake: 25M tokens for 1 year");

    // Yield farmer maximizes returns
    println!("\n🌾 Yield farmer staking strategy...");

    let yield_stake = governance.token_system.stake(
        "yield_farmer",
        &long_term_pool,
        12_000_000, // 12M tokens
        chrono::Duration::days(1095), // 3 years maximum
    ).await?;
    println!("   ✅ Maximum yield stake: 12M tokens for 3 years");

    // Governance enthusiast stakes for participation
    println!("\n🗳️ Governance enthusiast strategy...");

    let governance_stake = governance.token_system.stake(
        "governance_enthusiast",
        &long_term_pool,
        3_000_000, // 3M tokens
        chrono::Duration::days(365), // 1 year
    ).await?;

    // Lock remaining tokens for maximum voting power
    governance.token_system.lock_tokens(
        "governance_enthusiast",
        1_500_000, // 1.5M tokens
        chrono::Duration::days(180), // 6 months
    ).await?;
    println!("   ✅ Governance-focused: 3M staked + 1.5M locked");

    // Show updated staking statistics
    println!("\n📊 Staking Pool Status:");
    let pools = governance.token_system.get_active_pools();
    for pool in pools {
        println!("   {} - {} tokens staked ({}% capacity)",
                pool.name,
                pool.total_staked,
                (pool.total_staked * 100) / pool.capacity);
    }

    // Scenario 3: Voting Power Analysis
    println!("\n⚡ Scenario 3: Voting Power Analysis");
    println!("===================================");

    println!("📊 Voting power after staking and locking:");
    for (address, _, _, description) in &participants {
        let voting_power = governance.token_system.calculate_voting_power(address).await;
        let holder = governance.token_system.get_holder(address).unwrap();

        println!("   {}: {} voting power", description, voting_power);
        println!("     - Available: {} tokens", holder.balance);
        println!("     - Staked: {} tokens", holder.staked_balance);
        println!("     - Locked: {} tokens", holder.locked_balance);
        println!("     - Multiplier: {}", holder.voting_multiplier);
    }

    // Scenario 4: Governance Proposal with Staking Requirements
    println!("\n🏛️ Scenario 4: Governance with Staking Power");
    println!("=============================================");

    // Create a proposal that benefits stakers
    let staking_proposal = governance.create_proposal(
        ProposalType::ParameterUpdate {
            parameter: "staking_rewards_multiplier".to_string(),
            old_value: "1.0".to_string(),
            new_value: "1.25".to_string(),
        },
        "Increase Staking Rewards by 25%".to_string(),
        "Increase staking rewards multiplier to incentivize longer-term token holding and increase governance participation among committed community members.".to_string(),
        "governance_enthusiast".to_string(),
        Some(chrono::Duration::days(7)),
        PrivacyLevel::Public,
    ).await?;

    println!("📋 Created staking rewards proposal");

    // Vote with different staking-based voting powers
    let staking_votes = vec![
        ("long_term_whale", VoteType::For, "Higher rewards benefit long-term staking"),
        ("institutional_staker", VoteType::For, "Supports institutional staking strategy"),
        ("yield_farmer", VoteType::For, "Maximizes yield farming returns"),
        ("governance_enthusiast", VoteType::For, "Rewards governance participation"),
        ("active_trader", VoteType::Against, "Prefers trading over staking incentives"),
        ("casual_holder", VoteType::Abstain, "Need more information on impact"),
    ];

    println!("\n🗳️ Voting with staking-enhanced power:");
    for (voter, vote_type, justification) in staking_votes {
        let voting_power = governance.token_system.calculate_voting_power(voter).await;
        governance.vote(
            staking_proposal,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Public,
        ).await?;
        println!("   {} ({} power) voted {:?}: {}",
                voter, voting_power, vote_type, justification);
    }

    // Scenario 5: Reward Processing and Distribution
    println!("\n💰 Scenario 5: Reward Processing");
    println!("================================");

    // Process rewards for all staking positions
    println!("⏰ Processing staking rewards...");
    let processed_rewards = governance.token_system.process_reward_calculations().await?;
    println!("   ✅ Processed rewards for {} positions", processed_rewards);

    // Show individual position details
    println!("\n📈 Staking position details:");
    for (address, _, _, description) in &participants {
        let positions = governance.token_system.get_positions_by_staker(address);
        if !positions.is_empty() {
            println!("   {}:", description);
            for position in positions {
                let status_emoji = match position.status {
                    StakingStatus::Active => "🟢",
                    StakingStatus::Unstaking => "🟡",
                    StakingStatus::Withdrawable => "🔵",
                    StakingStatus::Withdrawn => "⚪",
                    StakingStatus::Slashed => "🔴",
                };
                println!("     {} {} tokens - {} rewards ({})",
                        status_emoji,
                        position.amount,
                        position.accrued_rewards,
                        position.status as u8);
            }
        }
    }

    // Scenario 6: Unstaking and Withdrawal Process
    println!("\n🔓 Scenario 6: Unstaking Process");
    println!("===============================");

    // Active trader decides to unstake from short-term pool
    println!("🔄 Active trader initiating unstaking...");

    // Request unstaking (simulate position maturity)
    let trader_position = governance.token_system.get_positions_by_staker("active_trader");
    if let Some(position) = trader_position.first() {
        // In a real scenario, we'd check if the position has matured
        println!("   📋 Position {} eligible for unstaking", position.id);

        // Note: The actual unstaking would require the position to be mature
        // This is a simplified example showing the process
        println!("   ⏳ Unstaking request would initiate 7-day lockup period");
        println!("   💰 Estimated rewards to be claimed: {} tokens", position.accrued_rewards);
    }

    // Scenario 7: Staking Impact on Governance
    println!("\n🎯 Scenario 7: Staking Impact Analysis");
    println!("=====================================");

    // Calculate total staking impact on governance
    let total_staked = governance.token_system.get_token_statistics().total_staked;
    let total_locked = governance.token_system.get_token_statistics().total_locked;
    let total_circulating = governance.token_system.get_token_statistics().total_balance;

    println!("📊 Governance impact metrics:");
    println!("   Total staked: {} tokens ({:.1}% of circulation)",
            total_staked,
            (total_staked as f64 / total_circulating as f64) * 100.0);
    println!("   Total locked: {} tokens ({:.1}% of circulation)",
            total_locked,
            (total_locked as f64 / total_circulating as f64) * 100.0);

    let staking_participation = (total_staked + total_locked) as f64 / total_circulating as f64;
    println!("   Staking participation rate: {:.1}%", staking_participation * 100.0);

    // Show governance concentration
    println!("\n🏆 Top governance power holders:");
    let mut governance_powers = Vec::new();
    for (address, _, _, description) in &participants {
        let power = governance.token_system.calculate_voting_power(address).await;
        governance_powers.push((description, power));
    }
    governance_powers.sort_by(|a, b| b.1.cmp(&a.1));

    for (i, (description, power)) in governance_powers.iter().enumerate() {
        let percentage = (*power as f64 / governance_powers.iter().map(|(_, p)| *p).sum::<u64>() as f64) * 100.0;
        println!("   {}. {} - {} power ({:.1}%)", i + 1, description, power, percentage);
    }

    // Final statistics
    println!("\n📈 Final Token and Staking Statistics");
    println!("====================================");

    let final_token_stats = governance.token_system.get_token_statistics();
    println!("Token distribution:");
    println!("  - Available: {} tokens", final_token_stats.total_balance);
    println!("  - Staked: {} tokens", final_token_stats.total_staked);
    println!("  - Locked: {} tokens", final_token_stats.total_locked);
    println!("  - Total circulation: {} tokens",
            final_token_stats.total_balance + final_token_stats.total_staked + final_token_stats.total_locked);

    println!("\nStaking infrastructure:");
    println!("  - Active pools: {}", final_token_stats.active_pools);
    println!("  - Total positions: {}", final_token_stats.total_positions);
    println!("  - Active positions: {}", final_token_stats.active_positions);

    println!("\nGovernance metrics:");
    let governance_stats = governance.get_governance_statistics().await;
    println!("  - Total voting power: {}", governance_stats.total_voting_power);
    println!("  - Staked voting power: {}", final_token_stats.total_staked + final_token_stats.total_locked);
    println!("  - Governance participation: {:.1}%",
            (governance_stats.staked_tokens as f64 / governance_stats.total_voting_power as f64) * 100.0);

    println!("\n🎉 Token staking example completed successfully!");
    println!("   • Created multiple staking pools with different terms");
    println!("   • Demonstrated various staking strategies");
    println!("   • Showed voting power enhancement through staking");
    println!("   • Illustrated reward processing and distribution");
    println!("   • Analyzed governance impact of staking mechanisms");

    Ok(())
}