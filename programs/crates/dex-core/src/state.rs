use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const REGISTRY_SEED: &[u8] = b"registry";
pub const LEADER_SEED: &[u8] = b"leader";
pub const SUBSCRIPTION_SEED: &[u8] = b"sub";

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccountType {
    Uninitialized = 0,
    RegistryConfig = 1,
    LeaderProfile = 2,
    CopySubscription = 3,
}

impl AccountType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Uninitialized),
            1 => Some(Self::RegistryConfig),
            2 => Some(Self::LeaderProfile),
            3 => Some(Self::CopySubscription),
            _ => None,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegistryConfig {
    pub account_type: AccountType,
    pub authority: Pubkey,
    pub leader_count: u32,
    pub total_intents_logged: u64,
    pub bump: u8,
}

impl RegistryConfig {
    pub const LEN: usize = 1 + 32 + 4 + 8 + 1;

    pub fn new(authority: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::RegistryConfig,
            authority,
            leader_count: 0,
            total_intents_logged: 0,
            bump,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct LeaderProfile {
    pub account_type: AccountType,
    pub leader: Pubkey,
    pub registrar: Pubkey,
    pub follower_count: u32,
    pub max_followers: u32,
    pub total_intents_logged: u64,
    pub is_active: bool,
    pub bump: u8,
}

impl LeaderProfile {
    pub const LEN: usize = 1 + 32 + 32 + 4 + 4 + 8 + 1 + 1;

    pub fn new(leader: Pubkey, registrar: Pubkey, max_followers: u32, bump: u8) -> Self {
        Self {
            account_type: AccountType::LeaderProfile,
            leader,
            registrar,
            follower_count: 0,
            max_followers,
            total_intents_logged: 0,
            is_active: true,
            bump,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct CopySubscription {
    pub account_type: AccountType,
    pub follower: Pubkey,
    pub leader: Pubkey,
    pub size_bps: u16,
    pub is_active: bool,
    pub intents_logged: u64,
    pub last_reference_sig: [u8; 64],
    pub bump: u8,
}

impl CopySubscription {
    pub const LEN: usize = 1 + 32 + 32 + 2 + 1 + 8 + 64 + 1;

    pub fn new(follower: Pubkey, leader: Pubkey, size_bps: u16, bump: u8) -> Self {
        Self {
            account_type: AccountType::CopySubscription,
            follower,
            leader,
            size_bps,
            is_active: true,
            intents_logged: 0,
            last_reference_sig: [0u8; 64],
            bump,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;

    #[test]
    fn account_sizes_match_borsh_encoding() {
        let registry = RegistryConfig::new(Pubkey::new_unique(), 255);
        let leader = LeaderProfile::new(Pubkey::new_unique(), Pubkey::new_unique(), 100, 254);
        let sub = CopySubscription::new(Pubkey::new_unique(), Pubkey::new_unique(), 5_000, 253);

        assert_eq!(registry.try_to_vec().unwrap().len(), RegistryConfig::LEN);
        assert_eq!(leader.try_to_vec().unwrap().len(), LeaderProfile::LEN);
        assert_eq!(sub.try_to_vec().unwrap().len(), CopySubscription::LEN);
    }
}
