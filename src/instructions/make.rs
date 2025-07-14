use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_log::log;
use pinocchio_token::state::TokenAccount;

use crate::state::Escrow;

pub fn process_make_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint_x, mint_y, maker_ata, vault, escrow, _system_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    let bump = unsafe{ *(data.as_ptr() as *const u8) }.to_le_bytes();
    let seed = [(b"escrow"), maker.key().as_slice(), bump.as_ref()];
    let seeds = &seed[..];

    let pda = pubkey::checked_create_program_address(seeds, &crate::ID).unwrap();
    assert_eq!(&pda, escrow.key());

    assert

    assert_eq!(unsafe {mint_x.owner()}, &pinocchio_token::ID); // Not needed since CPI since deposit will fail if not owned by token program
    assert_eq!(unsafe {mint_y.owner()}, &pinocchio_token::ID); // Check if mint_x and mint_y are owned by token program so we don't accept any random account
    assert!( unsafe { TokenAccount::from_account_info_unchecked(vault).unwrap().owner() == escrow.key() } ); // Check if vault is owned (authority) by escrow account

    if unsafe {escrow.owner()} != &crate::ID {
        log!("Creating Escrow Account");

        // Create Escrow Account
        pinocchio_system::instructions::CreateAccount {
            from: maker,
            to: escrow,
            lamports: Rent::get()?.minimum_balance(Escrow::LEN),
            space: Escrow::LEN as u64,
            owner: &crate::ID,
        }.invoke()?;

        // Populate Escrow Account
        let escrow_account = Escrow::from_account_info(&escrow);
        escrow_account.maker = *maker.key();
        escrow_account.mint_x = *mint_x.key();
        escrow_account.mint_y = *mint_y.key();
        escrow_account.amount = unsafe { *(data.as_ptr().add(1) as *const u64) };
        escrow_account.bump = unsafe { *data.as_ptr() };

        log!("Amount: {}", unsafe { *(data.as_ptr().add(1 + 8) as *const u64)});

        // Transfer mint_x (token being offered) from user ata to vault
        pinocchio_token::instructions::Transfer {
            from: maker_ata,
            to: vault,
            authority: maker,
            amount: unsafe { *(data.as_ptr().add(1 + 8) as *const u64)},
        }.invoke()?;
    }
    else {
        return Err(ProgramError::AccountAlreadyInitialized)
    }
    
    Ok(())
}