//! Delegation system for governance voting

use crate::error::{GovernanceError, GovernanceResult};
use crate::voting::VotingPower;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// A delegation relationship between two participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// The delegator (person giving their voting power)
    pub delegator: String,
    /// The delegate (person receiving voting power)
    pub delegate: String,
    /// Amount of voting power delegated
    pub power: DelegationPower,
    /// When the delegation was created
    pub created_at: DateTime<Utc>,
    /// When the delegation expires (if any)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the delegation is active
    pub active: bool,
    /// Delegation scope
    pub scope: DelegationScope,
    /// Metadata for additional information
    pub metadata: HashMap<String, String>,
}

/// Amount of voting power that can be delegated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationPower {
    /// Fixed amount of power
    Fixed(VotingPower),
    /// Percentage of delegator's total power
    Percentage(u8),
    /// All voting power
    All,
}

/// Scope of delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationScope {
    /// All proposals
    All,
    /// Specific proposal types
    ProposalTypes(Vec<String>),
    /// Specific proposals
    Proposals(Vec<String>),
    /// Proposals with specific tags
    Tags(Vec<String>),
}

/// Target for delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationTarget {
    /// Specific address
    Address(String),
    /// Multiple addresses with weights
    WeightedAddresses(Vec<(String, u8)>),
    /// Auto-delegate to top validators
    TopValidators(usize),
}

/// Configuration for delegation system
#[derive(Debug, Clone)]
pub struct DelegationConfig {
    /// Maximum delegation chain length
    pub max_delegation_chain: usize,
    /// Whether to allow self-delegation
    pub allow_self_delegation: bool,
    /// Maximum delegation percentage
    pub max_delegation_percentage: u8,
    /// Minimum delegation amount
    pub min_delegation_amount: VotingPower,
    /// Default delegation duration
    pub default_duration: chrono::Duration,
    /// Maximum delegation duration
    pub max_duration: chrono::Duration,
}

impl Default for DelegationConfig {
    fn default() -> Self {
        Self {
            max_delegation_chain: 5,
            allow_self_delegation: false,
            max_delegation_percentage: 100,
            min_delegation_amount: 1,
            default_duration: chrono::Duration::days(365), // 1 year
            max_duration: chrono::Duration::days(365 * 4), // 4 years
        }
    }
}

/// Delegation system manages all delegation operations
pub struct DelegationSystem {
    /// Configuration
    config: DelegationConfig,
    /// Active delegations by delegator
    delegations: HashMap<String, Vec<Delegation>>,
    /// Reverse index: delegate -> delegators
    delegate_index: HashMap<String, Vec<String>>,
    /// Cached delegation chains
    chain_cache: HashMap<String, Vec<String>>,
    /// Total delegated power by address
    delegated_power: HashMap<String, VotingPower>,
    /// Base voting power (non-delegated)
    base_power: HashMap<String, VotingPower>,
}

impl DelegationSystem {
    /// Create a new delegation system
    pub fn new(config: DelegationConfig) -> Self {
        Self {
            config,
            delegations: HashMap::new(),
            delegate_index: HashMap::new(),
            chain_cache: HashMap::new(),
            delegated_power: HashMap::new(),
            base_power: HashMap::new(),
        }
    }

    /// Set base voting power for an address
    pub async fn set_base_power(
        &mut self,
        address: String,
        power: VotingPower,
    ) -> GovernanceResult<()> {
        self.base_power.insert(address.clone(), power);

        // Recalculate delegated power
        self.recalculate_delegated_power(&address).await?;

        // Invalidate chain cache
        self.chain_cache.clear();

        Ok(())
    }

    /// Create a delegation
    pub async fn create_delegation(
        &mut self,
        delegator: String,
        delegate: String,
        power: DelegationPower,
        scope: DelegationScope,
        duration: Option<chrono::Duration>,
        metadata: HashMap<String, String>,
    ) -> GovernanceResult<()> {
        // Validate delegation
        self.validate_delegation(&delegator, &delegate, &power)?;

        // Check for circular delegation
        if self.would_create_circular_delegation(&delegator, &delegate)? {
            return Err(GovernanceError::CircularDelegationDetected);
        }

        let now = Utc::now();
        let duration = duration.unwrap_or(self.config.default_duration);

        if duration > self.config.max_duration {
            return Err(GovernanceError::InvalidRange {
                min: "0".to_string(),
                max: format!("{} days", self.config.max_duration.num_days()),
            });
        }

        let expires_at = if duration.num_seconds() > 0 {
            Some(now + duration)
        } else {
            None
        };

        let delegation = Delegation {
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            power,
            created_at: now,
            expires_at,
            active: true,
            scope,
            metadata,
        };

        // Store delegation
        self.delegations
            .entry(delegator.clone())
            .or_insert_with(Vec::new)
            .push(delegation);

        // Update delegate index
        self.delegate_index
            .entry(delegate.clone())
            .or_insert_with(Vec::new)
            .push(delegator.clone());

        // Recalculate delegated power
        self.recalculate_delegated_power(&delegator).await?;
        self.recalculate_delegated_power(&delegate).await?;

        // Invalidate chain cache
        self.chain_cache.clear();

        Ok(())
    }

    /// Revoke a delegation
    pub async fn revoke_delegation(
        &mut self,
        delegator: &str,
        delegate: &str,
    ) -> GovernanceResult<()> {
        let delegations = self.delegations.get_mut(delegator)
            .ok_or_else(|| GovernanceError::DelegationNotFound {
                delegator: delegator.to_string(),
                delegate: delegate.to_string(),
            })?;

        // Find and remove delegation
        let index = delegations.iter().position(|d| d.delegate == delegate)
            .ok_or_else(|| GovernanceError::DelegationNotFound {
                delegator: delegator.to_string(),
                delegate: delegate.to_string(),
            })?;

        delegations.remove(index);

        // Update delegate index
        if let Some(delegators) = self.delegate_index.get_mut(delegate) {
            delegators.retain(|d| d != delegator);
        }

        // Recalculate delegated power
        self.recalculate_delegated_power(delegator).await?;
        self.recalculate_delegated_power(delegate).await?;

        // Invalidate chain cache
        self.chain_cache.clear();

        Ok(())
    }

    /// Get delegations by delegator
    pub fn get_delegations_by_delegator(&self, delegator: &str) -> Vec<&Delegation> {
        self.delegations
            .get(delegator)
            .map(|delegations| {
                delegations
                    .iter()
                    .filter(|d| d.active && !self.is_delegation_expired(d))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get delegations by delegate
    pub fn get_delegations_by_delegate(&self, delegate: &str) -> Vec<&Delegation> {
        self.delegate_index
            .get(delegate)
            .map(|delegators| {
                delegators
                    .iter()
                    .filter_map(|delegator| {
                        self.delegations.get(delegator)?.iter().find(|d| {
                            d.delegate == delegate && d.active && !self.is_delegation_expired(d)
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get effective voting power for an address (including delegated power)
    pub async fn get_effective_voting_power(&self, address: &str) -> VotingPower {
        let base = self.base_power.get(address).copied().unwrap_or(0);
        let delegated = self.delegated_power.get(address).copied().unwrap_or(0);
        base + delegated
    }

    /// Get delegated power for an address
    pub fn get_delegated_power(&self, address: &str) -> VotingPower {
        self.delegated_power.get(address).copied().unwrap_or(0)
    }

    /// Get base power for an address
    pub fn get_base_power(&self, address: &str) -> VotingPower {
        self.base_power.get(address).copied().unwrap_or(0)
    }

    /// Calculate delegated voting power for a specific proposal
    pub async fn calculate_delegated_power_for_proposal(
        &self,
        delegate: &str,
        proposal_type: &str,
        proposal_tags: &[String],
    ) -> VotingPower {
        let mut total_power = 0;

        if let Some(delegators) = self.delegate_index.get(delegate) {
            for delegator in delegators {
                if let Some(delegations) = self.delegations.get(delegator) {
                    for delegation in delegations {
                        if delegation.delegate == delegate &&
                           delegation.active &&
                           !self.is_delegation_expired(delegation) &&
                           self.delegation_applies_to_proposal(delegation, proposal_type, proposal_tags) {

                            let delegator_base_power = self.base_power.get(delegator).copied().unwrap_or(0);
                            total_power += self.calculate_delegation_power(&delegation.power, delegator_base_power);
                        }
                    }
                }
            }
        }

        total_power
    }

    /// Get delegation chain for an address
    pub async fn get_delegation_chain(&mut self, address: &str) -> GovernanceResult<Vec<String>> {
        if let Some(cached_chain) = self.chain_cache.get(address) {
            return Ok(cached_chain.clone());
        }

        let chain = self.build_delegation_chain(address)?;
        self.chain_cache.insert(address.to_string(), chain.clone());
        Ok(chain)
    }

    /// Get delegation statistics
    pub fn get_delegation_statistics(&self) -> DelegationStatistics {
        let mut stats = DelegationStatistics::default();

        for delegations in self.delegations.values() {
            for delegation in delegations {
                if delegation.active && !self.is_delegation_expired(delegation) {
                    stats.total_delegations += 1;

                    match delegation.scope {
                        DelegationScope::All => stats.all_scope += 1,
                        DelegationScope::ProposalTypes(_) => stats.type_scope += 1,
                        DelegationScope::Proposals(_) => stats.proposal_scope += 1,
                        DelegationScope::Tags(_) => stats.tag_scope += 1,
                    }

                    if delegation.expires_at.is_some() {
                        stats.temporary_delegations += 1;
                    } else {
                        stats.permanent_delegations += 1;
                    }
                }
            }
        }

        stats.unique_delegators = self.delegations.len();
        stats.unique_delegates = self.delegate_index.len();

        stats
    }

    /// Process expired delegations
    pub async fn process_expired_delegations(&mut self) -> GovernanceResult<Vec<(String, String)>> {
        let now = Utc::now();
        let mut expired = Vec::new();

        for (delegator, delegations) in &mut self.delegations {
            delegations.retain(|delegation| {
                if let Some(expires_at) = delegation.expires_at {
                    if now > expires_at {
                        expired.push((delegator.clone(), delegation.delegate.clone()));
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
        }

        // Clean up delegate index
        for (delegator, delegate) in &expired {
            if let Some(delegators) = self.delegate_index.get_mut(delegate) {
                delegators.retain(|d| d != delegator);
            }
        }

        // Recalculate delegated power for affected addresses
        let mut affected_addresses = HashSet::new();
        for (delegator, delegate) in &expired {
            affected_addresses.insert(delegator.clone());
            affected_addresses.insert(delegate.clone());
        }

        for address in affected_addresses {
            self.recalculate_delegated_power(&address).await?;
        }

        // Invalidate chain cache
        self.chain_cache.clear();

        Ok(expired)
    }

    /// Auto-delegate to top validators
    pub async fn auto_delegate_to_top_validators(
        &mut self,
        delegator: String,
        power: DelegationPower,
        validator_count: usize,
        validator_powers: Vec<(String, VotingPower)>,
    ) -> GovernanceResult<()> {
        // Sort validators by power and take top N
        let mut sorted_validators = validator_powers;
        sorted_validators.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_validators.truncate(validator_count);

        if sorted_validators.is_empty() {
            return Err(GovernanceError::ResourceNotAvailable {
                resource: "validators".to_string(),
            });
        }

        // Calculate delegation per validator (equal distribution)
        let delegated_power = match &power {
            DelegationPower::Fixed(amount) => *amount,
            DelegationPower::Percentage(pct) => {
                let base = self.base_power.get(&delegator).copied().unwrap_or(0);
                (base * (*pct as u64)) / 100
            }
            DelegationPower::All => {
                self.base_power.get(&delegator).copied().unwrap_or(0)
            }
        };

        let power_per_validator = delegated_power / validator_count as u64;

        // Create delegations to each validator
        for (validator, _) in sorted_validators {
            self.create_delegation(
                delegator.clone(),
                validator,
                DelegationPower::Fixed(power_per_validator),
                DelegationScope::All,
                None,
                HashMap::new(),
            ).await?;
        }

        Ok(())
    }

    // Helper methods

    async fn recalculate_delegated_power(&mut self, address: &str) -> GovernanceResult<()> {
        let mut total_delegated = 0;

        if let Some(delegators) = self.delegate_index.get(address) {
            for delegator in delegators {
                if let Some(delegations) = self.delegations.get(delegator) {
                    for delegation in delegations {
                        if delegation.delegate == address &&
                           delegation.active &&
                           !self.is_delegation_expired(delegation) {

                            let delegator_base_power = self.base_power.get(delegator).copied().unwrap_or(0);
                            total_delegated += self.calculate_delegation_power(&delegation.power, delegator_base_power);
                        }
                    }
                }
            }
        }

        self.delegated_power.insert(address.to_string(), total_delegated);
        Ok(())
    }

    fn validate_delegation(
        &self,
        delegator: &str,
        delegate: &str,
        power: &DelegationPower,
    ) -> GovernanceResult<()> {
        // Check self-delegation
        if !self.config.allow_self_delegation && delegator == delegate {
            return Err(GovernanceError::SelfDelegationNotAllowed);
        }

        // Check if delegator has sufficient power
        let delegator_base_power = self.base_power.get(delegator).copied().unwrap_or(0);

        match power {
            DelegationPower::Fixed(amount) => {
                if *amount < self.config.min_delegation_amount {
                    return Err(GovernanceError::InvalidTokenAmount {
                        amount: amount.to_string(),
                    });
                }
                if *amount > delegator_base_power {
                    return Err(GovernanceError::InsufficientTokenBalance {
                        required: *amount,
                        available: delegator_base_power,
                    });
                }
            }
            DelegationPower::Percentage(pct) => {
                if *pct > self.config.max_delegation_percentage {
                    return Err(GovernanceError::InvalidRange {
                        min: "0".to_string(),
                        max: self.config.max_delegation_percentage.to_string(),
                    });
                }
            }
            DelegationPower::All => {
                if delegator_base_power == 0 {
                    return Err(GovernanceError::InsufficientTokenBalance {
                        required: 1,
                        available: 0,
                    });
                }
            }
        }

        Ok(())
    }

    fn would_create_circular_delegation(&self, delegator: &str, delegate: &str) -> GovernanceResult<bool> {
        let chain = self.build_delegation_chain(delegate)?;
        Ok(chain.contains(&delegator.to_string()))
    }

    fn build_delegation_chain(&self, address: &str) -> GovernanceResult<Vec<String>> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(address.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                return Err(GovernanceError::CircularDelegationDetected);
            }

            if chain.len() >= self.config.max_delegation_chain {
                break;
            }

            visited.insert(current.clone());
            chain.push(current.clone());

            // Find active delegations from current address
            if let Some(delegations) = self.delegations.get(&current) {
                for delegation in delegations {
                    if delegation.active && !self.is_delegation_expired(delegation) {
                        queue.push_back(delegation.delegate.clone());
                    }
                }
            }
        }

        Ok(chain)
    }

    fn is_delegation_expired(&self, delegation: &Delegation) -> bool {
        if let Some(expires_at) = delegation.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    fn delegation_applies_to_proposal(
        &self,
        delegation: &Delegation,
        proposal_type: &str,
        proposal_tags: &[String],
    ) -> bool {
        match &delegation.scope {
            DelegationScope::All => true,
            DelegationScope::ProposalTypes(types) => types.contains(&proposal_type.to_string()),
            DelegationScope::Proposals(_) => false, // Would need proposal ID
            DelegationScope::Tags(tags) => {
                tags.iter().any(|tag| proposal_tags.contains(tag))
            }
        }
    }

    fn calculate_delegation_power(&self, power: &DelegationPower, base_power: VotingPower) -> VotingPower {
        match power {
            DelegationPower::Fixed(amount) => *amount,
            DelegationPower::Percentage(pct) => (base_power * (*pct as u64)) / 100,
            DelegationPower::All => base_power,
        }
    }
}

/// Statistics for delegation analysis
#[derive(Debug, Default, Clone)]
pub struct DelegationStatistics {
    pub total_delegations: usize,
    pub unique_delegators: usize,
    pub unique_delegates: usize,
    pub all_scope: usize,
    pub type_scope: usize,
    pub proposal_scope: usize,
    pub tag_scope: usize,
    pub permanent_delegations: usize,
    pub temporary_delegations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_delegation() {
        let mut delegation_system = DelegationSystem::new(DelegationConfig::default());

        delegation_system.set_base_power("delegator".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("delegate".to_string(), 500).await.unwrap();

        delegation_system.create_delegation(
            "delegator".to_string(),
            "delegate".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        let delegations = delegation_system.get_delegations_by_delegator("delegator");
        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].delegate, "delegate");

        let effective_power = delegation_system.get_effective_voting_power("delegate").await;
        assert_eq!(effective_power, 1000); // 500 base + 500 delegated
    }

    #[tokio::test]
    async fn test_revoke_delegation() {
        let mut delegation_system = DelegationSystem::new(DelegationConfig::default());

        delegation_system.set_base_power("delegator".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("delegate".to_string(), 500).await.unwrap();

        delegation_system.create_delegation(
            "delegator".to_string(),
            "delegate".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        delegation_system.revoke_delegation("delegator", "delegate").await.unwrap();

        let delegations = delegation_system.get_delegations_by_delegator("delegator");
        assert_eq!(delegations.len(), 0);

        let effective_power = delegation_system.get_effective_voting_power("delegate").await;
        assert_eq!(effective_power, 500); // Only base power
    }

    #[tokio::test]
    async fn test_circular_delegation_detection() {
        let mut delegation_system = DelegationSystem::new(DelegationConfig::default());

        delegation_system.set_base_power("a".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("b".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("c".to_string(), 1000).await.unwrap();

        // a -> b
        delegation_system.create_delegation(
            "a".to_string(),
            "b".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        // b -> c
        delegation_system.create_delegation(
            "b".to_string(),
            "c".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        // c -> a should fail (circular)
        let result = delegation_system.create_delegation(
            "c".to_string(),
            "a".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await;

        assert!(matches!(result, Err(GovernanceError::CircularDelegationDetected)));
    }

    #[tokio::test]
    async fn test_percentage_delegation() {
        let mut delegation_system = DelegationSystem::new(DelegationConfig::default());

        delegation_system.set_base_power("delegator".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("delegate".to_string(), 500).await.unwrap();

        delegation_system.create_delegation(
            "delegator".to_string(),
            "delegate".to_string(),
            DelegationPower::Percentage(50), // 50% of 1000 = 500
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        let effective_power = delegation_system.get_effective_voting_power("delegate").await;
        assert_eq!(effective_power, 1000); // 500 base + 500 delegated
    }

    #[tokio::test]
    async fn test_delegation_scopes() {
        let mut delegation_system = DelegationSystem::new(DelegationConfig::default());

        delegation_system.set_base_power("delegator".to_string(), 1000).await.unwrap();

        // Create delegation with specific scope
        delegation_system.create_delegation(
            "delegator".to_string(),
            "delegate".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::ProposalTypes(vec!["parameter_update".to_string()]),
            None,
            HashMap::new(),
        ).await.unwrap();

        // Should apply to parameter updates
        let power = delegation_system.calculate_delegated_power_for_proposal(
            "delegate",
            "parameter_update",
            &[],
        ).await;
        assert_eq!(power, 500);

        // Should not apply to treasury spend
        let power = delegation_system.calculate_delegated_power_for_proposal(
            "delegate",
            "treasury_spend",
            &[],
        ).await;
        assert_eq!(power, 0);
    }

    #[tokio::test]
    async fn test_delegation_chain() {
        let mut delegation_system = DelegationSystem::new(DelegationConfig::default());

        delegation_system.set_base_power("a".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("b".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("c".to_string(), 1000).await.unwrap();

        // a -> b -> c
        delegation_system.create_delegation(
            "a".to_string(),
            "b".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        delegation_system.create_delegation(
            "b".to_string(),
            "c".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        let chain = delegation_system.get_delegation_chain("a").await.unwrap();
        assert_eq!(chain, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    }

    #[tokio::test]
    async fn test_delegation_statistics() {
        let mut delegation_system = DelegationSystem::new(DelegationConfig::default());

        delegation_system.set_base_power("delegator1".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("delegator2".to_string(), 1000).await.unwrap();
        delegation_system.set_base_power("delegate".to_string(), 500).await.unwrap();

        delegation_system.create_delegation(
            "delegator1".to_string(),
            "delegate".to_string(),
            DelegationPower::Fixed(500),
            DelegationScope::All,
            None,
            HashMap::new(),
        ).await.unwrap();

        delegation_system.create_delegation(
            "delegator2".to_string(),
            "delegate".to_string(),
            DelegationPower::Fixed(300),
            DelegationScope::ProposalTypes(vec!["parameter_update".to_string()]),
            Some(chrono::Duration::days(30)),
            HashMap::new(),
        ).await.unwrap();

        let stats = delegation_system.get_delegation_statistics();
        assert_eq!(stats.total_delegations, 2);
        assert_eq!(stats.unique_delegators, 2);
        assert_eq!(stats.unique_delegates, 1);
        assert_eq!(stats.all_scope, 1);
        assert_eq!(stats.type_scope, 1);
        assert_eq!(stats.permanent_delegations, 1);
        assert_eq!(stats.temporary_delegations, 1);
    }
}