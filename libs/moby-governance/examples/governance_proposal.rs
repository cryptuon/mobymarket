//! Example: Creating and voting on governance proposals

use moby_governance::{
    proposals::{ProposalType, ProposalPriority},
    system::{GovernanceSystem, ParticipationLevel, PrivacyLevel},
    voting::VoteType,
    error::GovernanceResult,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> GovernanceResult<()> {
    println!("🏛️ Moby Governance - Proposal Example");
    println!("=====================================\n");

    // Initialize governance system
    let mut governance = GovernanceSystem::new();
    governance.initialize(Default::default()).await?;

    println!("✅ Governance system initialized\n");

    // Register participants
    println!("👥 Registering participants...");

    // Whale trader with significant voting power
    governance.register_participant(
        "whale_trader".to_string(),
        5_000_000, // 5M tokens
        ParticipationLevel::Proposer,
        PrivacyLevel::Confidential,
    ).await?;

    // Institutional investor
    governance.register_participant(
        "institution".to_string(),
        10_000_000, // 10M tokens
        ParticipationLevel::Governor,
        PrivacyLevel::Public,
    ).await?;

    // Community representative
    governance.register_participant(
        "community_rep".to_string(),
        500_000, // 500K tokens
        ParticipationLevel::Voter,
        PrivacyLevel::Public,
    ).await?;

    // Protocol developer
    governance.register_participant(
        "developer".to_string(),
        1_000_000, // 1M tokens
        ParticipationLevel::Proposer,
        PrivacyLevel::Public,
    ).await?;

    println!("✅ Registered 4 participants\n");

    // Create a governance proposal to reduce trading fees
    println!("📋 Creating governance proposal...");

    let proposal_id = governance.create_proposal(
        ProposalType::ParameterUpdate {
            parameter: "trading_fee".to_string(),
            old_value: "0.003".to_string(), // 0.3%
            new_value: "0.002".to_string(),  // 0.2%
        },
        "Reduce Trading Fees for Whale Orders".to_string(),
        "This proposal aims to reduce trading fees from 0.3% to 0.2% to attract more whale trading volume and increase overall protocol revenue through volume rather than per-transaction fees.".to_string(),
        "whale_trader".to_string(),
        Some(chrono::Duration::days(7)), // 7-day voting period
        PrivacyLevel::Public,
    ).await?;

    println!("✅ Created proposal: {}", proposal_id);
    println!("   Title: Reduce Trading Fees for Whale Orders");
    println!("   Voting period: 7 days\n");

    // Cast votes from different participants
    println!("🗳️ Casting votes...");

    // Whale trader votes FOR (proposer supporting their own proposal)
    governance.vote(
        proposal_id,
        "whale_trader".to_string(),
        VoteType::For,
        Some("As the proposer, I believe this will significantly increase trading volume and benefit the entire ecosystem.".to_string()),
        PrivacyLevel::Confidential,
    ).await?;
    println!("✅ Whale trader voted FOR");

    // Institution votes FOR
    governance.vote(
        proposal_id,
        "institution".to_string(),
        VoteType::For,
        Some("Lower fees will make the platform more competitive and attract institutional volume.".to_string()),
        PrivacyLevel::Public,
    ).await?;
    println!("✅ Institution voted FOR");

    // Community representative votes AGAINST
    governance.vote(
        proposal_id,
        "community_rep".to_string(),
        VoteType::Against,
        Some("I'm concerned about reduced revenue affecting protocol development funding.".to_string()),
        PrivacyLevel::Public,
    ).await?;
    println!("✅ Community representative voted AGAINST");

    // Developer abstains
    governance.vote(
        proposal_id,
        "developer".to_string(),
        VoteType::Abstain,
        Some("As a protocol developer, I remain neutral on fee policy decisions.".to_string()),
        PrivacyLevel::Public,
    ).await?;
    println!("✅ Developer voted ABSTAIN\n");

    // Check voting results
    println!("📊 Checking voting results...");

    let stats = governance.get_governance_statistics().await;
    println!("   Total participants: {}", stats.total_participants);
    println!("   Total voting power: {}", stats.total_voting_power);
    println!("   Active proposals: {}", stats.active_proposals);

    // Get recent governance events
    let events = governance.get_recent_events(10, Some(PrivacyLevel::Public));
    println!("\n📈 Recent governance events ({} public events):", events.len());
    for (i, event) in events.iter().enumerate().take(5) {
        println!("   {}. {:?} by {} at {}",
                i + 1,
                event.event_type,
                event.participant,
                event.timestamp.format("%Y-%m-%d %H:%M:%S"));
    }

    // Create a second proposal for protocol upgrade
    println!("\n📋 Creating second proposal (Protocol Upgrade)...");

    let upgrade_proposal_id = governance.create_proposal(
        ProposalType::ProtocolUpgrade {
            version: "2.0.0".to_string(),
            description: "Enhanced Privacy Features".to_string(),
            code_hash: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        },
        "Implement Zero-Knowledge Privacy Features".to_string(),
        "This proposal introduces comprehensive zero-knowledge proof capabilities including stealth addresses, ring signatures, and confidential transaction amounts to enhance privacy for whale traders.".to_string(),
        "developer".to_string(),
        Some(chrono::Duration::days(14)), // 14-day voting period for upgrades
        PrivacyLevel::Public,
    ).await?;

    println!("✅ Created upgrade proposal: {}", upgrade_proposal_id);
    println!("   Title: Implement Zero-Knowledge Privacy Features");
    println!("   Voting period: 14 days\n");

    // Vote on the upgrade proposal
    println!("🗳️ Voting on upgrade proposal...");

    // Institution votes FOR
    governance.vote(
        upgrade_proposal_id,
        "institution".to_string(),
        VoteType::For,
        Some("Privacy features are essential for institutional adoption.".to_string()),
        PrivacyLevel::Public,
    ).await?;

    // Whale trader votes FOR
    governance.vote(
        upgrade_proposal_id,
        "whale_trader".to_string(),
        VoteType::For,
        Some("Enhanced privacy will attract more whale traders.".to_string()),
        PrivacyLevel::Confidential,
    ).await?;

    println!("✅ Votes cast on upgrade proposal\n");

    // Create a treasury spending proposal
    println!("📋 Creating treasury proposal...");

    let treasury_proposal_id = governance.create_proposal(
        ProposalType::TreasurySpend {
            recipient: "development_team".to_string(),
            amount: 1_000_000, // 1M tokens
            purpose: "Privacy feature development and security audit".to_string(),
        },
        "Fund Privacy Development Team".to_string(),
        "Allocate 1M MOBY tokens to fund the development team working on zero-knowledge privacy features and conduct a comprehensive security audit.".to_string(),
        "institution".to_string(),
        None, // Use default voting period
        PrivacyLevel::Public,
    ).await?;

    println!("✅ Created treasury proposal: {}", treasury_proposal_id);
    println!("   Allocation: 1M MOBY tokens");
    println!("   Purpose: Privacy development and security audit\n");

    // Show final statistics
    println!("📊 Final Governance Statistics");
    println!("==============================");

    let final_stats = governance.get_governance_statistics().await;
    println!("Total participants: {}", final_stats.total_participants);
    println!("Total proposals: {}", final_stats.total_proposals);
    println!("Active proposals: {}", final_stats.active_proposals);
    println!("Total voting power: {}", final_stats.total_voting_power);
    println!("Staked tokens: {}", final_stats.staked_tokens);
    println!("Treasury value: {}", final_stats.treasury_value);
    println!("Total events: {}", final_stats.total_events);

    println!("\n🎉 Governance proposal example completed successfully!");
    println!("   • Created 3 different types of proposals");
    println!("   • Demonstrated voting with different participation levels");
    println!("   • Showed privacy levels in action");
    println!("   • Tracked governance events and statistics");

    Ok(())
}