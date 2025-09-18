use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::error::TradeRegistryError;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyAction {
    Buy,
    Sell,
}

impl CopyAction {
    pub fn from_u8(value: u8) -> Result<Self, TradeRegistryError> {
        match value {
            0 => Ok(Self::Buy),
            1 => Ok(Self::Sell),
            _ => Err(TradeRegistryError::InvalidAccountData),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum TradeRegistryInstruction {
    /// Create the global registry PDA (authority-only, once).
    InitializeRegistry,
    /// Register a leader wallet followers can mirror.
    RegisterLeader { max_followers: u32 },
    /// Toggle leader active flag or follower cap.
    UpdateLeader {
        max_followers: Option<u32>,
        is_active: Option<bool>,
    },
    /// Follower subscribes to a leader with size in basis points (1 bps = 0.01%).
    Subscribe { size_bps: u16 },
    /// Follower closes an active subscription.
    Unsubscribe,
    /// Log an on-chain copy-trade intent for audit trail.
    LogCopyIntent {
        action: CopyAction,
        mint: Pubkey,
        amount_lamports: u64,
        reference_sig: [u8; 64],
    },
}

impl TradeRegistryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, TradeRegistryError> {
        Self::try_from_slice(input).map_err(|_| TradeRegistryError::InvalidAccountData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_instructions() {
        let cases = vec![
            TradeRegistryInstruction::InitializeRegistry,
            TradeRegistryInstruction::RegisterLeader { max_followers: 250 },
            TradeRegistryInstruction::UpdateLeader {
                max_followers: Some(500),
                is_active: Some(false),
            },
            TradeRegistryInstruction::Subscribe { size_bps: 2_500 },
            TradeRegistryInstruction::Unsubscribe,
            TradeRegistryInstruction::LogCopyIntent {
                action: CopyAction::Buy,
                mint: Pubkey::new_unique(),
                amount_lamports: 50_000_000,
                reference_sig: [7u8; 64],
            },
        ];

        for instruction in cases {
            let bytes = instruction.try_to_vec().unwrap();
            let decoded = TradeRegistryInstruction::unpack(&bytes).unwrap();
            assert_eq!(decoded, instruction);
        }
    }
}
