use std::convert::TryInto;
use solana_program::program_error::ProgramError;

use crate::error::EscrowError::InvalidInstruction;

pub enum EscrowInstruction {

    /** Starts the trade by creating and populating an escrow 
     account and transferring ownership of the given temp token account to the PDA
    **/
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the escrow
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The receiver account
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The token program
    InitEscrow {
        /// The amount party A is about to send
        amount: u64,
    },

    /// Accepts a trade
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
    ReleaseEscrow,
    Oracle
}


impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::ReleaseEscrow,
            2 => Self::Oracle,
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}