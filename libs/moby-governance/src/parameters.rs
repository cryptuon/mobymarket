//! Parameter management system for protocol governance

use crate::error::{GovernanceError, GovernanceResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Value types that parameters can hold
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Unsigned integer value
    UInteger(u64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<ParameterValue>),
    /// Object/map of key-value pairs
    Object(HashMap<String, ParameterValue>),
}

impl ParameterValue {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Integer(i) => i.to_string(),
            Self::UInteger(u) => u.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Self::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }

    /// Parse from string
    pub fn from_string(s: &str, value_type: &ParameterType) -> GovernanceResult<Self> {
        match value_type {
            ParameterType::String => Ok(Self::String(s.to_string())),
            ParameterType::Integer => {
                s.parse::<i64>()
                    .map(Self::Integer)
                    .map_err(|_| GovernanceError::InvalidParameterValue {
                        name: "unknown".to_string(),
                        value: s.to_string(),
                    })
            }
            ParameterType::UInteger => {
                s.parse::<u64>()
                    .map(Self::UInteger)
                    .map_err(|_| GovernanceError::InvalidParameterValue {
                        name: "unknown".to_string(),
                        value: s.to_string(),
                    })
            }
            ParameterType::Float => {
                s.parse::<f64>()
                    .map(Self::Float)
                    .map_err(|_| GovernanceError::InvalidParameterValue {
                        name: "unknown".to_string(),
                        value: s.to_string(),
                    })
            }
            ParameterType::Boolean => {
                s.parse::<bool>()
                    .map(Self::Boolean)
                    .map_err(|_| GovernanceError::InvalidParameterValue {
                        name: "unknown".to_string(),
                        value: s.to_string(),
                    })
            }
            _ => Err(GovernanceError::InvalidParameterValue {
                name: "unknown".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

/// Parameter type definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterType {
    String,
    Integer,
    UInteger,
    Float,
    Boolean,
    Array(Box<ParameterType>),
    Object,
}

/// Parameter constraints and validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraints {
    /// Minimum value (for numeric types)
    pub min_value: Option<f64>,
    /// Maximum value (for numeric types)
    pub max_value: Option<f64>,
    /// Allowed values (enum-like)
    pub allowed_values: Option<Vec<ParameterValue>>,
    /// Regular expression pattern (for strings)
    pub pattern: Option<String>,
    /// Minimum length (for strings/arrays)
    pub min_length: Option<usize>,
    /// Maximum length (for strings/arrays)
    pub max_length: Option<usize>,
    /// Custom validation function name
    pub custom_validator: Option<String>,
}

/// Protocol parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name/key
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type
    pub parameter_type: ParameterType,
    /// Current value
    pub current_value: ParameterValue,
    /// Default value
    pub default_value: ParameterValue,
    /// Value constraints
    pub constraints: Option<ParameterConstraints>,
    /// Whether parameter is mutable via governance
    pub mutable: bool,
    /// Whether parameter requires special permissions
    pub protected: bool,
    /// Category for organization
    pub category: String,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Who last updated it
    pub last_updated_by: Option<String>,
    /// Update history count
    pub update_count: u64,
}

/// Pending parameter update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterUpdate {
    /// Update identifier
    pub id: String,
    /// Parameter being updated
    pub parameter_name: String,
    /// Current value
    pub old_value: ParameterValue,
    /// Proposed new value
    pub new_value: ParameterValue,
    /// Who proposed the update
    pub proposed_by: String,
    /// When it was proposed
    pub proposed_at: DateTime<Utc>,
    /// When the timelock expires
    pub timelock_expires_at: DateTime<Utc>,
    /// Whether the update has been executed
    pub executed: bool,
    /// Execution timestamp
    pub executed_at: Option<DateTime<Utc>>,
    /// Justification for the change
    pub justification: String,
    /// Impact assessment
    pub impact_assessment: Option<String>,
}

/// Configuration for parameter management
#[derive(Debug, Clone)]
pub struct ParameterConfig {
    /// Default timelock duration for parameter updates
    pub default_timelock_duration: chrono::Duration,
    /// Minimum timelock duration
    pub min_timelock_duration: chrono::Duration,
    /// Maximum timelock duration
    pub max_timelock_duration: chrono::Duration,
    /// Whether to require impact assessment
    pub require_impact_assessment: bool,
    /// Maximum justification length
    pub max_justification_length: usize,
}

impl Default for ParameterConfig {
    fn default() -> Self {
        Self {
            default_timelock_duration: chrono::Duration::hours(24),
            min_timelock_duration: chrono::Duration::hours(1),
            max_timelock_duration: chrono::Duration::days(30),
            require_impact_assessment: true,
            max_justification_length: 2000,
        }
    }
}

/// Parameter manager handles protocol parameter updates
pub struct ParameterManager {
    /// Configuration
    config: ParameterConfig,
    /// All parameters
    parameters: HashMap<String, Parameter>,
    /// Pending updates
    pending_updates: HashMap<String, ParameterUpdate>,
    /// Parameter categories
    categories: HashMap<String, Vec<String>>,
    /// Update counter for IDs
    update_counter: u64,
}

impl ParameterManager {
    /// Create a new parameter manager
    pub fn new(config: ParameterConfig) -> Self {
        Self {
            config,
            parameters: HashMap::new(),
            pending_updates: HashMap::new(),
            categories: HashMap::new(),
            update_counter: 0,
        }
    }

    /// Register a new parameter
    pub async fn register_parameter(
        &mut self,
        name: String,
        description: String,
        parameter_type: ParameterType,
        default_value: ParameterValue,
        constraints: Option<ParameterConstraints>,
        mutable: bool,
        protected: bool,
        category: String,
        tags: Vec<String>,
    ) -> GovernanceResult<()> {
        if self.parameters.contains_key(&name) {
            return Err(GovernanceError::OperationFailed {
                reason: format!("Parameter {} already exists", name),
            });
        }

        // Validate default value against constraints
        self.validate_value(&default_value, &parameter_type, &constraints)?;

        let parameter = Parameter {
            name: name.clone(),
            description,
            parameter_type,
            current_value: default_value.clone(),
            default_value,
            constraints,
            mutable,
            protected,
            category: category.clone(),
            tags,
            last_updated: Utc::now(),
            last_updated_by: None,
            update_count: 0,
        };

        // Add to category index
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());

        self.parameters.insert(name, parameter);
        Ok(())
    }

    /// Propose a parameter update
    pub async fn propose_update(
        &mut self,
        parameter_name: String,
        new_value: ParameterValue,
        proposed_by: String,
        justification: String,
        impact_assessment: Option<String>,
        timelock_duration: Option<chrono::Duration>,
    ) -> GovernanceResult<String> {
        // Validate parameter exists
        let parameter = self.parameters.get(&parameter_name)
            .ok_or_else(|| GovernanceError::ParameterNotFound {
                name: parameter_name.clone(),
            })?;

        // Check if parameter is mutable
        if !parameter.mutable {
            return Err(GovernanceError::ParameterIsImmutable {
                name: parameter_name,
            });
        }

        // Validate new value
        self.validate_value(&new_value, &parameter.parameter_type, &parameter.constraints)?;

        // Validate justification
        if justification.len() > self.config.max_justification_length {
            return Err(GovernanceError::OperationFailed {
                reason: format!(
                    "Justification too long: {} > {}",
                    justification.len(),
                    self.config.max_justification_length
                ),
            });
        }

        // Check impact assessment requirement
        if self.config.require_impact_assessment && impact_assessment.is_none() {
            return Err(GovernanceError::OperationFailed {
                reason: "Impact assessment required".to_string(),
            });
        }

        // Calculate timelock
        let timelock_duration = timelock_duration.unwrap_or(self.config.default_timelock_duration);
        if timelock_duration < self.config.min_timelock_duration ||
           timelock_duration > self.config.max_timelock_duration {
            return Err(GovernanceError::InvalidRange {
                min: format!("{} hours", self.config.min_timelock_duration.num_hours()),
                max: format!("{} hours", self.config.max_timelock_duration.num_hours()),
            });
        }

        let now = Utc::now();
        self.update_counter += 1;
        let update_id = format!("param_update_{}", self.update_counter);

        let update = ParameterUpdate {
            id: update_id.clone(),
            parameter_name: parameter_name.clone(),
            old_value: parameter.current_value.clone(),
            new_value,
            proposed_by,
            proposed_at: now,
            timelock_expires_at: now + timelock_duration,
            executed: false,
            executed_at: None,
            justification,
            impact_assessment,
        };

        self.pending_updates.insert(update_id.clone(), update);
        Ok(update_id)
    }

    /// Execute a parameter update after timelock
    pub async fn execute_update(
        &mut self,
        update_id: &str,
        executed_by: String,
    ) -> GovernanceResult<()> {
        let update = self.pending_updates.get_mut(update_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Update {} not found", update_id),
            })?;

        if update.executed {
            return Err(GovernanceError::OperationFailed {
                reason: "Update already executed".to_string(),
            });
        }

        // Check timelock
        if Utc::now() < update.timelock_expires_at {
            return Err(GovernanceError::ParameterTimelockNotExpired);
        }

        // Get parameter
        let parameter = self.parameters.get_mut(&update.parameter_name)
            .ok_or_else(|| GovernanceError::ParameterNotFound {
                name: update.parameter_name.clone(),
            })?;

        // Apply update
        parameter.current_value = update.new_value.clone();
        parameter.last_updated = Utc::now();
        parameter.last_updated_by = Some(executed_by);
        parameter.update_count += 1;

        // Mark update as executed
        update.executed = true;
        update.executed_at = Some(Utc::now());

        Ok(())
    }

    /// Cancel a pending update
    pub async fn cancel_update(
        &mut self,
        update_id: &str,
        cancelled_by: &str,
    ) -> GovernanceResult<()> {
        let update = self.pending_updates.get(update_id)
            .ok_or_else(|| GovernanceError::OperationFailed {
                reason: format!("Update {} not found", update_id),
            })?;

        // Check authorization (simplified - could be more complex)
        if update.proposed_by != cancelled_by {
            return Err(GovernanceError::UnauthorizedAccess {
                action: "cancel parameter update".to_string(),
            });
        }

        if update.executed {
            return Err(GovernanceError::OperationFailed {
                reason: "Cannot cancel executed update".to_string(),
            });
        }

        self.pending_updates.remove(update_id);
        Ok(())
    }

    /// Get parameter value
    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.get(name)
    }

    /// Get parameter value
    pub fn get_parameter_value(&self, name: &str) -> Option<&ParameterValue> {
        self.parameters.get(name).map(|p| &p.current_value)
    }

    /// Get parameters by category
    pub fn get_parameters_by_category(&self, category: &str) -> Vec<&Parameter> {
        self.categories
            .get(category)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|name| self.parameters.get(name))
            .collect()
    }

    /// Get parameters by tag
    pub fn get_parameters_by_tag(&self, tag: &str) -> Vec<&Parameter> {
        self.parameters
            .values()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get pending updates
    pub fn get_pending_updates(&self) -> Vec<&ParameterUpdate> {
        self.pending_updates
            .values()
            .filter(|u| !u.executed)
            .collect()
    }

    /// Get pending update
    pub fn get_pending_update(&self, update_id: &str) -> Option<&ParameterUpdate> {
        self.pending_updates.get(update_id)
    }

    /// Reset parameter to default value
    pub async fn reset_to_default(
        &mut self,
        parameter_name: &str,
        reset_by: String,
    ) -> GovernanceResult<()> {
        let parameter = self.parameters.get_mut(parameter_name)
            .ok_or_else(|| GovernanceError::ParameterNotFound {
                name: parameter_name.to_string(),
            })?;

        if !parameter.mutable {
            return Err(GovernanceError::ParameterIsImmutable {
                name: parameter_name.to_string(),
            });
        }

        parameter.current_value = parameter.default_value.clone();
        parameter.last_updated = Utc::now();
        parameter.last_updated_by = Some(reset_by);
        parameter.update_count += 1;

        Ok(())
    }

    /// Batch update multiple parameters
    pub async fn batch_update(
        &mut self,
        updates: Vec<(String, ParameterValue)>,
        updated_by: String,
    ) -> GovernanceResult<Vec<String>> {
        let mut successful_updates = Vec::new();

        for (parameter_name, new_value) in updates {
            // Validate each parameter
            if let Some(parameter) = self.parameters.get(&parameter_name) {
                if parameter.mutable {
                    if self.validate_value(&new_value, &parameter.parameter_type, &parameter.constraints).is_ok() {
                        // Apply update
                        let parameter = self.parameters.get_mut(&parameter_name).unwrap();
                        parameter.current_value = new_value;
                        parameter.last_updated = Utc::now();
                        parameter.last_updated_by = Some(updated_by.clone());
                        parameter.update_count += 1;

                        successful_updates.push(parameter_name);
                    }
                }
            }
        }

        Ok(successful_updates)
    }

    /// Get parameter statistics
    pub fn get_parameter_statistics(&self) -> ParameterStatistics {
        let mut stats = ParameterStatistics::default();

        stats.total_parameters = self.parameters.len();

        for parameter in self.parameters.values() {
            if parameter.mutable {
                stats.mutable_parameters += 1;
            } else {
                stats.immutable_parameters += 1;
            }

            if parameter.protected {
                stats.protected_parameters += 1;
            }

            stats.total_updates += parameter.update_count as usize;

            match parameter.parameter_type {
                ParameterType::String => stats.string_parameters += 1,
                ParameterType::Integer => stats.integer_parameters += 1,
                ParameterType::UInteger => stats.uinteger_parameters += 1,
                ParameterType::Float => stats.float_parameters += 1,
                ParameterType::Boolean => stats.boolean_parameters += 1,
                ParameterType::Array(_) => stats.array_parameters += 1,
                ParameterType::Object => stats.object_parameters += 1,
            }
        }

        stats.pending_updates = self.pending_updates.values()
            .filter(|u| !u.executed)
            .count();

        stats.total_categories = self.categories.len();

        stats
    }

    /// Process expired timelocks
    pub async fn process_expired_timelocks(&mut self) -> GovernanceResult<Vec<String>> {
        let now = Utc::now();
        let mut ready_updates = Vec::new();

        for (update_id, update) in &self.pending_updates {
            if !update.executed && now >= update.timelock_expires_at {
                ready_updates.push(update_id.clone());
            }
        }

        Ok(ready_updates)
    }

    // Helper methods

    fn validate_value(
        &self,
        value: &ParameterValue,
        parameter_type: &ParameterType,
        constraints: &Option<ParameterConstraints>,
    ) -> GovernanceResult<()> {
        // Type validation
        match (value, parameter_type) {
            (ParameterValue::String(_), ParameterType::String) => {}
            (ParameterValue::Integer(_), ParameterType::Integer) => {}
            (ParameterValue::UInteger(_), ParameterType::UInteger) => {}
            (ParameterValue::Float(_), ParameterType::Float) => {}
            (ParameterValue::Boolean(_), ParameterType::Boolean) => {}
            (ParameterValue::Array(_), ParameterType::Array(_)) => {}
            (ParameterValue::Object(_), ParameterType::Object) => {}
            _ => {
                return Err(GovernanceError::InvalidParameterValue {
                    name: "type_mismatch".to_string(),
                    value: value.to_string(),
                });
            }
        }

        // Constraint validation
        if let Some(constraints) = constraints {
            self.validate_constraints(value, constraints)?;
        }

        Ok(())
    }

    fn validate_constraints(
        &self,
        value: &ParameterValue,
        constraints: &ParameterConstraints,
    ) -> GovernanceResult<()> {
        // Numeric range validation
        if let Some(min_val) = constraints.min_value {
            let numeric_value = match value {
                ParameterValue::Integer(i) => *i as f64,
                ParameterValue::UInteger(u) => *u as f64,
                ParameterValue::Float(f) => *f,
                _ => return Ok(()), // Skip for non-numeric types
            };

            if numeric_value < min_val {
                return Err(GovernanceError::InvalidParameterValue {
                    name: "min_value".to_string(),
                    value: value.to_string(),
                });
            }
        }

        if let Some(max_val) = constraints.max_value {
            let numeric_value = match value {
                ParameterValue::Integer(i) => *i as f64,
                ParameterValue::UInteger(u) => *u as f64,
                ParameterValue::Float(f) => *f,
                _ => return Ok(()), // Skip for non-numeric types
            };

            if numeric_value > max_val {
                return Err(GovernanceError::InvalidParameterValue {
                    name: "max_value".to_string(),
                    value: value.to_string(),
                });
            }
        }

        // Allowed values validation
        if let Some(allowed) = &constraints.allowed_values {
            if !allowed.contains(value) {
                return Err(GovernanceError::InvalidParameterValue {
                    name: "allowed_values".to_string(),
                    value: value.to_string(),
                });
            }
        }

        // Length validation
        let length = match value {
            ParameterValue::String(s) => s.len(),
            ParameterValue::Array(arr) => arr.len(),
            _ => return Ok(()), // Skip for other types
        };

        if let Some(min_len) = constraints.min_length {
            if length < min_len {
                return Err(GovernanceError::InvalidParameterValue {
                    name: "min_length".to_string(),
                    value: value.to_string(),
                });
            }
        }

        if let Some(max_len) = constraints.max_length {
            if length > max_len {
                return Err(GovernanceError::InvalidParameterValue {
                    name: "max_length".to_string(),
                    value: value.to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Statistics for parameter management
#[derive(Debug, Default, Clone)]
pub struct ParameterStatistics {
    pub total_parameters: usize,
    pub mutable_parameters: usize,
    pub immutable_parameters: usize,
    pub protected_parameters: usize,
    pub string_parameters: usize,
    pub integer_parameters: usize,
    pub uinteger_parameters: usize,
    pub float_parameters: usize,
    pub boolean_parameters: usize,
    pub array_parameters: usize,
    pub object_parameters: usize,
    pub total_updates: usize,
    pub pending_updates: usize,
    pub total_categories: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_parameter() {
        let mut manager = ParameterManager::new(ParameterConfig::default());

        manager.register_parameter(
            "trading_fee".to_string(),
            "Fee charged on trades".to_string(),
            ParameterType::Float,
            ParameterValue::Float(0.003),
            Some(ParameterConstraints {
                min_value: Some(0.0),
                max_value: Some(0.1),
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            true,
            false,
            "trading".to_string(),
            vec!["fee".to_string(), "trading".to_string()],
        ).await.unwrap();

        let parameter = manager.get_parameter("trading_fee").unwrap();
        assert_eq!(parameter.name, "trading_fee");
        assert_eq!(parameter.current_value, ParameterValue::Float(0.003));
        assert!(parameter.mutable);
    }

    #[tokio::test]
    async fn test_propose_and_execute_update() {
        let mut manager = ParameterManager::new(ParameterConfig {
            default_timelock_duration: chrono::Duration::seconds(1), // Short for testing
            ..ParameterConfig::default()
        });

        // Register parameter
        manager.register_parameter(
            "trading_fee".to_string(),
            "Fee charged on trades".to_string(),
            ParameterType::Float,
            ParameterValue::Float(0.003),
            Some(ParameterConstraints {
                min_value: Some(0.0),
                max_value: Some(0.1),
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            true,
            false,
            "trading".to_string(),
            vec!["fee".to_string()],
        ).await.unwrap();

        // Propose update
        let update_id = manager.propose_update(
            "trading_fee".to_string(),
            ParameterValue::Float(0.002),
            "proposer".to_string(),
            "Reduce fee to increase volume".to_string(),
            Some("Lower fees expected to increase trading volume by 20%".to_string()),
            None,
        ).await.unwrap();

        // Wait for timelock (in real code, this would be handled by a scheduler)
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        // Execute update
        manager.execute_update(&update_id, "executor".to_string()).await.unwrap();

        let parameter = manager.get_parameter("trading_fee").unwrap();
        assert_eq!(parameter.current_value, ParameterValue::Float(0.002));
        assert_eq!(parameter.update_count, 1);
    }

    #[tokio::test]
    async fn test_parameter_validation() {
        let mut manager = ParameterManager::new(ParameterConfig::default());

        // Register parameter with constraints
        manager.register_parameter(
            "max_order_size".to_string(),
            "Maximum order size".to_string(),
            ParameterType::UInteger,
            ParameterValue::UInteger(1000000),
            Some(ParameterConstraints {
                min_value: Some(1.0),
                max_value: Some(10000000.0),
                allowed_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                custom_validator: None,
            }),
            true,
            false,
            "trading".to_string(),
            vec!["order".to_string()],
        ).await.unwrap();

        // Try to propose invalid update (below minimum)
        let result = manager.propose_update(
            "max_order_size".to_string(),
            ParameterValue::UInteger(0),
            "proposer".to_string(),
            "Test update".to_string(),
            Some("Test".to_string()),
            None,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_immutable_parameter() {
        let mut manager = ParameterManager::new(ParameterConfig::default());

        // Register immutable parameter
        manager.register_parameter(
            "protocol_version".to_string(),
            "Protocol version".to_string(),
            ParameterType::String,
            ParameterValue::String("1.0.0".to_string()),
            None,
            false, // Not mutable
            true,  // Protected
            "system".to_string(),
            vec!["version".to_string()],
        ).await.unwrap();

        // Try to propose update to immutable parameter
        let result = manager.propose_update(
            "protocol_version".to_string(),
            ParameterValue::String("2.0.0".to_string()),
            "proposer".to_string(),
            "Upgrade protocol".to_string(),
            Some("Major upgrade".to_string()),
            None,
        ).await;

        assert!(matches!(result, Err(GovernanceError::ParameterIsImmutable { .. })));
    }

    #[tokio::test]
    async fn test_parameter_categories() {
        let mut manager = ParameterManager::new(ParameterConfig::default());

        // Register parameters in different categories
        manager.register_parameter(
            "trading_fee".to_string(),
            "Trading fee".to_string(),
            ParameterType::Float,
            ParameterValue::Float(0.003),
            None,
            true,
            false,
            "trading".to_string(),
            vec!["fee".to_string()],
        ).await.unwrap();

        manager.register_parameter(
            "withdrawal_fee".to_string(),
            "Withdrawal fee".to_string(),
            ParameterType::Float,
            ParameterValue::Float(0.001),
            None,
            true,
            false,
            "trading".to_string(),
            vec!["fee".to_string()],
        ).await.unwrap();

        manager.register_parameter(
            "protocol_version".to_string(),
            "Protocol version".to_string(),
            ParameterType::String,
            ParameterValue::String("1.0.0".to_string()),
            None,
            false,
            true,
            "system".to_string(),
            vec!["version".to_string()],
        ).await.unwrap();

        let trading_params = manager.get_parameters_by_category("trading");
        assert_eq!(trading_params.len(), 2);

        let system_params = manager.get_parameters_by_category("system");
        assert_eq!(system_params.len(), 1);

        let fee_params = manager.get_parameters_by_tag("fee");
        assert_eq!(fee_params.len(), 2);
    }

    #[tokio::test]
    async fn test_parameter_statistics() {
        let mut manager = ParameterManager::new(ParameterConfig::default());

        // Register various parameters
        manager.register_parameter(
            "trading_fee".to_string(),
            "Trading fee".to_string(),
            ParameterType::Float,
            ParameterValue::Float(0.003),
            None,
            true,
            false,
            "trading".to_string(),
            vec!["fee".to_string()],
        ).await.unwrap();

        manager.register_parameter(
            "max_orders".to_string(),
            "Max orders".to_string(),
            ParameterType::UInteger,
            ParameterValue::UInteger(1000),
            None,
            true,
            false,
            "trading".to_string(),
            vec!["limit".to_string()],
        ).await.unwrap();

        manager.register_parameter(
            "enabled".to_string(),
            "System enabled".to_string(),
            ParameterType::Boolean,
            ParameterValue::Boolean(true),
            None,
            false,
            true,
            "system".to_string(),
            vec!["status".to_string()],
        ).await.unwrap();

        let stats = manager.get_parameter_statistics();
        assert_eq!(stats.total_parameters, 3);
        assert_eq!(stats.mutable_parameters, 2);
        assert_eq!(stats.immutable_parameters, 1);
        assert_eq!(stats.protected_parameters, 1);
        assert_eq!(stats.float_parameters, 1);
        assert_eq!(stats.uinteger_parameters, 1);
        assert_eq!(stats.boolean_parameters, 1);
        assert_eq!(stats.total_categories, 2);
    }
}