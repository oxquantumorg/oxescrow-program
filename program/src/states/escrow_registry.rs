use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use crate::utils::constants::ESCROW_REGISTRY_STATE_LEN;

pub struct EscrowRegistryState {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub token_account_pubkey: Pubkey,
}

impl Sealed for EscrowRegistryState {}

impl IsInitialized for EscrowRegistryState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for EscrowRegistryState {
    const LEN: usize = ESCROW_REGISTRY_STATE_LEN;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, EscrowRegistryState::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            token_account_pubkey
        ) = array_refs![src, 1, 32, 32];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(EscrowRegistryState {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            token_account_pubkey: Pubkey::new_from_array(*token_account_pubkey),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, EscrowRegistryState::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            token_account_pubkey_dst
        ) = mut_array_refs![dst, 1, 32, 32];

        let EscrowRegistryState {
            is_initialized,
            initializer_pubkey,
            token_account_pubkey,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        token_account_pubkey_dst.copy_from_slice(token_account_pubkey.as_ref());
    }
}
