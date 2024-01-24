use solana_program::{
    account_info::AccountInfo,
    pubkey::Pubkey,
    msg,
    entrypoint::ProgramResult
};

pub fn handler(
    _accounts: &[AccountInfo],
    _program_id: &Pubkey,
) -> ProgramResult {
    msg!("Oracle Call!");
    Ok(())
}
