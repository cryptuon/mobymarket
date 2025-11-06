//! # Oracle Governance Example
//!
//! This example demonstrates governance mechanisms for the oracle system including:
//! - Parameter governance and voting
//! - Source addition/removal through governance
//! - Emergency procedures and circuit breakers
//! - Reputation-based governance
//! - Decentralized oracle management
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example oracle_governance
//! ```

use moby_oracle::*;
use moby_oracle::sources::*;
use moby_oracle::aggregation::*;
use moby_oracle::security::*;
use std::collections::HashMap;
use std::time::Duration;
use chrono::Utc;
use rust_decimal::Decimal;
use tokio::time::sleep;

/// Governance proposal types
#[derive(Debug, Clone)]
enum ProposalType {
    /// Add a new data source
    AddDataSource {
        source_id: String,
        source_type: DataSource,
        config: SourceConfig,
    },
    /// Remove an existing data source
    RemoveDataSource {
        source_id: String,
        reason: String,
    },
    /// Update aggregation parameters
    UpdateAggregationConfig {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    /// Update security parameters
    UpdateSecurityConfig {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    /// Emergency circuit breaker activation
    EmergencyCircuitBreaker {
        target: String,
        duration: Duration,
        reason: String,
    },
    /// Source weight adjustment
    AdjustSourceWeight {
        source: DataSource,
        old_weight: f64,
        new_weight: f64,
    },
}

/// Governance proposal
#[derive(Debug, Clone)]
struct GovernanceProposal {
    pub id: u64,
    pub proposal_type: ProposalType,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub voting_start: chrono::DateTime<Utc>,
    pub voting_end: chrono::DateTime<Utc>,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: ProposalStatus,
    pub execution_timestamp: Option<chrono::DateTime<Utc>>,
}

/// Proposal status
#[derive(Debug, Clone, PartialEq)]
enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

/// Governance participant
#[derive(Debug, Clone)]
struct GovernanceParticipant {
    pub address: String,
    pub voting_power: u64,
    pub reputation_score: f64,
    pub participation_rate: f64,
    pub role: ParticipantRole,
}

/// Participant roles in governance
#[derive(Debug, Clone)]
enum ParticipantRole {
    OracleOperator,
    DataProvider,
    Validator,
    WhaleTrader,
    CommunityMember,
}

/// Oracle governance system
struct OracleGovernance {
    proposals: HashMap<u64, GovernanceProposal>,
    participants: HashMap<String, GovernanceParticipant>,
    current_config: AggregationConfig,
    security_config: SecurityConfig,
    next_proposal_id: u64,
    quorum_threshold: f64,
    approval_threshold: f64,
}

impl OracleGovernance {
    fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            participants: HashMap::new(),
            current_config: AggregationConfig::default(),
            security_config: SecurityConfig::default(),
            next_proposal_id: 1,
            quorum_threshold: 0.3, // 30% participation required
            approval_threshold: 0.6, // 60% approval required
        }
    }

    fn add_participant(&mut self, participant: GovernanceParticipant) {
        self.participants.insert(participant.address.clone(), participant);
    }

    fn create_proposal(&mut self, proposal_type: ProposalType, proposer: String, title: String, description: String) -> u64 {
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let proposal = GovernanceProposal {
            id: proposal_id,
            proposal_type,
            proposer,
            title,
            description,
            voting_start: Utc::now(),
            voting_end: Utc::now() + chrono::Duration::days(7), // 7-day voting period
            votes_for: 0,
            votes_against: 0,
            status: ProposalStatus::Active,
            execution_timestamp: None,
        };

        self.proposals.insert(proposal_id, proposal);
        proposal_id
    }

    fn vote(&mut self, proposal_id: u64, voter: &str, support: bool) -> Result<()> {
        let participant = self.participants.get(voter)
            .ok_or_else(|| OracleError::GovernanceError {
                message: "Voter not registered".to_string(),
            })?;

        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| OracleError::GovernanceError {
                message: "Proposal not found".to_string(),
            })?;

        if proposal.status != ProposalStatus::Active {
            return Err(OracleError::GovernanceError {
                message: "Proposal is not active".to_string(),
            });
        }

        if Utc::now() > proposal.voting_end {
            return Err(OracleError::GovernanceError {
                message: "Voting period has ended".to_string(),
            });
        }

        // Calculate voting power based on role and reputation
        let voting_power = self.calculate_voting_power(participant);

        if support {
            proposal.votes_for += voting_power;
        } else {
            proposal.votes_against += voting_power;
        }

        Ok(())
    }

    fn calculate_voting_power(&self, participant: &GovernanceParticipant) -> u64 {
        let base_power = participant.voting_power;
        let reputation_multiplier = (participant.reputation_score * 1.5).min(2.0).max(0.5);
        let participation_multiplier = (participant.participation_rate * 1.2).min(1.5).max(0.8);

        let role_multiplier = match participant.role {
            ParticipantRole::OracleOperator => 1.5,
            ParticipantRole::DataProvider => 1.3,
            ParticipantRole::Validator => 1.2,
            ParticipantRole::WhaleTrader => 1.1,
            ParticipantRole::CommunityMember => 1.0,
        };

        (base_power as f64 * reputation_multiplier * participation_multiplier * role_multiplier) as u64
    }

    fn finalize_proposal(&mut self, proposal_id: u64) -> Result<()> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| OracleError::GovernanceError {
                message: "Proposal not found".to_string(),
            })?;

        if proposal.status != ProposalStatus::Active {
            return Ok(()); // Already finalized
        }

        if Utc::now() <= proposal.voting_end {
            return Err(OracleError::GovernanceError {
                message: "Voting period still active".to_string(),
            });
        }

        let total_votes = proposal.votes_for + proposal.votes_against;
        let total_voting_power: u64 = self.participants.values()
            .map(|p| self.calculate_voting_power(p))
            .sum();

        let participation_rate = total_votes as f64 / total_voting_power as f64;
        let approval_rate = if total_votes > 0 {
            proposal.votes_for as f64 / total_votes as f64
        } else {
            0.0
        };

        proposal.status = if participation_rate >= self.quorum_threshold && approval_rate >= self.approval_threshold {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        Ok(())
    }

    async fn execute_proposal(&mut self, proposal_id: u64) -> Result<()> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| OracleError::GovernanceError {
                message: "Proposal not found".to_string(),
            })?;

        if proposal.status != ProposalStatus::Passed {
            return Err(OracleError::GovernanceError {
                message: "Proposal has not passed".to_string(),
            });
        }

        // Execute the proposal based on its type
        match &proposal.proposal_type {
            ProposalType::UpdateAggregationConfig { parameter, new_value, .. } => {
                self.execute_aggregation_config_update(parameter, new_value)?;
            }
            ProposalType::UpdateSecurityConfig { parameter, new_value, .. } => {
                self.execute_security_config_update(parameter, new_value)?;
            }
            ProposalType::AdjustSourceWeight { source, new_weight, .. } => {
                self.current_config.source_weights.insert(*source, *new_weight);
            }
            ProposalType::AddDataSource { source_id, source_type, config } => {
                println!("Adding data source: {} ({:?})", source_id, source_type);
                // In a real implementation, this would register the new source
            }
            ProposalType::RemoveDataSource { source_id, reason } => {
                println!("Removing data source: {} (Reason: {})", source_id, reason);
                // In a real implementation, this would deregister the source
            }
            ProposalType::EmergencyCircuitBreaker { target, duration, reason } => {
                println!("Activating emergency circuit breaker on {} for {:?} (Reason: {})",
                    target, duration, reason);
                // In a real implementation, this would activate the circuit breaker
            }
        }

        proposal.status = ProposalStatus::Executed;
        proposal.execution_timestamp = Some(Utc::now());

        Ok(())
    }

    fn execute_aggregation_config_update(&mut self, parameter: &str, new_value: &str) -> Result<()> {
        match parameter {
            "min_sources" => {
                let value: usize = new_value.parse()
                    .map_err(|_| OracleError::ConfigurationError {
                        parameter: parameter.to_string(),
                        value: new_value.to_string(),
                    })?;
                self.current_config.min_sources = value;
            }
            "max_deviation" => {
                let value: f64 = new_value.parse()
                    .map_err(|_| OracleError::ConfigurationError {
                        parameter: parameter.to_string(),
                        value: new_value.to_string(),
                    })?;
                self.current_config.max_deviation = Decimal::from_f64_retain(value).unwrap();
            }
            "outlier_threshold" => {
                let value: f64 = new_value.parse()
                    .map_err(|_| OracleError::ConfigurationError {
                        parameter: parameter.to_string(),
                        value: new_value.to_string(),
                    })?;
                self.current_config.outlier_threshold = value;
            }
            _ => {
                return Err(OracleError::ConfigurationError {
                    parameter: parameter.to_string(),
                    value: new_value.to_string(),
                });
            }
        }

        Ok(())
    }

    fn execute_security_config_update(&mut self, parameter: &str, new_value: &str) -> Result<()> {
        match parameter {
            "max_price_deviation" => {
                let value: f64 = new_value.parse()
                    .map_err(|_| OracleError::ConfigurationError {
                        parameter: parameter.to_string(),
                        value: new_value.to_string(),
                    })?;
                self.security_config.max_price_deviation = value;
            }
            "circuit_breaker_threshold" => {
                let value: u32 = new_value.parse()
                    .map_err(|_| OracleError::ConfigurationError {
                        parameter: parameter.to_string(),
                        value: new_value.to_string(),
                    })?;
                self.security_config.circuit_breaker_threshold = value;
            }
            "mev_detection_sensitivity" => {
                let value: f64 = new_value.parse()
                    .map_err(|_| OracleError::ConfigurationError {
                        parameter: parameter.to_string(),
                        value: new_value.to_string(),
                    })?;
                self.security_config.mev_detection_sensitivity = value;
            }
            _ => {
                return Err(OracleError::ConfigurationError {
                    parameter: parameter.to_string(),
                    value: new_value.to_string(),
                });
            }
        }

        Ok(())
    }

    fn get_active_proposals(&self) -> Vec<&GovernanceProposal> {
        self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    fn get_proposal_stats(&self) -> (usize, usize, usize, usize) {
        let mut active = 0;
        let mut passed = 0;
        let mut rejected = 0;
        let mut executed = 0;

        for proposal in self.proposals.values() {
            match proposal.status {
                ProposalStatus::Active => active += 1,
                ProposalStatus::Passed => passed += 1,
                ProposalStatus::Rejected => rejected += 1,
                ProposalStatus::Executed => executed += 1,
                ProposalStatus::Cancelled => {}
            }
        }

        (active, passed, rejected, executed)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("🏛️  Moby Oracle - Governance System Example");
    println!("==========================================");

    // Step 1: Initialize governance system
    println!("\n🏗️  Initializing Governance System:");
    let mut governance = initialize_governance_system();

    // Step 2: Demonstrate proposal creation
    println!("\n📝 Creating Governance Proposals:");
    await create_sample_proposals(&mut governance).await?;

    // Step 3: Simulate voting process
    println!("\n🗳️  Conducting Voting Process:");
    await simulate_voting_process(&mut governance).await?;

    // Step 4: Finalize and execute proposals
    println!("\n⚖️  Finalizing and Executing Proposals:");
    await finalize_and_execute_proposals(&mut governance).await?;

    // Step 5: Demonstrate emergency governance
    println!("\n🚨 Emergency Governance Procedures:");
    await demonstrate_emergency_governance(&mut governance).await?;

    // Step 6: Source management through governance
    println!("\n🔧 Source Management via Governance:");
    await demonstrate_source_management(&mut governance).await?;

    // Step 7: Reputation-based governance
    println!("\n🏆 Reputation-Based Governance:");
    await demonstrate_reputation_governance(&mut governance).await?;

    // Step 8: Generate governance report
    println!("\n📊 Governance System Report:");
    generate_governance_report(&governance);

    println!("\n✅ Oracle governance example completed!");
    Ok(())
}

/// Initialize the governance system with participants
fn initialize_governance_system() -> OracleGovernance {
    let mut governance = OracleGovernance::new();

    // Add various types of governance participants
    let participants = vec![
        GovernanceParticipant {
            address: "oracle_operator_1".to_string(),
            voting_power: 1000,
            reputation_score: 0.95,
            participation_rate: 0.85,
            role: ParticipantRole::OracleOperator,
        },
        GovernanceParticipant {
            address: "oracle_operator_2".to_string(),
            voting_power: 800,
            reputation_score: 0.90,
            participation_rate: 0.90,
            role: ParticipantRole::OracleOperator,
        },
        GovernanceParticipant {
            address: "data_provider_chainlink".to_string(),
            voting_power: 1500,
            reputation_score: 0.98,
            participation_rate: 0.95,
            role: ParticipantRole::DataProvider,
        },
        GovernanceParticipant {
            address: "data_provider_pyth".to_string(),
            voting_power: 1200,
            reputation_score: 0.92,
            participation_rate: 0.80,
            role: ParticipantRole::DataProvider,
        },
        GovernanceParticipant {
            address: "validator_node_1".to_string(),
            voting_power: 600,
            reputation_score: 0.88,
            participation_rate: 0.75,
            role: ParticipantRole::Validator,
        },
        GovernanceParticipant {
            address: "whale_trader_alpha".to_string(),
            voting_power: 2000,
            reputation_score: 0.85,
            participation_rate: 0.60,
            role: ParticipantRole::WhaleTrader,
        },
        GovernanceParticipant {
            address: "whale_trader_beta".to_string(),
            voting_power: 1800,
            reputation_score: 0.80,
            participation_rate: 0.70,
            role: ParticipantRole::WhaleTrader,
        },
        GovernanceParticipant {
            address: "community_member_1".to_string(),
            voting_power: 300,
            reputation_score: 0.75,
            participation_rate: 0.50,
            role: ParticipantRole::CommunityMember,
        },
        GovernanceParticipant {
            address: "community_member_2".to_string(),
            voting_power: 250,
            reputation_score: 0.70,
            participation_rate: 0.65,
            role: ParticipantRole::CommunityMember,
        },
    ];

    for participant in participants {
        println!("   Added participant: {} ({:?}, {} voting power)",
            participant.address, participant.role, participant.voting_power);
        governance.add_participant(participant);
    }

    let total_voting_power: u64 = governance.participants.values()
        .map(|p| governance.calculate_voting_power(p))
        .sum();

    println!("   Total effective voting power: {}", total_voting_power);
    println!("   Quorum threshold: {:.1}%", governance.quorum_threshold * 100.0);
    println!("   Approval threshold: {:.1}%", governance.approval_threshold * 100.0);

    governance
}

/// Create sample governance proposals
async fn create_sample_proposals(governance: &mut OracleGovernance) -> Result<()> {
    let proposals = vec![
        (
            ProposalType::UpdateAggregationConfig {
                parameter: "min_sources".to_string(),
                old_value: "3".to_string(),
                new_value: "4".to_string(),
            },
            "Increase Minimum Source Requirement".to_string(),
            "Proposal to increase minimum required sources from 3 to 4 for improved reliability".to_string(),
            "oracle_operator_1".to_string(),
        ),
        (
            ProposalType::UpdateSecurityConfig {
                parameter: "max_price_deviation".to_string(),
                old_value: "0.05".to_string(),
                new_value: "0.03".to_string(),
            },
            "Tighten Price Deviation Limits".to_string(),
            "Reduce maximum allowed price deviation from 5% to 3% for better manipulation resistance".to_string(),
            "data_provider_chainlink".to_string(),
        ),
        (
            ProposalType::AdjustSourceWeight {
                source: DataSource::Chainlink,
                old_weight: 0.25,
                new_weight: 0.35,
            },
            "Increase Chainlink Weight".to_string(),
            "Increase Chainlink's weight in aggregation due to superior performance metrics".to_string(),
            "validator_node_1".to_string(),
        ),
        (
            ProposalType::AddDataSource {
                source_id: "uniswap_v3".to_string(),
                source_type: DataSource::API3,
                config: SourceConfig {
                    endpoint_url: "https://api.uniswap.org/v3".to_string(),
                    api_key: None,
                    timeout: Duration::from_secs(30),
                    retry_attempts: 3,
                    rate_limit: 100,
                    enabled: true,
                    metadata: HashMap::new(),
                },
            },
            "Add Uniswap V3 Data Source".to_string(),
            "Integrate Uniswap V3 as an additional DEX data source for improved price discovery".to_string(),
            "whale_trader_alpha".to_string(),
        ),
    ];

    for (proposal_type, title, description, proposer) in proposals {
        let proposal_id = governance.create_proposal(proposal_type, proposer, title.clone(), description);
        println!("   Created Proposal #{}: {}", proposal_id, title);
    }

    Ok(())
}

/// Simulate the voting process
async fn simulate_voting_process(governance: &mut OracleGovernance) -> Result<()> {
    let active_proposals = governance.get_active_proposals();
    println!("   Found {} active proposals", active_proposals.len());

    // Simulate voting over multiple rounds
    for round in 1..=3 {
        println!("\n   Voting Round {}:", round);

        for proposal in &active_proposals {
            let proposal_id = proposal.id;
            println!("     Proposal #{}: {}", proposal_id, proposal.title);

            // Simulate voting behavior based on participant roles and proposal type
            for participant in governance.participants.values() {
                let vote_probability = calculate_vote_probability(participant, &proposal.proposal_type);
                let support_probability = calculate_support_probability(participant, &proposal.proposal_type);

                if rand::random::<f64>() < vote_probability {
                    let support = rand::random::<f64>() < support_probability;

                    match governance.vote(proposal_id, &participant.address, support) {
                        Ok(()) => {
                            let voting_power = governance.calculate_voting_power(participant);
                            println!("       {} voted {} (power: {})",
                                participant.address,
                                if support { "FOR" } else { "AGAINST" },
                                voting_power);
                        }
                        Err(e) => {
                            println!("       {} vote failed: {}", participant.address, e);
                        }
                    }
                }
            }

            // Show current vote tally
            if let Some(proposal) = governance.proposals.get(&proposal_id) {
                let total_votes = proposal.votes_for + proposal.votes_against;
                if total_votes > 0 {
                    let approval_rate = proposal.votes_for as f64 / total_votes as f64;
                    println!("       Current tally: {} FOR, {} AGAINST ({:.1}% approval)",
                        proposal.votes_for, proposal.votes_against, approval_rate * 100.0);
                }
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

/// Calculate voting probability based on participant characteristics
fn calculate_vote_probability(participant: &GovernanceParticipant, proposal_type: &ProposalType) -> f64 {
    let base_probability = participant.participation_rate;

    let role_interest = match (&participant.role, proposal_type) {
        (ParticipantRole::OracleOperator, ProposalType::UpdateAggregationConfig { .. }) => 1.2,
        (ParticipantRole::DataProvider, ProposalType::AdjustSourceWeight { .. }) => 1.3,
        (ParticipantRole::Validator, ProposalType::UpdateSecurityConfig { .. }) => 1.2,
        (ParticipantRole::WhaleTrader, ProposalType::AddDataSource { .. }) => 1.1,
        _ => 1.0,
    };

    (base_probability * role_interest).min(1.0)
}

/// Calculate support probability based on participant characteristics
fn calculate_support_probability(participant: &GovernanceParticipant, proposal_type: &ProposalType) -> f64 {
    let base_support = 0.6; // 60% base support rate

    let role_alignment = match (&participant.role, proposal_type) {
        (ParticipantRole::OracleOperator, ProposalType::UpdateAggregationConfig { .. }) => 0.8,
        (ParticipantRole::DataProvider, ProposalType::AdjustSourceWeight { source, .. }) => {
            // Data providers support weight increases for their own sources
            match source {
                DataSource::Chainlink => 0.9,
                _ => 0.5,
            }
        }
        (ParticipantRole::Validator, ProposalType::UpdateSecurityConfig { .. }) => 0.85,
        (ParticipantRole::WhaleTrader, ProposalType::AddDataSource { .. }) => 0.75,
        _ => base_support,
    };

    role_alignment * participant.reputation_score
}

/// Finalize and execute passed proposals
async fn finalize_and_execute_proposals(governance: &mut OracleGovernance) -> Result<()> {
    // Simulate end of voting period
    for proposal in governance.proposals.values_mut() {
        proposal.voting_end = Utc::now() - chrono::Duration::seconds(1);
    }

    let proposal_ids: Vec<u64> = governance.proposals.keys().cloned().collect();

    for proposal_id in proposal_ids {
        // Finalize the proposal
        governance.finalize_proposal(proposal_id)?;

        if let Some(proposal) = governance.proposals.get(&proposal_id) {
            println!("   Proposal #{}: {} - {:?}",
                proposal_id, proposal.title, proposal.status);

            let total_votes = proposal.votes_for + proposal.votes_against;
            if total_votes > 0 {
                let approval_rate = proposal.votes_for as f64 / total_votes as f64;
                println!("     Final tally: {} FOR, {} AGAINST ({:.1}% approval)",
                    proposal.votes_for, proposal.votes_against, approval_rate * 100.0);
            }

            // Execute if passed
            if proposal.status == ProposalStatus::Passed {
                match governance.execute_proposal(proposal_id).await {
                    Ok(()) => {
                        println!("     ✅ Proposal executed successfully");
                    }
                    Err(e) => {
                        println!("     ❌ Execution failed: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Demonstrate emergency governance procedures
async fn demonstrate_emergency_governance(governance: &mut OracleGovernance) -> Result<()> {
    println!("   Simulating emergency scenario: Suspected oracle manipulation detected");

    // Create emergency proposal with shortened voting period
    let emergency_proposal_id = governance.create_proposal(
        ProposalType::EmergencyCircuitBreaker {
            target: "data_source_malicious".to_string(),
            duration: Duration::from_hours(24),
            reason: "Suspected price manipulation detected".to_string(),
        },
        "emergency_coordinator".to_string(),
        "Emergency Circuit Breaker Activation".to_string(),
        "Immediate circuit breaker activation due to detected anomalous behavior".to_string(),
    );

    // Shorten voting period for emergency
    if let Some(proposal) = governance.proposals.get_mut(&emergency_proposal_id) {
        proposal.voting_end = Utc::now() + chrono::Duration::hours(1); // 1-hour emergency voting
        println!("   Emergency Proposal #{} created with 1-hour voting window", emergency_proposal_id);
    }

    // Simulate rapid emergency voting
    println!("\n   Emergency voting in progress...");

    let critical_participants = vec![
        "oracle_operator_1",
        "oracle_operator_2",
        "data_provider_chainlink",
        "validator_node_1",
    ];

    for participant_addr in critical_participants {
        // Emergency procedures have higher participation rates
        if rand::random::<f64>() < 0.9 { // 90% emergency participation
            let support = rand::random::<f64>() < 0.8; // 80% support for security measures

            match governance.vote(emergency_proposal_id, participant_addr, support) {
                Ok(()) => {
                    if let Some(participant) = governance.participants.get(participant_addr) {
                        let voting_power = governance.calculate_voting_power(participant);
                        println!("     {} emergency vote: {} (power: {})",
                            participant_addr,
                            if support { "FOR" } else { "AGAINST" },
                            voting_power);
                    }
                }
                Err(e) => {
                    println!("     {} emergency vote failed: {}", participant_addr, e);
                }
            }
        }
    }

    // Fast-track finalization for emergency
    if let Some(proposal) = governance.proposals.get_mut(&emergency_proposal_id) {
        proposal.voting_end = Utc::now() - chrono::Duration::seconds(1);
    }

    governance.finalize_proposal(emergency_proposal_id)?;

    if let Some(proposal) = governance.proposals.get(&emergency_proposal_id) {
        println!("\n   Emergency proposal result: {:?}", proposal.status);

        if proposal.status == ProposalStatus::Passed {
            governance.execute_proposal(emergency_proposal_id).await?;
            println!("   🚨 Emergency circuit breaker activated");
        }
    }

    Ok(())
}

/// Demonstrate source management through governance
async fn demonstrate_source_management(governance: &mut OracleGovernance) -> Result<()> {
    // Create proposals for source management
    let source_proposals = vec![
        (
            ProposalType::RemoveDataSource {
                source_id: "unreliable_source".to_string(),
                reason: "Consistently poor performance and low reliability".to_string(),
            },
            "Remove Unreliable Data Source".to_string(),
        ),
        (
            ProposalType::AddDataSource {
                source_id: "new_defi_oracle".to_string(),
                source_type: DataSource::UMA,
                config: SourceConfig {
                    endpoint_url: "https://api.newdefi.com/v1".to_string(),
                    api_key: Some("test_key".to_string()),
                    timeout: Duration::from_secs(30),
                    retry_attempts: 3,
                    rate_limit: 150,
                    enabled: true,
                    metadata: HashMap::new(),
                },
            },
            "Add New DeFi Oracle Source".to_string(),
        ),
    ];

    for (proposal_type, title) in source_proposals {
        let proposal_id = governance.create_proposal(
            proposal_type,
            "source_manager".to_string(),
            title.clone(),
            format!("Source management proposal: {}", title),
        );

        println!("   Created source management proposal #{}: {}", proposal_id, title);

        // Simulate focused voting from relevant stakeholders
        let relevant_voters = vec![
            "oracle_operator_1",
            "oracle_operator_2",
            "data_provider_chainlink",
            "data_provider_pyth",
        ];

        for voter in relevant_voters {
            if rand::random::<f64>() < 0.8 { // 80% participation for source management
                let support = rand::random::<f64>() < 0.7; // 70% general support

                if let Ok(()) = governance.vote(proposal_id, voter, support) {
                    println!("     {} voted {}", voter, if support { "FOR" } else { "AGAINST" });
                }
            }
        }

        // Fast-track for demonstration
        if let Some(proposal) = governance.proposals.get_mut(&proposal_id) {
            proposal.voting_end = Utc::now() - chrono::Duration::seconds(1);
        }

        governance.finalize_proposal(proposal_id)?;

        if let Some(proposal) = governance.proposals.get(&proposal_id) {
            if proposal.status == ProposalStatus::Passed {
                governance.execute_proposal(proposal_id).await?;
                println!("     ✅ Source management proposal executed");
            } else {
                println!("     ❌ Source management proposal rejected");
            }
        }
    }

    Ok(())
}

/// Demonstrate reputation-based governance
async fn demonstrate_reputation_governance(governance: &mut OracleGovernance) -> Result<()> {
    println!("   Demonstrating how reputation affects governance participation:");

    // Show voting power calculations for different participants
    println!("\n   Voting Power Analysis:");
    for participant in governance.participants.values() {
        let base_power = participant.voting_power;
        let effective_power = governance.calculate_voting_power(participant);
        let multiplier = effective_power as f64 / base_power as f64;

        println!("     {} ({:?}):", participant.address, participant.role);
        println!("       Base Power: {}", base_power);
        println!("       Effective Power: {} ({:.2}x multiplier)", effective_power, multiplier);
        println!("       Reputation: {:.2}, Participation: {:.1}%",
            participant.reputation_score, participant.participation_rate * 100.0);
    }

    // Simulate reputation changes over time
    println!("\n   Simulating reputation evolution:");

    let mut governance_copy = governance.clone();

    // Simulate good behavior increasing reputation
    if let Some(participant) = governance_copy.participants.get_mut("community_member_1") {
        println!("   {} improving reputation through consistent participation:", participant.address);
        println!("     Before: Rep {:.2}, Power {}",
            participant.reputation_score,
            governance_copy.calculate_voting_power(participant));

        participant.reputation_score = 0.90; // Improved reputation
        participant.participation_rate = 0.85; // Increased participation

        println!("     After: Rep {:.2}, Power {}",
            participant.reputation_score,
            governance_copy.calculate_voting_power(participant));
    }

    // Simulate poor behavior decreasing reputation
    if let Some(participant) = governance_copy.participants.get_mut("whale_trader_beta") {
        println!("   {} reputation decline due to poor governance behavior:", participant.address);
        println!("     Before: Rep {:.2}, Power {}",
            participant.reputation_score,
            governance_copy.calculate_voting_power(participant));

        participant.reputation_score = 0.60; // Decreased reputation
        participant.participation_rate = 0.40; // Reduced participation

        println!("     After: Rep {:.2}, Power {}",
            participant.reputation_score,
            governance_copy.calculate_voting_power(participant));
    }

    // Create a test proposal to show reputation impact
    let test_proposal_id = governance.create_proposal(
        ProposalType::UpdateAggregationConfig {
            parameter: "outlier_threshold".to_string(),
            old_value: "2.0".to_string(),
            new_value: "2.5".to_string(),
        },
        "reputation_test".to_string(),
        "Reputation Impact Test Proposal".to_string(),
        "Testing how reputation affects governance outcomes".to_string(),
    );

    println!("\n   Testing reputation impact on proposal #{}", test_proposal_id);

    // High reputation participants vote
    let high_rep_votes = governance.vote(test_proposal_id, "data_provider_chainlink", true);
    let low_rep_votes = governance.vote(test_proposal_id, "community_member_2", true);

    if high_rep_votes.is_ok() && low_rep_votes.is_ok() {
        if let Some(proposal) = governance.proposals.get(&test_proposal_id) {
            println!("   High reputation vote impact: More significant due to multipliers");
            println!("   Total votes: {}", proposal.votes_for);
        }
    }

    Ok(())
}

/// Generate comprehensive governance report
fn generate_governance_report(governance: &OracleGovernance) {
    let (active, passed, rejected, executed) = governance.get_proposal_stats();

    let total_voting_power: u64 = governance.participants.values()
        .map(|p| governance.calculate_voting_power(p))
        .sum();

    let avg_reputation: f64 = governance.participants.values()
        .map(|p| p.reputation_score)
        .sum::<f64>() / governance.participants.len() as f64;

    let avg_participation: f64 = governance.participants.values()
        .map(|p| p.participation_rate)
        .sum::<f64>() / governance.participants.len() as f64;

    println!("╭─────────────────────────────────────────────────────────╮");
    println!("│                   GOVERNANCE REPORT                    │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ Participants: {:41} │", governance.participants.len());
    println!("│ Total Voting Power: {:35} │", total_voting_power);
    println!("│ Average Reputation: {:33.2} │", avg_reputation);
    println!("│ Average Participation: {:30.1}% │", avg_participation * 100.0);
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│                    PROPOSAL SUMMARY                    │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ Total Proposals: {:38} │", governance.proposals.len());
    println!("│ Active: {:45} │", active);
    println!("│ Passed: {:45} │", passed);
    println!("│ Rejected: {:43} │", rejected);
    println!("│ Executed: {:43} │", executed);
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│                  GOVERNANCE SETTINGS                   │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ Quorum Threshold: {:33.1}% │", governance.quorum_threshold * 100.0);
    println!("│ Approval Threshold: {:31.1}% │", governance.approval_threshold * 100.0);
    println!("│ Min Sources: {:40} │", governance.current_config.min_sources);
    println!("│ Max Price Deviation: {:30.1}% │", governance.security_config.max_price_deviation * 100.0);
    println!("│ Circuit Breaker Threshold: {:27} │", governance.security_config.circuit_breaker_threshold);
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│                     PARTICIPANT ROLES                  │");
    println!("├─────────────────────────────────────────────────────────┤");

    let mut role_counts = HashMap::new();
    for participant in governance.participants.values() {
        *role_counts.entry(format!("{:?}", participant.role)).or_insert(0) += 1;
    }

    for (role, count) in role_counts {
        println!("│ {:<20}: {:>30} │", role, count);
    }

    println!("╰─────────────────────────────────────────────────────────╯");
}