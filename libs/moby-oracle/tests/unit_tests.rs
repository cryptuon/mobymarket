//! Unit tests for individual oracle components
//!
//! This module contains focused unit tests for specific components of the oracle system,
//! testing individual functions and methods in isolation.

use moby_oracle::*;
use moby_oracle::sources::*;
use moby_oracle::aggregation::*;
use moby_oracle::security::*;
use tokio_test;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

mod source_tests {
    use super::*;

    #[test]
    fn test_data_point_creation() {
        let data_point = DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        };

        assert_eq!(data_point.source, DataSource::Chainlink);
        assert_eq!(data_point.symbol, "ETH/USD");
        assert_eq!(data_point.value, Decimal::from(2000));
        assert_eq!(data_point.confidence, 0.95);
        assert!(data_point.volume.is_some());
    }

    #[test]
    fn test_source_config_validation() {
        let config = SourceConfig {
            endpoint_url: "https://api.chainlinklabs.com".to_string(),
            api_key: Some("test_key".to_string()),
            timeout: std::time::Duration::from_secs(30),
            retry_attempts: 3,
            rate_limit: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        assert!(config.enabled);
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.rate_limit, 100);
        assert!(config.api_key.is_some());
    }

    #[test]
    fn test_source_health_status() {
        let health = SourceHealth {
            is_healthy: true,
            last_successful_fetch: Some(Utc::now()),
            consecutive_failures: 0,
            average_response_time: std::time::Duration::from_millis(150),
            error_rate: 0.01,
            status_message: "Operating normally".to_string(),
        };

        assert!(health.is_healthy);
        assert_eq!(health.consecutive_failures, 0);
        assert_eq!(health.error_rate, 0.01);
        assert!(health.last_successful_fetch.is_some());
    }

    #[test]
    fn test_data_source_enum() {
        assert_eq!(DataSource::Chainlink as u8, 0);
        assert_eq!(DataSource::Pyth as u8, 1);
        assert_eq!(DataSource::Band as u8, 2);
        assert_eq!(DataSource::API3 as u8, 3);
        assert_eq!(DataSource::UMA as u8, 4);

        // Test serialization/deserialization would work
        let source = DataSource::Chainlink;
        let serialized = serde_json::to_string(&source);
        assert!(serialized.is_ok());
    }
}

mod aggregation_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_aggregation_strategy_enum() {
        let median = AggregationStrategy::Median;
        let weighted = AggregationStrategy::WeightedAverage;
        let twap = AggregationStrategy::TimeWeightedAverage {
            window: Duration::from_secs(300)
        };

        assert_eq!(median, AggregationStrategy::Median);
        assert_eq!(weighted, AggregationStrategy::WeightedAverage);

        match twap {
            AggregationStrategy::TimeWeightedAverage { window } => {
                assert_eq!(window, Duration::from_secs(300));
            }
            _ => panic!("Expected TimeWeightedAverage variant"),
        }
    }

    #[test]
    fn test_quality_metrics_creation() {
        let mut source_reliability = HashMap::new();
        source_reliability.insert(DataSource::Chainlink, 0.95);
        source_reliability.insert(DataSource::Pyth, 0.90);

        let metrics = QualityMetrics {
            price_variance: Decimal::from_f64_retain(0.01).unwrap(),
            max_spread: Decimal::from_f64_retain(20.0).unwrap(),
            outlier_rate: 0.05,
            avg_latency_ms: 150,
            source_reliability,
        };

        assert_eq!(metrics.outlier_rate, 0.05);
        assert_eq!(metrics.avg_latency_ms, 150);
        assert_eq!(metrics.source_reliability.len(), 2);
        assert_eq!(metrics.source_reliability[&DataSource::Chainlink], 0.95);
    }

    #[test]
    fn test_whale_impact_analysis() {
        let analysis = WhaleImpactAnalysis {
            price_impact_bps: Decimal::from_f64_retain(50.0).unwrap(), // 0.5%
            liquidity_depth: Decimal::from(2_000_000),
            volatility_score: 0.15,
            whale_activity_detected: true,
            max_order_size: Some(Decimal::from(500_000)),
        };

        assert!(analysis.whale_activity_detected);
        assert_eq!(analysis.volatility_score, 0.15);
        assert_eq!(analysis.liquidity_depth, Decimal::from(2_000_000));
        assert!(analysis.max_order_size.is_some());
        assert_eq!(analysis.max_order_size.unwrap(), Decimal::from(500_000));
    }

    #[test]
    fn test_aggregation_config_defaults() {
        let config = AggregationConfig::default();

        assert_eq!(config.strategy, AggregationStrategy::Median);
        assert_eq!(config.min_sources, 3);
        assert!(config.fallback_strategy.is_some());
        assert_eq!(config.max_data_age, Duration::from_secs(300));
        assert_eq!(config.outlier_threshold, 2.0);
    }

    #[test]
    fn test_whale_aggregation_config() {
        let config = WhaleAggregationConfig::default();

        assert_eq!(config.whale_volume_threshold, Decimal::from(1_000_000));
        assert_eq!(config.liquidity_depth_levels, 10);
        assert_eq!(config.volatility_window, Duration::from_secs(3600));

        match config.impact_calculation {
            ImpactCalculationMethod::Linear { coefficient } => {
                assert_eq!(coefficient, 0.001);
            }
            _ => panic!("Expected Linear impact calculation method"),
        }
    }

    #[tokio::test]
    async fn test_aggregator_creation() {
        let config = AggregationConfig::default();
        let aggregator = Aggregator::new(config.clone());

        assert_eq!(aggregator.get_config().min_sources, config.min_sources);
        assert_eq!(aggregator.get_config().strategy, config.strategy);
    }

    #[test]
    fn test_impact_calculation_methods() {
        let linear = ImpactCalculationMethod::Linear { coefficient: 0.001 };
        let sqrt = ImpactCalculationMethod::SquareRoot { coefficient: 0.01 };
        let orderbook = ImpactCalculationMethod::OrderBook;
        let historical = ImpactCalculationMethod::Historical { lookback_periods: 24 };

        match linear {
            ImpactCalculationMethod::Linear { coefficient } => assert_eq!(coefficient, 0.001),
            _ => panic!("Expected Linear method"),
        }

        match sqrt {
            ImpactCalculationMethod::SquareRoot { coefficient } => assert_eq!(coefficient, 0.01),
            _ => panic!("Expected SquareRoot method"),
        }

        match orderbook {
            ImpactCalculationMethod::OrderBook => {}, // Correct
            _ => panic!("Expected OrderBook method"),
        }

        match historical {
            ImpactCalculationMethod::Historical { lookback_periods } => {
                assert_eq!(lookback_periods, 24);
            }
            _ => panic!("Expected Historical method"),
        }
    }
}

mod security_tests {
    use super::*;

    #[test]
    fn test_validation_result_structure() {
        let result = ValidationResult {
            is_valid: true,
            confidence_score: 0.85,
            checks_performed: vec![],
            security_warnings: vec![],
            fraud_risk: FraudRiskLevel::Low,
            integrity_verified: true,
            validated_at: Utc::now(),
        };

        assert!(result.is_valid);
        assert_eq!(result.confidence_score, 0.85);
        assert_eq!(result.fraud_risk, FraudRiskLevel::Low);
        assert!(result.integrity_verified);
    }

    #[test]
    fn test_validation_check_structure() {
        let check = ValidationCheck {
            check_name: "price_range".to_string(),
            passed: true,
            severity: ValidationSeverity::Low,
            details: Some("Price within expected range".to_string()),
            score_impact: 1.0,
        };

        assert_eq!(check.check_name, "price_range");
        assert!(check.passed);
        assert_eq!(check.severity, ValidationSeverity::Low);
        assert_eq!(check.score_impact, 1.0);
        assert!(check.details.is_some());
    }

    #[test]
    fn test_validation_severity_levels() {
        let severities = vec![
            ValidationSeverity::Low,
            ValidationSeverity::Medium,
            ValidationSeverity::High,
            ValidationSeverity::Critical,
        ];

        assert_eq!(severities.len(), 4);
        assert_eq!(severities[0], ValidationSeverity::Low);
        assert_eq!(severities[3], ValidationSeverity::Critical);
    }

    #[test]
    fn test_fraud_risk_levels() {
        let very_low = FraudRiskLevel::VeryLow;
        let low = FraudRiskLevel::Low;
        let medium = FraudRiskLevel::Medium;
        let high = FraudRiskLevel::High;
        let very_high = FraudRiskLevel::VeryHigh;

        // Test ordering
        assert!(very_low < low);
        assert!(low < medium);
        assert!(medium < high);
        assert!(high < very_high);
    }

    #[test]
    fn test_security_warnings() {
        let price_warning = SecurityWarning::PriceManipulationSuspected {
            deviation_percentage: 15.0,
            expected_range: (Decimal::from(1900), Decimal::from(2100)),
        };

        let mev_warning = SecurityWarning::MEVAttackSuspected {
            attack_type: MEVAttackType::Frontrunning,
            confidence: 0.8,
        };

        let stale_warning = SecurityWarning::StaleDataDetected {
            age_seconds: 600,
            max_allowed_age: 300,
        };

        match price_warning {
            SecurityWarning::PriceManipulationSuspected { deviation_percentage, .. } => {
                assert_eq!(deviation_percentage, 15.0);
            }
            _ => panic!("Expected PriceManipulationSuspected"),
        }

        match mev_warning {
            SecurityWarning::MEVAttackSuspected { attack_type, confidence } => {
                assert_eq!(confidence, 0.8);
                assert_eq!(attack_type, MEVAttackType::Frontrunning);
            }
            _ => panic!("Expected MEVAttackSuspected"),
        }

        match stale_warning {
            SecurityWarning::StaleDataDetected { age_seconds, max_allowed_age } => {
                assert_eq!(age_seconds, 600);
                assert_eq!(max_allowed_age, 300);
            }
            _ => panic!("Expected StaleDataDetected"),
        }
    }

    #[test]
    fn test_mev_attack_types() {
        let attack_types = vec![
            MEVAttackType::Frontrunning,
            MEVAttackType::Sandwich,
            MEVAttackType::OracleManipulation,
            MEVAttackType::FlashLoan,
            MEVAttackType::ArbitrageManipulation,
        ];

        assert_eq!(attack_types.len(), 5);

        // Test that each type can be serialized
        for attack_type in attack_types {
            let serialized = serde_json::to_string(&attack_type);
            assert!(serialized.is_ok());
        }
    }

    #[test]
    fn test_data_integrity_structure() {
        let mut source_integrity = HashMap::new();
        source_integrity.insert(DataSource::Chainlink, SourceIntegrityInfo {
            reputation_score: 0.95,
            last_good_data: Utc::now(),
            failure_count: 0,
            circuit_breaker_active: false,
            metadata: HashMap::new(),
        });

        let integrity = DataIntegrity {
            data_hash: "abc123...".to_string(),
            signature: Some("def456...".to_string()),
            public_key: Some("789xyz...".to_string()),
            merkle_proof: None,
            source_integrity,
        };

        assert_eq!(integrity.data_hash, "abc123...");
        assert!(integrity.signature.is_some());
        assert!(integrity.public_key.is_some());
        assert!(integrity.merkle_proof.is_none());
        assert_eq!(integrity.source_integrity.len(), 1);
    }

    #[test]
    fn test_merkle_proof_structure() {
        let proof = MerkleProof {
            root: "root_hash".to_string(),
            proof: vec!["hash1".to_string(), "hash2".to_string()],
            index: 5,
        };

        assert_eq!(proof.root, "root_hash");
        assert_eq!(proof.proof.len(), 2);
        assert_eq!(proof.index, 5);
    }

    #[test]
    fn test_source_integrity_info() {
        let mut metadata = HashMap::new();
        metadata.insert("last_check".to_string(), "2024-01-01T00:00:00Z".to_string());

        let info = SourceIntegrityInfo {
            reputation_score: 0.88,
            last_good_data: Utc::now(),
            failure_count: 2,
            circuit_breaker_active: false,
            metadata,
        };

        assert_eq!(info.reputation_score, 0.88);
        assert_eq!(info.failure_count, 2);
        assert!(!info.circuit_breaker_active);
        assert_eq!(info.metadata.len(), 1);
        assert!(info.metadata.contains_key("last_check"));
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();

        assert_eq!(config.max_price_deviation, 0.05);
        assert_eq!(config.max_data_age, Duration::from_secs(300));
        assert_eq!(config.min_correlation, 0.7);
        assert_eq!(config.mev_detection_sensitivity, 0.6);
        assert_eq!(config.circuit_breaker_threshold, 5);
        assert_eq!(config.reputation_decay_rate, 0.001);
    }

    #[test]
    fn test_fraud_risk_thresholds() {
        let thresholds = FraudRiskThresholds::default();

        assert_eq!(thresholds.very_low, 0.1);
        assert_eq!(thresholds.low, 0.25);
        assert_eq!(thresholds.medium, 0.5);
        assert_eq!(thresholds.high, 0.75);

        // Verify ordering
        assert!(thresholds.very_low < thresholds.low);
        assert!(thresholds.low < thresholds.medium);
        assert!(thresholds.medium < thresholds.high);
    }

    #[test]
    fn test_whale_security_config() {
        let config = WhaleSecurityConfig::default();

        assert_eq!(config.large_order_threshold, Decimal::from(1_000_000));
        assert_eq!(config.price_impact_threshold, 0.02);
        assert!(config.mev_protection_enabled);
        assert_eq!(config.slippage_protection, 0.005);
    }

    #[test]
    fn test_security_validator_creation() {
        let config = SecurityConfig::default();
        let validator = SecurityValidator::new(config.clone());

        assert_eq!(validator.get_config().max_price_deviation, config.max_price_deviation);
        assert_eq!(validator.get_config().min_correlation, config.min_correlation);
    }
}

mod error_tests {
    use super::*;

    #[test]
    fn test_oracle_error_variants() {
        let source_error = OracleError::DataSourceError {
            source: DataSource::Chainlink,
            details: "Connection failed".to_string(),
        };

        let aggregation_error = OracleError::AggregationError {
            strategy: "median".to_string(),
            reason: "Insufficient data".to_string(),
        };

        let validation_error = OracleError::ValidationError {
            check_name: "price_range".to_string(),
            reason: "Price out of range".to_string(),
        };

        match source_error {
            OracleError::DataSourceError { source, details } => {
                assert_eq!(source, DataSource::Chainlink);
                assert_eq!(details, "Connection failed");
            }
            _ => panic!("Expected DataSourceError"),
        }

        match aggregation_error {
            OracleError::AggregationError { strategy, reason } => {
                assert_eq!(strategy, "median");
                assert_eq!(reason, "Insufficient data");
            }
            _ => panic!("Expected AggregationError"),
        }

        match validation_error {
            OracleError::ValidationError { check_name, reason } => {
                assert_eq!(check_name, "price_range");
                assert_eq!(reason, "Price out of range");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_oracle_result_type() {
        let success: OracleResult<i32> = Ok(42);
        let failure: OracleResult<i32> = Err(OracleError::NetworkError {
            message: "Connection timeout".to_string(),
        });

        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        assert!(failure.is_err());
        match failure.unwrap_err() {
            OracleError::NetworkError { message } => {
                assert_eq!(message, "Connection timeout");
            }
            _ => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = OracleError::InvalidPriceValue {
            value: Decimal::from(-100),
        };

        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid price value"));
        assert!(error_string.contains("-100"));
    }
}

mod constants_tests {
    use super::*;

    #[test]
    fn test_oracle_constants() {
        assert_eq!(ORACLE_VERSION, "0.1.0");
        assert_eq!(MAX_DATA_SOURCES, 100);
        assert_eq!(DEFAULT_UPDATE_FREQUENCY_MS, 1000);
        assert_eq!(DEFAULT_DEVIATION_THRESHOLD, 0.005);
        assert_eq!(MAX_HISTORICAL_RETENTION_DAYS, 365);
        assert_eq!(DEFAULT_AGGREGATION_TIMEOUT_MS, 5000);
        assert_eq!(WHALE_VOLUME_THRESHOLD, 1_000_000.0);
    }

    #[test]
    fn test_type_aliases() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let error_result: Result<String> = Err(OracleError::ConfigurationError {
            parameter: "timeout".to_string(),
            value: "invalid".to_string(),
        });
        assert!(error_result.is_err());
    }
}

mod serialization_tests {
    use super::*;

    #[test]
    fn test_data_point_serialization() {
        let data_point = DataPoint {
            source: DataSource::Chainlink,
            symbol: "ETH/USD".to_string(),
            value: Decimal::from(2000),
            timestamp: Utc::now(),
            confidence: 0.95,
            volume: Some(Decimal::from(100_000)),
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&data_point);
        assert!(serialized.is_ok());

        let deserialized: Result<DataPoint, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let recovered = deserialized.unwrap();
        assert_eq!(recovered.source, data_point.source);
        assert_eq!(recovered.symbol, data_point.symbol);
        assert_eq!(recovered.value, data_point.value);
    }

    #[test]
    fn test_aggregation_strategy_serialization() {
        let strategies = vec![
            AggregationStrategy::Median,
            AggregationStrategy::WeightedAverage,
            AggregationStrategy::TimeWeightedAverage {
                window: Duration::from_secs(300)
            },
            AggregationStrategy::VolumeWeightedAverage,
            AggregationStrategy::Consensus {
                min_sources: 3,
                threshold: Decimal::from_f64_retain(0.02).unwrap(),
            },
        ];

        for strategy in strategies {
            let serialized = serde_json::to_string(&strategy);
            assert!(serialized.is_ok());

            let deserialized: Result<AggregationStrategy, _> = serde_json::from_str(&serialized.unwrap());
            assert!(deserialized.is_ok());
        }
    }

    #[test]
    fn test_security_warning_serialization() {
        let warnings = vec![
            SecurityWarning::PriceManipulationSuspected {
                deviation_percentage: 15.0,
                expected_range: (Decimal::from(1900), Decimal::from(2100)),
            },
            SecurityWarning::MEVAttackSuspected {
                attack_type: MEVAttackType::Frontrunning,
                confidence: 0.8,
            },
            SecurityWarning::StaleDataDetected {
                age_seconds: 600,
                max_allowed_age: 300,
            },
        ];

        for warning in warnings {
            let serialized = serde_json::to_string(&warning);
            assert!(serialized.is_ok());

            let deserialized: Result<SecurityWarning, _> = serde_json::from_str(&serialized.unwrap());
            assert!(deserialized.is_ok());
        }
    }
}