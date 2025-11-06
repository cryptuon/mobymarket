//! Protocol upgrade management system

use crate::error::{GovernanceError, GovernanceResult};
use crate::proposals::ProposalId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Types of protocol upgrades
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpgradeType {
    /// Minor version upgrade (bug fixes, small improvements)
    Minor {
        version: String,
        changes: Vec<String>,
    },
    /// Major version upgrade (significant new features)
    Major {
        version: String,
        breaking_changes: Vec<String>,
        new_features: Vec<String>,
    },
    /// Critical security patch
    Security {
        patch_id: String,
        vulnerabilities_fixed: Vec<String>,
        severity: SecuritySeverity,
    },
    /// Emergency hotfix
    Emergency {
        hotfix_id: String,
        issue_description: String,
        fix_description: String,
    },
    /// Smart contract upgrade
    Contract {
        contract_name: String,
        old_address: String,
        new_address: String,
        migration_required: bool,
    },
    /// Configuration update
    Configuration {
        config_name: String,
        old_config: HashMap<String, String>,
        new_config: HashMap<String, String>,
    },
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Protocol upgrade proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeProposal {
    /// Proposal identifier
    pub id: ProposalId,
    /// Type of upgrade
    pub upgrade_type: UpgradeType,
    /// Upgrade title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Technical specification
    pub technical_spec: String,
    /// Implementation details
    pub implementation_details: String,
    /// Testing plan
    pub testing_plan: String,
    /// Migration plan (if needed)
    pub migration_plan: Option<String>,
    /// Rollback plan
    pub rollback_plan: String,
    /// Risk assessment
    pub risk_assessment: String,
    /// Compatibility assessment
    pub compatibility_assessment: String,
    /// Who proposed the upgrade
    pub proposed_by: String,
    /// When proposed
    pub proposed_at: DateTime<Utc>,
    /// Current status
    pub status: UpgradeStatus,
    /// Target deployment date
    pub target_deployment: Option<DateTime<Utc>>,
    /// Actual deployment date
    pub deployed_at: Option<DateTime<Utc>>,
    /// Upgrade priority
    pub priority: UpgradePriority,
    /// Code hash for verification
    pub code_hash: Option<String>,
    /// Deployment artifacts
    pub artifacts: Vec<UpgradeArtifact>,
}

/// Status of upgrade proposals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpgradeStatus {
    /// Under review by technical committee
    UnderReview,
    /// In testing phase
    Testing,
    /// Ready for governance vote
    ReadyForVote,
    /// Currently being voted on
    Voting,
    /// Approved and ready for deployment
    Approved,
    /// Currently being deployed
    Deploying,
    /// Successfully deployed
    Deployed,
    /// Deployment failed
    DeploymentFailed,
    /// Rejected by vote
    Rejected,
    /// Cancelled
    Cancelled,
    /// Rolled back due to issues
    RolledBack,
}

/// Priority levels for upgrades
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpgradePriority {
    Low,
    Medium,
    High,
    Critical,
    Emergency,
}

/// Upgrade artifacts (code, configurations, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeArtifact {
    /// Artifact name
    pub name: String,
    /// Artifact type
    pub artifact_type: ArtifactType,
    /// Hash for verification
    pub hash: String,
    /// Size in bytes
    pub size: u64,
    /// IPFS or storage URL
    pub url: String,
    /// Description
    pub description: String,
}

/// Types of upgrade artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Compiled binary
    Binary,
    /// WebAssembly module
    Wasm,
    /// Smart contract bytecode
    Bytecode,
    /// Configuration file
    Configuration,
    /// Migration script
    Migration,
    /// Documentation
    Documentation,
}

/// Upgrade deployment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDeployment {
    /// Deployment identifier
    pub id: String,
    /// Associated upgrade proposal
    pub proposal_id: ProposalId,
    /// Deployment strategy
    pub strategy: DeploymentStrategy,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Deployment status
    pub status: DeploymentStatus,
    /// Who initiated deployment
    pub deployed_by: String,
    /// Deployment phases
    pub phases: Vec<DeploymentPhase>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Rollback information
    pub rollback_info: Option<RollbackInfo>,
}

/// Deployment strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    /// All at once deployment
    BigBang,
    /// Gradual rollout
    Rolling {
        percentage_per_phase: u8,
        phase_duration: chrono::Duration,
    },
    /// Blue-green deployment
    BlueGreen,
    /// Canary deployment
    Canary {
        canary_percentage: u8,
        monitoring_duration: chrono::Duration,
    },
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStatus {
    /// Deployment in progress
    InProgress,
    /// Successfully completed
    Completed,
    /// Failed
    Failed,
    /// Rolled back
    RolledBack,
    /// Paused
    Paused,
}

/// Individual deployment phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPhase {
    /// Phase number
    pub phase: u8,
    /// Phase description
    pub description: String,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Phase status
    pub status: DeploymentStatus,
    /// Success criteria
    pub success_criteria: Vec<String>,
    /// Metrics collected
    pub metrics: HashMap<String, f64>,
}

/// Rollback information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    /// Rollback identifier
    pub id: String,
    /// When rollback was initiated
    pub initiated_at: DateTime<Utc>,
    /// When rollback completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Reason for rollback
    pub reason: String,
    /// Who initiated rollback
    pub initiated_by: String,
    /// Rollback status
    pub status: DeploymentStatus,
}

/// Configuration for upgrade management
#[derive(Debug, Clone)]
pub struct UpgradeConfig {
    /// Minimum review period for upgrades
    pub min_review_period: chrono::Duration,
    /// Required approval percentage
    pub approval_threshold: u8,
    /// Emergency upgrade approval threshold
    pub emergency_approval_threshold: u8,
    /// Maximum rollout time
    pub max_rollout_time: chrono::Duration,
    /// Canary deployment percentage
    pub default_canary_percentage: u8,
    /// Monitoring duration after deployment
    pub post_deployment_monitoring: chrono::Duration,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            min_review_period: chrono::Duration::days(7),
            approval_threshold: 67, // 67%
            emergency_approval_threshold: 51, // 51% for emergencies
            max_rollout_time: chrono::Duration::hours(24),
            default_canary_percentage: 10,
            post_deployment_monitoring: chrono::Duration::hours(48),
        }
    }
}

/// Upgrade management system
pub struct UpgradeManager {
    /// Configuration
    config: UpgradeConfig,
    /// Upgrade proposals
    proposals: HashMap<ProposalId, UpgradeProposal>,
    /// Deployment records
    deployments: HashMap<String, UpgradeDeployment>,
    /// Current system version
    current_version: String,
    /// Version history
    version_history: Vec<VersionRecord>,
    /// Deployment counter
    deployment_counter: u64,
}

/// Version record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    /// Version identifier
    pub version: String,
    /// When deployed
    pub deployed_at: DateTime<Utc>,
    /// Associated proposal
    pub proposal_id: Option<ProposalId>,
    /// Deployment duration
    pub deployment_duration: chrono::Duration,
    /// Whether it was rolled back
    pub rolled_back: bool,
    /// Rollback reason if applicable
    pub rollback_reason: Option<String>,
}

impl UpgradeManager {
    /// Create a new upgrade manager
    pub fn new(config: UpgradeConfig, initial_version: String) -> Self {
        let initial_record = VersionRecord {
            version: initial_version.clone(),
            deployed_at: Utc::now(),
            proposal_id: None,
            deployment_duration: chrono::Duration::zero(),
            rolled_back: false,
            rollback_reason: None,
        };

        Self {
            config,
            proposals: HashMap::new(),
            deployments: HashMap::new(),
            current_version: initial_version,
            version_history: vec![initial_record],
            deployment_counter: 0,
        }
    }

    /// Create an upgrade proposal
    pub async fn create_upgrade_proposal(
        &mut self,
        upgrade_type: UpgradeType,
        title: String,
        description: String,
        technical_spec: String,
        implementation_details: String,
        testing_plan: String,
        migration_plan: Option<String>,
        rollback_plan: String,
        risk_assessment: String,
        compatibility_assessment: String,
        proposed_by: String,
        target_deployment: Option<DateTime<Utc>>,
        priority: UpgradePriority,
        artifacts: Vec<UpgradeArtifact>,
    ) -> GovernanceResult<ProposalId> {
        let proposal_id = Uuid::new_v4();

        // Validate proposal based on type and priority
        self.validate_upgrade_proposal(&upgrade_type, &priority)?;

        let proposal = UpgradeProposal {
            id: proposal_id,
            upgrade_type,
            title,
            description,
            technical_spec,
            implementation_details,
            testing_plan,
            migration_plan,
            rollback_plan,
            risk_assessment,
            compatibility_assessment,
            proposed_by,
            proposed_at: Utc::now(),
            status: UpgradeStatus::UnderReview,
            target_deployment,
            deployed_at: None,
            priority,
            code_hash: None,
            artifacts,
        };

        self.proposals.insert(proposal_id, proposal);
        Ok(proposal_id)
    }

    /// Update proposal status
    pub async fn update_proposal_status(
        &mut self,
        proposal_id: &ProposalId,
        new_status: UpgradeStatus,
        updated_by: String,
    ) -> GovernanceResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::UpgradeNotFound {
                id: proposal_id.to_string(),
            })?;

        // Validate status transition
        self.validate_status_transition(&proposal.status, &new_status)?;

        proposal.status = new_status;

        // Update deployment time if deployed
        if proposal.status == UpgradeStatus::Deployed {
            proposal.deployed_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Deploy an approved upgrade
    pub async fn deploy_upgrade(
        &mut self,
        proposal_id: &ProposalId,
        strategy: DeploymentStrategy,
        deployed_by: String,
    ) -> GovernanceResult<String> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| GovernanceError::UpgradeNotFound {
                id: proposal_id.to_string(),
            })?;

        if proposal.status != UpgradeStatus::Approved {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Proposal status is {:?}, expected Approved", proposal.status),
            });
        }

        // Check if emergency upgrade has expedited approval
        if matches!(proposal.priority, UpgradePriority::Emergency) {
            // Emergency upgrades can be deployed immediately
        } else {
            // Regular upgrades need review period
            let review_duration = Utc::now() - proposal.proposed_at;
            if review_duration < self.config.min_review_period {
                return Err(GovernanceError::OperationFailed {
                    reason: format!(
                        "Minimum review period not met: {} < {} days",
                        review_duration.num_days(),
                        self.config.min_review_period.num_days()
                    ),
                });
            }
        }

        // Create deployment record
        self.deployment_counter += 1;
        let deployment_id = format!("deploy_{}", self.deployment_counter);

        let deployment = UpgradeDeployment {
            id: deployment_id.clone(),
            proposal_id: *proposal_id,
            strategy,
            started_at: Utc::now(),
            completed_at: None,
            status: DeploymentStatus::InProgress,
            deployed_by,
            phases: Vec::new(),
            error_message: None,
            rollback_info: None,
        };

        // Update proposal status
        proposal.status = UpgradeStatus::Deploying;

        self.deployments.insert(deployment_id.clone(), deployment);
        Ok(deployment_id)
    }

    /// Complete a deployment
    pub async fn complete_deployment(
        &mut self,
        deployment_id: &str,
        success: bool,
        error_message: Option<String>,
    ) -> GovernanceResult<()> {
        let deployment = self.deployments.get_mut(deployment_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Deployment {} not found", deployment_id),
            })?;

        let proposal_id = deployment.proposal_id;
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| GovernanceError::UpgradeNotFound {
                id: proposal_id.to_string(),
            })?;

        if success {
            deployment.status = DeploymentStatus::Completed;
            deployment.completed_at = Some(Utc::now());
            proposal.status = UpgradeStatus::Deployed;
            proposal.deployed_at = Some(Utc::now());

            // Update current version
            self.update_current_version(&proposal.upgrade_type, proposal_id)?;
        } else {
            deployment.status = DeploymentStatus::Failed;
            deployment.error_message = error_message;
            proposal.status = UpgradeStatus::DeploymentFailed;
        }

        Ok(())
    }

    /// Initiate rollback of a deployment
    pub async fn initiate_rollback(
        &mut self,
        deployment_id: &str,
        reason: String,
        initiated_by: String,
    ) -> GovernanceResult<String> {
        let deployment = self.deployments.get_mut(deployment_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Deployment {} not found", deployment_id),
            })?;

        if deployment.status != DeploymentStatus::Completed {
            return Err(GovernanceError::OperationFailed {
                reason: "Can only rollback completed deployments".to_string(),
            });
        }

        let rollback_id = format!("rollback_{}", deployment_id);

        let rollback_info = RollbackInfo {
            id: rollback_id.clone(),
            initiated_at: Utc::now(),
            completed_at: None,
            reason,
            initiated_by,
            status: DeploymentStatus::InProgress,
        };

        deployment.rollback_info = Some(rollback_info);
        deployment.status = DeploymentStatus::RolledBack;

        // Update proposal status
        let proposal_id = deployment.proposal_id;
        if let Some(proposal) = self.proposals.get_mut(&proposal_id) {
            proposal.status = UpgradeStatus::RolledBack;
        }

        Ok(rollback_id)
    }

    /// Get upgrade proposal
    pub fn get_proposal(&self, proposal_id: &ProposalId) -> Option<&UpgradeProposal> {
        self.proposals.get(proposal_id)
    }

    /// Get proposals by status
    pub fn get_proposals_by_status(&self, status: &UpgradeStatus) -> Vec<&UpgradeProposal> {
        self.proposals.values()
            .filter(|p| p.status == *status)
            .collect()
    }

    /// Get proposals by priority
    pub fn get_proposals_by_priority(&self, priority: &UpgradePriority) -> Vec<&UpgradeProposal> {
        self.proposals.values()
            .filter(|p| p.priority == *priority)
            .collect()
    }

    /// Get deployment record
    pub fn get_deployment(&self, deployment_id: &str) -> Option<&UpgradeDeployment> {
        self.deployments.get(deployment_id)
    }

    /// Get current system version
    pub fn get_current_version(&self) -> &str {
        &self.current_version
    }

    /// Get version history
    pub fn get_version_history(&self) -> &[VersionRecord] {
        &self.version_history
    }

    /// Get upgrade statistics
    pub fn get_upgrade_statistics(&self) -> UpgradeStatistics {
        let mut stats = UpgradeStatistics::default();

        stats.total_proposals = self.proposals.len();
        stats.total_deployments = self.deployments.len();

        // Count by status
        for proposal in self.proposals.values() {
            match proposal.status {
                UpgradeStatus::UnderReview => stats.under_review += 1,
                UpgradeStatus::Testing => stats.testing += 1,
                UpgradeStatus::ReadyForVote => stats.ready_for_vote += 1,
                UpgradeStatus::Voting => stats.voting += 1,
                UpgradeStatus::Approved => stats.approved += 1,
                UpgradeStatus::Deploying => stats.deploying += 1,
                UpgradeStatus::Deployed => stats.deployed += 1,
                UpgradeStatus::DeploymentFailed => stats.deployment_failed += 1,
                UpgradeStatus::Rejected => stats.rejected += 1,
                UpgradeStatus::Cancelled => stats.cancelled += 1,
                UpgradeStatus::RolledBack => stats.rolled_back += 1,
            }

            match proposal.priority {
                UpgradePriority::Low => stats.low_priority += 1,
                UpgradePriority::Medium => stats.medium_priority += 1,
                UpgradePriority::High => stats.high_priority += 1,
                UpgradePriority::Critical => stats.critical_priority += 1,
                UpgradePriority::Emergency => stats.emergency_priority += 1,
            }
        }

        // Calculate success rate
        let total_attempted = stats.deployed + stats.deployment_failed + stats.rolled_back;
        if total_attempted > 0 {
            stats.success_rate = stats.deployed as f64 / total_attempted as f64;
        }

        stats.total_versions = self.version_history.len();

        stats
    }

    // Helper methods

    fn validate_upgrade_proposal(
        &self,
        upgrade_type: &UpgradeType,
        priority: &UpgradePriority,
    ) -> GovernanceResult<()> {
        // Validate based on upgrade type and priority
        match (upgrade_type, priority) {
            (UpgradeType::Emergency { .. }, UpgradePriority::Emergency) => {
                // Emergency upgrades must have emergency priority
                Ok(())
            }
            (UpgradeType::Security { severity, .. }, _) => {
                match severity {
                    SecuritySeverity::Critical => {
                        if !matches!(priority, UpgradePriority::Critical | UpgradePriority::Emergency) {
                            return Err(GovernanceError::OperationFailed {
                                reason: "Critical security fixes must have Critical or Emergency priority".to_string(),
                            });
                        }
                    }
                    SecuritySeverity::High => {
                        if matches!(priority, UpgradePriority::Low) {
                            return Err(GovernanceError::OperationFailed {
                                reason: "High severity security fixes cannot have Low priority".to_string(),
                            });
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn validate_status_transition(
        &self,
        from: &UpgradeStatus,
        to: &UpgradeStatus,
    ) -> GovernanceResult<()> {
        let valid = match (from, to) {
            (UpgradeStatus::UnderReview, UpgradeStatus::Testing) => true,
            (UpgradeStatus::Testing, UpgradeStatus::ReadyForVote) => true,
            (UpgradeStatus::ReadyForVote, UpgradeStatus::Voting) => true,
            (UpgradeStatus::Voting, UpgradeStatus::Approved) => true,
            (UpgradeStatus::Voting, UpgradeStatus::Rejected) => true,
            (UpgradeStatus::Approved, UpgradeStatus::Deploying) => true,
            (UpgradeStatus::Deploying, UpgradeStatus::Deployed) => true,
            (UpgradeStatus::Deploying, UpgradeStatus::DeploymentFailed) => true,
            (UpgradeStatus::Deployed, UpgradeStatus::RolledBack) => true,
            (_, UpgradeStatus::Cancelled) => true, // Can always cancel
            _ => false,
        };

        if !valid {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Invalid status transition: {:?} -> {:?}", from, to),
            });
        }

        Ok(())
    }

    fn update_current_version(
        &mut self,
        upgrade_type: &UpgradeType,
        proposal_id: ProposalId,
    ) -> GovernanceResult<()> {
        let new_version = match upgrade_type {
            UpgradeType::Minor { version, .. } |
            UpgradeType::Major { version, .. } => version.clone(),
            UpgradeType::Security { patch_id, .. } => {
                format!("{}-security-{}", self.current_version, patch_id)
            }
            UpgradeType::Emergency { hotfix_id, .. } => {
                format!("{}-hotfix-{}", self.current_version, hotfix_id)
            }
            _ => return Ok(()), // No version change for other types
        };

        let old_version = self.current_version.clone();
        self.current_version = new_version.clone();

        // Add to version history
        let deployment_duration = chrono::Duration::zero(); // Would be calculated in real implementation
        let version_record = VersionRecord {
            version: new_version,
            deployed_at: Utc::now(),
            proposal_id: Some(proposal_id),
            deployment_duration,
            rolled_back: false,
            rollback_reason: None,
        };

        self.version_history.push(version_record);

        Ok(())
    }
}

/// Statistics for upgrade management
#[derive(Debug, Default, Clone)]
pub struct UpgradeStatistics {
    pub total_proposals: usize,
    pub total_deployments: usize,
    pub under_review: usize,
    pub testing: usize,
    pub ready_for_vote: usize,
    pub voting: usize,
    pub approved: usize,
    pub deploying: usize,
    pub deployed: usize,
    pub deployment_failed: usize,
    pub rejected: usize,
    pub cancelled: usize,
    pub rolled_back: usize,
    pub low_priority: usize,
    pub medium_priority: usize,
    pub high_priority: usize,
    pub critical_priority: usize,
    pub emergency_priority: usize,
    pub success_rate: f64,
    pub total_versions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_upgrade_proposal() {
        let mut manager = UpgradeManager::new(UpgradeConfig::default(), "1.0.0".to_string());

        let proposal_id = manager.create_upgrade_proposal(
            UpgradeType::Minor {
                version: "1.1.0".to_string(),
                changes: vec!["Bug fixes".to_string(), "Performance improvements".to_string()],
            },
            "Minor Version Update".to_string(),
            "Update to version 1.1.0 with bug fixes".to_string(),
            "Technical details...".to_string(),
            "Implementation plan...".to_string(),
            "Testing strategy...".to_string(),
            None,
            "Rollback procedure...".to_string(),
            "Low risk upgrade".to_string(),
            "Backward compatible".to_string(),
            "developer".to_string(),
            None,
            UpgradePriority::Medium,
            vec![],
        ).await.unwrap();

        let proposal = manager.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, UpgradeStatus::UnderReview);
        assert_eq!(proposal.priority, UpgradePriority::Medium);

        if let UpgradeType::Minor { version, .. } = &proposal.upgrade_type {
            assert_eq!(version, "1.1.0");
        } else {
            panic!("Expected Minor upgrade type");
        }
    }

    #[tokio::test]
    async fn test_status_transitions() {
        let mut manager = UpgradeManager::new(UpgradeConfig::default(), "1.0.0".to_string());

        let proposal_id = manager.create_upgrade_proposal(
            UpgradeType::Minor {
                version: "1.1.0".to_string(),
                changes: vec!["Bug fixes".to_string()],
            },
            "Test Upgrade".to_string(),
            "Description".to_string(),
            "Spec".to_string(),
            "Implementation".to_string(),
            "Testing".to_string(),
            None,
            "Rollback".to_string(),
            "Risk".to_string(),
            "Compatibility".to_string(),
            "developer".to_string(),
            None,
            UpgradePriority::Medium,
            vec![],
        ).await.unwrap();

        // Test valid transitions
        manager.update_proposal_status(&proposal_id, UpgradeStatus::Testing, "reviewer".to_string()).await.unwrap();
        manager.update_proposal_status(&proposal_id, UpgradeStatus::ReadyForVote, "reviewer".to_string()).await.unwrap();
        manager.update_proposal_status(&proposal_id, UpgradeStatus::Voting, "governance".to_string()).await.unwrap();
        manager.update_proposal_status(&proposal_id, UpgradeStatus::Approved, "governance".to_string()).await.unwrap();

        let proposal = manager.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, UpgradeStatus::Approved);
    }

    #[tokio::test]
    async fn test_security_upgrade_validation() {
        let mut manager = UpgradeManager::new(UpgradeConfig::default(), "1.0.0".to_string());

        // Critical security fix should require high priority
        let result = manager.create_upgrade_proposal(
            UpgradeType::Security {
                patch_id: "SEC-001".to_string(),
                vulnerabilities_fixed: vec!["Buffer overflow".to_string()],
                severity: SecuritySeverity::Critical,
            },
            "Critical Security Fix".to_string(),
            "Fix critical vulnerability".to_string(),
            "Technical details...".to_string(),
            "Implementation...".to_string(),
            "Testing...".to_string(),
            None,
            "Rollback...".to_string(),
            "High risk if not fixed".to_string(),
            "Backward compatible".to_string(),
            "security_team".to_string(),
            None,
            UpgradePriority::Low, // This should fail
            vec![],
        ).await;

        assert!(result.is_err());

        // Same upgrade with correct priority should succeed
        let proposal_id = manager.create_upgrade_proposal(
            UpgradeType::Security {
                patch_id: "SEC-001".to_string(),
                vulnerabilities_fixed: vec!["Buffer overflow".to_string()],
                severity: SecuritySeverity::Critical,
            },
            "Critical Security Fix".to_string(),
            "Fix critical vulnerability".to_string(),
            "Technical details...".to_string(),
            "Implementation...".to_string(),
            "Testing...".to_string(),
            None,
            "Rollback...".to_string(),
            "High risk if not fixed".to_string(),
            "Backward compatible".to_string(),
            "security_team".to_string(),
            None,
            UpgradePriority::Critical,
            vec![],
        ).await.unwrap();

        let proposal = manager.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.priority, UpgradePriority::Critical);
    }

    #[tokio::test]
    async fn test_deployment_lifecycle() {
        let mut manager = UpgradeManager::new(UpgradeConfig::default(), "1.0.0".to_string());

        let proposal_id = manager.create_upgrade_proposal(
            UpgradeType::Minor {
                version: "1.1.0".to_string(),
                changes: vec!["Bug fixes".to_string()],
            },
            "Test Upgrade".to_string(),
            "Description".to_string(),
            "Spec".to_string(),
            "Implementation".to_string(),
            "Testing".to_string(),
            None,
            "Rollback".to_string(),
            "Risk".to_string(),
            "Compatibility".to_string(),
            "developer".to_string(),
            None,
            UpgradePriority::Medium,
            vec![],
        ).await.unwrap();

        // Move to approved status
        manager.update_proposal_status(&proposal_id, UpgradeStatus::Approved, "governance".to_string()).await.unwrap();

        // Deploy upgrade
        let deployment_id = manager.deploy_upgrade(
            &proposal_id,
            DeploymentStrategy::BigBang,
            "deployer".to_string(),
        ).await.unwrap();

        let deployment = manager.get_deployment(&deployment_id).unwrap();
        assert_eq!(deployment.status, DeploymentStatus::InProgress);

        // Complete deployment successfully
        manager.complete_deployment(&deployment_id, true, None).await.unwrap();

        let proposal = manager.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, UpgradeStatus::Deployed);
        assert_eq!(manager.get_current_version(), "1.1.0");
    }

    #[tokio::test]
    async fn test_rollback() {
        let mut manager = UpgradeManager::new(UpgradeConfig::default(), "1.0.0".to_string());

        let proposal_id = manager.create_upgrade_proposal(
            UpgradeType::Minor {
                version: "1.1.0".to_string(),
                changes: vec!["Bug fixes".to_string()],
            },
            "Test Upgrade".to_string(),
            "Description".to_string(),
            "Spec".to_string(),
            "Implementation".to_string(),
            "Testing".to_string(),
            None,
            "Rollback".to_string(),
            "Risk".to_string(),
            "Compatibility".to_string(),
            "developer".to_string(),
            None,
            UpgradePriority::Medium,
            vec![],
        ).await.unwrap();

        // Simulate successful deployment
        manager.update_proposal_status(&proposal_id, UpgradeStatus::Approved, "governance".to_string()).await.unwrap();
        let deployment_id = manager.deploy_upgrade(&proposal_id, DeploymentStrategy::BigBang, "deployer".to_string()).await.unwrap();
        manager.complete_deployment(&deployment_id, true, None).await.unwrap();

        // Initiate rollback
        let rollback_id = manager.initiate_rollback(
            &deployment_id,
            "Critical issue discovered".to_string(),
            "ops_team".to_string(),
        ).await.unwrap();

        assert!(!rollback_id.is_empty());

        let deployment = manager.get_deployment(&deployment_id).unwrap();
        assert_eq!(deployment.status, DeploymentStatus::RolledBack);
        assert!(deployment.rollback_info.is_some());
    }

    #[tokio::test]
    async fn test_upgrade_statistics() {
        let mut manager = UpgradeManager::new(UpgradeConfig::default(), "1.0.0".to_string());

        // Create various proposals
        for i in 0..5 {
            manager.create_upgrade_proposal(
                UpgradeType::Minor {
                    version: format!("1.{}.0", i + 1),
                    changes: vec!["Changes".to_string()],
                },
                format!("Upgrade {}", i + 1),
                "Description".to_string(),
                "Spec".to_string(),
                "Implementation".to_string(),
                "Testing".to_string(),
                None,
                "Rollback".to_string(),
                "Risk".to_string(),
                "Compatibility".to_string(),
                "developer".to_string(),
                None,
                UpgradePriority::Medium,
                vec![],
            ).await.unwrap();
        }

        let stats = manager.get_upgrade_statistics();
        assert_eq!(stats.total_proposals, 5);
        assert_eq!(stats.under_review, 5);
        assert_eq!(stats.medium_priority, 5);
        assert_eq!(stats.total_versions, 1); // Only initial version
    }
}