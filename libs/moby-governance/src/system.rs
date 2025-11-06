//! Main governance system coordinator

use crate::{
    delegation::{DelegationSystem, DelegationConfig},
    emergency::{EmergencySystem, EmergencyConfig},
    error::{GovernanceError, GovernanceResult},
    parameters::{ParameterManager, ParameterConfig},
    proposals::{ProposalManager, ProposalConfig, ProposalType, ProposalPriority, ProposalId},
    tokens::{TokenSystem, TokenConfig, GovernanceToken},
    treasury::{Treasury, TreasuryConfig},
    upgrades::{UpgradeManager, UpgradeConfig},
    voting::{VotingSystem, VotingConfig, VoteType, VotingStrategy},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main governance system configuration
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    /// Proposal management configuration
    pub proposal_config: ProposalConfig,
    /// Voting system configuration
    pub voting_config: VotingConfig,
    /// Delegation system configuration
    pub delegation_config: DelegationConfig,
    /// Token system configuration
    pub token_config: TokenConfig,
    /// Treasury configuration
    pub treasury_config: TreasuryConfig,
    /// Parameter management configuration
    pub parameter_config: ParameterConfig,
    /// Upgrade management configuration
    pub upgrade_config: UpgradeConfig,
    /// Emergency system configuration
    pub emergency_config: EmergencyConfig,
    /// Whether the system is initialized
    pub initialized: bool,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            proposal_config: ProposalConfig::default(),
            voting_config: VotingConfig::default(),
            delegation_config: DelegationConfig::default(),
            token_config: TokenConfig::default(),
            treasury_config: TreasuryConfig::default(),
            parameter_config: ParameterConfig::default(),
            upgrade_config: UpgradeConfig::default(),
            emergency_config: EmergencyConfig::default(),
            initialized: false,
        }
    }
}

/// Privacy level for different governance operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivacyLevel {
    /// Basic transparency
    Public,
    /// Partial privacy (some details hidden)
    Confidential,
    /// High privacy (minimal disclosure)
    Private,
    /// Anonymous operations
    Anonymous,
}

/// Governance participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Participant address/identifier
    pub address: String,
    /// Voting power
    pub voting_power: u64,
    /// Delegated voting power
    pub delegated_power: u64,
    /// Participation level
    pub participation_level: ParticipationLevel,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
    /// Reputation score
    pub reputation: u64,
    /// Privacy preferences
    pub privacy_level: PrivacyLevel,
}

/// Levels of governance participation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParticipationLevel {
    /// Observer only
    Observer,
    /// Can vote on proposals
    Voter,
    /// Can create proposals
    Proposer,
    /// Can review and approve proposals
    Reviewer,
    /// Full governance rights
    Governor,
}

/// Governance event for tracking and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvent {
    /// Event identifier
    pub id: String,
    /// Event type
    pub event_type: GovernanceEventType,
    /// Participant involved
    pub participant: String,
    /// Associated proposal (if any)
    pub proposal_id: Option<ProposalId>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event data
    pub data: HashMap<String, String>,
    /// Privacy level of this event
    pub privacy_level: PrivacyLevel,
}

/// Types of governance events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceEventType {
    /// Proposal created
    ProposalCreated,
    /// Vote cast
    VoteCast,
    /// Delegation created
    DelegationCreated,
    /// Delegation revoked
    DelegationRevoked,
    /// Proposal executed
    ProposalExecuted,
    /// Parameter updated
    ParameterUpdated,
    /// Emergency action taken
    EmergencyAction,
    /// Treasury action
    TreasuryAction,
    /// Upgrade deployed
    UpgradeDeployed,
    /// Token staked
    TokenStaked,
    /// Token unstaked
    TokenUnstaked,
}

/// Main governance system that coordinates all subsystems
pub struct GovernanceSystem {
    /// Configuration
    config: GovernanceConfig,
    /// Proposal management
    proposal_manager: ProposalManager,
    /// Voting system
    voting_system: VotingSystem,
    /// Delegation system
    delegation_system: DelegationSystem,
    /// Token system
    token_system: TokenSystem,
    /// Treasury management
    treasury: Treasury,
    /// Parameter management
    parameter_manager: ParameterManager,
    /// Upgrade management
    upgrade_manager: UpgradeManager,
    /// Emergency system
    emergency_system: EmergencySystem,
    /// Registered participants
    participants: HashMap<String, Participant>,
    /// Governance events
    events: Vec<GovernanceEvent>,
    /// Event counter
    event_counter: u64,
}

impl GovernanceSystem {
    /// Create a new governance system
    pub fn new() -> Self {
        let config = GovernanceConfig::default();

        // Create default governance token
        let governance_token = GovernanceToken {
            symbol: "MOBY".to_string(),
            name: "Moby Governance Token".to_string(),
            total_supply: 1_000_000_000,
            circulating_supply: 500_000_000,
            decimals: 18,
            metadata: HashMap::new(),
        };

        Self {
            proposal_manager: ProposalManager::new(config.proposal_config.clone()),
            voting_system: VotingSystem::new(config.voting_config.clone()),
            delegation_system: DelegationSystem::new(config.delegation_config.clone()),
            token_system: TokenSystem::new(config.token_config.clone(), governance_token),
            treasury: Treasury::new(
                config.treasury_config.clone(),
                vec!["treasury_signer_1".to_string(), "treasury_signer_2".to_string()],
                2,
            ),
            parameter_manager: ParameterManager::new(config.parameter_config.clone()),
            upgrade_manager: UpgradeManager::new(config.upgrade_config.clone(), "1.0.0".to_string()),
            emergency_system: EmergencySystem::new(config.emergency_config.clone()),
            participants: HashMap::new(),
            events: Vec::new(),
            event_counter: 0,
            config,
        }
    }

    /// Initialize the governance system with custom configuration
    pub async fn initialize(&mut self, config: GovernanceConfig) -> GovernanceResult<()> {
        if self.config.initialized {
            return Err(GovernanceError::SystemConfigurationError {
                message: "System already initialized".to_string(),
            });
        }

        self.config = config;
        self.config.initialized = true;

        // Initialize default parameters
        self.initialize_default_parameters().await?;

        // Record initialization event
        self.record_event(
            GovernanceEventType::ParameterUpdated,
            "system".to_string(),
            None,
            HashMap::from([("action".to_string(), "initialize".to_string())]),
            PrivacyLevel::Public,
        ).await?;

        Ok(())
    }

    /// Register a new governance participant
    pub async fn register_participant(
        &mut self,
        address: String,
        initial_token_balance: u64,
        participation_level: ParticipationLevel,
        privacy_level: PrivacyLevel,
    ) -> GovernanceResult<()> {
        if self.participants.contains_key(&address) {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Participant {} already registered", address),
            });
        }

        // Initialize token holder
        self.token_system.initialize_holder(address.clone(), initial_token_balance).await?;

        // Calculate initial voting power
        let voting_power = self.token_system.calculate_voting_power(&address).await;

        // Set voting power in voting and delegation systems
        self.voting_system.set_voting_power(address.clone(), voting_power).await?;
        self.delegation_system.set_base_power(address.clone(), voting_power).await?;

        let participant = Participant {
            address: address.clone(),
            voting_power,
            delegated_power: 0,
            participation_level,
            registered_at: Utc::now(),
            last_active: Utc::now(),
            reputation: 100, // Starting reputation
            privacy_level,
        };

        self.participants.insert(address.clone(), participant);

        // Record event (with appropriate privacy)
        self.record_event(
            GovernanceEventType::ProposalCreated, // Reusing for registration
            address,
            None,
            HashMap::from([
                ("action".to_string(), "register".to_string()),
                ("balance".to_string(), initial_token_balance.to_string()),
            ]),
            privacy_level,
        ).await?;

        Ok(())
    }

    /// Create a new governance proposal
    pub async fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        title: String,
        description: String,
        proposer: String,
        voting_duration: Option<chrono::Duration>,
        privacy_level: PrivacyLevel,
    ) -> GovernanceResult<ProposalId> {
        // Validate proposer
        let participant = self.participants.get_mut(&proposer)
            .ok_or_else(|| GovernanceError::UnauthorizedAccess {
                action: "create proposal".to_string(),
            })?;

        // Check participation level
        if !matches!(participant.participation_level,
            ParticipationLevel::Proposer |
            ParticipationLevel::Reviewer |
            ParticipationLevel::Governor
        ) {
            return Err(GovernanceError::UnauthorizedAccess {
                action: "create proposal".to_string(),
            });
        }

        // Check minimum token requirement
        if participant.voting_power < self.config.token_config.min_proposal_tokens {
            return Err(GovernanceError::InsufficientVotingPower {
                required: self.config.token_config.min_proposal_tokens,
                available: participant.voting_power,
            });
        }

        // Create proposal
        let proposal_id = self.proposal_manager.create_proposal(
            proposal_type,
            title,
            description,
            proposer.clone(),
            voting_duration,
            ProposalPriority::Medium,
            vec![],
            HashMap::new(),
        ).await?;

        // Set quorum requirement
        let total_voting_power = self.voting_system.total_eligible_power;
        self.proposal_manager.set_quorum_requirement(&proposal_id, total_voting_power).await?;

        // Update participant activity
        participant.last_active = Utc::now();

        // Record event
        self.record_event(
            GovernanceEventType::ProposalCreated,
            proposer,
            Some(proposal_id),
            HashMap::from([("proposal_type".to_string(), format!("{:?}", proposal_type))]),
            privacy_level,
        ).await?;

        Ok(proposal_id)
    }

    /// Cast a vote on a proposal
    pub async fn vote(
        &mut self,
        proposal_id: ProposalId,
        voter: String,
        vote_type: VoteType,
        justification: Option<String>,
        privacy_level: PrivacyLevel,
    ) -> GovernanceResult<()> {
        // Validate voter
        let participant = self.participants.get_mut(&voter)
            .ok_or_else(|| GovernanceError::UnauthorizedAccess {
                action: "vote".to_string(),
            })?;

        // Check participation level
        if matches!(participant.participation_level, ParticipationLevel::Observer) {
            return Err(GovernanceError::UnauthorizedAccess {
                action: "vote".to_string(),
            });
        }

        // Get proposal
        let proposal = self.proposal_manager.get_proposal(&proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound {
                id: proposal_id.to_string(),
            })?;

        // Cast vote
        self.voting_system.cast_vote(proposal, voter.clone(), vote_type.clone(), justification).await?;

        // Update vote counts in proposal
        let voting_result = self.voting_system.calculate_results(proposal, None).await?;
        self.proposal_manager.update_vote_counts(
            &proposal_id,
            voting_result.votes_for,
            voting_result.votes_against,
            voting_result.votes_abstain,
        ).await?;

        // Update participant activity
        participant.last_active = Utc::now();
        participant.reputation += 1; // Small reputation bonus for voting

        // Record event
        self.record_event(
            GovernanceEventType::VoteCast,
            voter,
            Some(proposal_id),
            HashMap::from([("vote_type".to_string(), format!("{:?}", vote_type))]),
            privacy_level,
        ).await?;

        Ok(())
    }

    /// Delegate voting power
    pub async fn delegate(
        &mut self,
        delegator: String,
        delegate: String,
        amount: u64,
        privacy_level: PrivacyLevel,
    ) -> GovernanceResult<()> {
        // Validate delegator
        let delegator_participant = self.participants.get_mut(&delegator)
            .ok_or_else(|| GovernanceError::UnauthorizedAccess {
                action: "delegate".to_string(),
            })?;

        // Validate delegate exists
        if !self.participants.contains_key(&delegate) {
            return Err(GovernanceError::InvalidAddress {
                address: delegate,
            });
        }

        // Create delegation
        self.delegation_system.create_delegation(
            delegator.clone(),
            delegate.clone(),
            crate::delegation::DelegationPower::Fixed(amount),
            crate::delegation::DelegationScope::All,
            None,
            HashMap::new(),
        ).await?;

        // Update participant activity
        delegator_participant.last_active = Utc::now();

        // Record event
        self.record_event(
            GovernanceEventType::DelegationCreated,
            delegator,
            None,
            HashMap::from([
                ("delegate".to_string(), delegate),
                ("amount".to_string(), amount.to_string()),
            ]),
            privacy_level,
        ).await?;

        Ok(())
    }

    /// Execute a passed proposal
    pub async fn execute_proposal(
        &mut self,
        proposal_id: ProposalId,
        executor: String,
    ) -> GovernanceResult<()> {
        // Validate executor
        let participant = self.participants.get_mut(&executor)
            .ok_or_else(|| GovernanceError::UnauthorizedAccess {
                action: "execute proposal".to_string(),
            })?;

        // Check participation level
        if !matches!(participant.participation_level,
            ParticipationLevel::Reviewer |
            ParticipationLevel::Governor
        ) {
            return Err(GovernanceError::UnauthorizedAccess {
                action: "execute proposal".to_string(),
            });
        }

        // Get and validate proposal
        let proposal = self.proposal_manager.get_proposal(&proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound {
                id: proposal_id.to_string(),
            })?;

        if !self.proposal_manager.has_proposal_passed(proposal) {
            return Err(GovernanceError::OperationFailed {
                reason: "Proposal has not passed".to_string(),
            });
        }

        // Execute based on proposal type
        match &proposal.proposal_type {
            ProposalType::ParameterUpdate { parameter, new_value, .. } => {
                self.execute_parameter_update(parameter, new_value, &executor).await?;
            }
            ProposalType::TreasurySpend { recipient, amount, .. } => {
                self.execute_treasury_spend(recipient, *amount, &executor).await?;
            }
            _ => {
                return Err(GovernanceError::OperationFailed {
                    reason: "Proposal type execution not implemented".to_string(),
                });
            }
        }

        // Update proposal status
        self.proposal_manager.update_status(&proposal_id, crate::proposals::ProposalStatus::Executed).await?;

        // Update participant activity
        participant.last_active = Utc::now();
        participant.reputation += 5; // Bonus for executing proposals

        // Record event
        self.record_event(
            GovernanceEventType::ProposalExecuted,
            executor,
            Some(proposal_id),
            HashMap::new(),
            PrivacyLevel::Public,
        ).await?;

        Ok(())
    }

    /// Get governance statistics
    pub async fn get_governance_statistics(&self) -> GovernanceStatistics {
        let mut stats = GovernanceStatistics::default();

        // Participant statistics
        stats.total_participants = self.participants.len();
        for participant in self.participants.values() {
            match participant.participation_level {
                ParticipationLevel::Observer => stats.observers += 1,
                ParticipationLevel::Voter => stats.voters += 1,
                ParticipationLevel::Proposer => stats.proposers += 1,
                ParticipationLevel::Reviewer => stats.reviewers += 1,
                ParticipationLevel::Governor => stats.governors += 1,
            }
        }

        // Proposal statistics
        let proposal_stats = self.proposal_manager.get_statistics();
        stats.total_proposals = proposal_stats.total_proposals;
        stats.active_proposals = proposal_stats.active_voting;
        stats.executed_proposals = proposal_stats.executed_proposals;

        // Token statistics
        let token_stats = self.token_system.get_token_statistics();
        stats.total_voting_power = token_stats.total_balance + token_stats.total_staked + token_stats.total_locked;
        stats.staked_tokens = token_stats.total_staked;

        // Treasury statistics
        let treasury_stats = self.treasury.get_treasury_statistics();
        stats.treasury_value = treasury_stats.total_assets;

        // Delegation statistics
        let delegation_stats = self.delegation_system.get_delegation_statistics();
        stats.total_delegations = delegation_stats.total_delegations;

        // Emergency statistics
        let emergency_stats = self.emergency_system.get_emergency_statistics();
        stats.emergency_actions = emergency_stats.executed;

        stats.total_events = self.events.len();

        stats
    }

    /// Get participant information
    pub fn get_participant(&self, address: &str) -> Option<&Participant> {
        self.participants.get(address)
    }

    /// Get recent governance events
    pub fn get_recent_events(&self, limit: usize, privacy_filter: Option<PrivacyLevel>) -> Vec<&GovernanceEvent> {
        let mut events: Vec<&GovernanceEvent> = self.events.iter().collect();

        // Filter by privacy level if specified
        if let Some(filter_level) = privacy_filter {
            events.retain(|event| event.privacy_level == filter_level);
        }

        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        events.into_iter().take(limit).collect()
    }

    /// Process scheduled tasks (should be called periodically)
    pub async fn process_scheduled_tasks(&mut self) -> GovernanceResult<()> {
        // Process expired proposals
        let _expired_proposals = self.proposal_manager.process_expired_proposals().await?;

        // Process expired delegations
        let _expired_delegations = self.delegation_system.process_expired_delegations().await?;

        // Process reward calculations
        let _processed_rewards = self.token_system.process_reward_calculations().await?;

        // Process parameter timelocks
        let _ready_parameters = self.parameter_manager.process_expired_timelocks().await?;

        // Evaluate emergency triggers (with dummy metrics)
        let current_metrics = HashMap::new(); // Would be populated with real metrics
        let _triggered_actions = self.emergency_system.evaluate_triggers(current_metrics).await?;

        Ok(())
    }

    // Helper methods

    async fn initialize_default_parameters(&mut self) -> GovernanceResult<()> {
        // Register default governance parameters
        self.parameter_manager.register_parameter(
            "voting_period".to_string(),
            "Default voting period for proposals".to_string(),
            crate::parameters::ParameterType::UInteger,
            crate::parameters::ParameterValue::UInteger(7 * 24 * 60 * 60), // 7 days in seconds
            None,
            true,
            false,
            "governance".to_string(),
            vec!["voting".to_string(), "time".to_string()],
        ).await?;

        self.parameter_manager.register_parameter(
            "quorum_percentage".to_string(),
            "Minimum quorum percentage for proposals".to_string(),
            crate::parameters::ParameterType::UInteger,
            crate::parameters::ParameterValue::UInteger(10), // 10%
            None,
            true,
            false,
            "governance".to_string(),
            vec!["voting".to_string(), "quorum".to_string()],
        ).await?;

        Ok(())
    }

    async fn execute_parameter_update(
        &mut self,
        parameter: &str,
        new_value: &str,
        executor: &str,
    ) -> GovernanceResult<()> {
        // This would update the actual parameter
        // For now, just record the event
        self.record_event(
            GovernanceEventType::ParameterUpdated,
            executor.to_string(),
            None,
            HashMap::from([
                ("parameter".to_string(), parameter.to_string()),
                ("new_value".to_string(), new_value.to_string()),
            ]),
            PrivacyLevel::Public,
        ).await?;

        Ok(())
    }

    async fn execute_treasury_spend(
        &mut self,
        _recipient: &str,
        _amount: u64,
        executor: &str,
    ) -> GovernanceResult<()> {
        // This would execute the actual treasury spend
        // For now, just record the event
        self.record_event(
            GovernanceEventType::TreasuryAction,
            executor.to_string(),
            None,
            HashMap::from([("action".to_string(), "spend".to_string())]),
            PrivacyLevel::Public,
        ).await?;

        Ok(())
    }

    async fn record_event(
        &mut self,
        event_type: GovernanceEventType,
        participant: String,
        proposal_id: Option<ProposalId>,
        data: HashMap<String, String>,
        privacy_level: PrivacyLevel,
    ) -> GovernanceResult<()> {
        self.event_counter += 1;
        let event_id = format!("event_{}", self.event_counter);

        let event = GovernanceEvent {
            id: event_id,
            event_type,
            participant,
            proposal_id,
            timestamp: Utc::now(),
            data,
            privacy_level,
        };

        self.events.push(event);
        Ok(())
    }
}

/// Overall governance statistics
#[derive(Debug, Default, Clone)]
pub struct GovernanceStatistics {
    pub total_participants: usize,
    pub observers: usize,
    pub voters: usize,
    pub proposers: usize,
    pub reviewers: usize,
    pub governors: usize,
    pub total_proposals: usize,
    pub active_proposals: usize,
    pub executed_proposals: usize,
    pub total_voting_power: u64,
    pub staked_tokens: u64,
    pub treasury_value: u64,
    pub total_delegations: usize,
    pub emergency_actions: usize,
    pub total_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposals::ProposalType;

    #[tokio::test]
    async fn test_governance_system_initialization() {
        let mut system = GovernanceSystem::new();

        let config = GovernanceConfig::default();
        system.initialize(config).await.unwrap();

        assert!(system.config.initialized);
    }

    #[tokio::test]
    async fn test_participant_registration() {
        let mut system = GovernanceSystem::new();
        system.initialize(GovernanceConfig::default()).await.unwrap();

        system.register_participant(
            "participant1".to_string(),
            10000,
            ParticipationLevel::Voter,
            PrivacyLevel::Public,
        ).await.unwrap();

        let participant = system.get_participant("participant1").unwrap();
        assert_eq!(participant.voting_power, 10000);
        assert_eq!(participant.participation_level, ParticipationLevel::Voter);
    }

    #[tokio::test]
    async fn test_proposal_creation_and_voting() {
        let mut system = GovernanceSystem::new();
        system.initialize(GovernanceConfig::default()).await.unwrap();

        // Register participants
        system.register_participant(
            "proposer".to_string(),
            10000,
            ParticipationLevel::Proposer,
            PrivacyLevel::Public,
        ).await.unwrap();

        system.register_participant(
            "voter1".to_string(),
            5000,
            ParticipationLevel::Voter,
            PrivacyLevel::Public,
        ).await.unwrap();

        // Create proposal
        let proposal_id = system.create_proposal(
            ProposalType::ParameterUpdate {
                parameter: "voting_period".to_string(),
                old_value: "604800".to_string(), // 7 days
                new_value: "432000".to_string(),  // 5 days
            },
            "Reduce voting period".to_string(),
            "Proposal to reduce voting period from 7 to 5 days".to_string(),
            "proposer".to_string(),
            None,
            PrivacyLevel::Public,
        ).await.unwrap();

        // Vote on proposal
        system.vote(
            proposal_id,
            "voter1".to_string(),
            VoteType::For,
            Some("I support this change".to_string()),
            PrivacyLevel::Public,
        ).await.unwrap();

        // Check that vote was recorded
        let vote = system.voting_system.get_vote(&proposal_id, "voter1").unwrap();
        assert_eq!(vote.vote_type, VoteType::For);
    }

    #[tokio::test]
    async fn test_delegation() {
        let mut system = GovernanceSystem::new();
        system.initialize(GovernanceConfig::default()).await.unwrap();

        // Register participants
        system.register_participant(
            "delegator".to_string(),
            10000,
            ParticipationLevel::Voter,
            PrivacyLevel::Public,
        ).await.unwrap();

        system.register_participant(
            "delegate".to_string(),
            5000,
            ParticipationLevel::Voter,
            PrivacyLevel::Public,
        ).await.unwrap();

        // Create delegation
        system.delegate(
            "delegator".to_string(),
            "delegate".to_string(),
            3000,
            PrivacyLevel::Public,
        ).await.unwrap();

        // Check delegation was created
        let delegations = system.delegation_system.get_delegations_by_delegator("delegator");
        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].delegate, "delegate");
    }

    #[tokio::test]
    async fn test_governance_statistics() {
        let mut system = GovernanceSystem::new();
        system.initialize(GovernanceConfig::default()).await.unwrap();

        // Register some participants
        system.register_participant(
            "voter1".to_string(),
            10000,
            ParticipationLevel::Voter,
            PrivacyLevel::Public,
        ).await.unwrap();

        system.register_participant(
            "proposer1".to_string(),
            15000,
            ParticipationLevel::Proposer,
            PrivacyLevel::Public,
        ).await.unwrap();

        let stats = system.get_governance_statistics().await;

        assert_eq!(stats.total_participants, 2);
        assert_eq!(stats.voters, 1);
        assert_eq!(stats.proposers, 1);
        assert_eq!(stats.total_voting_power, 25000);
    }

    #[tokio::test]
    async fn test_governance_events() {
        let mut system = GovernanceSystem::new();
        system.initialize(GovernanceConfig::default()).await.unwrap();

        // Register participant
        system.register_participant(
            "participant1".to_string(),
            10000,
            ParticipationLevel::Voter,
            PrivacyLevel::Public,
        ).await.unwrap();

        // Get recent events
        let events = system.get_recent_events(10, Some(PrivacyLevel::Public));

        // Should have initialization and registration events
        assert!(events.len() >= 2);
    }
}