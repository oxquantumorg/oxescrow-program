use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub fn handler(_accounts: &[AccountInfo], _program_id: &Pubkey) -> ProgramResult {
    msg!("Register Escrow!");
    Ok(())
}
