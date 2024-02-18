use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use crate::utils::constants::DEFAULT_ESCROW_STATE_LEN;

pub struct EscrowState {
    pub is_initialized: bool,
    pub caller_pubkey: Pubkey,
    pub initializer_pubkey: Pubkey,
    pub receiver_pubkey: Pubkey,
    pub temp_token_account_pubkey: Pubkey,
    pub escrow_amount: u64,
    pub expire_date: i64,
}

impl Sealed for EscrowState {}

impl IsInitialized for EscrowState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for EscrowState {
    const LEN: usize = DEFAULT_ESCROW_STATE_LEN;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, EscrowState::LEN];
        let (
            is_initialized,
            caller_pubkey,
            initializer_pubkey,
            receiver_pubkey,
            temp_token_account_pubkey,
            escrow_amount,
            expire_date,
        ) = array_refs![src, 1, 32, 32, 32, 32, 8, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(EscrowState {
            is_initialized,
            caller_pubkey: Pubkey::new_from_array(*caller_pubkey),
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            receiver_pubkey: Pubkey::new_from_array(*receiver_pubkey),
            temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
            escrow_amount: u64::from_le_bytes(*escrow_amount),
            expire_date: i64::from_le_bytes(*expire_date),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, EscrowState::LEN];
        let (
            is_initialized_dst,
            caller_pubkey_dst,
            initializer_pubkey_dst,
            receiver_pubkey_dst,
            temp_token_account_pubkey_dst,
            escrow_amount_dst,
            expire_date_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 32, 8, 8];

        let EscrowState {
            is_initialized,
            caller_pubkey,
            initializer_pubkey,
            receiver_pubkey,
            temp_token_account_pubkey,
            escrow_amount,
            expire_date,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        caller_pubkey_dst.copy_from_slice(caller_pubkey.as_ref());
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        receiver_pubkey_dst.copy_from_slice(receiver_pubkey.as_ref());
        temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
        *escrow_amount_dst = escrow_amount.to_le_bytes();
        *expire_date_dst = expire_date.to_le_bytes();
    }
}
