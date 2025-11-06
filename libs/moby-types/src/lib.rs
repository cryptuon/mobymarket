pub mod accounts;
pub mod orders;
pub mod events;
pub mod state;
pub mod error;
pub mod utils;

// Re-export commonly used types
pub use accounts::*;
pub use orders::*;
pub use events::*;
pub use state::*;
pub use error::*;
pub use utils::*;

// Re-export external dependencies that clients commonly need
pub use anchor_lang;
pub use solana_program;
pub use moby_math::{Price, SlippageCalculator};

// Common type aliases for whale trading
pub type WhaleAmount = u64;
pub type Timestamp = i64;
pub type AccountKey = solana_program::pubkey::Pubkey;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        // Test that basic types are accessible
        let _amount: WhaleAmount = 1000;
        let _time: Timestamp = chrono::Utc::now().timestamp();
        let _key: AccountKey = AccountKey::default();
    }
}