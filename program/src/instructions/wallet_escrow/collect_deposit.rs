use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, clock::Clock,
    entrypoint::ProgramResult, msg, program::invoke_signed, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    states::default_escrow::EscrowState,
    utils::{errors::EscrowError, token_lib, constants},
};
use spl_token::state::Account as TokenAccount;

/// Release Escrow Funds
///
///
/// Accounts expected:
/// 0. `[signer]` The account of the person taking the trade
/// 1. `[writable]` The taker's token account for the token they will receive should the trade go through
/// 2. `[writable]` The PDA's temp token account to get tokens from and eventually close
/// 3. `[writable]` The initializer's main account to send their rent fees to
/// 4. `[writable]` The escrow account holding the escrow info
/// 5. `[]` The token program
/// 6. `[]` The PDA account
pub fn handler(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    msg!("Release Escrow starting!");

    // let taker_account = next_account_info(account_info_iter)?;
    let receiver_token_account = next_account_info(account_info_iter)?;
    let pdas_temp_token_account = next_account_info(account_info_iter)?;
    let initializers_main_account = next_account_info(account_info_iter)?;
    let escrow_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[constants::ESCROW_SEED], program_id);
    msg!("Unpacking Escrow!");
    let escrow_info = EscrowState::unpack_from_slice(&escrow_account.try_borrow_data()?)?;
    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let receiver_token_account_info =
        TokenAccount::unpack(&receiver_token_account.try_borrow_data()?)?;

    // if !taker_account.is_signer {
    //     return Err(ProgramError::MissingRequiredSignature);
    // }

    msg!("Running Checks!");
    if current_timestamp < escrow_info.expire_date {
        return Err(EscrowError::EscrowNotMaturedYet.into());
    }

    msg!("Running Checks: Receiver PubKey! {:?}", receiver_token_account_info);
    if escrow_info.receiver_pubkey != receiver_token_account_info.owner {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Running Checks: Pda Token PubKey!");
    if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Running Checks: Initializer PubKey!");
    if escrow_info.initializer_pubkey != *initializers_main_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Transfer Tokens!");
    let amount = escrow_info.escrow_amount;
    let account_infos = &[
        pdas_temp_token_account.clone(),
        receiver_token_account.clone(),
        pda_account.clone(),
        token_program.clone(),
    ];

    let _ = token_lib::transfer_tokens(
        token_program.clone(),
        pdas_temp_token_account.clone(),
        receiver_token_account.clone(),
        pda,
        amount,
        account_infos,
        bump_seed,
    );

    msg!("Closing Token Account!");
    let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
        token_program.key,
        pdas_temp_token_account.key,
        initializers_main_account.key,
        &pda,
        &[&pda],
    )?;
    msg!("Calling the token program to close pda's temp account...");
    invoke_signed(
        &close_pdas_temp_acc_ix,
        &[
            pdas_temp_token_account.clone(),
            initializers_main_account.clone(),
            pda_account.clone(),
            token_program.clone(),
        ],
        &[&[&constants::ESCROW_SEED[..], &[bump_seed]]],
    )?;

    msg!("Closing the escrow account...");
    **initializers_main_account.lamports.borrow_mut() = initializers_main_account
        .lamports()
        .checked_add(escrow_account.lamports())
        .ok_or(EscrowError::AmountOverflow)?;
    **escrow_account.lamports.borrow_mut() = 0;
    *escrow_account.try_borrow_mut_data()? = &mut [];

    msg!("Escrow released successfully");
    Ok(())
}
