use solana_program::program_error::ProgramError;
use crate::utils::errors::EscrowError::InvalidInstruction;

pub enum EscrowRoutes {
    InitEscrow { amount: u64 },
    ReleaseEscrow,
    CollectDeposit,
    Oracle,
}

impl EscrowRoutes {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::ReleaseEscrow,
            2 => Self::CollectDeposit,
            3 => Self::Oracle,
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        if input.len() == 1 {
            Ok(u64::from(input[0]))
        } else {
            Err(InvalidInstruction.into())
        }
    }
}
