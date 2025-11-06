//! # Moby Governance 🐋🏛️
//!
//! A comprehensive decentralized governance system for whale trading protocols.
//!
//! This library provides all the necessary components for protocol governance including:
//! - Proposal creation and management
//! - Voting mechanisms with delegation
//! - Governance token and staking systems
//! - Parameter updates and protocol upgrades
//! - Treasury management
//! - Emergency procedures
//!
//! ## Features
//!
//! - **Proposal System**: Create, review, and execute governance proposals
//! - **Voting Mechanisms**: Multiple voting strategies with delegation support
//! - **Token Governance**: Governance token with staking and rewards
//! - **Parameter Management**: Protocol parameter updates through governance
//! - **Treasury Management**: Decentralized treasury with multi-sig controls
//! - **Emergency Controls**: Emergency pause and recovery mechanisms
//! - **Upgrade System**: Protocol upgrade governance with timelock
//!
//! ## Quick Start
//!
//! ```rust
//! use moby_governance::{GovernanceSystem, ProposalType, VotingPower};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize governance system
//!     let governance = GovernanceSystem::new().await?;
//!
//!     // Create a parameter update proposal
//!     let proposal_id = governance.create_proposal(
//!         ProposalType::ParameterUpdate {
//!             parameter: "trading_fee".to_string(),
//!             old_value: "0.003".to_string(),
//!             new_value: "0.002".to_string(),
//!         },
//!         "Reduce trading fees to increase volume".to_string(),
//!         chrono::Utc::now() + chrono::Duration::days(7),
//!     ).await?;
//!
//!     println!("✅ Proposal created: {}", proposal_id);
//!     Ok(())
//! }
//! ```

pub mod proposals;
pub mod voting;
pub mod delegation;
pub mod tokens;
pub mod treasury;
pub mod parameters;
pub mod upgrades;
pub mod emergency;
pub mod system;
pub mod error;

#[cfg(feature = "mock")]
pub mod mock;

// Re-export core types
pub use system::{GovernanceSystem, GovernanceConfig};
pub use proposals::{Proposal, ProposalType, ProposalStatus, ProposalId};
pub use voting::{Vote, VoteType, VotingPower, VotingResult, VotingStrategy};
pub use delegation::{Delegation, DelegationPower, DelegationTarget};
pub use tokens::{GovernanceToken, TokenAmount, StakingPool, StakingReward};
pub use treasury::{Treasury, TreasuryAction, TreasuryProposal};
pub use parameters::{ParameterManager, ParameterUpdate, ParameterValue};
pub use upgrades::{UpgradeManager, UpgradeProposal, UpgradeType};
pub use emergency::{EmergencySystem, EmergencyAction, EmergencyRole};
pub use error::{GovernanceError, GovernanceResult};

/// Result type alias for governance operations
pub type Result<T> = std::result::Result<T, GovernanceError>;

/// Current version of the governance system
pub const GOVERNANCE_VERSION: &str = "0.1.0";

/// Maximum number of active proposals
pub const MAX_ACTIVE_PROPOSALS: usize = 100;

/// Default voting period in seconds (7 days)
pub const DEFAULT_VOTING_PERIOD: u64 = 7 * 24 * 60 * 60;

/// Minimum voting power required to create proposal
pub const MIN_PROPOSAL_POWER: u64 = 1000;

/// Quorum percentage required for proposal to pass
pub const QUORUM_PERCENTAGE: u8 = 10;

/// Time lock duration for parameter updates (24 hours)
pub const TIMELOCK_DURATION: u64 = 24 * 60 * 60;