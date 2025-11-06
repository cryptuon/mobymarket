//! Voting system with delegation support

use crate::error::{GovernanceError, GovernanceResult};
use crate::proposals::{Proposal, ProposalId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Different types of votes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteType {
    /// Vote in favor
    For,
    /// Vote against
    Against,
    /// Abstain from voting
    Abstain,
}

/// Voting power represents the weight of a vote
pub type VotingPower = u64;

/// A single vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Proposal being voted on
    pub proposal_id: ProposalId,
    /// Voter identifier
    pub voter: String,
    /// Type of vote
    pub vote_type: VoteType,
    /// Voting power used
    pub voting_power: VotingPower,
    /// Timestamp when vote was cast
    pub timestamp: DateTime<Utc>,
    /// Optional justification
    pub justification: Option<String>,
    /// Whether this is a delegated vote
    pub is_delegated: bool,
    /// Original voter if this is a delegated vote
    pub original_voter: Option<String>,
}

/// Different voting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingStrategy {
    /// Simple majority (50% + 1)
    SimpleMajority,
    /// Supermajority (2/3)
    Supermajority,
    /// Absolute majority (more than 50% of total eligible voters)
    AbsoluteMajority,
    /// Unanimous (100%)
    Unanimous,
    /// Custom percentage threshold
    CustomThreshold(u8),
}

/// Result of a voting process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    /// Proposal that was voted on
    pub proposal_id: ProposalId,
    /// Total votes for
    pub votes_for: VotingPower,
    /// Total votes against
    pub votes_against: VotingPower,
    /// Total abstained votes
    pub votes_abstain: VotingPower,
    /// Total voting power that participated
    pub total_votes: VotingPower,
    /// Total eligible voting power
    pub total_eligible: VotingPower,
    /// Whether the proposal passed
    pub passed: bool,
    /// Percentage in favor
    pub percentage_for: f64,
    /// Percentage against
    pub percentage_against: f64,
    /// Participation rate
    pub participation_rate: f64,
    /// Voting strategy used
    pub strategy: VotingStrategy,
    /// Whether quorum was reached
    pub quorum_reached: bool,
}

/// Configuration for voting system
#[derive(Debug, Clone)]
pub struct VotingConfig {
    /// Default voting strategy
    pub default_strategy: VotingStrategy,
    /// Minimum voting power to participate
    pub min_voting_power: VotingPower,
    /// Whether to allow vote changes
    pub allow_vote_changes: bool,
    /// Whether to require justification for votes
    pub require_justification: bool,
    /// Maximum justification length
    pub max_justification_length: usize,
}

impl Default for VotingConfig {
    fn default() -> Self {
        Self {
            default_strategy: VotingStrategy::SimpleMajority,
            min_voting_power: 1,
            allow_vote_changes: true,
            require_justification: false,
            max_justification_length: 1000,
        }
    }
}

/// Voting system manages all voting operations
pub struct VotingSystem {
    /// Configuration
    config: VotingConfig,
    /// Votes by proposal
    proposal_votes: HashMap<ProposalId, HashMap<String, Vote>>,
    /// Voting power by address
    voting_power: HashMap<String, VotingPower>,
    /// Total eligible voting power
    total_eligible_power: VotingPower,
    /// Voting results cache
    results_cache: HashMap<ProposalId, VotingResult>,
}

impl VotingSystem {
    /// Create a new voting system
    pub fn new(config: VotingConfig) -> Self {
        Self {
            config,
            proposal_votes: HashMap::new(),
            voting_power: HashMap::new(),
            total_eligible_power: 0,
            results_cache: HashMap::new(),
        }
    }

    /// Set voting power for an address
    pub async fn set_voting_power(
        &mut self,
        address: String,
        power: VotingPower,
    ) -> GovernanceResult<()> {
        let old_power = self.voting_power.get(&address).copied().unwrap_or(0);

        self.voting_power.insert(address, power);

        // Update total eligible power
        if power > old_power {
            self.total_eligible_power += power - old_power;
        } else {
            self.total_eligible_power -= old_power - power;
        }

        // Invalidate results cache since voting power changed
        self.results_cache.clear();

        Ok(())
    }

    /// Get voting power for an address
    pub fn get_voting_power(&self, address: &str) -> VotingPower {
        self.voting_power.get(address).copied().unwrap_or(0)
    }

    /// Cast a vote on a proposal
    pub async fn cast_vote(
        &mut self,
        proposal: &Proposal,
        voter: String,
        vote_type: VoteType,
        justification: Option<String>,
    ) -> GovernanceResult<()> {
        // Validate voting eligibility
        self.validate_voting_eligibility(proposal, &voter, &justification)?;

        let voting_power = self.get_voting_power(&voter);
        if voting_power < self.config.min_voting_power {
            return Err(GovernanceError::InsufficientVotingPower {
                required: self.config.min_voting_power,
                available: voting_power,
            });
        }

        // Check if already voted
        let proposal_votes = self.proposal_votes
            .entry(proposal.id)
            .or_insert_with(HashMap::new);

        if proposal_votes.contains_key(&voter) && !self.config.allow_vote_changes {
            return Err(GovernanceError::AlreadyVoted { voter });
        }

        // Create vote
        let vote = Vote {
            proposal_id: proposal.id,
            voter: voter.clone(),
            vote_type,
            voting_power,
            timestamp: Utc::now(),
            justification,
            is_delegated: false,
            original_voter: None,
        };

        // Store vote
        proposal_votes.insert(voter, vote);

        // Invalidate results cache for this proposal
        self.results_cache.remove(&proposal.id);

        Ok(())
    }

    /// Cast a delegated vote
    pub async fn cast_delegated_vote(
        &mut self,
        proposal: &Proposal,
        delegate: String,
        original_voter: String,
        vote_type: VoteType,
        delegated_power: VotingPower,
        justification: Option<String>,
    ) -> GovernanceResult<()> {
        // Validate voting eligibility for delegate
        self.validate_voting_eligibility(proposal, &delegate, &justification)?;

        // Check delegate has sufficient voting power
        let delegate_power = self.get_voting_power(&delegate);
        if delegate_power < self.config.min_voting_power {
            return Err(GovernanceError::InsufficientVotingPower {
                required: self.config.min_voting_power,
                available: delegate_power,
            });
        }

        // Check if original voter already voted directly
        let proposal_votes = self.proposal_votes
            .entry(proposal.id)
            .or_insert_with(HashMap::new);

        if proposal_votes.contains_key(&original_voter) {
            return Err(GovernanceError::AlreadyVoted { voter: original_voter });
        }

        // Create delegated vote
        let vote = Vote {
            proposal_id: proposal.id,
            voter: delegate.clone(),
            vote_type,
            voting_power: delegated_power,
            timestamp: Utc::now(),
            justification,
            is_delegated: true,
            original_voter: Some(original_voter.clone()),
        };

        // Store vote under original voter's key to prevent double voting
        proposal_votes.insert(original_voter, vote);

        // Invalidate results cache for this proposal
        self.results_cache.remove(&proposal.id);

        Ok(())
    }

    /// Get vote for a voter on a proposal
    pub fn get_vote(&self, proposal_id: &ProposalId, voter: &str) -> Option<&Vote> {
        self.proposal_votes
            .get(proposal_id)?
            .get(voter)
    }

    /// Get all votes for a proposal
    pub fn get_proposal_votes(&self, proposal_id: &ProposalId) -> Vec<&Vote> {
        self.proposal_votes
            .get(proposal_id)
            .map(|votes| votes.values().collect())
            .unwrap_or_default()
    }

    /// Calculate voting results for a proposal
    pub async fn calculate_results(
        &mut self,
        proposal: &Proposal,
        strategy: Option<VotingStrategy>,
    ) -> GovernanceResult<VotingResult> {
        // Check cache first
        if let Some(cached_result) = self.results_cache.get(&proposal.id) {
            return Ok(cached_result.clone());
        }

        let votes = self.get_proposal_votes(&proposal.id);
        let strategy = strategy.unwrap_or(self.config.default_strategy.clone());

        let mut votes_for = 0;
        let mut votes_against = 0;
        let mut votes_abstain = 0;

        for vote in &votes {
            match vote.vote_type {
                VoteType::For => votes_for += vote.voting_power,
                VoteType::Against => votes_against += vote.voting_power,
                VoteType::Abstain => votes_abstain += vote.voting_power,
            }
        }

        let total_votes = votes_for + votes_against + votes_abstain;
        let total_decisive_votes = votes_for + votes_against; // Exclude abstentions

        let percentage_for = if total_decisive_votes > 0 {
            (votes_for as f64 / total_decisive_votes as f64) * 100.0
        } else {
            0.0
        };

        let percentage_against = if total_decisive_votes > 0 {
            (votes_against as f64 / total_decisive_votes as f64) * 100.0
        } else {
            0.0
        };

        let participation_rate = if self.total_eligible_power > 0 {
            (total_votes as f64 / self.total_eligible_power as f64) * 100.0
        } else {
            0.0
        };

        // Check if quorum is reached
        let quorum_reached = total_votes >= proposal.quorum_required;

        // Determine if proposal passed based on strategy
        let passed = quorum_reached && self.evaluate_strategy(&strategy, votes_for, votes_against, self.total_eligible_power);

        let result = VotingResult {
            proposal_id: proposal.id,
            votes_for,
            votes_against,
            votes_abstain,
            total_votes,
            total_eligible: self.total_eligible_power,
            passed,
            percentage_for,
            percentage_against,
            participation_rate,
            strategy,
            quorum_reached,
        };

        // Cache result
        self.results_cache.insert(proposal.id, result.clone());

        Ok(result)
    }

    /// Get voting statistics for a proposal
    pub async fn get_voting_statistics(&self, proposal_id: &ProposalId) -> VotingStatistics {
        let votes = self.get_proposal_votes(proposal_id);

        let mut stats = VotingStatistics::default();
        stats.total_votes = votes.len();

        for vote in votes {
            match vote.vote_type {
                VoteType::For => stats.votes_for += 1,
                VoteType::Against => stats.votes_against += 1,
                VoteType::Abstain => stats.votes_abstain += 1,
            }

            if vote.is_delegated {
                stats.delegated_votes += 1;
            } else {
                stats.direct_votes += 1;
            }

            if vote.justification.is_some() {
                stats.justified_votes += 1;
            }

            stats.total_voting_power += vote.voting_power;
        }

        if stats.total_votes > 0 {
            stats.average_voting_power = stats.total_voting_power / stats.total_votes as u64;
        }

        stats
    }

    /// Remove a vote (if allowed)
    pub async fn remove_vote(
        &mut self,
        proposal_id: &ProposalId,
        voter: &str,
    ) -> GovernanceResult<()> {
        if !self.config.allow_vote_changes {
            return Err(GovernanceError::OperationFailed {
                reason: "Vote changes not allowed".to_string(),
            });
        }

        if let Some(proposal_votes) = self.proposal_votes.get_mut(proposal_id) {
            proposal_votes.remove(voter);

            // Invalidate results cache
            self.results_cache.remove(proposal_id);
        }

        Ok(())
    }

    /// Get voter turnout for a proposal
    pub fn get_voter_turnout(&self, proposal_id: &ProposalId) -> f64 {
        let votes = self.get_proposal_votes(proposal_id);
        let participating_power: VotingPower = votes.iter()
            .map(|vote| vote.voting_power)
            .sum();

        if self.total_eligible_power > 0 {
            (participating_power as f64 / self.total_eligible_power as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get voting distribution
    pub fn get_voting_distribution(&self, proposal_id: &ProposalId) -> VotingDistribution {
        let votes = self.get_proposal_votes(proposal_id);

        let mut distribution = VotingDistribution::default();

        for vote in votes {
            distribution.total_power += vote.voting_power;

            match vote.vote_type {
                VoteType::For => {
                    distribution.for_votes += 1;
                    distribution.for_power += vote.voting_power;
                }
                VoteType::Against => {
                    distribution.against_votes += 1;
                    distribution.against_power += vote.voting_power;
                }
                VoteType::Abstain => {
                    distribution.abstain_votes += 1;
                    distribution.abstain_power += vote.voting_power;
                }
            }
        }

        // Calculate percentages
        if distribution.total_power > 0 {
            distribution.for_percentage = (distribution.for_power as f64 / distribution.total_power as f64) * 100.0;
            distribution.against_percentage = (distribution.against_power as f64 / distribution.total_power as f64) * 100.0;
            distribution.abstain_percentage = (distribution.abstain_power as f64 / distribution.total_power as f64) * 100.0;
        }

        distribution
    }

    // Helper methods

    fn validate_voting_eligibility(
        &self,
        proposal: &Proposal,
        voter: &str,
        justification: &Option<String>,
    ) -> GovernanceResult<()> {
        // Check if proposal is in voting state
        if !matches!(proposal.status, crate::proposals::ProposalStatus::Voting) {
            return Err(GovernanceError::InvalidProposalState {
                expected: "Voting".to_string(),
                actual: format!("{:?}", proposal.status),
            });
        }

        // Check voting period
        let now = Utc::now();
        if now < proposal.voting_start {
            return Err(GovernanceError::VotingPeriodNotStarted);
        }
        if now > proposal.voting_end {
            return Err(GovernanceError::VotingPeriodEnded);
        }

        // Check if voter has voting power
        if !self.voting_power.contains_key(voter) {
            return Err(GovernanceError::InsufficientVotingPower {
                required: self.config.min_voting_power,
                available: 0,
            });
        }

        // Check justification requirement
        if self.config.require_justification && justification.is_none() {
            return Err(GovernanceError::OperationFailed {
                reason: "Justification required for voting".to_string(),
            });
        }

        // Check justification length
        if let Some(justification) = justification {
            if justification.len() > self.config.max_justification_length {
                return Err(GovernanceError::OperationFailed {
                    reason: format!(
                        "Justification too long: {} > {}",
                        justification.len(),
                        self.config.max_justification_length
                    ),
                });
            }
        }

        Ok(())
    }

    fn evaluate_strategy(
        &self,
        strategy: &VotingStrategy,
        votes_for: VotingPower,
        votes_against: VotingPower,
        total_eligible: VotingPower,
    ) -> bool {
        let total_decisive = votes_for + votes_against;

        match strategy {
            VotingStrategy::SimpleMajority => {
                votes_for > votes_against
            }
            VotingStrategy::Supermajority => {
                if total_decisive == 0 {
                    false
                } else {
                    (votes_for as f64 / total_decisive as f64) >= 2.0 / 3.0
                }
            }
            VotingStrategy::AbsoluteMajority => {
                if total_eligible == 0 {
                    false
                } else {
                    (votes_for as f64 / total_eligible as f64) > 0.5
                }
            }
            VotingStrategy::Unanimous => {
                votes_against == 0 && votes_for > 0
            }
            VotingStrategy::CustomThreshold(threshold) => {
                if total_decisive == 0 {
                    false
                } else {
                    let percentage = (votes_for as f64 / total_decisive as f64) * 100.0;
                    percentage >= *threshold as f64
                }
            }
        }
    }
}

/// Statistics for voting analysis
#[derive(Debug, Default, Clone)]
pub struct VotingStatistics {
    pub total_votes: usize,
    pub votes_for: usize,
    pub votes_against: usize,
    pub votes_abstain: usize,
    pub direct_votes: usize,
    pub delegated_votes: usize,
    pub justified_votes: usize,
    pub total_voting_power: VotingPower,
    pub average_voting_power: VotingPower,
}

/// Distribution of votes
#[derive(Debug, Default, Clone)]
pub struct VotingDistribution {
    pub for_votes: usize,
    pub against_votes: usize,
    pub abstain_votes: usize,
    pub for_power: VotingPower,
    pub against_power: VotingPower,
    pub abstain_power: VotingPower,
    pub total_power: VotingPower,
    pub for_percentage: f64,
    pub against_percentage: f64,
    pub abstain_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposals::{ProposalManager, ProposalConfig, ProposalType, ProposalPriority, ProposalStatus};
    use std::collections::HashMap;

    async fn create_test_proposal() -> Proposal {
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

        manager.get_proposal(&id).unwrap().clone()
    }

    #[tokio::test]
    async fn test_voting_power_management() {
        let mut voting_system = VotingSystem::new(VotingConfig::default());

        voting_system.set_voting_power("voter1".to_string(), 1000).await.unwrap();
        voting_system.set_voting_power("voter2".to_string(), 2000).await.unwrap();

        assert_eq!(voting_system.get_voting_power("voter1"), 1000);
        assert_eq!(voting_system.get_voting_power("voter2"), 2000);
        assert_eq!(voting_system.total_eligible_power, 3000);
    }

    #[tokio::test]
    async fn test_cast_vote() {
        let mut voting_system = VotingSystem::new(VotingConfig::default());
        let proposal = create_test_proposal().await;

        voting_system.set_voting_power("voter1".to_string(), 1000).await.unwrap();

        voting_system.cast_vote(
            &proposal,
            "voter1".to_string(),
            VoteType::For,
            Some("I support this proposal".to_string()),
        ).await.unwrap();

        let vote = voting_system.get_vote(&proposal.id, "voter1").unwrap();
        assert_eq!(vote.vote_type, VoteType::For);
        assert_eq!(vote.voting_power, 1000);
        assert!(!vote.is_delegated);
    }

    #[tokio::test]
    async fn test_delegated_vote() {
        let mut voting_system = VotingSystem::new(VotingConfig::default());
        let proposal = create_test_proposal().await;

        voting_system.set_voting_power("delegate".to_string(), 1000).await.unwrap();

        voting_system.cast_delegated_vote(
            &proposal,
            "delegate".to_string(),
            "original_voter".to_string(),
            VoteType::Against,
            500,
            None,
        ).await.unwrap();

        let vote = voting_system.get_vote(&proposal.id, "original_voter").unwrap();
        assert_eq!(vote.vote_type, VoteType::Against);
        assert_eq!(vote.voting_power, 500);
        assert!(vote.is_delegated);
        assert_eq!(vote.original_voter, Some("original_voter".to_string()));
    }

    #[tokio::test]
    async fn test_calculate_results() {
        let mut voting_system = VotingSystem::new(VotingConfig::default());
        let mut proposal = create_test_proposal().await;
        proposal.quorum_required = 1000;

        // Set up voters
        voting_system.set_voting_power("voter1".to_string(), 1000).await.unwrap();
        voting_system.set_voting_power("voter2".to_string(), 2000).await.unwrap();
        voting_system.set_voting_power("voter3".to_string(), 500).await.unwrap();

        // Cast votes
        voting_system.cast_vote(&proposal, "voter1".to_string(), VoteType::For, None).await.unwrap();
        voting_system.cast_vote(&proposal, "voter2".to_string(), VoteType::Against, None).await.unwrap();
        voting_system.cast_vote(&proposal, "voter3".to_string(), VoteType::Abstain, None).await.unwrap();

        let result = voting_system.calculate_results(&proposal, None).await.unwrap();

        assert_eq!(result.votes_for, 1000);
        assert_eq!(result.votes_against, 2000);
        assert_eq!(result.votes_abstain, 500);
        assert_eq!(result.total_votes, 3500);
        assert!(result.quorum_reached);
        assert!(!result.passed); // Against > For
    }

    #[tokio::test]
    async fn test_voting_strategies() {
        let mut voting_system = VotingSystem::new(VotingConfig::default());
        let mut proposal = create_test_proposal().await;
        proposal.quorum_required = 1000;

        // Set up voters - 60% for, 40% against
        voting_system.set_voting_power("voter1".to_string(), 6000).await.unwrap();
        voting_system.set_voting_power("voter2".to_string(), 4000).await.unwrap();

        voting_system.cast_vote(&proposal, "voter1".to_string(), VoteType::For, None).await.unwrap();
        voting_system.cast_vote(&proposal, "voter2".to_string(), VoteType::Against, None).await.unwrap();

        // Simple majority - should pass
        let result = voting_system.calculate_results(&proposal, Some(VotingStrategy::SimpleMajority)).await.unwrap();
        assert!(result.passed);

        // Supermajority (66.7%) - should fail
        let result = voting_system.calculate_results(&proposal, Some(VotingStrategy::Supermajority)).await.unwrap();
        assert!(!result.passed);

        // Custom threshold 50% - should pass
        let result = voting_system.calculate_results(&proposal, Some(VotingStrategy::CustomThreshold(50))).await.unwrap();
        assert!(result.passed);

        // Custom threshold 70% - should fail
        let result = voting_system.calculate_results(&proposal, Some(VotingStrategy::CustomThreshold(70))).await.unwrap();
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_voting_statistics() {
        let mut voting_system = VotingSystem::new(VotingConfig::default());
        let proposal = create_test_proposal().await;

        voting_system.set_voting_power("voter1".to_string(), 1000).await.unwrap();
        voting_system.set_voting_power("voter2".to_string(), 2000).await.unwrap();

        voting_system.cast_vote(&proposal, "voter1".to_string(), VoteType::For, Some("Justified".to_string())).await.unwrap();
        voting_system.cast_vote(&proposal, "voter2".to_string(), VoteType::Against, None).await.unwrap();

        let stats = voting_system.get_voting_statistics(&proposal.id).await;

        assert_eq!(stats.total_votes, 2);
        assert_eq!(stats.votes_for, 1);
        assert_eq!(stats.votes_against, 1);
        assert_eq!(stats.direct_votes, 2);
        assert_eq!(stats.justified_votes, 1);
        assert_eq!(stats.total_voting_power, 3000);
        assert_eq!(stats.average_voting_power, 1500);
    }

    #[tokio::test]
    async fn test_voting_distribution() {
        let mut voting_system = VotingSystem::new(VotingConfig::default());
        let proposal = create_test_proposal().await;

        voting_system.set_voting_power("voter1".to_string(), 3000).await.unwrap();
        voting_system.set_voting_power("voter2".to_string(), 1000).await.unwrap();

        voting_system.cast_vote(&proposal, "voter1".to_string(), VoteType::For, None).await.unwrap();
        voting_system.cast_vote(&proposal, "voter2".to_string(), VoteType::Against, None).await.unwrap();

        let distribution = voting_system.get_voting_distribution(&proposal.id);

        assert_eq!(distribution.for_votes, 1);
        assert_eq!(distribution.against_votes, 1);
        assert_eq!(distribution.for_power, 3000);
        assert_eq!(distribution.against_power, 1000);
        assert_eq!(distribution.total_power, 4000);
        assert_eq!(distribution.for_percentage, 75.0);
        assert_eq!(distribution.against_percentage, 25.0);
    }
}