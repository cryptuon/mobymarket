//! Proposal management system for governance

use crate::error::{GovernanceError, GovernanceResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for proposals
pub type ProposalId = Uuid;

/// Different types of governance proposals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    /// Update a protocol parameter
    ParameterUpdate {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    /// Treasury spending proposal
    TreasurySpend {
        recipient: String,
        amount: u64,
        purpose: String,
    },
    /// Protocol upgrade proposal
    ProtocolUpgrade {
        version: String,
        description: String,
        code_hash: String,
    },
    /// Emergency action proposal
    EmergencyAction {
        action: String,
        justification: String,
    },
    /// General governance proposal
    General {
        title: String,
        description: String,
        actions: Vec<String>,
    },
    /// Constitutional amendment
    Constitutional {
        amendment: String,
        section: String,
    },
}

/// Current status of a proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is in draft state
    Draft,
    /// Proposal is under review
    UnderReview,
    /// Active voting period
    Voting,
    /// Voting has ended, awaiting execution
    Passed,
    /// Proposal was rejected
    Rejected,
    /// Proposal was executed
    Executed,
    /// Proposal was cancelled
    Cancelled,
    /// Proposal expired without execution
    Expired,
}

/// Priority level for proposals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// A governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique identifier
    pub id: ProposalId,
    /// Type of proposal
    pub proposal_type: ProposalType,
    /// Proposal title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Proposal author
    pub author: String,
    /// Current status
    pub status: ProposalStatus,
    /// Priority level
    pub priority: ProposalPriority,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Voting start time
    pub voting_start: DateTime<Utc>,
    /// Voting end time
    pub voting_end: DateTime<Utc>,
    /// Execution deadline
    pub execution_deadline: Option<DateTime<Utc>>,
    /// Required voting power for passage
    pub quorum_required: u64,
    /// Current voting results
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    /// Total voting power participating
    pub total_votes: u64,
    /// Metadata for additional information
    pub metadata: HashMap<String, String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Dependencies on other proposals
    pub dependencies: Vec<ProposalId>,
    /// Whether the proposal is executable
    pub executable: bool,
}

/// Configuration for proposal creation
#[derive(Debug, Clone)]
pub struct ProposalConfig {
    /// Minimum voting period duration
    pub min_voting_period: chrono::Duration,
    /// Maximum voting period duration
    pub max_voting_period: chrono::Duration,
    /// Default voting period
    pub default_voting_period: chrono::Duration,
    /// Minimum quorum percentage
    pub min_quorum_percentage: u8,
    /// Maximum number of active proposals per author
    pub max_proposals_per_author: usize,
    /// Whether proposals require review before voting
    pub require_review: bool,
    /// Execution timeout after passage
    pub execution_timeout: chrono::Duration,
}

impl Default for ProposalConfig {
    fn default() -> Self {
        Self {
            min_voting_period: chrono::Duration::days(1),
            max_voting_period: chrono::Duration::days(30),
            default_voting_period: chrono::Duration::days(7),
            min_quorum_percentage: 5,
            max_proposals_per_author: 3,
            require_review: true,
            execution_timeout: chrono::Duration::days(7),
        }
    }
}

/// Proposal manager handles creation, validation, and lifecycle
pub struct ProposalManager {
    /// Configuration
    config: ProposalConfig,
    /// Active proposals
    proposals: HashMap<ProposalId, Proposal>,
    /// Proposals by author
    author_proposals: HashMap<String, Vec<ProposalId>>,
    /// Proposals by status
    status_index: HashMap<ProposalStatus, Vec<ProposalId>>,
    /// Proposals by type
    type_index: HashMap<String, Vec<ProposalId>>,
}

impl ProposalManager {
    /// Create a new proposal manager
    pub fn new(config: ProposalConfig) -> Self {
        Self {
            config,
            proposals: HashMap::new(),
            author_proposals: HashMap::new(),
            status_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// Create a new proposal
    pub async fn create_proposal(
        &mut self,
        proposal_type: ProposalType,
        title: String,
        description: String,
        author: String,
        voting_period: Option<chrono::Duration>,
        priority: ProposalPriority,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> GovernanceResult<ProposalId> {
        // Validate author proposal limit
        let author_count = self.author_proposals
            .get(&author)
            .map(|proposals| proposals.len())
            .unwrap_or(0);

        if author_count >= self.config.max_proposals_per_author {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Author {} has reached maximum proposal limit", author),
            });
        }

        // Validate voting period
        let voting_duration = voting_period.unwrap_or(self.config.default_voting_period);
        if voting_duration < self.config.min_voting_period {
            return Err(GovernanceError::InvalidRange {
                min: format!("{} days", self.config.min_voting_period.num_days()),
                max: format!("{} days", self.config.max_voting_period.num_days()),
            });
        }

        let now = Utc::now();
        let id = Uuid::new_v4();

        // Determine voting start time based on review requirement
        let voting_start = if self.config.require_review {
            now + chrono::Duration::days(1) // 1 day review period
        } else {
            now
        };

        let voting_end = voting_start + voting_duration;
        let execution_deadline = Some(voting_end + self.config.execution_timeout);

        let proposal = Proposal {
            id,
            proposal_type: proposal_type.clone(),
            title,
            description,
            author: author.clone(),
            status: if self.config.require_review {
                ProposalStatus::UnderReview
            } else {
                ProposalStatus::Voting
            },
            priority,
            created_at: now,
            voting_start,
            voting_end,
            execution_deadline,
            quorum_required: 0, // Will be set based on total voting power
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            total_votes: 0,
            metadata,
            tags,
            dependencies: Vec::new(),
            executable: self.is_proposal_executable(&proposal_type),
        };

        // Store proposal
        self.proposals.insert(id, proposal);

        // Update indices
        self.author_proposals
            .entry(author)
            .or_insert_with(Vec::new)
            .push(id);

        self.status_index
            .entry(proposal.status.clone())
            .or_insert_with(Vec::new)
            .push(id);

        let type_key = self.proposal_type_key(&proposal_type);
        self.type_index
            .entry(type_key)
            .or_insert_with(Vec::new)
            .push(id);

        Ok(id)
    }

    /// Get a proposal by ID
    pub fn get_proposal(&self, id: &ProposalId) -> Option<&Proposal> {
        self.proposals.get(id)
    }

    /// Get a mutable proposal by ID
    pub fn get_proposal_mut(&mut self, id: &ProposalId) -> Option<&mut Proposal> {
        self.proposals.get_mut(id)
    }

    /// Update proposal status
    pub async fn update_status(
        &mut self,
        id: &ProposalId,
        new_status: ProposalStatus,
    ) -> GovernanceResult<()> {
        let proposal = self.proposals.get_mut(id)
            .ok_or_else(|| GovernanceError::ProposalNotFound {
                id: id.to_string(),
            })?;

        let old_status = proposal.status.clone();

        // Validate status transition
        self.validate_status_transition(&old_status, &new_status)?;

        // Update proposal status
        proposal.status = new_status.clone();

        // Update status index
        if let Some(old_list) = self.status_index.get_mut(&old_status) {
            old_list.retain(|&x| x != *id);
        }

        self.status_index
            .entry(new_status)
            .or_insert_with(Vec::new)
            .push(*id);

        Ok(())
    }

    /// Cancel a proposal
    pub async fn cancel_proposal(
        &mut self,
        id: &ProposalId,
        author: &str,
    ) -> GovernanceResult<()> {
        let proposal = self.proposals.get(id)
            .ok_or_else(|| GovernanceError::ProposalNotFound {
                id: id.to_string(),
            })?;

        // Verify author can cancel
        if proposal.author != author {
            return Err(GovernanceError::UnauthorizedAccess {
                action: "cancel proposal".to_string(),
            });
        }

        // Can only cancel if not yet executed
        if matches!(proposal.status, ProposalStatus::Executed) {
            return Err(GovernanceError::InvalidProposalState {
                expected: "non-executed".to_string(),
                actual: "executed".to_string(),
            });
        }

        self.update_status(id, ProposalStatus::Cancelled).await
    }

    /// Get proposals by status
    pub fn get_proposals_by_status(&self, status: &ProposalStatus) -> Vec<&Proposal> {
        self.status_index
            .get(status)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|id| self.proposals.get(id))
            .collect()
    }

    /// Get proposals by author
    pub fn get_proposals_by_author(&self, author: &str) -> Vec<&Proposal> {
        self.author_proposals
            .get(author)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|id| self.proposals.get(id))
            .collect()
    }

    /// Get proposals by type
    pub fn get_proposals_by_type(&self, proposal_type: &ProposalType) -> Vec<&Proposal> {
        let type_key = self.proposal_type_key(proposal_type);
        self.type_index
            .get(&type_key)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|id| self.proposals.get(id))
            .collect()
    }

    /// Check if voting period is active
    pub fn is_voting_active(&self, proposal: &Proposal) -> bool {
        let now = Utc::now();
        proposal.status == ProposalStatus::Voting &&
            now >= proposal.voting_start &&
            now <= proposal.voting_end
    }

    /// Check if proposal has passed
    pub fn has_proposal_passed(&self, proposal: &Proposal) -> bool {
        if proposal.total_votes < proposal.quorum_required {
            return false;
        }

        proposal.votes_for > proposal.votes_against
    }

    /// Set quorum requirement based on total voting power
    pub async fn set_quorum_requirement(
        &mut self,
        id: &ProposalId,
        total_voting_power: u64,
    ) -> GovernanceResult<()> {
        let proposal = self.proposals.get_mut(id)
            .ok_or_else(|| GovernanceError::ProposalNotFound {
                id: id.to_string(),
            })?;

        let quorum_percentage = self.config.min_quorum_percentage as u64;
        proposal.quorum_required = (total_voting_power * quorum_percentage) / 100;

        Ok(())
    }

    /// Update vote counts
    pub async fn update_vote_counts(
        &mut self,
        id: &ProposalId,
        votes_for: u64,
        votes_against: u64,
        votes_abstain: u64,
    ) -> GovernanceResult<()> {
        let proposal = self.proposals.get_mut(id)
            .ok_or_else(|| GovernanceError::ProposalNotFound {
                id: id.to_string(),
            })?;

        proposal.votes_for = votes_for;
        proposal.votes_against = votes_against;
        proposal.votes_abstain = votes_abstain;
        proposal.total_votes = votes_for + votes_against + votes_abstain;

        Ok(())
    }

    /// Add dependency between proposals
    pub async fn add_dependency(
        &mut self,
        id: &ProposalId,
        dependency: ProposalId,
    ) -> GovernanceResult<()> {
        let proposal = self.proposals.get_mut(id)
            .ok_or_else(|| GovernanceError::ProposalNotFound {
                id: id.to_string(),
            })?;

        if !proposal.dependencies.contains(&dependency) {
            proposal.dependencies.push(dependency);
        }

        Ok(())
    }

    /// Check if all dependencies are met
    pub fn are_dependencies_met(&self, proposal: &Proposal) -> bool {
        proposal.dependencies.iter().all(|dep_id| {
            self.proposals.get(dep_id)
                .map(|dep| dep.status == ProposalStatus::Executed)
                .unwrap_or(false)
        })
    }

    /// Get proposal statistics
    pub fn get_statistics(&self) -> ProposalStatistics {
        let mut stats = ProposalStatistics::default();

        for proposal in self.proposals.values() {
            stats.total_proposals += 1;

            match proposal.status {
                ProposalStatus::Draft => stats.draft_proposals += 1,
                ProposalStatus::UnderReview => stats.under_review += 1,
                ProposalStatus::Voting => stats.active_voting += 1,
                ProposalStatus::Passed => stats.passed_proposals += 1,
                ProposalStatus::Rejected => stats.rejected_proposals += 1,
                ProposalStatus::Executed => stats.executed_proposals += 1,
                ProposalStatus::Cancelled => stats.cancelled_proposals += 1,
                ProposalStatus::Expired => stats.expired_proposals += 1,
            }

            match proposal.priority {
                ProposalPriority::Low => stats.low_priority += 1,
                ProposalPriority::Medium => stats.medium_priority += 1,
                ProposalPriority::High => stats.high_priority += 1,
                ProposalPriority::Critical => stats.critical_priority += 1,
            }
        }

        stats
    }

    /// Process expired proposals
    pub async fn process_expired_proposals(&mut self) -> GovernanceResult<Vec<ProposalId>> {
        let now = Utc::now();
        let mut expired_ids = Vec::new();

        let voting_proposals: Vec<ProposalId> = self.status_index
            .get(&ProposalStatus::Voting)
            .unwrap_or(&Vec::new())
            .clone();

        for id in voting_proposals {
            if let Some(proposal) = self.proposals.get(&id) {
                if now > proposal.voting_end {
                    if self.has_proposal_passed(proposal) {
                        self.update_status(&id, ProposalStatus::Passed).await?;
                    } else {
                        self.update_status(&id, ProposalStatus::Rejected).await?;
                    }
                }
            }
        }

        // Check for execution deadline expiry
        let passed_proposals: Vec<ProposalId> = self.status_index
            .get(&ProposalStatus::Passed)
            .unwrap_or(&Vec::new())
            .clone();

        for id in passed_proposals {
            if let Some(proposal) = self.proposals.get(&id) {
                if let Some(deadline) = proposal.execution_deadline {
                    if now > deadline {
                        self.update_status(&id, ProposalStatus::Expired).await?;
                        expired_ids.push(id);
                    }
                }
            }
        }

        Ok(expired_ids)
    }

    // Helper methods

    fn validate_status_transition(
        &self,
        from: &ProposalStatus,
        to: &ProposalStatus,
    ) -> GovernanceResult<()> {
        let valid = match (from, to) {
            (ProposalStatus::Draft, ProposalStatus::UnderReview) => true,
            (ProposalStatus::UnderReview, ProposalStatus::Voting) => true,
            (ProposalStatus::UnderReview, ProposalStatus::Rejected) => true,
            (ProposalStatus::Voting, ProposalStatus::Passed) => true,
            (ProposalStatus::Voting, ProposalStatus::Rejected) => true,
            (ProposalStatus::Passed, ProposalStatus::Executed) => true,
            (ProposalStatus::Passed, ProposalStatus::Expired) => true,
            (_, ProposalStatus::Cancelled) => true,
            _ => false,
        };

        if !valid {
            return Err(GovernanceError::InvalidProposalState {
                expected: format!("valid transition from {:?}", from),
                actual: format!("{:?}", to),
            });
        }

        Ok(())
    }

    fn proposal_type_key(&self, proposal_type: &ProposalType) -> String {
        match proposal_type {
            ProposalType::ParameterUpdate { .. } => "parameter_update".to_string(),
            ProposalType::TreasurySpend { .. } => "treasury_spend".to_string(),
            ProposalType::ProtocolUpgrade { .. } => "protocol_upgrade".to_string(),
            ProposalType::EmergencyAction { .. } => "emergency_action".to_string(),
            ProposalType::General { .. } => "general".to_string(),
            ProposalType::Constitutional { .. } => "constitutional".to_string(),
        }
    }

    fn is_proposal_executable(&self, proposal_type: &ProposalType) -> bool {
        matches!(proposal_type,
            ProposalType::ParameterUpdate { .. } |
            ProposalType::TreasurySpend { .. } |
            ProposalType::ProtocolUpgrade { .. } |
            ProposalType::EmergencyAction { .. }
        )
    }
}

/// Statistics for proposal management
#[derive(Debug, Default, Clone)]
pub struct ProposalStatistics {
    pub total_proposals: usize,
    pub draft_proposals: usize,
    pub under_review: usize,
    pub active_voting: usize,
    pub passed_proposals: usize,
    pub rejected_proposals: usize,
    pub executed_proposals: usize,
    pub cancelled_proposals: usize,
    pub expired_proposals: usize,
    pub low_priority: usize,
    pub medium_priority: usize,
    pub high_priority: usize,
    pub critical_priority: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_proposal() {
        let mut manager = ProposalManager::new(ProposalConfig::default());

        let id = manager.create_proposal(
            ProposalType::ParameterUpdate {
                parameter: "trading_fee".to_string(),
                old_value: "0.003".to_string(),
                new_value: "0.002".to_string(),
            },
            "Reduce trading fees".to_string(),
            "Lower trading fees to increase volume".to_string(),
            "author1".to_string(),
            None,
            ProposalPriority::Medium,
            vec!["trading".to_string(), "fees".to_string()],
            HashMap::new(),
        ).await.unwrap();

        let proposal = manager.get_proposal(&id).unwrap();
        assert_eq!(proposal.title, "Reduce trading fees");
        assert_eq!(proposal.author, "author1");
        assert_eq!(proposal.status, ProposalStatus::UnderReview);
    }

    #[tokio::test]
    async fn test_update_status() {
        let mut manager = ProposalManager::new(ProposalConfig::default());

        let id = manager.create_proposal(
            ProposalType::General {
                title: "Test".to_string(),
                description: "Test proposal".to_string(),
                actions: vec!["action1".to_string()],
            },
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            "author1".to_string(),
            None,
            ProposalPriority::Low,
            vec![],
            HashMap::new(),
        ).await.unwrap();

        manager.update_status(&id, ProposalStatus::Voting).await.unwrap();

        let proposal = manager.get_proposal(&id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Voting);
    }

    #[tokio::test]
    async fn test_proposal_voting_status() {
        let mut manager = ProposalManager::new(ProposalConfig {
            require_review: false,
            ..ProposalConfig::default()
        });

        let id = manager.create_proposal(
            ProposalType::General {
                title: "Test".to_string(),
                description: "Test proposal".to_string(),
                actions: vec!["action1".to_string()],
            },
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            "author1".to_string(),
            None,
            ProposalPriority::Low,
            vec![],
            HashMap::new(),
        ).await.unwrap();

        let proposal = manager.get_proposal(&id).unwrap();
        assert!(manager.is_voting_active(proposal));
    }

    #[tokio::test]
    async fn test_proposal_dependencies() {
        let mut manager = ProposalManager::new(ProposalConfig::default());

        let dep_id = manager.create_proposal(
            ProposalType::General {
                title: "Dependency".to_string(),
                description: "Dependency proposal".to_string(),
                actions: vec!["action1".to_string()],
            },
            "Dependency Proposal".to_string(),
            "A dependency proposal".to_string(),
            "author1".to_string(),
            None,
            ProposalPriority::Low,
            vec![],
            HashMap::new(),
        ).await.unwrap();

        let main_id = manager.create_proposal(
            ProposalType::General {
                title: "Main".to_string(),
                description: "Main proposal".to_string(),
                actions: vec!["action2".to_string()],
            },
            "Main Proposal".to_string(),
            "A main proposal".to_string(),
            "author1".to_string(),
            None,
            ProposalPriority::Low,
            vec![],
            HashMap::new(),
        ).await.unwrap();

        manager.add_dependency(&main_id, dep_id).await.unwrap();

        let main_proposal = manager.get_proposal(&main_id).unwrap();
        assert!(!manager.are_dependencies_met(main_proposal));

        // Execute dependency
        manager.update_status(&dep_id, ProposalStatus::Executed).await.unwrap();
        assert!(manager.are_dependencies_met(main_proposal));
    }

    #[test]
    fn test_proposal_statistics() {
        let mut manager = ProposalManager::new(ProposalConfig::default());

        // Add some test data
        let proposal1 = Proposal {
            id: Uuid::new_v4(),
            proposal_type: ProposalType::General {
                title: "Test".to_string(),
                description: "Test".to_string(),
                actions: vec![],
            },
            title: "Test 1".to_string(),
            description: "Description".to_string(),
            author: "author1".to_string(),
            status: ProposalStatus::Voting,
            priority: ProposalPriority::High,
            created_at: Utc::now(),
            voting_start: Utc::now(),
            voting_end: Utc::now() + chrono::Duration::days(7),
            execution_deadline: None,
            quorum_required: 1000,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            total_votes: 0,
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            executable: false,
        };

        manager.proposals.insert(proposal1.id, proposal1);

        let stats = manager.get_statistics();
        assert_eq!(stats.total_proposals, 1);
        assert_eq!(stats.active_voting, 1);
        assert_eq!(stats.high_priority, 1);
    }
}