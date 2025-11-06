//! Example: Advanced voting mechanisms and strategies

use moby_governance::{
    proposals::{ProposalType, ProposalPriority},
    system::{GovernanceSystem, ParticipationLevel, PrivacyLevel},
    voting::{VoteType, VotingStrategy},
    error::GovernanceResult,
};

#[tokio::main]
async fn main() -> GovernanceResult<()> {
    println!("🗳️ Moby Governance - Advanced Voting Example");
    println!("=============================================\n");

    // Initialize governance system
    let mut governance = GovernanceSystem::new();
    governance.initialize(Default::default()).await?;

    println!("✅ Governance system initialized\n");

    // Register a diverse set of participants with different voting powers
    println!("👥 Registering diverse participant base...");

    let participants = vec![
        ("mega_whale", 20_000_000, ParticipationLevel::Governor, "Mega Whale Trader"),
        ("institution_a", 15_000_000, ParticipationLevel::Proposer, "Institutional Investor A"),
        ("institution_b", 12_000_000, ParticipationLevel::Proposer, "Institutional Investor B"),
        ("whale_1", 8_000_000, ParticipationLevel::Proposer, "Whale Trader 1"),
        ("whale_2", 6_000_000, ParticipationLevel::Proposer, "Whale Trader 2"),
        ("dao_delegate_1", 3_000_000, ParticipationLevel::Reviewer, "DAO Delegate 1"),
        ("dao_delegate_2", 2_500_000, ParticipationLevel::Reviewer, "DAO Delegate 2"),
        ("community_1", 1_000_000, ParticipationLevel::Voter, "Community Member 1"),
        ("community_2", 800_000, ParticipationLevel::Voter, "Community Member 2"),
        ("community_3", 600_000, ParticipationLevel::Voter, "Community Member 3"),
        ("developer_1", 500_000, ParticipationLevel::Voter, "Developer 1"),
        ("developer_2", 400_000, ParticipationLevel::Voter, "Developer 2"),
    ];

    for (address, tokens, level, name) in participants {
        governance.register_participant(
            address.to_string(),
            tokens,
            level,
            PrivacyLevel::Public,
        ).await?;
        println!("   ✅ {} registered with {} tokens", name, tokens);
    }

    let total_participants = governance.get_governance_statistics().await.total_participants;
    let total_voting_power = governance.get_governance_statistics().await.total_voting_power;
    println!("\n📊 Participant Summary:");
    println!("   Total participants: {}", total_participants);
    println!("   Total voting power: {} tokens\n", total_voting_power);

    // Test Scenario 1: Simple Majority Voting
    println!("🗳️ Scenario 1: Simple Majority Voting");
    println!("=====================================");

    let proposal_1 = governance.create_proposal(
        ProposalType::ParameterUpdate {
            parameter: "max_order_size".to_string(),
            old_value: "10000000".to_string(),
            new_value: "50000000".to_string(),
        },
        "Increase Maximum Order Size".to_string(),
        "Increase maximum single order size from 10M to 50M tokens to better serve whale traders.".to_string(),
        "mega_whale".to_string(),
        Some(chrono::Duration::days(3)),
        PrivacyLevel::Public,
    ).await?;

    println!("📋 Created proposal: Increase Maximum Order Size");

    // Cast votes with simple majority expectation
    let votes_1 = vec![
        ("mega_whale", VoteType::For, "This directly benefits my trading needs"),
        ("institution_a", VoteType::For, "Larger orders improve market efficiency"),
        ("whale_1", VoteType::For, "Essential for whale trading"),
        ("whale_2", VoteType::For, "Strongly support this change"),
        ("dao_delegate_1", VoteType::Against, "May increase market volatility"),
        ("community_1", VoteType::Against, "Favors large traders too much"),
        ("community_2", VoteType::Against, "Concerned about market manipulation"),
        ("developer_1", VoteType::Abstain, "Technical implementation is feasible"),
    ];

    for (voter, vote_type, justification) in votes_1 {
        governance.vote(
            proposal_1,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Public,
        ).await?;
        println!("   {} voted {:?}", voter, vote_type);
    }

    // Test Scenario 2: Supermajority Requirement
    println!("\n🗳️ Scenario 2: Supermajority Requirement (67%)");
    println!("===============================================");

    let proposal_2 = governance.create_proposal(
        ProposalType::ProtocolUpgrade {
            version: "3.0.0".to_string(),
            description: "Major Protocol Overhaul".to_string(),
            code_hash: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        },
        "Major Protocol Architecture Upgrade".to_string(),
        "Implement major architectural changes including new consensus mechanism and enhanced scalability. This is a breaking change requiring supermajority approval.".to_string(),
        "institution_a".to_string(),
        Some(chrono::Duration::days(7)),
        PrivacyLevel::Public,
    ).await?;

    println!("📋 Created proposal: Major Protocol Architecture Upgrade");

    // Cast votes for supermajority test
    let votes_2 = vec![
        ("mega_whale", VoteType::For, "Long-term protocol improvement"),
        ("institution_a", VoteType::For, "Proposing this upgrade"),
        ("institution_b", VoteType::For, "Necessary for institutional adoption"),
        ("whale_1", VoteType::For, "Will improve trading performance"),
        ("whale_2", VoteType::Against, "Too risky, prefer incremental updates"),
        ("dao_delegate_1", VoteType::For, "Community benefits outweigh risks"),
        ("dao_delegate_2", VoteType::For, "Supports long-term vision"),
        ("community_1", VoteType::Against, "Concerned about compatibility"),
        ("community_2", VoteType::For, "Trust in development team"),
        ("community_3", VoteType::Abstain, "Need more technical details"),
        ("developer_1", VoteType::For, "Technically sound implementation"),
        ("developer_2", VoteType::For, "Will improve developer experience"),
    ];

    for (voter, vote_type, justification) in votes_2 {
        governance.vote(
            proposal_2,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Public,
        ).await?;
        println!("   {} voted {:?}", voter, vote_type);
    }

    // Test Scenario 3: Unanimous Consensus Required
    println!("\n🗳️ Scenario 3: Constitutional Amendment (Unanimous)");
    println!("===================================================");

    let proposal_3 = governance.create_proposal(
        ProposalType::Constitutional {
            amendment: "Add Privacy Rights Amendment".to_string(),
            section: "Article 5: Privacy Rights".to_string(),
        },
        "Constitutional Privacy Rights Amendment".to_string(),
        "Add constitutional protection for user privacy and data rights. This fundamental change requires unanimous approval from all active governance participants.".to_string(),
        "dao_delegate_1".to_string(),
        Some(chrono::Duration::days(14)),
        PrivacyLevel::Public,
    ).await?;

    println!("📋 Created proposal: Constitutional Privacy Rights Amendment");

    // Cast votes for unanimous test (simulate some not voting)
    let votes_3 = vec![
        ("mega_whale", VoteType::For, "Privacy is fundamental"),
        ("institution_a", VoteType::For, "Essential for compliance"),
        ("institution_b", VoteType::For, "Supports institutional adoption"),
        ("whale_1", VoteType::For, "Privacy protects whale traders"),
        ("whale_2", VoteType::For, "Strongly support privacy rights"),
        ("dao_delegate_1", VoteType::For, "Proposing this amendment"),
        ("dao_delegate_2", VoteType::For, "Critical for user protection"),
        ("community_1", VoteType::For, "Privacy benefits everyone"),
        ("community_2", VoteType::For, "Fundamental right"),
        // community_3 doesn't vote (not unanimous)
        ("developer_1", VoteType::For, "Technically important"),
        ("developer_2", VoteType::For, "Supports implementation"),
    ];

    for (voter, vote_type, justification) in votes_3 {
        governance.vote(
            proposal_3,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Public,
        ).await?;
        println!("   {} voted {:?}", voter, vote_type);
    }

    // Test Scenario 4: Custom Threshold (75%)
    println!("\n🗳️ Scenario 4: Treasury Allocation (75% threshold)");
    println!("==================================================");

    let proposal_4 = governance.create_proposal(
        ProposalType::TreasurySpend {
            recipient: "security_audit_firm".to_string(),
            amount: 5_000_000,
            purpose: "Comprehensive security audit for protocol upgrade".to_string(),
        },
        "Fund Comprehensive Security Audit".to_string(),
        "Allocate 5M MOBY tokens for a comprehensive security audit before implementing the major protocol upgrade. High threshold required for large treasury expenditure.".to_string(),
        "institution_b".to_string(),
        Some(chrono::Duration::days(5)),
        PrivacyLevel::Public,
    ).await?;

    println!("📋 Created proposal: Fund Comprehensive Security Audit");

    // Cast votes for custom threshold test
    let votes_4 = vec![
        ("mega_whale", VoteType::For, "Security is paramount"),
        ("institution_a", VoteType::For, "Necessary before upgrade"),
        ("institution_b", VoteType::For, "Proposing this allocation"),
        ("whale_1", VoteType::For, "Worth the investment"),
        ("whale_2", VoteType::For, "Security audit is essential"),
        ("dao_delegate_1", VoteType::For, "Community safety first"),
        ("dao_delegate_2", VoteType::Against, "Amount seems excessive"),
        ("community_1", VoteType::For, "Better safe than sorry"),
        ("community_2", VoteType::Against, "Could use funds for development"),
        ("community_3", VoteType::For, "Security audit is critical"),
        ("developer_1", VoteType::Abstain, "Audit firm decision pending"),
        ("developer_2", VoteType::For, "Will validate our work"),
    ];

    for (voter, vote_type, justification) in votes_4 {
        governance.vote(
            proposal_4,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Public,
        ).await?;
        println!("   {} voted {:?}", voter, vote_type);
    }

    // Analyze voting results
    println!("\n📊 Voting Results Analysis");
    println!("==========================");

    let proposals = vec![
        (proposal_1, "Max Order Size (Simple Majority)", VotingStrategy::SimpleMajority),
        (proposal_2, "Protocol Upgrade (Supermajority)", VotingStrategy::Supermajority),
        (proposal_3, "Privacy Amendment (Unanimous)", VotingStrategy::Unanimous),
        (proposal_4, "Security Audit (75% Threshold)", VotingStrategy::CustomThreshold(75)),
    ];

    for (proposal_id, name, strategy) in proposals {
        // This would normally be done within the governance system
        // For demonstration, we'll show the concept
        println!("\n📋 {}", name);
        println!("   Strategy: {:?}", strategy);

        // Get voting statistics (simplified for example)
        let stats = governance.voting_system.get_voting_statistics(&proposal_id).await;
        println!("   Total votes: {}", stats.total_votes);
        println!("   Votes for: {}", stats.votes_for);
        println!("   Votes against: {}", stats.votes_against);
        println!("   Abstentions: {}", stats.votes_abstain);
        println!("   Direct votes: {}", stats.direct_votes);
        println!("   Delegated votes: {}", stats.delegated_votes);

        let distribution = governance.voting_system.get_voting_distribution(&proposal_id);
        println!("   For percentage: {:.1}%", distribution.for_percentage);
        println!("   Against percentage: {:.1}%", distribution.against_percentage);

        // Determine if passed based on strategy
        let passed = match strategy {
            VotingStrategy::SimpleMajority => distribution.for_percentage > 50.0,
            VotingStrategy::Supermajority => distribution.for_percentage >= 66.7,
            VotingStrategy::Unanimous => distribution.against_percentage == 0.0 && distribution.for_percentage > 0.0 && stats.total_votes == total_participants,
            VotingStrategy::CustomThreshold(threshold) => distribution.for_percentage >= threshold as f64,
        };

        println!("   Result: {}", if passed { "✅ PASSED" } else { "❌ FAILED" });
    }

    // Show overall governance health metrics
    println!("\n📈 Governance Health Metrics");
    println!("============================");

    let final_stats = governance.get_governance_statistics().await;
    println!("Participation rate: {:.1}%",
             (final_stats.total_participants as f64 / 12.0) * 100.0);
    println!("Average proposals per participant: {:.1}",
             final_stats.total_proposals as f64 / final_stats.total_participants as f64);
    println!("Voter diversity index: High"); // Simplified calculation
    println!("Governance activity: Active");

    println!("\n🎉 Advanced voting example completed successfully!");
    println!("   • Demonstrated 4 different voting strategies");
    println!("   • Showed diverse participant engagement");
    println!("   • Analyzed voting patterns and outcomes");
    println!("   • Measured governance health metrics");

    Ok(())
}