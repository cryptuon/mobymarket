//! Emergency governance system for critical situations

use crate::error::{GovernanceError, GovernanceResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Types of emergency actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyAction {
    /// Pause all system operations
    SystemPause {
        duration: Option<chrono::Duration>,
        reason: String,
    },
    /// Resume paused operations
    SystemResume {
        verification_required: bool,
    },
    /// Emergency parameter change
    ParameterOverride {
        parameter: String,
        old_value: String,
        new_value: String,
        duration: Option<chrono::Duration>,
    },
    /// Freeze specific accounts
    AccountFreeze {
        accounts: Vec<String>,
        reason: String,
    },
    /// Unfreeze accounts
    AccountUnfreeze {
        accounts: Vec<String>,
    },
    /// Emergency fund recovery
    FundRecovery {
        amount: u64,
        token: String,
        destination: String,
        justification: String,
    },
    /// Circuit breaker activation
    CircuitBreaker {
        component: String,
        trigger_condition: String,
    },
    /// Emergency contract upgrade
    EmergencyUpgrade {
        contract: String,
        new_implementation: String,
        bypass_governance: bool,
    },
    /// Force transaction reversal
    TransactionReversal {
        transaction_hash: String,
        reason: String,
    },
}

/// Emergency roles with different authorization levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyRole {
    /// Guardian with limited emergency powers
    Guardian,
    /// Emergency coordinator with broader powers
    EmergencyCoordinator,
    /// Security officer for security-related emergencies
    SecurityOfficer,
    /// Technical lead for technical emergencies
    TechnicalLead,
    /// Multi-sig committee member
    CommitteeMember,
}

/// Emergency authorization levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthorizationLevel {
    /// Single person can authorize
    Individual,
    /// Requires multiple people
    MultiSig { required: usize },
    /// Requires committee consensus
    Committee { threshold_percentage: u8 },
    /// Automatic based on conditions
    Automatic,
}

/// Emergency response record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyResponse {
    /// Response identifier
    pub id: String,
    /// Emergency action taken
    pub action: EmergencyAction,
    /// Who initiated the action
    pub initiated_by: String,
    /// Role of the initiator
    pub initiator_role: EmergencyRole,
    /// When action was initiated
    pub initiated_at: DateTime<Utc>,
    /// Authorization level required
    pub authorization_level: AuthorizationLevel,
    /// Current status
    pub status: EmergencyStatus,
    /// Authorizations received
    pub authorizations: Vec<EmergencyAuthorization>,
    /// When action was executed
    pub executed_at: Option<DateTime<Utc>>,
    /// When action expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Detailed justification
    pub justification: String,
    /// Risk assessment
    pub risk_assessment: String,
    /// Impact analysis
    pub impact_analysis: String,
    /// Evidence/supporting data
    pub evidence: Vec<String>,
    /// Post-action report
    pub post_action_report: Option<String>,
}

/// Status of emergency actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyStatus {
    /// Pending authorization
    PendingAuthorization,
    /// Authorized and ready for execution
    Authorized,
    /// Currently being executed
    Executing,
    /// Successfully executed
    Executed,
    /// Execution failed
    Failed,
    /// Action was rejected
    Rejected,
    /// Action was cancelled
    Cancelled,
    /// Action has expired
    Expired,
    /// Action was reversed
    Reversed,
}

/// Authorization for emergency actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAuthorization {
    /// Who provided authorization
    pub authorizer: String,
    /// Role of authorizer
    pub role: EmergencyRole,
    /// Authorization timestamp
    pub timestamp: DateTime<Utc>,
    /// Digital signature
    pub signature: String,
    /// Additional comments
    pub comments: Option<String>,
}

/// Emergency system state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemState {
    /// Normal operation
    Normal,
    /// System is paused
    Paused,
    /// Emergency mode active
    Emergency,
    /// Maintenance mode
    Maintenance,
    /// Recovery mode
    Recovery,
}

/// Emergency trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyTrigger {
    /// Trigger identifier
    pub id: String,
    /// Trigger name
    pub name: String,
    /// Condition description
    pub condition: String,
    /// Automatic action to take
    pub action: EmergencyAction,
    /// Whether trigger is active
    pub active: bool,
    /// Monitoring parameters
    pub parameters: HashMap<String, f64>,
    /// Threshold values
    pub thresholds: HashMap<String, f64>,
    /// Last evaluation time
    pub last_evaluated: DateTime<Utc>,
    /// Number of times triggered
    pub trigger_count: u64,
}

/// Configuration for emergency system
#[derive(Debug, Clone)]
pub struct EmergencyConfig {
    /// Maximum emergency response time
    pub max_response_time: chrono::Duration,
    /// Default authorization timeout
    pub authorization_timeout: chrono::Duration,
    /// Required signatures for multi-sig actions
    pub multi_sig_threshold: usize,
    /// Committee consensus threshold
    pub committee_threshold: u8,
    /// Maximum emergency action duration
    pub max_action_duration: chrono::Duration,
    /// Cooldown period between emergency actions
    pub cooldown_period: chrono::Duration,
}

impl Default for EmergencyConfig {
    fn default() -> Self {
        Self {
            max_response_time: chrono::Duration::minutes(15),
            authorization_timeout: chrono::Duration::hours(2),
            multi_sig_threshold: 2,
            committee_threshold: 67, // 67%
            max_action_duration: chrono::Duration::hours(24),
            cooldown_period: chrono::Duration::hours(1),
        }
    }
}

/// Emergency management system
pub struct EmergencySystem {
    /// Configuration
    config: EmergencyConfig,
    /// Current system state
    system_state: SystemState,
    /// Emergency responses
    responses: HashMap<String, EmergencyResponse>,
    /// Emergency personnel and their roles
    emergency_personnel: HashMap<String, EmergencyRole>,
    /// Active triggers
    triggers: HashMap<String, EmergencyTrigger>,
    /// Response counter for IDs
    response_counter: u64,
    /// Trigger counter for IDs
    trigger_counter: u64,
    /// Last emergency action timestamp
    last_emergency: Option<DateTime<Utc>>,
}

impl EmergencySystem {
    /// Create a new emergency system
    pub fn new(config: EmergencyConfig) -> Self {
        Self {
            config,
            system_state: SystemState::Normal,
            responses: HashMap::new(),
            emergency_personnel: HashMap::new(),
            triggers: HashMap::new(),
            response_counter: 0,
            trigger_counter: 0,
            last_emergency: None,
        }
    }

    /// Add emergency personnel
    pub async fn add_emergency_personnel(
        &mut self,
        address: String,
        role: EmergencyRole,
    ) -> GovernanceResult<()> {
        self.emergency_personnel.insert(address, role);
        Ok(())
    }

    /// Remove emergency personnel
    pub async fn remove_emergency_personnel(&mut self, address: &str) -> GovernanceResult<()> {
        self.emergency_personnel.remove(address);
        Ok(())
    }

    /// Initiate emergency action
    pub async fn initiate_emergency_action(
        &mut self,
        action: EmergencyAction,
        initiated_by: String,
        justification: String,
        risk_assessment: String,
        impact_analysis: String,
        evidence: Vec<String>,
    ) -> GovernanceResult<String> {
        // Check if initiator has emergency role
        let initiator_role = self.emergency_personnel.get(&initiated_by)
            .ok_or_else(|| GovernanceError::EmergencyActionNotAuthorized {
                action: "initiate emergency action".to_string(),
            })?;

        // Check cooldown period
        if let Some(last_emergency) = self.last_emergency {
            let time_since_last = Utc::now() - last_emergency;
            if time_since_last < self.config.cooldown_period {
                return Err(GovernanceError::OperationFailed {
                    reason: format!(
                        "Cooldown period not met: {} < {} minutes",
                        time_since_last.num_minutes(),
                        self.config.cooldown_period.num_minutes()
                    ),
                });
            }
        }

        // Validate action authorization
        let authorization_level = self.get_required_authorization(&action, initiator_role)?;

        self.response_counter += 1;
        let response_id = format!("emergency_{}", self.response_counter);

        let now = Utc::now();
        let expires_at = self.calculate_expiration(&action, now);

        let response = EmergencyResponse {
            id: response_id.clone(),
            action: action.clone(),
            initiated_by: initiated_by.clone(),
            initiator_role: initiator_role.clone(),
            initiated_at: now,
            authorization_level: authorization_level.clone(),
            status: EmergencyStatus::PendingAuthorization,
            authorizations: Vec::new(),
            executed_at: None,
            expires_at,
            justification,
            risk_assessment,
            impact_analysis,
            evidence,
            post_action_report: None,
        };

        // Check if can be auto-authorized
        if matches!(authorization_level, AuthorizationLevel::Individual) {
            // Individual can authorize their own action
            self.authorize_action(&response_id, initiated_by, "Self-authorization for individual action".to_string()).await?;
        }

        self.responses.insert(response_id.clone(), response);
        Ok(response_id)
    }

    /// Authorize an emergency action
    pub async fn authorize_action(
        &mut self,
        response_id: &str,
        authorizer: String,
        comments: String,
    ) -> GovernanceResult<()> {
        // Check if authorizer has emergency role
        let authorizer_role = self.emergency_personnel.get(&authorizer)
            .ok_or_else(|| GovernanceError::EmergencyActionNotAuthorized {
                action: "authorize emergency action".to_string(),
            })?;

        let response = self.responses.get_mut(response_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Emergency response {} not found", response_id),
            })?;

        if response.status != EmergencyStatus::PendingAuthorization {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Response status is {:?}, expected PendingAuthorization", response.status),
            });
        }

        // Check if already authorized by this person
        if response.authorizations.iter().any(|auth| auth.authorizer == authorizer) {
            return Err(GovernanceError::OperationFailed {
                reason: "Already authorized by this person".to_string(),
            });
        }

        // Add authorization
        let authorization = EmergencyAuthorization {
            authorizer: authorizer.clone(),
            role: authorizer_role.clone(),
            timestamp: Utc::now(),
            signature: format!("sig_{}", authorizer), // Simplified signature
            comments: Some(comments),
        };

        response.authorizations.push(authorization);

        // Check if sufficient authorization is received
        if self.is_sufficiently_authorized(response)? {
            response.status = EmergencyStatus::Authorized;

            // Execute immediately for critical actions
            if self.should_execute_immediately(&response.action) {
                self.execute_emergency_action(response_id).await?;
            }
        }

        Ok(())
    }

    /// Execute an authorized emergency action
    pub async fn execute_emergency_action(&mut self, response_id: &str) -> GovernanceResult<()> {
        let response = self.responses.get_mut(response_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Emergency response {} not found", response_id),
            })?;

        if response.status != EmergencyStatus::Authorized {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Response status is {:?}, expected Authorized", response.status),
            });
        }

        response.status = EmergencyStatus::Executing;

        // Execute the specific action
        let execution_result = self.execute_specific_action(&response.action).await;

        match execution_result {
            Ok(()) => {
                response.status = EmergencyStatus::Executed;
                response.executed_at = Some(Utc::now());
                self.last_emergency = Some(Utc::now());

                // Update system state if needed
                self.update_system_state_for_action(&response.action);
            }
            Err(e) => {
                response.status = EmergencyStatus::Failed;
                return Err(e);
            }
        }

        Ok(())
    }

    /// Reject an emergency action
    pub async fn reject_action(
        &mut self,
        response_id: &str,
        rejected_by: String,
        reason: String,
    ) -> GovernanceResult<()> {
        let _rejector_role = self.emergency_personnel.get(&rejected_by)
            .ok_or_else(|| GovernanceError::EmergencyActionNotAuthorized {
                action: "reject emergency action".to_string(),
            })?;

        let response = self.responses.get_mut(response_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Emergency response {} not found", response_id),
            })?;

        if !matches!(response.status, EmergencyStatus::PendingAuthorization | EmergencyStatus::Authorized) {
            return Err(GovernanceError::OperationFailed {
                reason: "Cannot reject executed or failed actions".to_string(),
            });
        }

        response.status = EmergencyStatus::Rejected;
        response.post_action_report = Some(format!("Rejected by {}: {}", rejected_by, reason));

        Ok(())
    }

    /// Add emergency trigger
    pub async fn add_trigger(
        &mut self,
        name: String,
        condition: String,
        action: EmergencyAction,
        parameters: HashMap<String, f64>,
        thresholds: HashMap<String, f64>,
    ) -> GovernanceResult<String> {
        self.trigger_counter += 1;
        let trigger_id = format!("trigger_{}", self.trigger_counter);

        let trigger = EmergencyTrigger {
            id: trigger_id.clone(),
            name,
            condition,
            action,
            active: true,
            parameters,
            thresholds,
            last_evaluated: Utc::now(),
            trigger_count: 0,
        };

        self.triggers.insert(trigger_id.clone(), trigger);
        Ok(trigger_id)
    }

    /// Evaluate triggers and execute automatic actions
    pub async fn evaluate_triggers(
        &mut self,
        current_metrics: HashMap<String, f64>,
    ) -> GovernanceResult<Vec<String>> {
        let mut triggered_actions = Vec::new();

        for trigger in self.triggers.values_mut() {
            if !trigger.active {
                continue;
            }

            trigger.last_evaluated = Utc::now();

            // Evaluate trigger condition
            if self.evaluate_trigger_condition(trigger, &current_metrics)? {
                trigger.trigger_count += 1;

                // Initiate automatic emergency action
                let response_id = self.initiate_emergency_action(
                    trigger.action.clone(),
                    "system".to_string(), // System-initiated
                    format!("Automatic trigger: {}", trigger.name),
                    "Automatically assessed based on trigger conditions".to_string(),
                    "System impact from trigger condition".to_string(),
                    vec![format!("Trigger: {} exceeded threshold", trigger.name)],
                ).await?;

                triggered_actions.push(response_id);
            }
        }

        Ok(triggered_actions)
    }

    /// Get current system state
    pub fn get_system_state(&self) -> &SystemState {
        &self.system_state
    }

    /// Get emergency response
    pub fn get_response(&self, response_id: &str) -> Option<&EmergencyResponse> {
        self.responses.get(response_id)
    }

    /// Get active emergency responses
    pub fn get_active_responses(&self) -> Vec<&EmergencyResponse> {
        self.responses.values()
            .filter(|r| matches!(r.status,
                EmergencyStatus::PendingAuthorization |
                EmergencyStatus::Authorized |
                EmergencyStatus::Executing |
                EmergencyStatus::Executed
            ))
            .collect()
    }

    /// Get emergency statistics
    pub fn get_emergency_statistics(&self) -> EmergencyStatistics {
        let mut stats = EmergencyStatistics::default();

        stats.total_responses = self.responses.len();
        stats.total_personnel = self.emergency_personnel.len();
        stats.total_triggers = self.triggers.len();

        for response in self.responses.values() {
            match response.status {
                EmergencyStatus::PendingAuthorization => stats.pending_authorization += 1,
                EmergencyStatus::Authorized => stats.authorized += 1,
                EmergencyStatus::Executing => stats.executing += 1,
                EmergencyStatus::Executed => stats.executed += 1,
                EmergencyStatus::Failed => stats.failed += 1,
                EmergencyStatus::Rejected => stats.rejected += 1,
                EmergencyStatus::Cancelled => stats.cancelled += 1,
                EmergencyStatus::Expired => stats.expired += 1,
                EmergencyStatus::Reversed => stats.reversed += 1,
            }
        }

        // Count by role
        for role in self.emergency_personnel.values() {
            match role {
                EmergencyRole::Guardian => stats.guardians += 1,
                EmergencyRole::EmergencyCoordinator => stats.coordinators += 1,
                EmergencyRole::SecurityOfficer => stats.security_officers += 1,
                EmergencyRole::TechnicalLead => stats.technical_leads += 1,
                EmergencyRole::CommitteeMember => stats.committee_members += 1,
            }
        }

        stats.active_triggers = self.triggers.values().filter(|t| t.active).count();

        // Calculate response time (simplified)
        let executed_responses: Vec<_> = self.responses.values()
            .filter(|r| r.status == EmergencyStatus::Executed)
            .collect();

        if !executed_responses.is_empty() {
            let total_response_time: i64 = executed_responses.iter()
                .filter_map(|r| r.executed_at.map(|exec| (exec - r.initiated_at).num_minutes()))
                .sum();

            stats.average_response_time_minutes = total_response_time as f64 / executed_responses.len() as f64;
        }

        stats
    }

    // Helper methods

    async fn execute_specific_action(&mut self, action: &EmergencyAction) -> GovernanceResult<()> {
        match action {
            EmergencyAction::SystemPause { duration: _, reason: _ } => {
                self.system_state = SystemState::Paused;
                Ok(())
            }
            EmergencyAction::SystemResume { verification_required: _ } => {
                if self.system_state == SystemState::Paused {
                    self.system_state = SystemState::Normal;
                }
                Ok(())
            }
            EmergencyAction::AccountFreeze { accounts: _, reason: _ } => {
                // Implementation would freeze the specified accounts
                Ok(())
            }
            EmergencyAction::AccountUnfreeze { accounts: _ } => {
                // Implementation would unfreeze the specified accounts
                Ok(())
            }
            EmergencyAction::CircuitBreaker { component: _, trigger_condition: _ } => {
                // Implementation would activate circuit breaker
                Ok(())
            }
            _ => {
                // Other actions would be implemented based on specific requirements
                Ok(())
            }
        }
    }

    fn get_required_authorization(
        &self,
        action: &EmergencyAction,
        initiator_role: &EmergencyRole,
    ) -> GovernanceResult<AuthorizationLevel> {
        let level = match action {
            EmergencyAction::SystemPause { .. } => {
                match initiator_role {
                    EmergencyRole::EmergencyCoordinator => AuthorizationLevel::Individual,
                    _ => AuthorizationLevel::MultiSig { required: self.config.multi_sig_threshold },
                }
            }
            EmergencyAction::SystemResume { .. } => {
                AuthorizationLevel::MultiSig { required: self.config.multi_sig_threshold }
            }
            EmergencyAction::FundRecovery { .. } => {
                AuthorizationLevel::Committee { threshold_percentage: self.config.committee_threshold }
            }
            EmergencyAction::EmergencyUpgrade { bypass_governance: true, .. } => {
                AuthorizationLevel::Committee { threshold_percentage: 75 } // Higher threshold for bypassing governance
            }
            _ => {
                AuthorizationLevel::MultiSig { required: self.config.multi_sig_threshold }
            }
        };

        Ok(level)
    }

    fn is_sufficiently_authorized(&self, response: &EmergencyResponse) -> GovernanceResult<bool> {
        match &response.authorization_level {
            AuthorizationLevel::Individual => Ok(response.authorizations.len() >= 1),
            AuthorizationLevel::MultiSig { required } => Ok(response.authorizations.len() >= *required),
            AuthorizationLevel::Committee { threshold_percentage } => {
                let committee_members = self.emergency_personnel.values()
                    .filter(|role| matches!(role, EmergencyRole::CommitteeMember))
                    .count();

                let committee_authorizations = response.authorizations.iter()
                    .filter(|auth| matches!(auth.role, EmergencyRole::CommitteeMember))
                    .count();

                let percentage = if committee_members > 0 {
                    (committee_authorizations * 100) / committee_members
                } else {
                    0
                };

                Ok(percentage >= *threshold_percentage as usize)
            }
            AuthorizationLevel::Automatic => Ok(true),
        }
    }

    fn should_execute_immediately(&self, action: &EmergencyAction) -> bool {
        matches!(action,
            EmergencyAction::SystemPause { .. } |
            EmergencyAction::CircuitBreaker { .. } |
            EmergencyAction::AccountFreeze { .. }
        )
    }

    fn calculate_expiration(&self, action: &EmergencyAction, start_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match action {
            EmergencyAction::SystemPause { duration: Some(duration), .. } => {
                Some(start_time + *duration)
            }
            EmergencyAction::ParameterOverride { duration: Some(duration), .. } => {
                Some(start_time + *duration)
            }
            _ => Some(start_time + self.config.max_action_duration),
        }
    }

    fn update_system_state_for_action(&mut self, action: &EmergencyAction) {
        match action {
            EmergencyAction::SystemPause { .. } => {
                self.system_state = SystemState::Paused;
            }
            EmergencyAction::SystemResume { .. } => {
                if self.system_state == SystemState::Paused {
                    self.system_state = SystemState::Normal;
                }
            }
            _ => {}
        }
    }

    fn evaluate_trigger_condition(
        &self,
        trigger: &EmergencyTrigger,
        current_metrics: &HashMap<String, f64>,
    ) -> GovernanceResult<bool> {
        // Simplified trigger evaluation
        for (param, threshold) in &trigger.thresholds {
            if let Some(current_value) = current_metrics.get(param) {
                if *current_value > *threshold {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

/// Statistics for emergency system analysis
#[derive(Debug, Default, Clone)]
pub struct EmergencyStatistics {
    pub total_responses: usize,
    pub total_personnel: usize,
    pub total_triggers: usize,
    pub pending_authorization: usize,
    pub authorized: usize,
    pub executing: usize,
    pub executed: usize,
    pub failed: usize,
    pub rejected: usize,
    pub cancelled: usize,
    pub expired: usize,
    pub reversed: usize,
    pub guardians: usize,
    pub coordinators: usize,
    pub security_officers: usize,
    pub technical_leads: usize,
    pub committee_members: usize,
    pub active_triggers: usize,
    pub average_response_time_minutes: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_emergency_personnel() {
        let mut emergency_system = EmergencySystem::new(EmergencyConfig::default());

        emergency_system.add_emergency_personnel(
            "guardian1".to_string(),
            EmergencyRole::Guardian,
        ).await.unwrap();

        emergency_system.add_emergency_personnel(
            "coordinator1".to_string(),
            EmergencyRole::EmergencyCoordinator,
        ).await.unwrap();

        assert_eq!(emergency_system.emergency_personnel.len(), 2);
        assert_eq!(
            emergency_system.emergency_personnel.get("guardian1"),
            Some(&EmergencyRole::Guardian)
        );
    }

    #[tokio::test]
    async fn test_initiate_emergency_action() {
        let mut emergency_system = EmergencySystem::new(EmergencyConfig::default());

        // Add emergency coordinator
        emergency_system.add_emergency_personnel(
            "coordinator1".to_string(),
            EmergencyRole::EmergencyCoordinator,
        ).await.unwrap();

        let response_id = emergency_system.initiate_emergency_action(
            EmergencyAction::SystemPause {
                duration: Some(chrono::Duration::hours(1)),
                reason: "Critical security issue".to_string(),
            },
            "coordinator1".to_string(),
            "System compromise detected".to_string(),
            "High risk of fund loss".to_string(),
            "Trading will be halted".to_string(),
            vec!["Security alert log".to_string()],
        ).await.unwrap();

        let response = emergency_system.get_response(&response_id).unwrap();
        assert_eq!(response.status, EmergencyStatus::Authorized); // Should be auto-authorized for coordinator
        assert_eq!(response.initiated_by, "coordinator1");
    }

    #[tokio::test]
    async fn test_multi_sig_authorization() {
        let mut emergency_system = EmergencySystem::new(EmergencyConfig {
            multi_sig_threshold: 2,
            ..EmergencyConfig::default()
        });

        // Add multiple personnel
        emergency_system.add_emergency_personnel(
            "guardian1".to_string(),
            EmergencyRole::Guardian,
        ).await.unwrap();

        emergency_system.add_emergency_personnel(
            "guardian2".to_string(),
            EmergencyRole::Guardian,
        ).await.unwrap();

        emergency_system.add_emergency_personnel(
            "guardian3".to_string(),
            EmergencyRole::Guardian,
        ).await.unwrap();

        // Initiate action requiring multi-sig
        let response_id = emergency_system.initiate_emergency_action(
            EmergencyAction::FundRecovery {
                amount: 1000000,
                token: "USDC".to_string(),
                destination: "recovery_address".to_string(),
                justification: "Hack recovery".to_string(),
            },
            "guardian1".to_string(),
            "Funds need to be recovered".to_string(),
            "Risk of total loss".to_string(),
            "1M USDC will be moved to safe address".to_string(),
            vec!["Hack evidence".to_string()],
        ).await.unwrap();

        let response = emergency_system.get_response(&response_id).unwrap();
        assert_eq!(response.status, EmergencyStatus::PendingAuthorization);

        // First authorization
        emergency_system.authorize_action(
            &response_id,
            "guardian2".to_string(),
            "I approve this recovery".to_string(),
        ).await.unwrap();

        let response = emergency_system.get_response(&response_id).unwrap();
        assert_eq!(response.status, EmergencyStatus::PendingAuthorization); // Still pending

        // Second authorization (should reach threshold)
        emergency_system.authorize_action(
            &response_id,
            "guardian3".to_string(),
            "Approved for recovery".to_string(),
        ).await.unwrap();

        let response = emergency_system.get_response(&response_id).unwrap();
        assert_eq!(response.status, EmergencyStatus::Authorized); // Now authorized
        assert_eq!(response.authorizations.len(), 2);
    }

    #[tokio::test]
    async fn test_system_pause_resume() {
        let mut emergency_system = EmergencySystem::new(EmergencyConfig::default());

        emergency_system.add_emergency_personnel(
            "coordinator1".to_string(),
            EmergencyRole::EmergencyCoordinator,
        ).await.unwrap();

        // Pause system
        let pause_id = emergency_system.initiate_emergency_action(
            EmergencyAction::SystemPause {
                duration: None,
                reason: "Emergency maintenance".to_string(),
            },
            "coordinator1".to_string(),
            "System needs immediate pause".to_string(),
            "Low risk".to_string(),
            "Brief trading halt".to_string(),
            vec![],
        ).await.unwrap();

        emergency_system.execute_emergency_action(&pause_id).await.unwrap();

        assert_eq!(emergency_system.get_system_state(), &SystemState::Paused);

        // Resume system
        let resume_id = emergency_system.initiate_emergency_action(
            EmergencyAction::SystemResume {
                verification_required: true,
            },
            "coordinator1".to_string(),
            "Maintenance complete".to_string(),
            "Low risk".to_string(),
            "Resume normal operations".to_string(),
            vec![],
        ).await.unwrap();

        // Need additional authorization for resume
        emergency_system.add_emergency_personnel(
            "guardian1".to_string(),
            EmergencyRole::Guardian,
        ).await.unwrap();

        emergency_system.authorize_action(
            &resume_id,
            "guardian1".to_string(),
            "Verified system is ready".to_string(),
        ).await.unwrap();

        emergency_system.execute_emergency_action(&resume_id).await.unwrap();

        assert_eq!(emergency_system.get_system_state(), &SystemState::Normal);
    }

    #[tokio::test]
    async fn test_emergency_triggers() {
        let mut emergency_system = EmergencySystem::new(EmergencyConfig::default());

        // Add system user for automatic actions
        emergency_system.add_emergency_personnel(
            "system".to_string(),
            EmergencyRole::EmergencyCoordinator,
        ).await.unwrap();

        // Add trigger for high transaction volume
        let _trigger_id = emergency_system.add_trigger(
            "High Volume Circuit Breaker".to_string(),
            "Transaction volume > 1000 per minute".to_string(),
            EmergencyAction::CircuitBreaker {
                component: "trading_engine".to_string(),
                trigger_condition: "volume_spike".to_string(),
            },
            HashMap::from([("volume".to_string(), 500.0)]),
            HashMap::from([("volume".to_string(), 1000.0)]),
        ).await.unwrap();

        // Simulate metrics that trigger the condition
        let current_metrics = HashMap::from([("volume".to_string(), 1500.0)]);

        let triggered_actions = emergency_system.evaluate_triggers(current_metrics).await.unwrap();

        assert_eq!(triggered_actions.len(), 1);

        let response = emergency_system.get_response(&triggered_actions[0]).unwrap();
        assert_eq!(response.initiated_by, "system");
        assert!(matches!(response.action, EmergencyAction::CircuitBreaker { .. }));
    }

    #[tokio::test]
    async fn test_emergency_statistics() {
        let mut emergency_system = EmergencySystem::new(EmergencyConfig::default());

        // Add personnel
        emergency_system.add_emergency_personnel("guardian1".to_string(), EmergencyRole::Guardian).await.unwrap();
        emergency_system.add_emergency_personnel("coordinator1".to_string(), EmergencyRole::EmergencyCoordinator).await.unwrap();
        emergency_system.add_emergency_personnel("security1".to_string(), EmergencyRole::SecurityOfficer).await.unwrap();

        // Create some responses
        let _response_id = emergency_system.initiate_emergency_action(
            EmergencyAction::SystemPause {
                duration: Some(chrono::Duration::hours(1)),
                reason: "Test".to_string(),
            },
            "coordinator1".to_string(),
            "Test action".to_string(),
            "Test risk".to_string(),
            "Test impact".to_string(),
            vec![],
        ).await.unwrap();

        let stats = emergency_system.get_emergency_statistics();

        assert_eq!(stats.total_personnel, 3);
        assert_eq!(stats.guardians, 1);
        assert_eq!(stats.coordinators, 1);
        assert_eq!(stats.security_officers, 1);
        assert_eq!(stats.total_responses, 1);
        assert_eq!(stats.authorized, 1); // Coordinator can self-authorize
    }
}