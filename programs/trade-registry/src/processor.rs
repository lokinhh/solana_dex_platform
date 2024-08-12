use borsh::{BorshDeserialize, BorshSerialize};
use dex_core::{
    assert_pda, AccountType,
    CopyAction, CopySubscription, LeaderProfile, RegistryConfig, TradeRegistryError,
    TradeRegistryInstruction, LEADER_SEED, REGISTRY_SEED, SUBSCRIPTION_SEED,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction: TradeRegistryInstruction,
    ) -> ProgramResult {
        match instruction {
            TradeRegistryInstruction::InitializeRegistry => {
                Self::initialize_registry(program_id, accounts)
            }
            TradeRegistryInstruction::RegisterLeader { max_followers } => {
                Self::register_leader(program_id, accounts, max_followers)
            }
            TradeRegistryInstruction::UpdateLeader {
                max_followers,
                is_active,
            } => Self::update_leader(program_id, accounts, max_followers, is_active),
            TradeRegistryInstruction::Subscribe { size_bps } => {
                Self::subscribe(program_id, accounts, size_bps)
            }
            TradeRegistryInstruction::Unsubscribe => Self::unsubscribe(program_id, accounts),
            TradeRegistryInstruction::LogCopyIntent {
                action,
                mint,
                amount_lamports,
                reference_sig,
            } => Self::log_copy_intent(
                program_id,
                accounts,
                action,
                mint,
                amount_lamports,
                reference_sig,
            ),
        }
    }

    fn initialize_registry(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let registry = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if registry.owner != &solana_program::system_program::id() && registry.owner != program_id
        {
            return Err(TradeRegistryError::InvalidAccountOwner.into());
        }
        if !registry.data_is_empty() {
            return Err(TradeRegistryError::AccountAlreadyInitialized.into());
        }

        let bump = assert_pda(registry.key, &[REGISTRY_SEED], program_id)?;
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(RegistryConfig::LEN);
        let seeds: &[&[u8]] = &[REGISTRY_SEED, &[bump]];

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                registry.key,
                lamports,
                RegistryConfig::LEN as u64,
                program_id,
            ),
            &[payer.clone(), registry.clone(), system_program.clone()],
            &[seeds],
        )?;

        let config = RegistryConfig::new(*authority.key, bump);
        config.serialize(&mut &mut registry.data.borrow_mut()[..])?;

        msg!("TradeRegistry: registry initialized");
        Ok(())
    }

    fn register_leader(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        max_followers: u32,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let registry = next_account_info(account_info_iter)?;
        let leader_account = next_account_info(account_info_iter)?;
        let leader_profile = next_account_info(account_info_iter)?;
        let registrar = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !registrar.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut registry_data = load_registry(registry, program_id)?;
        if registry_data.authority != *registrar.key {
            return Err(TradeRegistryError::Unauthorized.into());
        }

        if max_followers == 0 {
            return Err(TradeRegistryError::InvalidAccountData.into());
        }
        if !leader_profile.data_is_empty() {
            return Err(TradeRegistryError::AccountAlreadyInitialized.into());
        }

        let bump = assert_pda(
            leader_profile.key,
            &[LEADER_SEED, leader_account.key.as_ref()],
            program_id,
        )?;
        create_pda_account(
            payer,
            leader_profile,
            system_program,
            program_id,
            LeaderProfile::LEN,
            &[LEADER_SEED, leader_account.key.as_ref(), &[bump]],
        )?;

        let profile = LeaderProfile::new(*leader_account.key, *registrar.key, max_followers, bump);
        profile.serialize(&mut &mut leader_profile.data.borrow_mut()[..])?;

        registry_data.leader_count = registry_data
            .leader_count
            .checked_add(1)
            .ok_or(TradeRegistryError::ArithmeticOverflow)?;
        registry_data.serialize(&mut &mut registry.data.borrow_mut()[..])?;

        msg!("TradeRegistry: leader registered {}", leader_account.key);
        Ok(())
    }

    fn update_leader(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        max_followers: Option<u32>,
        is_active: Option<bool>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let registry = next_account_info(account_info_iter)?;
        let leader_profile = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;

        if !authority.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let registry_data = load_registry(registry, program_id)?;
        if registry_data.authority != *authority.key {
            return Err(TradeRegistryError::Unauthorized.into());
        }

        let mut profile = load_leader(leader_profile, program_id)?;
        if let Some(cap) = max_followers {
            if cap == 0 || cap < profile.follower_count {
                return Err(TradeRegistryError::InvalidAccountData.into());
            }
            profile.max_followers = cap;
        }
        if let Some(active) = is_active {
            profile.is_active = active;
        }
        profile.serialize(&mut &mut leader_profile.data.borrow_mut()[..])?;

        msg!(
            "TradeRegistry: leader {} updated active={}",
            profile.leader,
            profile.is_active
        );
        Ok(())
    }

    fn subscribe(program_id: &Pubkey, accounts: &[AccountInfo], size_bps: u16) -> ProgramResult {
        if size_bps == 0 || size_bps > 10_000 {
            return Err(TradeRegistryError::InvalidSizeBps.into());
        }

        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let leader_profile = next_account_info(account_info_iter)?;
        let subscription = next_account_info(account_info_iter)?;
        let follower = next_account_info(account_info_iter)?;
        let leader = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !follower.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let profile = load_leader(leader_profile, program_id)?;
        if !profile.is_active {
            return Err(TradeRegistryError::LeaderInactive.into());
        }
        if profile.leader != *leader.key {
            return Err(TradeRegistryError::InvalidAccountData.into());
        }
        if profile.follower_count >= profile.max_followers {
            return Err(TradeRegistryError::LeaderFollowerCapReached.into());
        }
        if !subscription.data_is_empty() {
            return Err(TradeRegistryError::AccountAlreadyInitialized.into());
        }

        let bump = assert_pda(
            subscription.key,
            &[
                SUBSCRIPTION_SEED,
                follower.key.as_ref(),
                leader.key.as_ref(),
            ],
            program_id,
        )?;
        create_pda_account(
            payer,
            subscription,
            system_program,
            program_id,
            CopySubscription::LEN,
            &[
                SUBSCRIPTION_SEED,
                follower.key.as_ref(),
                leader.key.as_ref(),
                &[bump],
            ],
        )?;

        let sub = CopySubscription::new(*follower.key, *leader.key, size_bps, bump);
        sub.serialize(&mut &mut subscription.data.borrow_mut()[..])?;

        let mut profile = profile;
        profile.follower_count = profile
            .follower_count
            .checked_add(1)
            .ok_or(TradeRegistryError::ArithmeticOverflow)?;
        profile.serialize(&mut &mut leader_profile.data.borrow_mut()[..])?;

        msg!(
            "TradeRegistry: {} subscribed to {} at {} bps",
            follower.key,
            leader.key,
            size_bps
        );
        Ok(())
    }

    fn unsubscribe(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let leader_profile = next_account_info(account_info_iter)?;
        let subscription = next_account_info(account_info_iter)?;
        let follower = next_account_info(account_info_iter)?;

        if !follower.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut sub = load_subscription(subscription, program_id)?;
        if sub.follower != *follower.key {
            return Err(TradeRegistryError::Unauthorized.into());
        }
        if !sub.is_active {
            return Ok(());
        }

        sub.is_active = false;
        sub.serialize(&mut &mut subscription.data.borrow_mut()[..])?;

        let mut profile = load_leader(leader_profile, program_id)?;
        if profile.leader != sub.leader {
            return Err(TradeRegistryError::InvalidAccountData.into());
        }
        profile.follower_count = profile.follower_count.saturating_sub(1);
        profile.serialize(&mut &mut leader_profile.data.borrow_mut()[..])?;

        msg!("TradeRegistry: {} unsubscribed from {}", sub.follower, sub.leader);
        Ok(())
    }

    fn log_copy_intent(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        action: CopyAction,
        mint: Pubkey,
        amount_lamports: u64,
        reference_sig: [u8; 64],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let registry = next_account_info(account_info_iter)?;
        let leader_profile = next_account_info(account_info_iter)?;
        let subscription = next_account_info(account_info_iter)?;
        let follower = next_account_info(account_info_iter)?;

        if !follower.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut registry_data = load_registry(registry, program_id)?;
        let mut profile = load_leader(leader_profile, program_id)?;
        let mut sub = load_subscription(subscription, program_id)?;

        if sub.follower != *follower.key || !sub.is_active {
            return Err(TradeRegistryError::Unauthorized.into());
        }
        if profile.leader != sub.leader {
            return Err(TradeRegistryError::InvalidAccountData.into());
        }
        if !profile.is_active {
            return Err(TradeRegistryError::LeaderInactive.into());
        }

        sub.intents_logged = sub
            .intents_logged
            .checked_add(1)
            .ok_or(TradeRegistryError::ArithmeticOverflow)?;
        sub.last_reference_sig = reference_sig;
        sub.serialize(&mut &mut subscription.data.borrow_mut()[..])?;

        profile.total_intents_logged = profile
            .total_intents_logged
            .checked_add(1)
            .ok_or(TradeRegistryError::ArithmeticOverflow)?;
        profile.serialize(&mut &mut leader_profile.data.borrow_mut()[..])?;

        registry_data.total_intents_logged = registry_data
            .total_intents_logged
            .checked_add(1)
            .ok_or(TradeRegistryError::ArithmeticOverflow)?;
        registry_data.serialize(&mut &mut registry.data.borrow_mut()[..])?;

        let action_label = match action {
            CopyAction::Buy => "COPY_BUY",
            CopyAction::Sell => "COPY_SELL",
        };
        msg!(
            "TradeRegistry: {} follower={} leader={} mint={} lamports={}",
            action_label,
            sub.follower,
            sub.leader,
            mint,
            amount_lamports
        );
        Ok(())
    }
}

fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    new_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    program_id: &Pubkey,
    space: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);
    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            new_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[payer.clone(), new_account.clone(), system_program.clone()],
        &[signer_seeds],
    )
}

fn load_registry(account: &AccountInfo, program_id: &Pubkey) -> Result<RegistryConfig, ProgramError> {
    if account.owner != program_id {
        return Err(TradeRegistryError::InvalidAccountOwner.into());
    }
    let data = account.try_borrow_data()?;
    let config = RegistryConfig::try_from_slice(&data)
        .map_err(|_| TradeRegistryError::InvalidAccountData)?;
    if config.account_type != AccountType::RegistryConfig {
        return Err(TradeRegistryError::InvalidAccountData.into());
    }
    Ok(config)
}

fn load_leader(account: &AccountInfo, program_id: &Pubkey) -> Result<LeaderProfile, ProgramError> {
    if account.owner != program_id {
        return Err(TradeRegistryError::InvalidAccountOwner.into());
    }
    let data = account.try_borrow_data()?;
    let profile = LeaderProfile::try_from_slice(&data)
        .map_err(|_| TradeRegistryError::InvalidAccountData)?;
    if profile.account_type != AccountType::LeaderProfile {
        return Err(TradeRegistryError::InvalidAccountData.into());
    }
    Ok(profile)
}

fn load_subscription(
    account: &AccountInfo,
    program_id: &Pubkey,
) -> Result<CopySubscription, ProgramError> {
    if account.owner != program_id {
        return Err(TradeRegistryError::InvalidAccountOwner.into());
    }
    let data = account.try_borrow_data()?;
    let sub = CopySubscription::try_from_slice(&data)
        .map_err(|_| TradeRegistryError::InvalidAccountData)?;
    if sub.account_type != AccountType::CopySubscription {
        return Err(TradeRegistryError::InvalidAccountData.into());
    }
    Ok(sub)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dex_core::find_registry_pda;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn registry_pda_matches_helper() {
        let program_id = Pubkey::new_unique();
        let (expected, _) = find_registry_pda(&program_id);
        let bump = assert_pda(&expected, &[REGISTRY_SEED], &program_id).unwrap();
        assert!(bump <= 255);
    }
}
