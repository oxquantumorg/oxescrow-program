use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    states::default_escrow::EscrowState,
    utils::{self, errors::EscrowError, token_lib},
};

/** Initialize Escrow
**/
///
/// Accounts expected:
///
/// 0. `[]` The account of the person initializing the escrow
/// 1. `[]` The account of the receiver
/// 2. `[writable]` Temporary token account that should be created prior to this instruction and owned by the caller
/// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
/// 4. `[]` The rent sysvar
/// 5. `[]` The token program
/// 6. `[signer]` The caller / relayer
pub fn handler(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    msg!("Escrow starting!");
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let receiver_account = next_account_info(account_info_iter)?;
    let temp_token_account = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let token_program = next_account_info(account_info_iter)?;
    let caller = next_account_info(account_info_iter)?;

    if !caller.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
        return Err(EscrowError::NotRentExempt.into());
    }

    msg!("Escrow unpacking!");
    let mut escrow_info = EscrowState::unpack_from_slice(&escrow_account.try_borrow_data()?)?;
    if escrow_info.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    escrow_info.is_initialized = true;
    escrow_info.caller_pubkey = *caller.key;
    escrow_info.initializer_pubkey = *initializer.key;
    escrow_info.receiver_pubkey = *receiver_account.key;
    escrow_info.temp_token_account_pubkey = *temp_token_account.key;
    escrow_info.escrow_amount = amount;
    escrow_info.expire_date = Clock::get().unwrap().unix_timestamp + utils::constants::ESCROW_WAIT_TIME_SEC;

    msg!("Escrow packing!");
    EscrowState::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;
    let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

    let authority_type = spl_token::instruction::AuthorityType::AccountOwner;
    let account_infos = &[
        temp_token_account.clone(),
        caller.clone(),
        token_program.clone(),
    ];

    let _ = token_lib::change_account_authority(
        token_program.clone(),
        temp_token_account.clone(),
        pda,
        authority_type,
        caller.clone(),
        account_infos,
    );

    msg!("Escrow creation was successful");
    Ok(())
}
