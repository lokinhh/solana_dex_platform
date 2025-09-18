pub mod error;
pub mod instruction;
pub mod pda;
pub mod state;
pub mod transaction;
pub mod validation;

pub use error::TradeRegistryError;
pub use instruction::{CopyAction, TradeRegistryInstruction};
pub use pda::{find_leader_pda, find_registry_pda, find_subscription_pda};
pub use state::{
    AccountType, CopySubscription, LeaderProfile, RegistryConfig, REGISTRY_SEED, LEADER_SEED,
    SUBSCRIPTION_SEED,
};
pub use transaction::{InstructionAccounts, RegistryTransactionBuilder};
pub use validation::{
    scale_copy_amount_lamports, validate_leader_cap, validate_lamports, validate_subscribe_bps,
    MAX_FOLLOWERS_CAP, MAX_SUBSCRIBE_BPS, MIN_SUBSCRIBE_BPS,
};
