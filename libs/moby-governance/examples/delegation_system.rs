//! Example: Delegation and liquid democracy mechanisms

use moby_governance::{
    delegation::{DelegationPower, DelegationScope},
    proposals::{ProposalType, ProposalPriority},
    system::{GovernanceSystem, ParticipationLevel, PrivacyLevel},
    voting::VoteType,
    error::GovernanceResult,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> GovernanceResult<()> {
    println!("🔗 Moby Governance - Delegation System Example");
    println!("===============================================\n");

    // Initialize governance system
    let mut governance = GovernanceSystem::new();
    governance.initialize(Default::default()).await?;

    println!("✅ Governance system initialized\n");

    // Register participants with diverse profiles
    println!("👥 Registering diverse participant ecosystem...");

    // Whale traders (high voting power, may delegate)
    governance.register_participant(
        "alpha_whale".to_string(),
        25_000_000, // 25M tokens
        ParticipationLevel::Governor,
        PrivacyLevel::Confidential,
    ).await?;

    governance.register_participant(
        "beta_whale".to_string(),
        20_000_000, // 20M tokens
        ParticipationLevel::Proposer,
        PrivacyLevel::Confidential,
    ).await?;

    // Institutional delegates (trusted by multiple entities)
    governance.register_participant(
        "institutional_delegate_1".to_string(),
        5_000_000, // 5M tokens
        ParticipationLevel::Reviewer,
        PrivacyLevel::Public,
    ).await?;

    governance.register_participant(
        "institutional_delegate_2".to_string(),
        3_000_000, // 3M tokens
        ParticipationLevel::Reviewer,
        PrivacyLevel::Public,
    ).await?;

    // Specialized delegates
    governance.register_participant(
        "technical_delegate".to_string(),
        2_000_000, // 2M tokens
        ParticipationLevel::Proposer,
        PrivacyLevel::Public,
    ).await?;

    governance.register_participant(
        "governance_expert".to_string(),
        1_500_000, // 1.5M tokens
        ParticipationLevel::Proposer,
        PrivacyLevel::Public,
    ).await?;

    // Community members (may delegate to experts)
    let community_members = vec![
        ("community_alice", 800_000, ParticipationLevel::Voter),
        ("community_bob", 600_000, ParticipationLevel::Voter),
        ("community_charlie", 400_000, ParticipationLevel::Voter),
        ("community_diana", 300_000, ParticipationLevel::Voter),
        ("community_eve", 250_000, ParticipationLevel::Voter),
        ("retail_trader_1", 150_000, ParticipationLevel::Voter),
        ("retail_trader_2", 100_000, ParticipationLevel::Voter),
        ("retail_trader_3", 75_000, ParticipationLevel::Voter),
    ];

    for (address, tokens, level) in community_members {
        governance.register_participant(
            address.to_string(),
            tokens,
            level,
            PrivacyLevel::Public,
        ).await?;
    }

    // Passive holders (will delegate most of their power)
    governance.register_participant(
        "passive_holder_1".to_string(),
        10_000_000, // 10M tokens but passive
        ParticipationLevel::Observer,
        PrivacyLevel::Anonymous,
    ).await?;

    governance.register_participant(
        "passive_holder_2".to_string(),
        8_000_000, // 8M tokens but passive
        ParticipationLevel::Observer,
        PrivacyLevel::Anonymous,
    ).await?;

    println!("✅ Registered {} participants\n", governance.get_governance_statistics().await.total_participants);

    // Scenario 1: Basic Delegation
    println!("🔗 Scenario 1: Basic Delegation Setup");
    println!("=====================================");

    // Community members delegate to governance expert
    println!("🎯 Community delegates to governance expert...");
    let community_delegations = vec![
        ("community_alice", 600_000),  // Partial delegation
        ("community_bob", 600_000),    // Full delegation
        ("community_charlie", 300_000), // Partial delegation
        ("community_diana", 300_000),  // Full delegation
    ];

    for (delegator, amount) in community_delegations {
        governance.delegate(
            delegator.to_string(),
            "governance_expert".to_string(),
            amount,
            PrivacyLevel::Public,
        ).await?;
        println!("   {} delegated {} tokens", delegator, amount);
    }

    // Technical users delegate to technical delegate
    println!("\n🔧 Technical delegation...");
    governance.delegate(
        "retail_trader_1".to_string(),
        "technical_delegate".to_string(),
        150_000, // Full delegation
        PrivacyLevel::Public,
    ).await?;
    println!("   retail_trader_1 delegated 150K tokens to technical_delegate");

    // Passive holders delegate to institutional delegates
    println!("\n🏛️ Institutional delegation...");
    governance.delegate(
        "passive_holder_1".to_string(),
        "institutional_delegate_1".to_string(),
        8_000_000, // Most of their power
        PrivacyLevel::Anonymous,
    ).await?;
    println!("   passive_holder_1 delegated 8M tokens to institutional_delegate_1");

    governance.delegate(
        "passive_holder_2".to_string(),
        "institutional_delegate_2".to_string(),
        6_000_000, // Most of their power
        PrivacyLevel::Anonymous,
    ).await?;
    println!("   passive_holder_2 delegated 6M tokens to institutional_delegate_2");

    // Show delegation statistics
    let delegation_stats = governance.delegation_system.get_delegation_statistics();
    println!("\n📊 Delegation Statistics:");
    println!("   Total delegations: {}", delegation_stats.total_delegations);
    println!("   Unique delegators: {}", delegation_stats.unique_delegators);
    println!("   Unique delegates: {}", delegation_stats.unique_delegates);

    // Scenario 2: Scoped Delegation
    println!("\n🔗 Scenario 2: Scoped Delegation");
    println!("================================");

    // Beta whale delegates only for specific proposal types
    println!("🐋 Whale delegates for specific proposal types...");

    // Create scoped delegations
    governance.delegation_system.create_delegation(
        "beta_whale".to_string(),
        "technical_delegate".to_string(),
        DelegationPower::Fixed(5_000_000),
        DelegationScope::ProposalTypes(vec!["protocol_upgrade".to_string()]),
        None,
        HashMap::new(),
    ).await?;
    println!("   beta_whale delegated 5M for protocol upgrades to technical_delegate");

    governance.delegation_system.create_delegation(
        "beta_whale".to_string(),
        "institutional_delegate_1".to_string(),
        DelegationPower::Fixed(3_000_000),
        DelegationScope::ProposalTypes(vec!["treasury_spend".to_string()]),
        None,
        HashMap::new(),
    ).await?;
    println!("   beta_whale delegated 3M for treasury spending to institutional_delegate_1");

    // Community member delegates by topic
    governance.delegation_system.create_delegation(
        "community_eve".to_string(),
        "governance_expert".to_string(),
        DelegationPower::Percentage(80), // 80% of their power
        DelegationScope::Tags(vec!["governance".to_string(), "parameters".to_string()]),
        None,
        HashMap::new(),
    ).await?;
    println!("   community_eve delegated 80% for governance/parameter proposals");

    // Scenario 3: Delegation Chains
    println!("\n🔗 Scenario 3: Delegation Chains");
    println!("=================================");

    // Create a delegation chain: retail -> community -> expert -> whale
    println!("⛓️ Creating delegation chain...");

    // First level: retail traders to community leader
    governance.delegate(
        "retail_trader_2".to_string(),
        "community_alice".to_string(),
        100_000,
        PrivacyLevel::Public,
    ).await?;
    println!("   retail_trader_2 → community_alice (100K)");

    governance.delegate(
        "retail_trader_3".to_string(),
        "community_alice".to_string(),
        75_000,
        PrivacyLevel::Public,
    ).await?;
    println!("   retail_trader_3 → community_alice (75K)");

    // Second level: community leader to governance expert (already done above)
    println!("   community_alice → governance_expert (600K, already established)");

    // Third level: governance expert to alpha whale
    governance.delegate(
        "governance_expert".to_string(),
        "alpha_whale".to_string(),
        500_000,
        PrivacyLevel::Public,
    ).await?;
    println!("   governance_expert → alpha_whale (500K)");

    // Get delegation chain
    let chain = governance.delegation_system.get_delegation_chain("retail_trader_2").await?;
    println!("\n🔍 Delegation chain for retail_trader_2:");
    for (i, address) in chain.iter().enumerate() {
        println!("   {}. {}", i + 1, address);
    }

    // Scenario 4: Create and Vote on Proposals with Delegated Power
    println!("\n🔗 Scenario 4: Voting with Delegated Power");
    println!("===========================================");

    // Create a technical proposal
    println!("📋 Creating technical proposal...");
    let technical_proposal = governance.create_proposal(
        ProposalType::ProtocolUpgrade {
            version: "2.1.0".to_string(),
            description: "Enhanced whale trading algorithms".to_string(),
            code_hash: "0xtech123456789abcdef".to_string(),
        },
        "Implement Enhanced Whale Trading Algorithms".to_string(),
        "Upgrade the trading engine with advanced algorithms specifically optimized for large whale orders, including improved slippage protection and MEV resistance.".to_string(),
        "technical_delegate".to_string(),
        Some(chrono::Duration::days(7)),
        PrivacyLevel::Public,
    ).await?;

    // Create a governance proposal
    println!("📋 Creating governance proposal...");
    let governance_proposal = governance.create_proposal(
        ProposalType::ParameterUpdate {
            parameter: "voting_period".to_string(),
            old_value: "604800".to_string(), // 7 days
            new_value: "432000".to_string(),  // 5 days
        },
        "Reduce Voting Period for Faster Governance".to_string(),
        "Reduce the default voting period from 7 days to 5 days to enable faster governance decisions while maintaining adequate deliberation time.".to_string(),
        "governance_expert".to_string(),
        Some(chrono::Duration::days(5)),
        PrivacyLevel::Public,
    ).await?;

    // Vote with direct power
    println!("\n🗳️ Direct voting...");
    governance.vote(
        technical_proposal,
        "alpha_whale".to_string(),
        VoteType::For,
        Some("Technical improvements benefit whale trading".to_string()),
        PrivacyLevel::Confidential,
    ).await?;
    println!("   alpha_whale voted FOR technical proposal");

    // Vote using delegated power (this would be implemented in the voting system)
    println!("\n🏛️ Delegated voting...");

    // Calculate effective voting power including delegations
    let institutional_power = governance.delegation_system.get_effective_voting_power("institutional_delegate_1").await;
    let technical_power = governance.delegation_system.get_effective_voting_power("technical_delegate").await;
    let governance_expert_power = governance.delegation_system.get_effective_voting_power("governance_expert").await;

    println!("   institutional_delegate_1 effective power: {} tokens", institutional_power);
    println!("   technical_delegate effective power: {} tokens", technical_power);
    println!("   governance_expert effective power: {} tokens", governance_expert_power);

    // Simulate delegated votes
    governance.voting_system.cast_delegated_vote(
        &governance.proposal_manager.get_proposal(&technical_proposal).unwrap(),
        "technical_delegate".to_string(),
        "beta_whale".to_string(), // Original delegator
        VoteType::For,
        5_000_000, // Delegated power for technical proposals
        Some("Using delegated power from beta_whale for technical proposal".to_string()),
    ).await?;
    println!("   technical_delegate voted FOR using beta_whale's delegated power");

    // Scenario 5: Auto-delegation to Top Validators
    println!("\n🔗 Scenario 5: Auto-delegation to Top Validators");
    println!("================================================");

    // Simulate validator rankings
    let validator_powers = vec![
        ("institutional_delegate_1".to_string(), 13_000_000),
        ("institutional_delegate_2".to_string(), 9_000_000),
        ("technical_delegate".to_string(), 7_150_000),
        ("governance_expert".to_string(), 3_275_000),
    ];

    // Auto-delegate from a new passive holder
    governance.register_participant(
        "auto_delegator".to_string(),
        5_000_000,
        ParticipationLevel::Observer,
        PrivacyLevel::Public,
    ).await?;

    println!("🤖 Auto-delegating to top 3 validators...");
    governance.delegation_system.auto_delegate_to_top_validators(
        "auto_delegator".to_string(),
        DelegationPower::Percentage(90), // Delegate 90% of power
        3, // Top 3 validators
        validator_powers,
    ).await?;

    let auto_delegations = governance.delegation_system.get_delegations_by_delegator("auto_delegator");
    println!("   Created {} auto-delegations", auto_delegations.len());
    for delegation in auto_delegations {
        println!("     → {} ({} tokens)", delegation.delegate,
                match &delegation.power {
                    DelegationPower::Fixed(amount) => amount.to_string(),
                    _ => "calculated".to_string()
                });
    }

    // Show final statistics and analysis
    println!("\n📊 Final Delegation Analysis");
    println!("============================");

    let final_delegation_stats = governance.delegation_system.get_delegation_statistics();
    println!("Total delegations: {}", final_delegation_stats.total_delegations);
    println!("Unique delegators: {}", final_delegation_stats.unique_delegators);
    println!("Unique delegates: {}", final_delegation_stats.unique_delegates);
    println!("All scope delegations: {}", final_delegation_stats.all_scope);
    println!("Type-specific delegations: {}", final_delegation_stats.type_scope);
    println!("Tag-specific delegations: {}", final_delegation_stats.tag_scope);

    println!("\n🏆 Top Delegates by Effective Power:");
    let top_delegates = vec![
        ("institutional_delegate_1", governance.delegation_system.get_effective_voting_power("institutional_delegate_1").await),
        ("institutional_delegate_2", governance.delegation_system.get_effective_voting_power("institutional_delegate_2").await),
        ("technical_delegate", governance.delegation_system.get_effective_voting_power("technical_delegate").await),
        ("governance_expert", governance.delegation_system.get_effective_voting_power("governance_expert").await),
        ("alpha_whale", governance.delegation_system.get_effective_voting_power("alpha_whale").await),
    ];

    let mut sorted_delegates = top_delegates;
    sorted_delegates.sort_by(|a, b| b.1.cmp(&a.1));

    for (i, (delegate, power)) in sorted_delegates.iter().enumerate() {
        println!("   {}. {} - {} tokens", i + 1, delegate, power);
    }

    println!("\n🎉 Delegation system example completed successfully!");
    println!("   • Demonstrated basic, scoped, and chained delegations");
    println!("   • Showed auto-delegation to top validators");
    println!("   • Illustrated liquid democracy in action");
    println!("   • Analyzed delegation patterns and effectiveness");

    Ok(())
}