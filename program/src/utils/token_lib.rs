use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program::invoke_signed, pubkey::Pubkey,
};

use spl_token::instruction::AuthorityType;

use crate::utils::constants;

pub fn change_account_authority(
    token_program_account: AccountInfo,
    sender_token_account: AccountInfo,
    new_authority_pubkey: Pubkey,
    authority_type: AuthorityType,
    owner_account: AccountInfo,
    account_infos: &[AccountInfo],
) -> ProgramResult {
    let instruction = spl_token::instruction::set_authority(
        token_program_account.key,
        sender_token_account.key,
        Some(&new_authority_pubkey),
        authority_type,
        owner_account.key,
        &[&owner_account.key],
    )?;

    msg!("Calling the token program to transfer token account ownership...");
    invoke(&instruction, account_infos)?;

    Ok(())
}

pub fn transfer_tokens(
    token_program_account: AccountInfo,
    sender_token_account: AccountInfo,
    receiver_token_account: AccountInfo,
    authority_pubkey: Pubkey,
    amount: u64,
    account_infos: &[AccountInfo],
    bump_seed: u8,
) -> ProgramResult {
    let instruction = spl_token::instruction::transfer(
        &token_program_account.key,
        &sender_token_account.key,
        &receiver_token_account.key,
        &authority_pubkey,
        &[&authority_pubkey],
        amount,
    )?;
    msg!("Calling the token program to transfer tokens to the taker...");
    invoke_signed(
        &instruction,
        account_infos,
        &[&[&constants::ESCROW_SEED[..], &[bump_seed]]],
    )?;

    Ok(())
}
