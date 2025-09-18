use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::system_program;

use crate::{
    find_leader_pda, find_registry_pda, find_subscription_pda, CopyAction,
    TradeRegistryInstruction,
};

/// Account metas required for each on-chain instruction (for client / worker builders).
#[derive(Debug, Clone)]
pub struct InstructionAccounts {
    pub metas: Vec<AccountMeta>,
    pub data: Vec<u8>,
}

pub struct RegistryTransactionBuilder {
    program_id: Pubkey,
}

impl RegistryTransactionBuilder {
    pub fn new(program_id: Pubkey) -> Self {
        Self { program_id }
    }

    pub fn initialize_registry(
        &self,
        payer: Pubkey,
        authority: Pubkey,
    ) -> InstructionAccounts {
        let (registry, _) = find_registry_pda(&self.program_id);
        let data = borsh::to_vec(&TradeRegistryInstruction::InitializeRegistry).expect("encode");

        InstructionAccounts {
            metas: vec![
                AccountMeta::new(payer, true),
                AccountMeta::new(registry, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        }
    }

    pub fn register_leader(
        &self,
        payer: Pubkey,
        registrar: Pubkey,
        leader: Pubkey,
        max_followers: u32,
    ) -> InstructionAccounts {
        let (registry, _) = find_registry_pda(&self.program_id);
        let (leader_profile, _) = find_leader_pda(&self.program_id, &leader);
        let data = borsh::to_vec(&TradeRegistryInstruction::RegisterLeader { max_followers })
            .expect("encode");

        InstructionAccounts {
            metas: vec![
                AccountMeta::new(payer, true),
                AccountMeta::new(registry, false),
                AccountMeta::new_readonly(leader, false),
                AccountMeta::new(leader_profile, false),
                AccountMeta::new_readonly(registrar, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        }
    }

    pub fn subscribe(
        &self,
        payer: Pubkey,
        follower: Pubkey,
        leader: Pubkey,
        size_bps: u16,
    ) -> InstructionAccounts {
        let (leader_profile, _) = find_leader_pda(&self.program_id, &leader);
        let (subscription, _) = find_subscription_pda(&self.program_id, &follower, &leader);
        let data = borsh::to_vec(&TradeRegistryInstruction::Subscribe { size_bps }).expect("encode");

        InstructionAccounts {
            metas: vec![
                AccountMeta::new(payer, true),
                AccountMeta::new(leader_profile, false),
                AccountMeta::new(subscription, false),
                AccountMeta::new_readonly(follower, true),
                AccountMeta::new_readonly(leader, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        }
    }

    pub fn log_copy_intent(
        &self,
        follower: Pubkey,
        leader: Pubkey,
        action: CopyAction,
        mint: Pubkey,
        amount_lamports: u64,
        reference_sig: [u8; 64],
    ) -> InstructionAccounts {
        let (registry, _) = find_registry_pda(&self.program_id);
        let (leader_profile, _) = find_leader_pda(&self.program_id, &leader);
        let (subscription, _) = find_subscription_pda(&self.program_id, &follower, &leader);

        let data = borsh::to_vec(&TradeRegistryInstruction::LogCopyIntent {
            action,
            mint,
            amount_lamports,
            reference_sig,
        })
        .expect("encode");

        InstructionAccounts {
            metas: vec![
                AccountMeta::new(registry, false),
                AccountMeta::new(leader_profile, false),
                AccountMeta::new(subscription, false),
                AccountMeta::new_readonly(follower, true),
            ],
            data,
        }
    }

    pub fn into_instruction(&self, accounts: InstructionAccounts) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: accounts.metas,
            data: accounts.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_initialize_accounts() {
        let program_id = Pubkey::new_unique();
        let builder = RegistryTransactionBuilder::new(program_id);
        let ix = builder.initialize_registry(Pubkey::new_unique(), Pubkey::new_unique());
        assert_eq!(ix.metas.len(), 4);
        assert!(!ix.data.is_empty());
    }

    #[test]
    fn builds_log_copy_intent() {
        let program_id = Pubkey::new_unique();
        let builder = RegistryTransactionBuilder::new(program_id);
        let ix = builder.log_copy_intent(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            CopyAction::Buy,
            Pubkey::new_unique(),
            1_000_000,
            [0u8; 64],
        );
        assert_eq!(ix.metas.len(), 4);
    }
}
