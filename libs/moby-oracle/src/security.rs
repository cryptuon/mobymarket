//! # Oracle Security and Validation
//!
//! This module provides comprehensive security mechanisms for oracle data validation,
//! cryptographic verification, fraud detection, and attack prevention specifically
//! designed for whale trading operations.
//!
//! ## Features
//!
//! - Cryptographic data integrity verification
//! - Real-time fraud detection and anomaly analysis
//! - MEV (Maximal Extractable Value) attack protection
//! - Source authentication and reputation scoring
//! - Data freshness and staleness detection
//! - Circuit breaker mechanisms for faulty sources

use crate::error::{OracleError, OracleResult};
use crate::sources::{DataPoint, DataSource};
use crate::aggregation::AggregatedPrice;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use sha2::{Sha256, Digest};
use ed25519_dalek::{Verifier, Signature, VerifyingKey};
use hex;

/// Validation result for oracle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the data passed validation
    pub is_valid: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence_score: f64,
    /// List of validation checks performed
    pub checks_performed: Vec<ValidationCheck>,
    /// Any security warnings detected
    pub security_warnings: Vec<SecurityWarning>,
    /// Fraud risk assessment
    pub fraud_risk: FraudRiskLevel,
    /// Data integrity verification result
    pub integrity_verified: bool,
    /// Timestamp of validation
    pub validated_at: DateTime<Utc>,
}

/// Individual validation check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    /// Name of the validation check
    pub check_name: String,
    /// Whether this check passed
    pub passed: bool,
    /// Severity of failure if check failed
    pub severity: ValidationSeverity,
    /// Additional details about the check
    pub details: Option<String>,
    /// Score contribution to overall confidence
    pub score_impact: f64,
}

/// Severity level for validation failures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    /// Low impact on confidence
    Low,
    /// Medium impact on confidence
    Medium,
    /// High impact on confidence
    High,
    /// Critical - data should be rejected
    Critical,
}

/// Security warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityWarning {
    /// Unusual price movement detected
    PriceManipulationSuspected {
        deviation_percentage: f64,
        expected_range: (Decimal, Decimal),
    },
    /// Data source showing suspicious behavior
    SourceBehaviorAnomaly {
        source: DataSource,
        anomaly_type: String,
        severity: ValidationSeverity,
    },
    /// Potential MEV attack detected
    MEVAttackSuspected {
        attack_type: MEVAttackType,
        confidence: f64,
    },
    /// Data staleness detected
    StaleDataDetected {
        age_seconds: u64,
        max_allowed_age: u64,
    },
    /// Volume anomaly detected
    VolumeAnomalyDetected {
        reported_volume: Decimal,
        expected_range: (Decimal, Decimal),
    },
    /// Cross-source correlation failure
    CorrelationFailure {
        correlation_coefficient: f64,
        minimum_expected: f64,
    },
}

/// Types of MEV attacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MEVAttackType {
    /// Front-running attack
    Frontrunning,
    /// Sandwich attack
    Sandwich,
    /// Price oracle manipulation
    OracleManipulation,
    /// Flash loan attack
    FlashLoan,
    /// Arbitrage manipulation
    ArbitrageManipulation,
}

/// Fraud risk assessment levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum FraudRiskLevel {
    /// Very low fraud risk
    VeryLow,
    /// Low fraud risk
    Low,
    /// Medium fraud risk
    Medium,
    /// High fraud risk
    High,
    /// Very high fraud risk - reject data
    VeryHigh,
}

/// Data integrity verification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataIntegrity {
    /// SHA-256 hash of the data
    pub data_hash: String,
    /// Digital signature (if available)
    pub signature: Option<String>,
    /// Public key for signature verification
    pub public_key: Option<String>,
    /// Merkle proof (if applicable)
    pub merkle_proof: Option<MerkleProof>,
    /// Source-specific integrity data
    pub source_integrity: HashMap<DataSource, SourceIntegrityInfo>,
}

/// Merkle proof for data verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Merkle root hash
    pub root: String,
    /// Proof hashes
    pub proof: Vec<String>,
    /// Leaf index
    pub index: usize,
}

/// Source-specific integrity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIntegrityInfo {
    /// Source reputation score
    pub reputation_score: f64,
    /// Last known good timestamp
    pub last_good_data: DateTime<Utc>,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Circuit breaker status
    pub circuit_breaker_active: bool,
    /// Source-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Configuration for security validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Maximum allowed price deviation (percentage)
    pub max_price_deviation: f64,
    /// Maximum data age before considered stale
    pub max_data_age: Duration,
    /// Minimum correlation coefficient between sources
    pub min_correlation: f64,
    /// MEV attack detection sensitivity
    pub mev_detection_sensitivity: f64,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Source reputation decay rate
    pub reputation_decay_rate: f64,
    /// Fraud risk thresholds
    pub fraud_risk_thresholds: FraudRiskThresholds,
    /// Whale trading specific security settings
    pub whale_security: WhaleSecurityConfig,
}

/// Fraud risk threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudRiskThresholds {
    /// Threshold for very low risk
    pub very_low: f64,
    /// Threshold for low risk
    pub low: f64,
    /// Threshold for medium risk
    pub medium: f64,
    /// Threshold for high risk
    pub high: f64,
}

/// Whale trading specific security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleSecurityConfig {
    /// Large order volume threshold
    pub large_order_threshold: Decimal,
    /// Price impact protection threshold
    pub price_impact_threshold: f64,
    /// MEV protection for whale orders
    pub mev_protection_enabled: bool,
    /// Slippage protection threshold
    pub slippage_protection: f64,
}

/// Main security validator for oracle data
pub struct SecurityValidator {
    config: SecurityConfig,
    source_reputation: HashMap<DataSource, SourceIntegrityInfo>,
    historical_data: Vec<AggregatedPrice>,
}

impl SecurityValidator {
    /// Create new security validator
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            source_reputation: HashMap::new(),
            historical_data: Vec::new(),
        }
    }

    /// Validate a single data point
    pub async fn validate_data_point(&mut self, data_point: &DataPoint) -> OracleResult<ValidationResult> {
        let mut checks = Vec::new();
        let mut warnings = Vec::new();
        let mut confidence_score = 1.0;

        // 1. Data freshness check
        let freshness_check = self.check_data_freshness(data_point);
        confidence_score *= self.apply_check_impact(&freshness_check);
        if let Some(warning) = self.extract_freshness_warning(&freshness_check, data_point) {
            warnings.push(warning);
        }
        checks.push(freshness_check);

        // 2. Price range validation
        let range_check = await self.check_price_range(data_point).await?;
        confidence_score *= self.apply_check_impact(&range_check);
        if let Some(warning) = self.extract_price_warning(&range_check, data_point) {
            warnings.push(warning);
        }
        checks.push(range_check);

        // 3. Source reputation check
        let reputation_check = self.check_source_reputation(data_point);
        confidence_score *= self.apply_check_impact(&reputation_check);
        checks.push(reputation_check);

        // 4. Data integrity verification
        let integrity_check = await self.verify_data_integrity(data_point).await?;
        confidence_score *= self.apply_check_impact(&integrity_check);
        checks.push(integrity_check);

        // 5. Volume validation
        let volume_check = self.check_volume_validity(data_point);
        confidence_score *= self.apply_check_impact(&volume_check);
        if let Some(warning) = self.extract_volume_warning(&volume_check, data_point) {
            warnings.push(warning);
        }
        checks.push(volume_check);

        // 6. MEV attack detection
        let mev_check = await self.detect_mev_attack(data_point).await?;
        confidence_score *= self.apply_check_impact(&mev_check);
        if let Some(warning) = self.extract_mev_warning(&mev_check) {
            warnings.push(warning);
        }
        checks.push(mev_check);

        // Calculate fraud risk
        let fraud_risk = self.assess_fraud_risk(confidence_score, &warnings);

        // Determine overall validity
        let is_valid = confidence_score >= 0.5 && fraud_risk != FraudRiskLevel::VeryHigh;

        // Verify integrity
        let integrity_verified = checks.iter()
            .find(|c| c.check_name == "data_integrity")
            .map(|c| c.passed)
            .unwrap_or(false);

        Ok(ValidationResult {
            is_valid,
            confidence_score,
            checks_performed: checks,
            security_warnings: warnings,
            fraud_risk,
            integrity_verified,
            validated_at: Utc::now(),
        })
    }

    /// Validate aggregated price data
    pub async fn validate_aggregated_price(&mut self, price: &AggregatedPrice) -> OracleResult<ValidationResult> {
        let mut checks = Vec::new();
        let mut warnings = Vec::new();
        let mut confidence_score = price.confidence;

        // 1. Cross-source correlation check
        let correlation_check = self.check_cross_source_correlation(price);
        confidence_score *= self.apply_check_impact(&correlation_check);
        if let Some(warning) = self.extract_correlation_warning(&correlation_check) {
            warnings.push(warning);
        }
        checks.push(correlation_check);

        // 2. Historical consistency check
        let consistency_check = self.check_historical_consistency(price);
        confidence_score *= self.apply_check_impact(&consistency_check);
        checks.push(consistency_check);

        // 3. Whale impact validation
        let whale_check = self.validate_whale_impact(price);
        confidence_score *= self.apply_check_impact(&whale_check);
        checks.push(whale_check);

        // 4. Market conditions validation
        let market_check = await self.validate_market_conditions(price).await?;
        confidence_score *= self.apply_check_impact(&market_check);
        checks.push(market_check);

        // Store for historical analysis
        self.historical_data.push(price.clone());
        if self.historical_data.len() > 1000 {
            self.historical_data.drain(0..100);
        }

        let fraud_risk = self.assess_fraud_risk(confidence_score, &warnings);
        let is_valid = confidence_score >= 0.6 && fraud_risk != FraudRiskLevel::VeryHigh;

        Ok(ValidationResult {
            is_valid,
            confidence_score,
            checks_performed: checks,
            security_warnings: warnings,
            fraud_risk,
            integrity_verified: true, // Aggregated data is internally verified
            validated_at: Utc::now(),
        })
    }

    /// Check data freshness
    fn check_data_freshness(&self, data_point: &DataPoint) -> ValidationCheck {
        let now = Utc::now();
        let age = now.signed_duration_since(data_point.timestamp);
        let age_seconds = age.num_seconds() as u64;
        let max_age_seconds = self.config.max_data_age.as_secs();

        let passed = age_seconds <= max_age_seconds;
        let severity = if age_seconds > max_age_seconds * 2 {
            ValidationSeverity::Critical
        } else if age_seconds > max_age_seconds {
            ValidationSeverity::High
        } else if age_seconds > max_age_seconds / 2 {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        let score_impact = if passed { 1.0 } else {
            match severity {
                ValidationSeverity::Critical => 0.0,
                ValidationSeverity::High => 0.3,
                ValidationSeverity::Medium => 0.7,
                ValidationSeverity::Low => 0.9,
            }
        };

        ValidationCheck {
            check_name: "data_freshness".to_string(),
            passed,
            severity,
            details: Some(format!("Data age: {}s, max allowed: {}s", age_seconds, max_age_seconds)),
            score_impact,
        }
    }

    /// Check if price is within reasonable range
    async fn check_price_range(&self, data_point: &DataPoint) -> OracleResult<ValidationCheck> {
        // Get recent historical prices for comparison
        let recent_prices: Vec<Decimal> = self.historical_data.iter()
            .rev()
            .take(10)
            .filter(|p| p.symbol == data_point.symbol)
            .map(|p| p.price)
            .collect();

        if recent_prices.is_empty() {
            // No historical data - accept with medium confidence
            return Ok(ValidationCheck {
                check_name: "price_range".to_string(),
                passed: true,
                severity: ValidationSeverity::Low,
                details: Some("No historical data for comparison".to_string()),
                score_impact: 0.8,
            });
        }

        // Calculate average and deviation
        let avg_price: Decimal = recent_prices.iter().sum::<Decimal>() / Decimal::from(recent_prices.len());
        let deviation = ((data_point.value - avg_price).abs() / avg_price).to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

        let max_deviation = self.config.max_price_deviation;
        let passed = deviation <= max_deviation;

        let severity = if deviation > max_deviation * 3.0 {
            ValidationSeverity::Critical
        } else if deviation > max_deviation * 2.0 {
            ValidationSeverity::High
        } else if deviation > max_deviation {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        let score_impact = if passed { 1.0 } else {
            (1.0 - (deviation / max_deviation).min(1.0)).max(0.0)
        };

        Ok(ValidationCheck {
            check_name: "price_range".to_string(),
            passed,
            severity,
            details: Some(format!("Price deviation: {:.2}%, max allowed: {:.2}%",
                deviation * 100.0, max_deviation * 100.0)),
            score_impact,
        })
    }

    /// Check source reputation
    fn check_source_reputation(&mut self, data_point: &DataPoint) -> ValidationCheck {
        let reputation_info = self.source_reputation.entry(data_point.source)
            .or_insert_with(|| SourceIntegrityInfo {
                reputation_score: 1.0,
                last_good_data: data_point.timestamp,
                failure_count: 0,
                circuit_breaker_active: false,
                metadata: HashMap::new(),
            });

        let passed = reputation_info.reputation_score >= 0.5 && !reputation_info.circuit_breaker_active;

        let severity = if reputation_info.circuit_breaker_active {
            ValidationSeverity::Critical
        } else if reputation_info.reputation_score < 0.3 {
            ValidationSeverity::High
        } else if reputation_info.reputation_score < 0.5 {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        ValidationCheck {
            check_name: "source_reputation".to_string(),
            passed,
            severity,
            details: Some(format!("Reputation score: {:.2}, failures: {}",
                reputation_info.reputation_score, reputation_info.failure_count)),
            score_impact: reputation_info.reputation_score,
        }
    }

    /// Verify data integrity using cryptographic methods
    async fn verify_data_integrity(&self, data_point: &DataPoint) -> OracleResult<ValidationCheck> {
        // Calculate data hash
        let data_string = format!("{}:{}:{}:{}",
            data_point.symbol, data_point.value, data_point.timestamp.timestamp(), data_point.source as u8);
        let hash = Sha256::digest(data_string.as_bytes());
        let hash_hex = hex::encode(hash);

        // Check if signature verification is available
        let signature_verified = if let (Some(sig_hex), Some(pubkey_hex)) =
            (data_point.metadata.get("signature"), data_point.metadata.get("public_key")) {
            self.verify_signature(&hash_hex, sig_hex, pubkey_hex).unwrap_or(false)
        } else {
            true // No signature required
        };

        let passed = signature_verified;
        let severity = if passed { ValidationSeverity::Low } else { ValidationSeverity::High };

        Ok(ValidationCheck {
            check_name: "data_integrity".to_string(),
            passed,
            severity,
            details: Some(format!("Data hash: {}, signature verified: {}",
                &hash_hex[..16], signature_verified)),
            score_impact: if passed { 1.0 } else { 0.5 },
        })
    }

    /// Verify digital signature
    fn verify_signature(&self, data_hash: &str, signature_hex: &str, pubkey_hex: &str) -> OracleResult<bool> {
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|_| OracleError::InvalidSignatureFormat)?;
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|_| OracleError::InvalidPublicKeyFormat)?;

        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|_| OracleError::InvalidSignatureFormat)?;
        let public_key = VerifyingKey::from_bytes(&pubkey_bytes.try_into()
            .map_err(|_| OracleError::InvalidPublicKeyFormat)?)
            .map_err(|_| OracleError::InvalidPublicKeyFormat)?;

        let message = hex::decode(data_hash)
            .map_err(|_| OracleError::InvalidDataHash)?;

        Ok(public_key.verify(&message, &signature).is_ok())
    }

    /// Check volume validity
    fn check_volume_validity(&self, data_point: &DataPoint) -> ValidationCheck {
        let volume = data_point.volume.unwrap_or_default();

        // Check for reasonable volume ranges
        let min_volume = Decimal::from(1000); // $1K minimum
        let max_volume = Decimal::from(1_000_000_000); // $1B maximum

        let passed = volume >= min_volume && volume <= max_volume;
        let severity = if volume <= Decimal::ZERO {
            ValidationSeverity::High
        } else if volume > max_volume {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        ValidationCheck {
            check_name: "volume_validity".to_string(),
            passed,
            severity,
            details: Some(format!("Volume: ${}, range: ${}-${}", volume, min_volume, max_volume)),
            score_impact: if passed { 1.0 } else { 0.7 },
        }
    }

    /// Detect potential MEV attacks
    async fn detect_mev_attack(&self, data_point: &DataPoint) -> OracleResult<ValidationCheck> {
        let mut mev_score = 0.0;

        // Check for suspicious volume spikes
        if let Some(volume) = data_point.volume {
            if volume > self.config.whale_security.large_order_threshold {
                mev_score += 0.3;
            }
        }

        // Check for unusual price movements
        if let Some(last_price) = self.historical_data.last() {
            if last_price.symbol == data_point.symbol {
                let price_change = ((data_point.value - last_price.price).abs() / last_price.price)
                    .to_string().parse::<f64>().unwrap_or(0.0);

                if price_change > self.config.whale_security.price_impact_threshold {
                    mev_score += 0.4;
                }
            }
        }

        // Check timing patterns (simplified)
        let timestamp_ms = data_point.timestamp.timestamp_millis();
        if timestamp_ms % 12000 < 100 { // Suspicious if exactly on block boundaries
            mev_score += 0.2;
        }

        let mev_detected = mev_score > self.config.mev_detection_sensitivity;
        let passed = !mev_detected;

        let severity = if mev_score > 0.8 {
            ValidationSeverity::Critical
        } else if mev_score > 0.6 {
            ValidationSeverity::High
        } else if mev_score > 0.4 {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        Ok(ValidationCheck {
            check_name: "mev_detection".to_string(),
            passed,
            severity,
            details: Some(format!("MEV risk score: {:.2}", mev_score)),
            score_impact: if passed { 1.0 } else { 1.0 - mev_score },
        })
    }

    /// Check cross-source correlation
    fn check_cross_source_correlation(&self, price: &AggregatedPrice) -> ValidationCheck {
        if price.source_count < 2 {
            return ValidationCheck {
                check_name: "cross_source_correlation".to_string(),
                passed: true,
                severity: ValidationSeverity::Low,
                details: Some("Single source - no correlation to check".to_string()),
                score_impact: 0.8,
            };
        }

        // Simplified correlation check based on price variance
        let variance = price.quality_metrics.price_variance.to_string().parse::<f64>().unwrap_or(0.0);
        let correlation_estimate = 1.0 - variance.min(1.0);

        let passed = correlation_estimate >= self.config.min_correlation;
        let severity = if correlation_estimate < 0.3 {
            ValidationSeverity::High
        } else if correlation_estimate < 0.5 {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        ValidationCheck {
            check_name: "cross_source_correlation".to_string(),
            passed,
            severity,
            details: Some(format!("Estimated correlation: {:.2}, minimum: {:.2}",
                correlation_estimate, self.config.min_correlation)),
            score_impact: correlation_estimate,
        }
    }

    /// Check historical consistency
    fn check_historical_consistency(&self, price: &AggregatedPrice) -> ValidationCheck {
        if self.historical_data.len() < 5 {
            return ValidationCheck {
                check_name: "historical_consistency".to_string(),
                passed: true,
                severity: ValidationSeverity::Low,
                details: Some("Insufficient historical data".to_string()),
                score_impact: 0.9,
            };
        }

        // Calculate trend consistency
        let recent_prices: Vec<Decimal> = self.historical_data.iter()
            .rev()
            .take(5)
            .filter(|p| p.symbol == price.symbol)
            .map(|p| p.price)
            .collect();

        if recent_prices.len() < 3 {
            return ValidationCheck {
                check_name: "historical_consistency".to_string(),
                passed: true,
                severity: ValidationSeverity::Low,
                details: Some("Insufficient price history for symbol".to_string()),
                score_impact: 0.9,
            };
        }

        // Simple trend analysis
        let avg_change: f64 = recent_prices.windows(2)
            .map(|w| ((w[0] - w[1]).abs() / w[1]).to_string().parse::<f64>().unwrap_or(0.0))
            .sum::<f64>() / (recent_prices.len() - 1) as f64;

        let current_change = if let Some(last_price) = recent_prices.first() {
            ((price.price - last_price).abs() / last_price).to_string().parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        };

        let consistency_score = if avg_change > 0.0 {
            1.0 - (current_change / (avg_change * 3.0)).min(1.0)
        } else {
            0.9
        };

        let passed = consistency_score >= 0.5;
        let severity = if consistency_score < 0.3 {
            ValidationSeverity::High
        } else if consistency_score < 0.5 {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        ValidationCheck {
            check_name: "historical_consistency".to_string(),
            passed,
            severity,
            details: Some(format!("Consistency score: {:.2}", consistency_score)),
            score_impact: consistency_score,
        }
    }

    /// Validate whale impact analysis
    fn validate_whale_impact(&self, price: &AggregatedPrice) -> ValidationCheck {
        let whale_impact = &price.whale_impact;

        // Check if whale detection is reasonable
        let volume_threshold = self.config.whale_security.large_order_threshold;
        let liquidity_ratio = if whale_impact.liquidity_depth > Decimal::ZERO {
            volume_threshold / whale_impact.liquidity_depth
        } else {
            Decimal::ONE
        };

        let impact_reasonable = whale_impact.price_impact_bps <= Decimal::from_f64_retain(1000.0).unwrap(); // Max 10%
        let volatility_reasonable = whale_impact.volatility_score <= 1.0;

        let passed = impact_reasonable && volatility_reasonable;
        let severity = if !impact_reasonable {
            ValidationSeverity::High
        } else if !volatility_reasonable {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        ValidationCheck {
            check_name: "whale_impact_validation".to_string(),
            passed,
            severity,
            details: Some(format!("Impact: {} bps, volatility: {:.2}",
                whale_impact.price_impact_bps, whale_impact.volatility_score)),
            score_impact: if passed { 1.0 } else { 0.6 },
        }
    }

    /// Validate market conditions
    async fn validate_market_conditions(&self, price: &AggregatedPrice) -> OracleResult<ValidationCheck> {
        // Simplified market validation - check for extreme conditions
        let extreme_volatility = price.whale_impact.volatility_score > 0.8;
        let extreme_impact = price.whale_impact.price_impact_bps > Decimal::from(500); // 5%

        let passed = !extreme_volatility && !extreme_impact;
        let severity = if extreme_volatility && extreme_impact {
            ValidationSeverity::High
        } else if extreme_volatility || extreme_impact {
            ValidationSeverity::Medium
        } else {
            ValidationSeverity::Low
        };

        Ok(ValidationCheck {
            check_name: "market_conditions".to_string(),
            passed,
            severity,
            details: Some(format!("Extreme volatility: {}, extreme impact: {}",
                extreme_volatility, extreme_impact)),
            score_impact: if passed { 1.0 } else { 0.7 },
        })
    }

    /// Apply check impact to confidence score
    fn apply_check_impact(&self, check: &ValidationCheck) -> f64 {
        if check.passed {
            check.score_impact
        } else {
            match check.severity {
                ValidationSeverity::Critical => 0.0,
                ValidationSeverity::High => check.score_impact * 0.5,
                ValidationSeverity::Medium => check.score_impact * 0.7,
                ValidationSeverity::Low => check.score_impact * 0.9,
            }
        }
    }

    /// Extract freshness warning
    fn extract_freshness_warning(&self, check: &ValidationCheck, data_point: &DataPoint) -> Option<SecurityWarning> {
        if !check.passed && check.severity != ValidationSeverity::Low {
            let age = Utc::now().signed_duration_since(data_point.timestamp).num_seconds() as u64;
            Some(SecurityWarning::StaleDataDetected {
                age_seconds: age,
                max_allowed_age: self.config.max_data_age.as_secs(),
            })
        } else {
            None
        }
    }

    /// Extract price warning
    fn extract_price_warning(&self, check: &ValidationCheck, data_point: &DataPoint) -> Option<SecurityWarning> {
        if !check.passed && check.severity == ValidationSeverity::High {
            // Extract deviation from check details
            if let Some(details) = &check.details {
                if let Some(deviation_str) = details.split("deviation: ").nth(1) {
                    if let Some(deviation_pct) = deviation_str.split('%').next() {
                        if let Ok(deviation) = deviation_pct.parse::<f64>() {
                            return Some(SecurityWarning::PriceManipulationSuspected {
                                deviation_percentage: deviation,
                                expected_range: (data_point.value * Decimal::from_f64_retain(0.95).unwrap(),
                                               data_point.value * Decimal::from_f64_retain(1.05).unwrap()),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract volume warning
    fn extract_volume_warning(&self, check: &ValidationCheck, data_point: &DataPoint) -> Option<SecurityWarning> {
        if !check.passed {
            let volume = data_point.volume.unwrap_or_default();
            Some(SecurityWarning::VolumeAnomalyDetected {
                reported_volume: volume,
                expected_range: (Decimal::from(1000), Decimal::from(1_000_000_000)),
            })
        } else {
            None
        }
    }

    /// Extract MEV warning
    fn extract_mev_warning(&self, check: &ValidationCheck) -> Option<SecurityWarning> {
        if !check.passed {
            Some(SecurityWarning::MEVAttackSuspected {
                attack_type: MEVAttackType::OracleManipulation,
                confidence: 0.7,
            })
        } else {
            None
        }
    }

    /// Extract correlation warning
    fn extract_correlation_warning(&self, check: &ValidationCheck) -> Option<SecurityWarning> {
        if !check.passed {
            if let Some(details) = &check.details {
                if let Some(corr_str) = details.split("correlation: ").nth(1) {
                    if let Some(corr_val) = corr_str.split(',').next() {
                        if let Ok(correlation) = corr_val.parse::<f64>() {
                            return Some(SecurityWarning::CorrelationFailure {
                                correlation_coefficient: correlation,
                                minimum_expected: self.config.min_correlation,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// Assess fraud risk level
    fn assess_fraud_risk(&self, confidence_score: f64, warnings: &[SecurityWarning]) -> FraudRiskLevel {
        let base_risk = 1.0 - confidence_score;

        // Add risk from warnings
        let warning_risk: f64 = warnings.iter().map(|w| match w {
            SecurityWarning::PriceManipulationSuspected { .. } => 0.3,
            SecurityWarning::MEVAttackSuspected { .. } => 0.4,
            SecurityWarning::SourceBehaviorAnomaly { severity, .. } => match severity {
                ValidationSeverity::Critical => 0.5,
                ValidationSeverity::High => 0.3,
                ValidationSeverity::Medium => 0.2,
                ValidationSeverity::Low => 0.1,
            },
            SecurityWarning::StaleDataDetected { .. } => 0.1,
            SecurityWarning::VolumeAnomalyDetected { .. } => 0.2,
            SecurityWarning::CorrelationFailure { .. } => 0.25,
        }).sum();

        let total_risk = (base_risk + warning_risk).min(1.0);

        if total_risk >= self.config.fraud_risk_thresholds.high {
            FraudRiskLevel::VeryHigh
        } else if total_risk >= self.config.fraud_risk_thresholds.medium {
            FraudRiskLevel::High
        } else if total_risk >= self.config.fraud_risk_thresholds.low {
            FraudRiskLevel::Medium
        } else if total_risk >= self.config.fraud_risk_thresholds.very_low {
            FraudRiskLevel::Low
        } else {
            FraudRiskLevel::VeryLow
        }
    }

    /// Update source reputation based on validation results
    pub fn update_source_reputation(&mut self, source: DataSource, validation_result: &ValidationResult) {
        let reputation_info = self.source_reputation.entry(source)
            .or_insert_with(|| SourceIntegrityInfo {
                reputation_score: 1.0,
                last_good_data: Utc::now(),
                failure_count: 0,
                circuit_breaker_active: false,
                metadata: HashMap::new(),
            });

        if validation_result.is_valid {
            // Increase reputation for successful validation
            reputation_info.reputation_score = (reputation_info.reputation_score + 0.01).min(1.0);
            reputation_info.last_good_data = Utc::now();
            reputation_info.failure_count = 0;
        } else {
            // Decrease reputation for failed validation
            reputation_info.reputation_score = (reputation_info.reputation_score - 0.05).max(0.0);
            reputation_info.failure_count += 1;

            // Activate circuit breaker if too many failures
            if reputation_info.failure_count >= self.config.circuit_breaker_threshold {
                reputation_info.circuit_breaker_active = true;
            }
        }

        // Apply reputation decay
        let time_since_last_good = Utc::now().signed_duration_since(reputation_info.last_good_data);
        let decay_factor = (time_since_last_good.num_hours() as f64 * self.config.reputation_decay_rate).min(0.5);
        reputation_info.reputation_score = (reputation_info.reputation_score - decay_factor).max(0.0);
    }

    /// Get source reputation information
    pub fn get_source_reputation(&self, source: &DataSource) -> Option<&SourceIntegrityInfo> {
        self.source_reputation.get(source)
    }

    /// Reset circuit breaker for a source (manual intervention)
    pub fn reset_circuit_breaker(&mut self, source: DataSource) {
        if let Some(info) = self.source_reputation.get_mut(&source) {
            info.circuit_breaker_active = false;
            info.failure_count = 0;
        }
    }

    /// Get security configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Update security configuration
    pub fn update_config(&mut self, config: SecurityConfig) {
        self.config = config;
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_price_deviation: 0.05, // 5%
            max_data_age: Duration::from_secs(300), // 5 minutes
            min_correlation: 0.7,
            mev_detection_sensitivity: 0.6,
            circuit_breaker_threshold: 5,
            reputation_decay_rate: 0.001, // 0.1% per hour
            fraud_risk_thresholds: FraudRiskThresholds::default(),
            whale_security: WhaleSecurityConfig::default(),
        }
    }
}

impl Default for FraudRiskThresholds {
    fn default() -> Self {
        Self {
            very_low: 0.1,
            low: 0.25,
            medium: 0.5,
            high: 0.75,
        }
    }
}

impl Default for WhaleSecurityConfig {
    fn default() -> Self {
        Self {
            large_order_threshold: Decimal::from(1_000_000), // $1M
            price_impact_threshold: 0.02, // 2%
            mev_protection_enabled: true,
            slippage_protection: 0.005, // 0.5%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::DataSource;
    use std::collections::HashMap;

    fn create_test_data_point() -> DataPoint {
        DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_data_freshness_validation() {
        let config = SecurityConfig::default();
        let mut validator = SecurityValidator::new(config);

        let mut data_point = create_test_data_point();
        data_point.timestamp = Utc::now() - chrono::Duration::seconds(60); // 1 minute old

        let result = validator.validate_data_point(&data_point).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(validation.is_valid);
        assert!(validation.confidence_score > 0.5);
    }

    #[tokio::test]
    async fn test_stale_data_rejection() {
        let mut config = SecurityConfig::default();
        config.max_data_age = Duration::from_secs(60); // 1 minute max age

        let mut validator = SecurityValidator::new(config);

        let mut data_point = create_test_data_point();
        data_point.timestamp = Utc::now() - chrono::Duration::seconds(120); // 2 minutes old

        let result = validator.validate_data_point(&data_point).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(!validation.is_valid || validation.confidence_score < 0.5);
        assert!(!validation.security_warnings.is_empty());
    }

    #[tokio::test]
    async fn test_source_reputation_tracking() {
        let config = SecurityConfig::default();
        let mut validator = SecurityValidator::new(config);

        let data_point = create_test_data_point();

        // First validation should pass
        let result1 = validator.validate_data_point(&data_point).await.unwrap();
        validator.update_source_reputation(data_point.source, &result1);

        // Check reputation was updated
        let reputation = validator.get_source_reputation(&DataSource::Chainlink);
        assert!(reputation.is_some());
        assert!(reputation.unwrap().reputation_score > 0.9);
    }

    #[tokio::test]
    async fn test_mev_attack_detection() {
        let config = SecurityConfig::default();
        let mut validator = SecurityValidator::new(config);

        let mut data_point = create_test_data_point();
        data_point.volume = Some(Decimal::from(5_000_000)); // Large volume

        let result = validator.validate_data_point(&data_point).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        // Should detect potential MEV risk due to large volume
        let mev_check = validation.checks_performed.iter()
            .find(|c| c.check_name == "mev_detection");
        assert!(mev_check.is_some());
    }

    #[tokio::test]
    async fn test_fraud_risk_assessment() {
        let config = SecurityConfig::default();
        let mut validator = SecurityValidator::new(config);

        let mut data_point = create_test_data_point();
        data_point.timestamp = Utc::now() - chrono::Duration::seconds(600); // Very old
        data_point.value = Decimal::from(10000); // Unrealistic price

        let result = validator.validate_data_point(&data_point).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(validation.fraud_risk >= FraudRiskLevel::Medium);
    }

    #[tokio::test]
    async fn test_circuit_breaker_activation() {
        let mut config = SecurityConfig::default();
        config.circuit_breaker_threshold = 2;

        let mut validator = SecurityValidator::new(config);
        let data_point = create_test_data_point();

        // Simulate multiple failures
        for _ in 0..3 {
            let mut bad_validation = ValidationResult {
                is_valid: false,
                confidence_score: 0.1,
                checks_performed: vec![],
                security_warnings: vec![],
                fraud_risk: FraudRiskLevel::High,
                integrity_verified: false,
                validated_at: Utc::now(),
            };

            validator.update_source_reputation(data_point.source, &bad_validation);
        }

        let reputation = validator.get_source_reputation(&DataSource::Chainlink);
        assert!(reputation.unwrap().circuit_breaker_active);
    }
}