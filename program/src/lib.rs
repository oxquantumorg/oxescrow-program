pub mod instructions;
pub mod utils;
pub mod states;

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
use crate::states::escrow_instruction::EscrowInstruction;
use crate::instructions::default_escrow;

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = EscrowInstruction::unpack(instruction_data)?;

    match instruction {
        EscrowInstruction::InitEscrow { amount } => {
            msg!("Instruction: Init Escrow");
            let _ = default_escrow::init_escrow::handler(accounts, amount, program_id);
        }
        EscrowInstruction::ReleaseEscrow => {
            msg!("Instruction: Release Escrow");
            let _ = default_escrow::release_escrow::handler(accounts, program_id);
        }
        EscrowInstruction::CollectDeposit => {
            msg!("Instruction: Oracle Call");
            let _ = default_escrow::oracle_call::handler(accounts, program_id);
        }
        EscrowInstruction::Oracle => {
            msg!("Instruction: Oracle Call");
            let _ = default_escrow::oracle_call::handler(accounts, program_id);
        }
    }

    Ok(())
}
