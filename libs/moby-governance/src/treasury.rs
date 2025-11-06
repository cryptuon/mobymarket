//! Treasury management system for governance

use crate::error::{GovernanceError, GovernanceResult};
use crate::proposals::ProposalId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Treasury action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreasuryAction {
    /// Transfer funds to an address
    Transfer {
        recipient: String,
        amount: u64,
        token: String,
        purpose: String,
    },
    /// Grant funding for a project
    Grant {
        grantee: String,
        amount: u64,
        token: String,
        milestone_based: bool,
        milestones: Vec<TreasuryMilestone>,
    },
    /// Stake treasury funds
    Stake {
        validator: String,
        amount: u64,
        token: String,
        duration: chrono::Duration,
    },
    /// Invest in yield-generating protocols
    Invest {
        protocol: String,
        amount: u64,
        token: String,
        expected_yield: f64,
    },
    /// Emergency withdrawal
    EmergencyWithdraw {
        amount: u64,
        token: String,
        justification: String,
    },
    /// Token swap
    Swap {
        from_token: String,
        to_token: String,
        amount: u64,
        min_output: u64,
    },
}

/// Treasury milestone for grant tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryMilestone {
    /// Milestone identifier
    pub id: String,
    /// Description
    pub description: String,
    /// Amount to release
    pub amount: u64,
    /// Completion criteria
    pub completion_criteria: String,
    /// Whether completed
    pub completed: bool,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Reviewer who approved
    pub reviewed_by: Option<String>,
}

/// Treasury proposal for governance voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryProposal {
    /// Proposal identifier
    pub id: ProposalId,
    /// Treasury action being proposed
    pub action: TreasuryAction,
    /// Detailed justification
    pub justification: String,
    /// Impact assessment
    pub impact_assessment: String,
    /// Risk assessment
    pub risk_assessment: String,
    /// Expected outcomes
    pub expected_outcomes: Vec<String>,
    /// Success metrics
    pub success_metrics: Vec<String>,
    /// Proposal status
    pub status: TreasuryProposalStatus,
    /// Who proposed it
    pub proposed_by: String,
    /// When proposed
    pub proposed_at: DateTime<Utc>,
    /// Execution deadline
    pub execution_deadline: Option<DateTime<Utc>>,
}

/// Status of treasury proposals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreasuryProposalStatus {
    /// Under review
    UnderReview,
    /// Active voting
    Voting,
    /// Approved and ready for execution
    Approved,
    /// Executed successfully
    Executed,
    /// Rejected by vote
    Rejected,
    /// Cancelled
    Cancelled,
    /// Expired without execution
    Expired,
}

/// Treasury balance for a specific token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryBalance {
    /// Token identifier
    pub token: String,
    /// Available balance
    pub available: u64,
    /// Allocated but not yet spent
    pub allocated: u64,
    /// Staked amount
    pub staked: u64,
    /// Invested amount
    pub invested: u64,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Treasury transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryTransaction {
    /// Transaction ID
    pub id: String,
    /// Transaction type
    pub transaction_type: TreasuryTransactionType,
    /// Token involved
    pub token: String,
    /// Amount
    pub amount: u64,
    /// Counterparty (recipient/sender)
    pub counterparty: String,
    /// Associated proposal ID
    pub proposal_id: Option<ProposalId>,
    /// Transaction hash (if on-chain)
    pub tx_hash: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Status
    pub status: TreasuryTransactionStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of treasury transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreasuryTransactionType {
    /// Incoming deposit
    Deposit,
    /// Outgoing transfer
    Transfer,
    /// Grant payment
    Grant,
    /// Staking transaction
    Stake,
    /// Investment transaction
    Investment,
    /// Swap transaction
    Swap,
    /// Fee collection
    Fee,
    /// Yield/reward collection
    Yield,
}

/// Status of treasury transactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreasuryTransactionStatus {
    /// Pending execution
    Pending,
    /// Successfully executed
    Executed,
    /// Failed execution
    Failed,
    /// Cancelled
    Cancelled,
}

/// Configuration for treasury management
#[derive(Debug, Clone)]
pub struct TreasuryConfig {
    /// Minimum proposal threshold (percentage of treasury)
    pub min_proposal_threshold: f64,
    /// Maximum single proposal amount (percentage of treasury)
    pub max_single_proposal: f64,
    /// Diversification requirements
    pub max_token_concentration: f64,
    /// Required approvals for large transactions
    pub large_transaction_threshold: u64,
    /// Emergency withdrawal limits
    pub emergency_withdrawal_limit: f64,
    /// Yield target percentage
    pub target_yield: f64,
}

impl Default for TreasuryConfig {
    fn default() -> Self {
        Self {
            min_proposal_threshold: 0.001, // 0.1%
            max_single_proposal: 0.1,      // 10%
            max_token_concentration: 0.5,  // 50%
            large_transaction_threshold: 1_000_000,
            emergency_withdrawal_limit: 0.05, // 5%
            target_yield: 0.08, // 8%
        }
    }
}

/// Treasury management system
pub struct Treasury {
    /// Configuration
    config: TreasuryConfig,
    /// Token balances
    balances: HashMap<String, TreasuryBalance>,
    /// Treasury proposals
    proposals: HashMap<ProposalId, TreasuryProposal>,
    /// Transaction history
    transactions: Vec<TreasuryTransaction>,
    /// Active allocations (committed but not spent)
    allocations: HashMap<ProposalId, u64>,
    /// Multi-sig signers
    signers: Vec<String>,
    /// Required signatures for transactions
    required_signatures: usize,
    /// Transaction counter
    transaction_counter: u64,
}

impl Treasury {
    /// Create a new treasury
    pub fn new(config: TreasuryConfig, signers: Vec<String>, required_signatures: usize) -> Self {
        Self {
            config,
            balances: HashMap::new(),
            proposals: HashMap::new(),
            transactions: Vec::new(),
            allocations: HashMap::new(),
            signers,
            required_signatures,
            transaction_counter: 0,
        }
    }

    /// Add funds to treasury
    pub async fn deposit(
        &mut self,
        token: String,
        amount: u64,
        from: String,
        tx_hash: Option<String>,
    ) -> GovernanceResult<String> {
        // Update balance
        let balance = self.balances.entry(token.clone()).or_insert_with(|| TreasuryBalance {
            token: token.clone(),
            available: 0,
            allocated: 0,
            staked: 0,
            invested: 0,
            last_updated: Utc::now(),
        });

        balance.available += amount;
        balance.last_updated = Utc::now();

        // Record transaction
        self.transaction_counter += 1;
        let transaction_id = format!("treasury_tx_{}", self.transaction_counter);

        let transaction = TreasuryTransaction {
            id: transaction_id.clone(),
            transaction_type: TreasuryTransactionType::Deposit,
            token,
            amount,
            counterparty: from,
            proposal_id: None,
            tx_hash,
            timestamp: Utc::now(),
            status: TreasuryTransactionStatus::Executed,
            metadata: HashMap::new(),
        };

        self.transactions.push(transaction);
        Ok(transaction_id)
    }

    /// Create a treasury proposal
    pub async fn create_proposal(
        &mut self,
        action: TreasuryAction,
        justification: String,
        impact_assessment: String,
        risk_assessment: String,
        expected_outcomes: Vec<String>,
        success_metrics: Vec<String>,
        proposed_by: String,
        execution_deadline: Option<DateTime<Utc>>,
    ) -> GovernanceResult<ProposalId> {
        // Validate proposal against treasury policies
        self.validate_treasury_action(&action)?;

        let proposal_id = Uuid::new_v4();

        let proposal = TreasuryProposal {
            id: proposal_id,
            action,
            justification,
            impact_assessment,
            risk_assessment,
            expected_outcomes,
            success_metrics,
            status: TreasuryProposalStatus::UnderReview,
            proposed_by,
            proposed_at: Utc::now(),
            execution_deadline,
        };

        self.proposals.insert(proposal_id, proposal);
        Ok(proposal_id)
    }

    /// Execute an approved treasury proposal
    pub async fn execute_proposal(
        &mut self,
        proposal_id: &ProposalId,
        executed_by: String,
        signatures: Vec<String>,
    ) -> GovernanceResult<String> {
        // Verify signatures
        if signatures.len() < self.required_signatures {
            return Err(GovernanceError::MultiSigThresholdNotMet {
                required: self.required_signatures as u8,
                provided: signatures.len() as u8,
            });
        }

        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Proposal {} not found", proposal_id),
            })?;

        if proposal.status != TreasuryProposalStatus::Approved {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Proposal status is {:?}, expected Approved", proposal.status),
            });
        }

        // Check execution deadline
        if let Some(deadline) = proposal.execution_deadline {
            if Utc::now() > deadline {
                proposal.status = TreasuryProposalStatus::Expired;
                return Err(GovernanceError::OperationFailed {
                    reason: "Proposal execution deadline has passed".to_string(),
                });
            }
        }

        // Execute the action
        let transaction_id = self.execute_treasury_action(&proposal.action, Some(*proposal_id)).await?;

        // Update proposal status
        proposal.status = TreasuryProposalStatus::Executed;

        Ok(transaction_id)
    }

    /// Get treasury balance for a token
    pub fn get_balance(&self, token: &str) -> Option<&TreasuryBalance> {
        self.balances.get(token)
    }

    /// Get total treasury value (simplified - assumes all tokens have same value)
    pub fn get_total_value(&self) -> u64 {
        self.balances.values()
            .map(|balance| balance.available + balance.allocated + balance.staked + balance.invested)
            .sum()
    }

    /// Get available balance for a token
    pub fn get_available_balance(&self, token: &str) -> u64 {
        self.balances.get(token)
            .map(|b| b.available)
            .unwrap_or(0)
    }

    /// Allocate funds for a proposal
    pub async fn allocate_funds(
        &mut self,
        proposal_id: ProposalId,
        token: &str,
        amount: u64,
    ) -> GovernanceResult<()> {
        let balance = self.balances.get_mut(token)
            .ok_or_else(|| GovernanceError::InsufficientTreasuryFunds {
                required: amount,
                available: 0,
            })?;

        if balance.available < amount {
            return Err(GovernanceError::InsufficientTreasuryFunds {
                required: amount,
                available: balance.available,
            });
        }

        // Move from available to allocated
        balance.available -= amount;
        balance.allocated += amount;
        balance.last_updated = Utc::now();

        // Track allocation
        self.allocations.insert(proposal_id, amount);

        Ok(())
    }

    /// Release allocated funds
    pub async fn release_allocation(
        &mut self,
        proposal_id: ProposalId,
        token: &str,
    ) -> GovernanceResult<()> {
        let amount = self.allocations.remove(&proposal_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("No allocation found for proposal {}", proposal_id),
            })?;

        let balance = self.balances.get_mut(token)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Token {} not found", token),
            })?;

        // Move from allocated back to available
        balance.allocated -= amount;
        balance.available += amount;
        balance.last_updated = Utc::now();

        Ok(())
    }

    /// Get treasury statistics
    pub fn get_treasury_statistics(&self) -> TreasuryStatistics {
        let mut stats = TreasuryStatistics::default();

        // Calculate totals
        for balance in self.balances.values() {
            stats.total_assets += balance.available + balance.allocated + balance.staked + balance.invested;
            stats.available_funds += balance.available;
            stats.allocated_funds += balance.allocated;
            stats.staked_funds += balance.staked;
            stats.invested_funds += balance.invested;
        }

        stats.total_tokens = self.balances.len();
        stats.total_proposals = self.proposals.len();
        stats.total_transactions = self.transactions.len();

        // Count proposal statuses
        for proposal in self.proposals.values() {
            match proposal.status {
                TreasuryProposalStatus::UnderReview => stats.proposals_under_review += 1,
                TreasuryProposalStatus::Voting => stats.proposals_voting += 1,
                TreasuryProposalStatus::Approved => stats.proposals_approved += 1,
                TreasuryProposalStatus::Executed => stats.proposals_executed += 1,
                TreasuryProposalStatus::Rejected => stats.proposals_rejected += 1,
                TreasuryProposalStatus::Cancelled => stats.proposals_cancelled += 1,
                TreasuryProposalStatus::Expired => stats.proposals_expired += 1,
            }
        }

        // Calculate diversification
        if stats.total_assets > 0 {
            let mut max_concentration = 0.0;
            for balance in self.balances.values() {
                let total_balance = balance.available + balance.allocated + balance.staked + balance.invested;
                let concentration = total_balance as f64 / stats.total_assets as f64;
                if concentration > max_concentration {
                    max_concentration = concentration;
                }
            }
            stats.diversification_ratio = 1.0 - max_concentration;
        }

        stats
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Option<&TreasuryProposal> {
        self.proposals.get(proposal_id)
    }

    /// Get proposals by status
    pub fn get_proposals_by_status(&self, status: &TreasuryProposalStatus) -> Vec<&TreasuryProposal> {
        self.proposals.values()
            .filter(|p| p.status == *status)
            .collect()
    }

    /// Get recent transactions
    pub fn get_recent_transactions(&self, limit: usize) -> Vec<&TreasuryTransaction> {
        let mut transactions = self.transactions.iter().collect::<Vec<_>>();
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        transactions.into_iter().take(limit).collect()
    }

    // Helper methods

    async fn execute_treasury_action(
        &mut self,
        action: &TreasuryAction,
        proposal_id: Option<ProposalId>,
    ) -> GovernanceResult<String> {
        self.transaction_counter += 1;
        let transaction_id = format!("treasury_tx_{}", self.transaction_counter);

        let transaction = match action {
            TreasuryAction::Transfer { recipient, amount, token, purpose: _ } => {
                self.execute_transfer(token, *amount, recipient.clone())?;

                TreasuryTransaction {
                    id: transaction_id.clone(),
                    transaction_type: TreasuryTransactionType::Transfer,
                    token: token.clone(),
                    amount: *amount,
                    counterparty: recipient.clone(),
                    proposal_id,
                    tx_hash: None, // Would be filled by actual blockchain transaction
                    timestamp: Utc::now(),
                    status: TreasuryTransactionStatus::Executed,
                    metadata: HashMap::new(),
                }
            }
            TreasuryAction::Grant { grantee, amount, token, milestone_based: _, milestones: _ } => {
                self.execute_transfer(token, *amount, grantee.clone())?;

                TreasuryTransaction {
                    id: transaction_id.clone(),
                    transaction_type: TreasuryTransactionType::Grant,
                    token: token.clone(),
                    amount: *amount,
                    counterparty: grantee.clone(),
                    proposal_id,
                    tx_hash: None,
                    timestamp: Utc::now(),
                    status: TreasuryTransactionStatus::Executed,
                    metadata: HashMap::new(),
                }
            }
            TreasuryAction::Stake { validator, amount, token, duration: _ } => {
                self.execute_stake(token, *amount, validator.clone())?;

                TreasuryTransaction {
                    id: transaction_id.clone(),
                    transaction_type: TreasuryTransactionType::Stake,
                    token: token.clone(),
                    amount: *amount,
                    counterparty: validator.clone(),
                    proposal_id,
                    tx_hash: None,
                    timestamp: Utc::now(),
                    status: TreasuryTransactionStatus::Executed,
                    metadata: HashMap::new(),
                }
            }
            _ => {
                return Err(GovernanceError::OperationFailed {
                    reason: "Action type not yet implemented".to_string(),
                });
            }
        };

        self.transactions.push(transaction);
        Ok(transaction_id)
    }

    fn execute_transfer(&mut self, token: &str, amount: u64, _recipient: String) -> GovernanceResult<()> {
        let balance = self.balances.get_mut(token)
            .ok_or_else(|| GovernanceError::InsufficientTreasuryFunds {
                required: amount,
                available: 0,
            })?;

        if balance.available < amount {
            return Err(GovernanceError::InsufficientTreasuryFunds {
                required: amount,
                available: balance.available,
            });
        }

        balance.available -= amount;
        balance.last_updated = Utc::now();

        Ok(())
    }

    fn execute_stake(&mut self, token: &str, amount: u64, _validator: String) -> GovernanceResult<()> {
        let balance = self.balances.get_mut(token)
            .ok_or_else(|| GovernanceError::InsufficientTreasuryFunds {
                required: amount,
                available: 0,
            })?;

        if balance.available < amount {
            return Err(GovernanceError::InsufficientTreasuryFunds {
                required: amount,
                available: balance.available,
            });
        }

        balance.available -= amount;
        balance.staked += amount;
        balance.last_updated = Utc::now();

        Ok(())
    }

    fn validate_treasury_action(&self, action: &TreasuryAction) -> GovernanceResult<()> {
        match action {
            TreasuryAction::Transfer { amount, token, .. } |
            TreasuryAction::Grant { amount, token, .. } => {
                let available = self.get_available_balance(token);
                if *amount > available {
                    return Err(GovernanceError::InsufficientTreasuryFunds {
                        required: *amount,
                        available,
                    });
                }

                // Check percentage limits
                let total_value = self.get_total_value();
                let percentage = *amount as f64 / total_value as f64;

                if percentage < self.config.min_proposal_threshold {
                    return Err(GovernanceError::OperationFailed {
                        reason: format!("Amount below minimum threshold of {}%",
                                      self.config.min_proposal_threshold * 100.0),
                    });
                }

                if percentage > self.config.max_single_proposal {
                    return Err(GovernanceError::OperationFailed {
                        reason: format!("Amount exceeds maximum single proposal of {}%",
                                      self.config.max_single_proposal * 100.0),
                    });
                }
            }
            TreasuryAction::EmergencyWithdraw { amount, .. } => {
                let total_value = self.get_total_value();
                let percentage = *amount as f64 / total_value as f64;

                if percentage > self.config.emergency_withdrawal_limit {
                    return Err(GovernanceError::OperationFailed {
                        reason: format!("Emergency withdrawal exceeds limit of {}%",
                                      self.config.emergency_withdrawal_limit * 100.0),
                    });
                }
            }
            _ => {} // Other validations as needed
        }

        Ok(())
    }
}

/// Statistics for treasury analysis
#[derive(Debug, Default, Clone)]
pub struct TreasuryStatistics {
    pub total_assets: u64,
    pub available_funds: u64,
    pub allocated_funds: u64,
    pub staked_funds: u64,
    pub invested_funds: u64,
    pub total_tokens: usize,
    pub total_proposals: usize,
    pub total_transactions: usize,
    pub proposals_under_review: usize,
    pub proposals_voting: usize,
    pub proposals_approved: usize,
    pub proposals_executed: usize,
    pub proposals_rejected: usize,
    pub proposals_cancelled: usize,
    pub proposals_expired: usize,
    pub diversification_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_treasury_deposit() {
        let mut treasury = Treasury::new(
            TreasuryConfig::default(),
            vec!["signer1".to_string(), "signer2".to_string()],
            2,
        );

        let tx_id = treasury.deposit(
            "USDC".to_string(),
            1000000,
            "depositor".to_string(),
            Some("0x123...".to_string()),
        ).await.unwrap();

        assert!(!tx_id.is_empty());

        let balance = treasury.get_balance("USDC").unwrap();
        assert_eq!(balance.available, 1000000);

        let stats = treasury.get_treasury_statistics();
        assert_eq!(stats.total_assets, 1000000);
        assert_eq!(stats.total_transactions, 1);
    }

    #[tokio::test]
    async fn test_create_treasury_proposal() {
        let mut treasury = Treasury::new(
            TreasuryConfig::default(),
            vec!["signer1".to_string()],
            1,
        );

        // Add funds first
        treasury.deposit("USDC".to_string(), 1000000, "depositor".to_string(), None).await.unwrap();

        let proposal_id = treasury.create_proposal(
            TreasuryAction::Transfer {
                recipient: "recipient".to_string(),
                amount: 50000,
                token: "USDC".to_string(),
                purpose: "Development grant".to_string(),
            },
            "Fund development work".to_string(),
            "Will accelerate feature development".to_string(),
            "Low risk, established team".to_string(),
            vec!["Faster development".to_string()],
            vec!["Features delivered on time".to_string()],
            "proposer".to_string(),
            None,
        ).await.unwrap();

        let proposal = treasury.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, TreasuryProposalStatus::UnderReview);

        if let TreasuryAction::Transfer { amount, .. } = &proposal.action {
            assert_eq!(*amount, 50000);
        } else {
            panic!("Expected Transfer action");
        }
    }

    #[tokio::test]
    async fn test_fund_allocation() {
        let mut treasury = Treasury::new(
            TreasuryConfig::default(),
            vec!["signer1".to_string()],
            1,
        );

        // Add funds
        treasury.deposit("USDC".to_string(), 1000000, "depositor".to_string(), None).await.unwrap();

        let proposal_id = Uuid::new_v4();

        // Allocate funds
        treasury.allocate_funds(proposal_id, "USDC", 100000).await.unwrap();

        let balance = treasury.get_balance("USDC").unwrap();
        assert_eq!(balance.available, 900000);
        assert_eq!(balance.allocated, 100000);

        // Release allocation
        treasury.release_allocation(proposal_id, "USDC").await.unwrap();

        let balance = treasury.get_balance("USDC").unwrap();
        assert_eq!(balance.available, 1000000);
        assert_eq!(balance.allocated, 0);
    }

    #[tokio::test]
    async fn test_treasury_limits() {
        let mut treasury = Treasury::new(
            TreasuryConfig {
                max_single_proposal: 0.05, // 5%
                ..TreasuryConfig::default()
            },
            vec!["signer1".to_string()],
            1,
        );

        // Add funds
        treasury.deposit("USDC".to_string(), 1000000, "depositor".to_string(), None).await.unwrap();

        // Try to create proposal that exceeds limits
        let result = treasury.create_proposal(
            TreasuryAction::Transfer {
                recipient: "recipient".to_string(),
                amount: 100000, // 10% of treasury, exceeds 5% limit
                token: "USDC".to_string(),
                purpose: "Large grant".to_string(),
            },
            "Large grant proposal".to_string(),
            "High impact project".to_string(),
            "Medium risk".to_string(),
            vec!["Major advancement".to_string()],
            vec!["Success metrics".to_string()],
            "proposer".to_string(),
            None,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_treasury_statistics() {
        let mut treasury = Treasury::new(
            TreasuryConfig::default(),
            vec!["signer1".to_string()],
            1,
        );

        // Add different tokens
        treasury.deposit("USDC".to_string(), 600000, "depositor".to_string(), None).await.unwrap();
        treasury.deposit("ETH".to_string(), 400000, "depositor".to_string(), None).await.unwrap();

        // Stake some funds
        treasury.execute_stake("USDC", 100000, "validator".to_string()).unwrap();

        let stats = treasury.get_treasury_statistics();
        assert_eq!(stats.total_assets, 1000000);
        assert_eq!(stats.available_funds, 900000);
        assert_eq!(stats.staked_funds, 100000);
        assert_eq!(stats.total_tokens, 2);

        // Check diversification (should be less than 1.0 since we have concentration)
        assert!(stats.diversification_ratio > 0.0);
        assert!(stats.diversification_ratio < 1.0);
    }
}