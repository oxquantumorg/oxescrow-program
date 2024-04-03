use std::str::FromStr;

use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::{IsInitialized, Pack}, pubkey::Pubkey, system_instruction::create_account, sysvar::{rent::Rent, Sysvar}
};

use spl_associated_token_account::instruction as token_account_instruction;

use crate::{
    states::default_escrow::EscrowState,
    utils::{self, constants, errors::EscrowError},
};
use sha2::{Digest, Sha256};
use spl_token::{id, state::Mint};

/** Initialize Escrow

    Accounts expected:

    0. [] The account of the person initializing the escrow
    1. [] The account of the receiver
    2. [writable] Temporary token account that should be created prior to this instruction and owned by the rentFeePayer
    3. [writable] The escrow account, it will hold all necessary info about the trade.
    4. [] The rent sysvar
    5. [] The token program
    6. [signer] The rentFeePayer / relayer

**/
pub fn handler(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    msg!("Escrow starting!");
    
    let account_info_iter = &mut accounts.iter();
    let seller_account = next_account_info(account_info_iter)?;
    // let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let caller_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let mint_pubkey = Pubkey::from_str(&constants::USDC_MINT);

    // Generate a nonce for creating the associated token account
    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;

    // Generate random string using timestamp and seller's address
    let seller_address = seller_account.key.to_bytes();
    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_le_bytes());
    hasher.update(&seller_address);
    let hash_result = hasher.finalize();
    let random_string = hex::encode(hash_result);
    let nonce = random_string.as_bytes();

    msg!("Generated nonce {}", nonce[0]);

    // Calculate the PDA for creating the associated token account
    let (escrow_pubkey, escrow_pubkey_bump_seed) = Pubkey::find_program_address(&[&[nonce[0]]], program_id);
    msg!("Generated escrow account {}", escrow_pubkey);

    // Create the associated token account instruction
    let instruction = token_account_instruction::create_associated_token_account(
        &escrow_pubkey,    // Owner of the token account (PDA)
        &escrow_pubkey,    // PDA
        &mint_pubkey.unwrap(),    // Owner of the token account (PDA)
        &token_program.key,      // Token mint
    );

    // Invoke the instruction
    invoke_signed(
        &instruction,
        &[
            caller_account.clone(),  
            token_program.clone(), 
            // mint_pubkey.clone()
        ],
        &[&[&[nonce[0]], &[escrow_pubkey_bump_seed]]],
    )?;
    
    //     if !rentFeePayer.is_signer {
    //         return Err(ProgramError::MissingRequiredSignature);
    //     }
    
    //     if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
    //         return Err(EscrowError::NotRentExempt.into());
    //     }
    
    //     msg!("Escrow unpacking!");
    //     let mut escrow_info = EscrowState::unpack_from_slice(&escrow_account.try_borrow_data()?)?;
    //     if escrow_info.is_initialized() {
    //         return Err(ProgramError::AccountAlreadyInitialized);
    //     }
    
    //     escrow_info.is_initialized = true;
    //     escrow_info.rentFeePayer_pubkey = *rentFeePayer.key;
    //     escrow_info.creator_account_pubkey = *creator_account.key;
    //     escrow_info.receiver_pubkey = *receiver_account.key;
    //     escrow_info.temp_token_account_pubkey = *temp_token_account.key;
    //     escrow_info.escrow_amount = amount;
    //     escrow_info.expire_date =
    //         Clock::get().unwrap().unix_timestamp + utils::constants::ESCROW_WAIT_TIME_SEC;
    
    //     msg!("Escrow packing!");
    //     EscrowState::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;

    msg!("Escrow creation was successful");
    Ok(())
}