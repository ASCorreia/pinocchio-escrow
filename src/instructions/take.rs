use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, pubkey::find_program_address, ProgramResult};

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        taker, maker, mint_x, mint_y, taker_ata_x, taker_ata_y, maker_ata_y, vault, escrow,
        _token_program, _system_program
    ] = accounts else {
        return  Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };
    
    let escrow_account = Escrow::from_account_info(escrow);
    assert_eq!(escrow_account.mint_x, *mint_x.key());
    assert_eq!(escrow_account.mint_y, *mint_y.key());

    let vault_account = pinocchio_token::state::TokenAccount::from_account_info(vault)?;

    let seed = [(b"escrow"), maker.key().as_slice(), &[escrow_account.bump]];
    let seeds = &seed[..];
    let escrow_pda = find_program_address(seeds, &crate::ID).0;
    assert_eq!(*escrow.key(), escrow_pda);

    pinocchio_token::instructions::Transfer {
        from: taker_ata_y,
        to: maker_ata_y,
        authority: taker,
        amount: escrow_account.amount,
    }.invoke()?;

    let bump = [escrow_account.bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.key()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault,
        to: taker_ata_x,
        authority: escrow,
        amount: vault_account.amount(),
    }.invoke_signed(&[seeds.clone()])?;

    pinocchio_token::instructions::CloseAccount {
        account: vault,
        destination: maker,
        authority: escrow,
    }.invoke_signed(&[seeds])?;

    unsafe { 
        *maker.borrow_mut_lamports_unchecked() += *escrow.borrow_lamports_unchecked();
        *escrow.borrow_mut_lamports_unchecked() = 0 
    };

    Ok(())
}
