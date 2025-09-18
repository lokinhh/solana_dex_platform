use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::{
    error::TradeRegistryError,
    state::{LEADER_SEED, REGISTRY_SEED, SUBSCRIPTION_SEED},
};

pub fn find_registry_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REGISTRY_SEED], program_id)
}

pub fn find_leader_pda(program_id: &Pubkey, leader: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[LEADER_SEED, leader.as_ref()], program_id)
}

pub fn find_subscription_pda(
    program_id: &Pubkey,
    follower: &Pubkey,
    leader: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SUBSCRIPTION_SEED, follower.as_ref(), leader.as_ref()],
        program_id,
    )
}

pub fn assert_pda(
    account: &Pubkey,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<u8, TradeRegistryError> {
    let (expected, bump) = Pubkey::find_program_address(seeds, program_id);
    if account == &expected {
        Ok(bump)
    } else {
        Err(TradeRegistryError::InvalidPda)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdas_are_deterministic() {
        let program_id = Pubkey::new_unique();
        let leader = Pubkey::new_unique();
        let follower = Pubkey::new_unique();

        let (registry_a, bump_a) = find_registry_pda(&program_id);
        let (registry_b, _) = find_registry_pda(&program_id);
        assert_eq!(registry_a, registry_b);
        assert!(bump_a <= 255);

        let (leader_pda, _) = find_leader_pda(&program_id, &leader);
        assert_ne!(leader_pda, registry_a);

        let (sub_pda, _) = find_subscription_pda(&program_id, &follower, &leader);
        assert_ne!(sub_pda, leader_pda);
    }
}
