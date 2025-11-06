//! Example: Parameter management and protocol updates

use moby_governance::{
    parameters::{ParameterType, ParameterValue, ParameterConstraints},
    proposals::{ProposalType, ProposalPriority},
    system::{GovernanceSystem, ParticipationLevel, PrivacyLevel},
    voting::VoteType,
    error::GovernanceResult,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> GovernanceResult<()> {
    println!("⚙️ Moby Governance - Parameter Management Example");
    println!("=================================================\n");

    // Initialize governance system
    let mut governance = GovernanceSystem::new();
    governance.initialize(Default::default()).await?;

    println!("✅ Governance system initialized\n");

    // Register key participants
    println!("👥 Registering governance participants...");

    governance.register_participant(
        "protocol_team".to_string(),
        3_000_000,
        ParticipationLevel::Governor,
        PrivacyLevel::Public,
    ).await?;

    governance.register_participant(
        "whale_council".to_string(),
        15_000_000,
        ParticipationLevel::Proposer,
        PrivacyLevel::Confidential,
    ).await?;

    governance.register_participant(
        "community_delegate".to_string(),
        2_000_000,
        ParticipationLevel::Reviewer,
        PrivacyLevel::Public,
    ).await?;

    governance.register_participant(
        "institutional_node".to_string(),
        8_000_000,
        ParticipationLevel::Proposer,
        PrivacyLevel::Public,
    ).await?;

    println!("✅ Registered participants with governance roles\n");

    // Setup comprehensive parameter system
    println!("⚙️ Setting up protocol parameters...");

    // Trading parameters
    let trading_params = vec![
        (
            "base_trading_fee",
            "Base fee charged on all trades",
            ParameterType::Float,
            ParameterValue::Float(0.003), // 0.3%
            Some(ParameterConstraints {
                min_value: Some(0.0001), // 0.01%
                max_value: Some(0.01),   // 1%
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            "trading",
            vec!["fee", "trading", "revenue"],
        ),
        (
            "whale_discount_threshold",
            "Minimum trade size for whale discount",
            ParameterType::UInteger,
            ParameterValue::UInteger(1_000_000), // 1M tokens
            Some(ParameterConstraints {
                min_value: Some(100_000.0),   // 100K
                max_value: Some(100_000_000.0), // 100M
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            "trading",
            vec!["whale", "discount", "threshold"],
        ),
        (
            "max_slippage_protection",
            "Maximum slippage protection percentage",
            ParameterType::Float,
            ParameterValue::Float(0.05), // 5%
            Some(ParameterConstraints {
                min_value: Some(0.001), // 0.1%
                max_value: Some(0.2),   // 20%
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            "trading",
            vec!["slippage", "protection", "whale"],
        ),
    ];

    for (name, desc, param_type, default_val, constraints, category, tags) in trading_params {
        governance.parameter_manager.register_parameter(
            name.to_string(),
            desc.to_string(),
            param_type,
            default_val,
            constraints,
            true,  // mutable
            false, // not protected
            category.to_string(),
            tags.into_iter().map(|s| s.to_string()).collect(),
        ).await?;
        println!("   ✅ Registered parameter: {}", name);
    }

    // Governance parameters
    let governance_params = vec![
        (
            "proposal_threshold",
            "Minimum tokens required to create proposal",
            ParameterType::UInteger,
            ParameterValue::UInteger(100_000),
            Some(ParameterConstraints {
                min_value: Some(1_000.0),
                max_value: Some(10_000_000.0),
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            "governance",
            vec!["proposal", "threshold", "governance"],
        ),
        (
            "voting_delay",
            "Delay before voting starts (seconds)",
            ParameterType::UInteger,
            ParameterValue::UInteger(86400), // 24 hours
            Some(ParameterConstraints {
                min_value: Some(3600.0),   // 1 hour
                max_value: Some(604800.0), // 7 days
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            "governance",
            vec!["voting", "delay", "governance"],
        ),
        (
            "execution_timelock",
            "Timelock before executing passed proposals (seconds)",
            ParameterType::UInteger,
            ParameterValue::UInteger(172800), // 48 hours
            Some(ParameterConstraints {
                min_value: Some(86400.0),  // 24 hours
                max_value: Some(604800.0), // 7 days
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            "governance",
            vec!["execution", "timelock", "security"],
        ),
    ];

    for (name, desc, param_type, default_val, constraints, category, tags) in governance_params {
        governance.parameter_manager.register_parameter(
            name.to_string(),
            desc.to_string(),
            param_type,
            default_val,
            constraints,
            true,
            false,
            category.to_string(),
            tags.into_iter().map(|s| s.to_string()).collect(),
        ).await?;
        println!("   ✅ Registered parameter: {}", name);
    }

    // Security parameters (some protected)
    let security_params = vec![
        (
            "emergency_pause_authority",
            "Addresses authorized for emergency pause",
            ParameterType::Array(Box::new(ParameterType::String)),
            ParameterValue::Array(vec![
                ParameterValue::String("emergency_multisig".to_string()),
                ParameterValue::String("protocol_team".to_string()),
            ]),
            None,
            "security",
            vec!["emergency", "pause", "authority"],
            true, // protected
        ),
        (
            "max_daily_withdrawal",
            "Maximum daily withdrawal limit",
            ParameterType::UInteger,
            ParameterValue::UInteger(50_000_000), // 50M tokens
            Some(ParameterConstraints {
                min_value: Some(1_000_000.0),   // 1M
                max_value: Some(1_000_000_000.0), // 1B
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            "security",
            vec!["withdrawal", "limit", "security"],
            false, // not protected
        ),
    ];

    for (name, desc, param_type, default_val, constraints, category, tags, protected) in security_params {
        governance.parameter_manager.register_parameter(
            name.to_string(),
            desc.to_string(),
            param_type,
            default_val,
            constraints,
            true,
            protected,
            category.to_string(),
            tags.into_iter().map(|s| s.to_string()).collect(),
        ).await?;
        println!("   ✅ Registered parameter: {} {}", name, if protected { "(protected)" } else { "" });
    }

    println!("\n📊 Parameter system overview:");
    let param_stats = governance.parameter_manager.get_parameter_statistics();
    println!("   Total parameters: {}", param_stats.total_parameters);
    println!("   Mutable parameters: {}", param_stats.mutable_parameters);
    println!("   Protected parameters: {}", param_stats.protected_parameters);
    println!("   Categories: {}", param_stats.total_categories);

    // Scenario 1: Routine Parameter Update
    println!("\n⚙️ Scenario 1: Routine Trading Fee Adjustment");
    println!("==============================================");

    // Propose reducing trading fees to be more competitive
    let fee_update_proposal = governance.create_proposal(
        ProposalType::ParameterUpdate {
            parameter: "base_trading_fee".to_string(),
            old_value: "0.003".to_string(), // 0.3%
            new_value: "0.0025".to_string(), // 0.25%
        },
        "Reduce Base Trading Fee to Increase Competitiveness".to_string(),
        "Market analysis shows our 0.3% trading fee is above industry average. Reducing to 0.25% should increase volume while maintaining healthy revenue margins.".to_string(),
        "whale_council".to_string(),
        Some(chrono::Duration::days(5)),
        PrivacyLevel::Public,
    ).await?;

    println!("📋 Created fee reduction proposal");

    // Vote on the proposal
    let fee_votes = vec![
        ("whale_council", VoteType::For, "Will increase our trading volume"),
        ("institutional_node", VoteType::For, "Competitive fees attract institutional flow"),
        ("community_delegate", VoteType::Against, "Concerned about revenue impact"),
        ("protocol_team", VoteType::Abstain, "Neutral on fee policy decisions"),
    ];

    for (voter, vote_type, justification) in fee_votes {
        governance.vote(
            fee_update_proposal,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Public,
        ).await?;
        println!("   {} voted {:?}: {}", voter, vote_type, justification);
    }

    // Propose the parameter update with timelock
    let fee_update_id = governance.parameter_manager.propose_update(
        "base_trading_fee".to_string(),
        ParameterValue::Float(0.0025),
        "whale_council".to_string(),
        "Approved by governance vote to increase competitiveness".to_string(),
        Some("Expected 15-20% increase in trading volume based on price elasticity analysis".to_string()),
        Some(chrono::Duration::hours(24)), // 24-hour timelock
    ).await?;

    println!("⏳ Parameter update proposed with 24-hour timelock: {}", fee_update_id);

    // Scenario 2: Emergency Parameter Override
    println!("\n🚨 Scenario 2: Emergency Withdrawal Limit Update");
    println!("================================================");

    // Simulate emergency situation requiring immediate parameter change
    println!("🚨 Emergency: Unusual withdrawal patterns detected");

    let emergency_update_proposal = governance.create_proposal(
        ProposalType::EmergencyAction {
            action: "Parameter Override: max_daily_withdrawal".to_string(),
            justification: "Suspicious large withdrawals detected, temporary limit reduction needed".to_string(),
        },
        "Emergency Withdrawal Limit Reduction".to_string(),
        "Reduce daily withdrawal limit from 50M to 10M tokens due to suspicious activity patterns detected by security monitoring systems.".to_string(),
        "protocol_team".to_string(),
        Some(chrono::Duration::hours(6)), // Fast-track voting
        PrivacyLevel::Confidential, // Sensitive security information
    ).await?;

    println!("📋 Created emergency parameter update proposal");

    // Emergency voting (expedited)
    let emergency_votes = vec![
        ("protocol_team", VoteType::For, "Security team recommendation"),
        ("institutional_node", VoteType::For, "Agree with security measures"),
        ("whale_council", VoteType::Against, "Limit too restrictive for whale operations"),
        ("community_delegate", VoteType::For, "Security is paramount"),
    ];

    for (voter, vote_type, justification) in emergency_votes {
        governance.vote(
            emergency_update_proposal,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Confidential,
        ).await?;
        println!("   {} voted {:?} (emergency vote)", voter, vote_type);
    }

    // Emergency parameter update (minimal timelock)
    let emergency_update_id = governance.parameter_manager.propose_update(
        "max_daily_withdrawal".to_string(),
        ParameterValue::UInteger(10_000_000), // Reduce to 10M
        "protocol_team".to_string(),
        "Emergency response to suspicious withdrawal patterns".to_string(),
        Some("Temporary measure to prevent potential security breach".to_string()),
        Some(chrono::Duration::hours(1)), // Minimal timelock for emergency
    ).await?;

    println!("⚡ Emergency parameter update proposed: {}", emergency_update_id);

    // Scenario 3: Comprehensive Governance Reform
    println!("\n🏛️ Scenario 3: Comprehensive Governance Parameter Reform");
    println!("=========================================================");

    // Propose multiple related governance improvements
    let governance_reform_proposal = governance.create_proposal(
        ProposalType::General {
            title: "Governance Efficiency Improvements".to_string(),
            description: "Multiple parameter updates to improve governance efficiency".to_string(),
            actions: vec![
                "Reduce proposal threshold from 100K to 50K tokens".to_string(),
                "Reduce voting delay from 24h to 12h".to_string(),
                "Reduce execution timelock from 48h to 24h".to_string(),
            ],
        },
        "Governance Efficiency Reform Package".to_string(),
        "Comprehensive reform to make governance more efficient and accessible while maintaining security. Reduces barriers to participation and speeds up decision-making.".to_string(),
        "community_delegate".to_string(),
        Some(chrono::Duration::days(10)), // Longer period for comprehensive changes
        PrivacyLevel::Public,
    ).await?;

    println!("📋 Created comprehensive governance reform proposal");

    // Vote on governance reform
    let reform_votes = vec![
        ("community_delegate", VoteType::For, "Proposing these efficiency improvements"),
        ("whale_council", VoteType::For, "Faster governance benefits whale trading"),
        ("institutional_node", VoteType::For, "Improved efficiency with maintained security"),
        ("protocol_team", VoteType::Against, "Concerned about reduced security margins"),
    ];

    for (voter, vote_type, justification) in reform_votes {
        governance.vote(
            governance_reform_proposal,
            voter.to_string(),
            vote_type.clone(),
            Some(justification.to_string()),
            PrivacyLevel::Public,
        ).await?;
        println!("   {} voted {:?}: {}", voter, vote_type, justification);
    }

    // Batch parameter updates
    let batch_updates = vec![
        ("proposal_threshold".to_string(), ParameterValue::UInteger(50_000)),
        ("voting_delay".to_string(), ParameterValue::UInteger(43_200)), // 12 hours
        ("execution_timelock".to_string(), ParameterValue::UInteger(86_400)), // 24 hours
    ];

    let successful_updates = governance.parameter_manager.batch_update(
        batch_updates,
        "community_delegate".to_string(),
    ).await?;

    println!("📦 Batch parameter update completed:");
    for param in successful_updates {
        println!("   ✅ Updated: {}", param);
    }

    // Scenario 4: Parameter Analysis and Monitoring
    println!("\n📊 Scenario 4: Parameter Analysis and Impact Assessment");
    println!("=======================================================");

    // Show current parameter values by category
    println!("📋 Current parameter values by category:");

    println!("\n🔄 Trading Parameters:");
    let trading_params = governance.parameter_manager.get_parameters_by_category("trading");
    for param in trading_params {
        println!("   {}: {} (updated {} times)",
                param.name,
                param.current_value.to_string(),
                param.update_count);
    }

    println!("\n🏛️ Governance Parameters:");
    let gov_params = governance.parameter_manager.get_parameters_by_category("governance");
    for param in gov_params {
        println!("   {}: {} (updated {} times)",
                param.name,
                param.current_value.to_string(),
                param.update_count);
    }

    println!("\n🔒 Security Parameters:");
    let security_params = governance.parameter_manager.get_parameters_by_category("security");
    for param in security_params {
        let status = if param.protected { " [PROTECTED]" } else { "" };
        println!("   {}: {}{} (updated {} times)",
                param.name,
                param.current_value.to_string(),
                status,
                param.update_count);
    }

    // Show pending parameter updates
    println!("\n⏳ Pending Parameter Updates:");
    let pending_updates = governance.parameter_manager.get_pending_updates();
    if pending_updates.is_empty() {
        println!("   No pending updates");
    } else {
        for update in pending_updates {
            let time_left = update.timelock_expires_at - chrono::Utc::now();
            println!("   {} → {} (executes in {} minutes)",
                    update.parameter_name,
                    update.new_value.to_string(),
                    time_left.num_minutes());
        }
    }

    // Scenario 5: Parameter History and Audit Trail
    println!("\n📜 Scenario 5: Parameter Audit Trail");
    println!("====================================");

    // Simulate parameter history tracking
    println!("📈 Parameter change history (simulated):");

    let change_history = vec![
        ("base_trading_fee", "2024-01-15", "0.005 → 0.003", "Initial launch adjustment"),
        ("whale_discount_threshold", "2024-02-03", "500000 → 1000000", "Increase whale threshold"),
        ("max_slippage_protection", "2024-02-20", "0.03 → 0.05", "Better whale protection"),
        ("proposal_threshold", "Today", "100000 → 50000", "Governance reform"),
    ];

    for (param, date, change, reason) in change_history {
        println!("   {} [{}]: {} - {}", date, param, change, reason);
    }

    // Show parameter impact metrics (simulated)
    println!("\n📊 Parameter Impact Metrics:");
    println!("   Trading fee reduction: +18% volume increase");
    println!("   Whale threshold increase: +25% whale participation");
    println!("   Slippage protection: +12% whale satisfaction");
    println!("   Proposal threshold reduction: +40% governance participation");

    // Final statistics
    println!("\n📈 Final Parameter Management Statistics");
    println!("=======================================");

    let final_param_stats = governance.parameter_manager.get_parameter_statistics();
    println!("Total parameters: {}", final_param_stats.total_parameters);
    println!("Total updates: {}", final_param_stats.total_updates);
    println!("Pending updates: {}", final_param_stats.pending_updates);
    println!("Parameter types:");
    println!("  - String: {}", final_param_stats.string_parameters);
    println!("  - Integer: {}", final_param_stats.integer_parameters);
    println!("  - UInteger: {}", final_param_stats.uinteger_parameters);
    println!("  - Float: {}", final_param_stats.float_parameters);
    println!("  - Boolean: {}", final_param_stats.boolean_parameters);
    println!("  - Array: {}", final_param_stats.array_parameters);

    println!("\n🎉 Parameter management example completed successfully!");
    println!("   • Registered comprehensive parameter system");
    println!("   • Demonstrated routine and emergency updates");
    println!("   • Showed batch updates and governance reforms");
    println!("   • Illustrated parameter monitoring and audit trails");
    println!("   • Tracked impact metrics and governance efficiency");

    Ok(())
}