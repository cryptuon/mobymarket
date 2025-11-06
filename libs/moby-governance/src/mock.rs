//! Mock implementations for testing and development

use crate::{
    delegation::{DelegationPower, DelegationScope},
    emergency::{EmergencyAction, EmergencyRole},
    error::GovernanceResult,
    parameters::{ParameterType, ParameterValue},
    proposals::{ProposalType, ProposalPriority},
    system::{GovernanceSystem, ParticipationLevel, PrivacyLevel},
    tokens::GovernanceToken,
    treasury::TreasuryAction,
    upgrades::{UpgradeType, UpgradePriority},
    voting::VoteType,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Mock governance system for testing
pub struct MockGovernanceSystem {
    pub system: GovernanceSystem,
    pub mock_data: MockData,
}

/// Mock data for testing scenarios
#[derive(Debug, Default)]
pub struct MockData {
    pub participants: Vec<MockParticipant>,
    pub proposals: Vec<MockProposal>,
    pub votes: Vec<MockVote>,
    pub delegations: Vec<MockDelegation>,
    pub treasury_actions: Vec<MockTreasuryAction>,
    pub emergency_actions: Vec<MockEmergencyAction>,
}

#[derive(Debug, Clone)]
pub struct MockParticipant {
    pub address: String,
    pub name: String,
    pub token_balance: u64,
    pub participation_level: ParticipationLevel,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone)]
pub struct MockProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub proposer: String,
    pub priority: ProposalPriority,
}

#[derive(Debug, Clone)]
pub struct MockVote {
    pub voter: String,
    pub proposal_id: String,
    pub vote_type: VoteType,
    pub justification: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MockDelegation {
    pub delegator: String,
    pub delegate: String,
    pub amount: u64,
    pub scope: DelegationScope,
}

#[derive(Debug, Clone)]
pub struct MockTreasuryAction {
    pub action: TreasuryAction,
    pub proposer: String,
    pub justification: String,
}

#[derive(Debug, Clone)]
pub struct MockEmergencyAction {
    pub action: EmergencyAction,
    pub initiator: String,
    pub role: EmergencyRole,
    pub justification: String,
}

impl MockGovernanceSystem {
    /// Create a new mock governance system
    pub async fn new() -> GovernanceResult<Self> {
        let mut system = GovernanceSystem::new();

        // Initialize with default configuration
        system.initialize(crate::system::GovernanceConfig::default()).await?;

        let mut mock_system = Self {
            system,
            mock_data: MockData::default(),
        };

        // Set up default mock data
        mock_system.setup_default_data().await?;

        Ok(mock_system)
    }

    /// Set up default mock data for testing
    async fn setup_default_data(&mut self) -> GovernanceResult<()> {
        // Create mock participants
        self.add_mock_participant(MockParticipant {
            address: "whale_trader_1".to_string(),
            name: "Alpha Whale".to_string(),
            token_balance: 1_000_000,
            participation_level: ParticipationLevel::Governor,
            privacy_level: PrivacyLevel::Public,
        }).await?;

        self.add_mock_participant(MockParticipant {
            address: "institution_1".to_string(),
            name: "Institutional Investor".to_string(),
            token_balance: 5_000_000,
            participation_level: ParticipationLevel::Proposer,
            privacy_level: PrivacyLevel::Confidential,
        }).await?;

        self.add_mock_participant(MockParticipant {
            address: "community_member_1".to_string(),
            name: "Community Representative".to_string(),
            token_balance: 50_000,
            participation_level: ParticipationLevel::Voter,
            privacy_level: PrivacyLevel::Public,
        }).await?;

        self.add_mock_participant(MockParticipant {
            address: "dao_delegate_1".to_string(),
            name: "DAO Delegate".to_string(),
            token_balance: 100_000,
            participation_level: ParticipationLevel::Reviewer,
            privacy_level: PrivacyLevel::Public,
        }).await?;

        self.add_mock_participant(MockParticipant {
            address: "anonymous_trader".to_string(),
            name: "Anonymous Trader".to_string(),
            token_balance: 2_000_000,
            participation_level: ParticipationLevel::Proposer,
            privacy_level: PrivacyLevel::Anonymous,
        }).await?;

        // Add emergency personnel
        self.system.emergency_system.add_emergency_personnel(
            "emergency_coordinator".to_string(),
            EmergencyRole::EmergencyCoordinator,
        ).await?;

        self.system.emergency_system.add_emergency_personnel(
            "security_officer".to_string(),
            EmergencyRole::SecurityOfficer,
        ).await?;

        Ok(())
    }

    /// Add a mock participant
    pub async fn add_mock_participant(&mut self, participant: MockParticipant) -> GovernanceResult<()> {
        self.system.register_participant(
            participant.address.clone(),
            participant.token_balance,
            participant.participation_level.clone(),
            participant.privacy_level.clone(),
        ).await?;

        self.mock_data.participants.push(participant);
        Ok(())
    }

    /// Create a mock governance proposal
    pub async fn create_mock_proposal(&mut self, proposal: MockProposal) -> GovernanceResult<Uuid> {
        let proposal_id = self.system.create_proposal(
            proposal.proposal_type.clone(),
            proposal.title.clone(),
            proposal.description.clone(),
            proposal.proposer.clone(),
            None,
            PrivacyLevel::Public,
        ).await?;

        let mut mock_proposal = proposal;
        mock_proposal.id = proposal_id.to_string();
        self.mock_data.proposals.push(mock_proposal);

        Ok(proposal_id)
    }

    /// Cast a mock vote
    pub async fn cast_mock_vote(&mut self, vote: MockVote) -> GovernanceResult<()> {
        let proposal_id = Uuid::parse_str(&vote.proposal_id)
            .map_err(|_| crate::error::GovernanceError::InvalidHash {
                hash: vote.proposal_id.clone(),
            })?;

        self.system.vote(
            proposal_id,
            vote.voter.clone(),
            vote.vote_type.clone(),
            vote.justification.clone(),
            PrivacyLevel::Public,
        ).await?;

        self.mock_data.votes.push(vote);
        Ok(())
    }

    /// Create a mock delegation
    pub async fn create_mock_delegation(&mut self, delegation: MockDelegation) -> GovernanceResult<()> {
        self.system.delegate(
            delegation.delegator.clone(),
            delegation.delegate.clone(),
            delegation.amount,
            PrivacyLevel::Public,
        ).await?;

        self.mock_data.delegations.push(delegation);
        Ok(())
    }

    /// Get mock governance scenarios for testing
    pub fn get_test_scenarios() -> Vec<MockScenario> {
        vec![
            MockScenario::basic_voting(),
            MockScenario::whale_trading_governance(),
            MockScenario::emergency_response(),
            MockScenario::treasury_management(),
            MockScenario::delegation_cascade(),
            MockScenario::parameter_upgrade(),
        ]
    }

    /// Run a specific test scenario
    pub async fn run_scenario(&mut self, scenario: MockScenario) -> GovernanceResult<MockScenarioResult> {
        let start_time = chrono::Utc::now();
        let mut events = Vec::new();

        for step in scenario.steps {
            let step_result = self.execute_scenario_step(step).await?;
            events.push(step_result);
        }

        let end_time = chrono::Utc::now();
        let duration = end_time - start_time;

        Ok(MockScenarioResult {
            scenario_name: scenario.name,
            duration,
            events,
            final_statistics: self.system.get_governance_statistics().await,
        })
    }

    async fn execute_scenario_step(&mut self, step: MockScenarioStep) -> GovernanceResult<MockStepResult> {
        let step_start = chrono::Utc::now();

        let result = match step.action {
            MockAction::CreateProposal { proposal } => {
                let proposal_id = self.create_mock_proposal(proposal).await?;
                format!("Created proposal: {}", proposal_id)
            }
            MockAction::Vote { vote } => {
                self.cast_mock_vote(vote).await?;
                "Vote cast successfully".to_string()
            }
            MockAction::Delegate { delegation } => {
                self.create_mock_delegation(delegation).await?;
                "Delegation created successfully".to_string()
            }
            MockAction::EmergencyAction { action, initiator } => {
                self.system.emergency_system.initiate_emergency_action(
                    action,
                    initiator,
                    "Mock emergency action".to_string(),
                    "Low risk mock scenario".to_string(),
                    "Minimal impact for testing".to_string(),
                    vec!["Mock evidence".to_string()],
                ).await?;
                "Emergency action initiated".to_string()
            }
            MockAction::ProcessScheduledTasks => {
                self.system.process_scheduled_tasks().await?;
                "Scheduled tasks processed".to_string()
            }
        };

        let step_end = chrono::Utc::now();

        Ok(MockStepResult {
            step_name: step.name,
            duration: step_end - step_start,
            result,
            success: true,
        })
    }
}

/// Mock scenario for testing different governance situations
#[derive(Debug, Clone)]
pub struct MockScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<MockScenarioStep>,
}

#[derive(Debug, Clone)]
pub struct MockScenarioStep {
    pub name: String,
    pub action: MockAction,
    pub expected_outcome: String,
}

#[derive(Debug, Clone)]
pub enum MockAction {
    CreateProposal { proposal: MockProposal },
    Vote { vote: MockVote },
    Delegate { delegation: MockDelegation },
    EmergencyAction { action: EmergencyAction, initiator: String },
    ProcessScheduledTasks,
}

#[derive(Debug)]
pub struct MockScenarioResult {
    pub scenario_name: String,
    pub duration: chrono::Duration,
    pub events: Vec<MockStepResult>,
    pub final_statistics: crate::system::GovernanceStatistics,
}

#[derive(Debug)]
pub struct MockStepResult {
    pub step_name: String,
    pub duration: chrono::Duration,
    pub result: String,
    pub success: bool,
}

impl MockScenario {
    /// Basic voting scenario
    pub fn basic_voting() -> Self {
        Self {
            name: "Basic Voting".to_string(),
            description: "Simple proposal creation and voting".to_string(),
            steps: vec![
                MockScenarioStep {
                    name: "Create Trading Fee Proposal".to_string(),
                    action: MockAction::CreateProposal {
                        proposal: MockProposal {
                            id: String::new(), // Will be set during execution
                            title: "Reduce Trading Fees".to_string(),
                            description: "Proposal to reduce trading fees from 0.3% to 0.25%".to_string(),
                            proposal_type: ProposalType::ParameterUpdate {
                                parameter: "trading_fee".to_string(),
                                old_value: "0.003".to_string(),
                                new_value: "0.0025".to_string(),
                            },
                            proposer: "institution_1".to_string(),
                            priority: ProposalPriority::Medium,
                        },
                    },
                    expected_outcome: "Proposal created successfully".to_string(),
                },
            ],
        }
    }

    /// Whale trading governance scenario
    pub fn whale_trading_governance() -> Self {
        Self {
            name: "Whale Trading Governance".to_string(),
            description: "Large whale proposes changes to trading parameters".to_string(),
            steps: vec![
                MockScenarioStep {
                    name: "Whale Proposes Slippage Changes".to_string(),
                    action: MockAction::CreateProposal {
                        proposal: MockProposal {
                            id: String::new(),
                            title: "Optimize Slippage for Large Orders".to_string(),
                            description: "Adjust slippage parameters to better serve whale trades".to_string(),
                            proposal_type: ProposalType::ParameterUpdate {
                                parameter: "max_slippage".to_string(),
                                old_value: "0.05".to_string(),
                                new_value: "0.02".to_string(),
                            },
                            proposer: "whale_trader_1".to_string(),
                            priority: ProposalPriority::High,
                        },
                    },
                    expected_outcome: "Whale proposal created".to_string(),
                },
                MockScenarioStep {
                    name: "Community Delegates to Whale".to_string(),
                    action: MockAction::Delegate {
                        delegation: MockDelegation {
                            delegator: "community_member_1".to_string(),
                            delegate: "whale_trader_1".to_string(),
                            amount: 25_000,
                            scope: DelegationScope::ProposalTypes(vec!["parameter_update".to_string()]),
                        },
                    },
                    expected_outcome: "Delegation successful".to_string(),
                },
            ],
        }
    }

    /// Emergency response scenario
    pub fn emergency_response() -> Self {
        Self {
            name: "Emergency Response".to_string(),
            description: "Emergency system pause due to security issue".to_string(),
            steps: vec![
                MockScenarioStep {
                    name: "Initiate Emergency Pause".to_string(),
                    action: MockAction::EmergencyAction {
                        action: EmergencyAction::SystemPause {
                            duration: Some(chrono::Duration::hours(2)),
                            reason: "Suspicious activity detected".to_string(),
                        },
                        initiator: "emergency_coordinator".to_string(),
                    },
                    expected_outcome: "System paused successfully".to_string(),
                },
            ],
        }
    }

    /// Treasury management scenario
    pub fn treasury_management() -> Self {
        Self {
            name: "Treasury Management".to_string(),
            description: "Proposal for treasury fund allocation".to_string(),
            steps: vec![
                MockScenarioStep {
                    name: "Propose Development Grant".to_string(),
                    action: MockAction::CreateProposal {
                        proposal: MockProposal {
                            id: String::new(),
                            title: "Development Grant for Privacy Features".to_string(),
                            description: "Allocate 500,000 MOBY for privacy feature development".to_string(),
                            proposal_type: ProposalType::TreasurySpend {
                                recipient: "dev_team".to_string(),
                                amount: 500_000,
                                purpose: "Privacy feature development".to_string(),
                            },
                            proposer: "whale_trader_1".to_string(),
                            priority: ProposalPriority::Medium,
                        },
                    },
                    expected_outcome: "Treasury proposal created".to_string(),
                },
            ],
        }
    }

    /// Delegation cascade scenario
    pub fn delegation_cascade() -> Self {
        Self {
            name: "Delegation Cascade".to_string(),
            description: "Multiple levels of delegation forming a cascade".to_string(),
            steps: vec![
                MockScenarioStep {
                    name: "First Level Delegation".to_string(),
                    action: MockAction::Delegate {
                        delegation: MockDelegation {
                            delegator: "community_member_1".to_string(),
                            delegate: "dao_delegate_1".to_string(),
                            amount: 30_000,
                            scope: DelegationScope::All,
                        },
                    },
                    expected_outcome: "First delegation created".to_string(),
                },
                MockScenarioStep {
                    name: "Second Level Delegation".to_string(),
                    action: MockAction::Delegate {
                        delegation: MockDelegation {
                            delegator: "dao_delegate_1".to_string(),
                            delegate: "whale_trader_1".to_string(),
                            amount: 50_000,
                            scope: DelegationScope::All,
                        },
                    },
                    expected_outcome: "Second delegation created".to_string(),
                },
            ],
        }
    }

    /// Parameter upgrade scenario
    pub fn parameter_upgrade() -> Self {
        Self {
            name: "Parameter Upgrade".to_string(),
            description: "Systematic upgrade of multiple parameters".to_string(),
            steps: vec![
                MockScenarioStep {
                    name: "Propose Fee Structure Update".to_string(),
                    action: MockAction::CreateProposal {
                        proposal: MockProposal {
                            id: String::new(),
                            title: "Comprehensive Fee Structure Overhaul".to_string(),
                            description: "Update all fee parameters for better competitiveness".to_string(),
                            proposal_type: ProposalType::ParameterUpdate {
                                parameter: "fee_structure".to_string(),
                                old_value: "current".to_string(),
                                new_value: "optimized".to_string(),
                            },
                            proposer: "institution_1".to_string(),
                            priority: ProposalPriority::High,
                        },
                    },
                    expected_outcome: "Fee structure proposal created".to_string(),
                },
                MockScenarioStep {
                    name: "Process Scheduled Tasks".to_string(),
                    action: MockAction::ProcessScheduledTasks,
                    expected_outcome: "Tasks processed successfully".to_string(),
                },
            ],
        }
    }
}

/// Mock data generators for testing
pub struct MockDataGenerator;

impl MockDataGenerator {
    /// Generate a realistic whale trading proposal
    pub fn whale_trading_proposal(proposer: &str) -> MockProposal {
        MockProposal {
            id: String::new(),
            title: "Whale Trading Optimization Proposal".to_string(),
            description: "Optimize trading parameters specifically for large volume whale trades including reduced slippage tolerance, extended settlement times, and priority execution queues.".to_string(),
            proposal_type: ProposalType::ParameterUpdate {
                parameter: "whale_trading_config".to_string(),
                old_value: "standard".to_string(),
                new_value: "whale_optimized".to_string(),
            },
            proposer: proposer.to_string(),
            priority: ProposalPriority::High,
        }
    }

    /// Generate a privacy enhancement proposal
    pub fn privacy_enhancement_proposal(proposer: &str) -> MockProposal {
        MockProposal {
            id: String::new(),
            title: "Enhanced Privacy Features Implementation".to_string(),
            description: "Implement advanced zero-knowledge proofs for enhanced transaction privacy including stealth addresses, ring signatures, and confidential amounts.".to_string(),
            proposal_type: ProposalType::ProtocolUpgrade {
                version: "2.0.0".to_string(),
                description: "Privacy upgrade".to_string(),
                code_hash: "0x123456789abcdef".to_string(),
            },
            proposer: proposer.to_string(),
            priority: ProposalPriority::Critical,
        }
    }

    /// Generate emergency fund recovery action
    pub fn emergency_fund_recovery() -> EmergencyAction {
        EmergencyAction::FundRecovery {
            amount: 10_000_000,
            token: "USDC".to_string(),
            destination: "emergency_recovery_address".to_string(),
            justification: "Critical security vulnerability discovered requiring immediate fund protection".to_string(),
        }
    }

    /// Generate treasury diversification action
    pub fn treasury_diversification_action() -> TreasuryAction {
        TreasuryAction::Invest {
            protocol: "yield_farming_protocol".to_string(),
            amount: 5_000_000,
            token: "USDC".to_string(),
            expected_yield: 0.08, // 8% APY
        }
    }

    /// Generate upgrade proposal for whale trading features
    pub fn whale_trading_upgrade() -> UpgradeType {
        UpgradeType::Major {
            version: "2.1.0".to_string(),
            breaking_changes: vec![
                "New whale detection algorithm".to_string(),
                "Enhanced privacy for large orders".to_string(),
            ],
            new_features: vec![
                "Dynamic fee scaling based on order size".to_string(),
                "Priority execution queues for verified whales".to_string(),
                "Advanced slippage protection".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_governance_system_creation() {
        let mock_system = MockGovernanceSystem::new().await.unwrap();

        assert_eq!(mock_system.mock_data.participants.len(), 5);

        // Check that participants were registered in the system
        assert!(mock_system.system.get_participant("whale_trader_1").is_some());
        assert!(mock_system.system.get_participant("institution_1").is_some());
    }

    #[tokio::test]
    async fn test_basic_voting_scenario() {
        let mut mock_system = MockGovernanceSystem::new().await.unwrap();

        let scenario = MockScenario::basic_voting();
        let result = mock_system.run_scenario(scenario).await.unwrap();

        assert_eq!(result.scenario_name, "Basic Voting");
        assert_eq!(result.events.len(), 1);
        assert!(result.events[0].success);
    }

    #[tokio::test]
    async fn test_whale_trading_scenario() {
        let mut mock_system = MockGovernanceSystem::new().await.unwrap();

        let scenario = MockScenario::whale_trading_governance();
        let result = mock_system.run_scenario(scenario).await.unwrap();

        assert_eq!(result.scenario_name, "Whale Trading Governance");
        assert_eq!(result.events.len(), 2);

        // Check that delegation was successful
        let delegations = mock_system.system.delegation_system.get_delegations_by_delegator("community_member_1");
        assert_eq!(delegations.len(), 1);
    }

    #[tokio::test]
    async fn test_emergency_response_scenario() {
        let mut mock_system = MockGovernanceSystem::new().await.unwrap();

        let scenario = MockScenario::emergency_response();
        let result = mock_system.run_scenario(scenario).await.unwrap();

        assert_eq!(result.scenario_name, "Emergency Response");
        assert!(result.events[0].success);
    }

    #[tokio::test]
    async fn test_mock_data_generators() {
        let whale_proposal = MockDataGenerator::whale_trading_proposal("whale_trader_1");
        assert_eq!(whale_proposal.proposer, "whale_trader_1");
        assert_eq!(whale_proposal.priority, ProposalPriority::High);

        let privacy_proposal = MockDataGenerator::privacy_enhancement_proposal("institution_1");
        assert_eq!(privacy_proposal.priority, ProposalPriority::Critical);

        let emergency_action = MockDataGenerator::emergency_fund_recovery();
        assert!(matches!(emergency_action, EmergencyAction::FundRecovery { .. }));
    }

    #[tokio::test]
    async fn test_all_scenarios() {
        let scenarios = MockGovernanceSystem::get_test_scenarios();
        assert_eq!(scenarios.len(), 6);

        for scenario in scenarios {
            let mut mock_system = MockGovernanceSystem::new().await.unwrap();
            let result = mock_system.run_scenario(scenario).await;

            // All scenarios should execute without errors
            assert!(result.is_ok());
        }
    }
}