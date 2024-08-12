//! Solana on-chain program — copy-trade registry with PDAs for leaders and subscriptions.

mod processor;

use dex_core::TradeRegistryInstruction;
use processor::Processor;
use solana_program::{account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TradeRegistryInstruction::unpack(instruction_data)?;
    Processor::process(program_id, accounts, instruction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dex_core::{CopyAction, TradeRegistryInstruction};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn routes_copy_buy_intent() {
        let instruction = TradeRegistryInstruction::LogCopyIntent {
            action: CopyAction::Buy,
            mint: Pubkey::new_unique(),
            amount_lamports: 1,
            reference_sig: [0u8; 64],
        };
        let bytes = borsh::to_vec(&instruction).unwrap();
        // Missing accounts should fail, not panic.
        let result = process_instruction(&Pubkey::new_unique(), &[], &bytes);
        assert!(result.is_err());
    }
}
