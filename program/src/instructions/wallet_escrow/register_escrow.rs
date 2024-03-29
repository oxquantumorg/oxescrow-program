use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    states::escrow_registry::EscrowRegistryState,
    utils::{self, errors::EscrowError},
};

/** Initialize Escrow
**/
///
/// Accounts expected:
///
/// 0. `[signer]` The account of the person initializing the escrow
/// 1. `[writable]` Token account that should be created prior to this instruction and owned by the initializer
/// 2. `[writable]` The escrow account, it will hold all necessary info about the trade.
/// 3. `[]` The rent sysvar
/// 4. `[]` The token program
pub fn handler(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    msg!("Escrow starting!");
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let temp_token_account = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let token_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
        return Err(EscrowError::NotRentExempt.into());
    }

    msg!("Escrow unpacking!");
    let mut escrow_wallet_registry = EscrowRegistryState::unpack_from_slice(&escrow_account.try_borrow_data()?)?;
    if escrow_wallet_registry.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    escrow_wallet_registry.is_initialized = true;
    escrow_wallet_registry.initializer_pubkey = *initializer.key;
    escrow_wallet_registry.token_account_pubkey = *temp_token_account.key;

    msg!("Escrow packing!");
    EscrowRegistryState::pack(escrow_wallet_registry, &mut escrow_account.try_borrow_mut_data()?)?;
    let (pda, _bump_seed) =
        Pubkey::find_program_address(&[utils::constants::ESCROW_WALLET_SEED], program_id);

    let owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        temp_token_account.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        initializer.key,
        &[&initializer.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    invoke(
        &owner_change_ix,
        &[
            temp_token_account.clone(),
            initializer.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("Escrow have been registered successfully");
    Ok(())
}
