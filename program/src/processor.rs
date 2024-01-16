use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke, invoke_signed}, clock::Clock
};

use crate::{instruction::EscrowInstruction, error::EscrowError, state::Escrow};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> Result<Escrow, ProgramError> {
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                Self::process_init_escrow(accounts, amount, program_id)
            },
            EscrowInstruction::ReleaseEscrow => {
                msg!("Instruction: ReleaseEscrow");
                Self::process_release_escrow(accounts, program_id)
            },
            EscrowInstruction::Oracle => {
                msg!("Instruction: Oracle");
                Self::process_echo_oracle()
            }
        }
    }
    
    fn process_echo_oracle() -> Result<Escrow, ProgramError> {
        msg!("Echo from oracle!");
        let empty_escrow = Escrow::default(); 
        Ok(empty_escrow)
    }
    
    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> Result<Escrow, ProgramError>  {
        msg!("Escrow starting!");
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let temp_token_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        msg!("Escrow unpacking!");
        let mut escrow_info = Escrow::unpack_from_slice(&escrow_account.try_borrow_data()?)?;
        let mut escrow_info_copy = Escrow::unpack_from_slice(&escrow_account.try_borrow_data()?)?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.temp_token_account_pubkey = *temp_token_account.key;
        escrow_info.escrow_amount = amount;
        escrow_info.expire_date = Clock::get().unwrap().unix_timestamp + 20000;

        escrow_info_copy.is_initialized = true;
        escrow_info_copy.initializer_pubkey = *initializer.key;
        escrow_info_copy.temp_token_account_pubkey = *temp_token_account.key;
        escrow_info_copy.escrow_amount = amount;
        escrow_info_copy.expire_date = Clock::get().unwrap().unix_timestamp + 20000;
        
        msg!("Escrow packing!");
        Escrow::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        let token_program = next_account_info(account_info_iter)?;
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
                ],
            )?;

        msg!("Escrow creation was successful");
        Ok(escrow_info_copy)
    }
   
    fn process_release_escrow(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> Result<Escrow, ProgramError> {
        let account_info_iter = &mut accounts.iter();
        let taker = next_account_info(account_info_iter)?;
    
        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        let takers_token_to_receive_account = next_account_info(account_info_iter)?;
    
        let pdas_temp_token_account = next_account_info(account_info_iter)?;
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
    
        let initializers_main_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
    
        let escrow_info = Escrow::unpack(&escrow_account.try_borrow_data()?)?;
        let current_timestamp = Clock::get().unwrap().unix_timestamp;

        if current_timestamp < escrow_info.expire_date {
            return Err(EscrowError::EscrowNotMaturedYet.into());
        }
    
        if escrow_info.receiver_pubkey != *taker.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
    
        if escrow_info.initializer_pubkey != *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
    
        let token_program = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key,
            pdas_temp_token_account.key,
            takers_token_to_receive_account.key,
            &pda,
            &[&pda],
            escrow_info.escrow_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the taker...");
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pdas_temp_token_account.clone(),
                takers_token_to_receive_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            pdas_temp_token_account.key,
            initializers_main_account.key,
            &pda,
            &[&pda]
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
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        msg!("Closing the escrow account...");
        **initializers_main_account.lamports.borrow_mut() = initializers_main_account.lamports()
        .checked_add(escrow_account.lamports())
        .ok_or(EscrowError::AmountOverflow)?;
        **escrow_account.lamports.borrow_mut() = 0;
        *escrow_account.try_borrow_mut_data()? = &mut [];
        
        Ok(escrow_info)
    }
}