use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeRegistryError {
    InvalidPda,
    AccountAlreadyInitialized,
    AccountNotInitialized,
    LeaderInactive,
    LeaderFollowerCapReached,
    InvalidSizeBps,
    Unauthorized,
    InvalidAccountOwner,
    InvalidAccountData,
    ArithmeticOverflow,
}

impl From<TradeRegistryError> for ProgramError {
    fn from(error: TradeRegistryError) -> Self {
        match error {
            TradeRegistryError::InvalidPda => ProgramError::InvalidSeeds,
            TradeRegistryError::AccountAlreadyInitialized => ProgramError::AccountAlreadyInitialized,
            TradeRegistryError::AccountNotInitialized => ProgramError::UninitializedAccount,
            TradeRegistryError::LeaderInactive => ProgramError::Custom(1),
            TradeRegistryError::LeaderFollowerCapReached => ProgramError::Custom(2),
            TradeRegistryError::InvalidSizeBps => ProgramError::InvalidInstructionData,
            TradeRegistryError::Unauthorized => ProgramError::IllegalOwner,
            TradeRegistryError::InvalidAccountOwner => ProgramError::IncorrectProgramId,
            TradeRegistryError::InvalidAccountData => ProgramError::InvalidAccountData,
            TradeRegistryError::ArithmeticOverflow => ProgramError::ArithmeticOverflow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_custom_errors() {
        assert_eq!(
            ProgramError::from(TradeRegistryError::LeaderInactive),
            ProgramError::Custom(1)
        );
    }
}
