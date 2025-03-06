use pinocchio::{self, account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::find_program_address, ProgramResult};
use pinocchio_token::{instructions::{CloseAccount, Transfer}, state::TokenAccount};

use crate::state::Escrow;

pub fn process_refund_instruction(accounts: &[AccountInfo], _data: &[u8])-> ProgramResult{
    let [
        maker, mint_x, maker_ata_x, vault, escrow, _token_program, _system_program
    ] = accounts else{
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    let escrow_account = Escrow::from_account_info_unchecked(escrow);
    assert_eq!(escrow_account.mint_x, *mint_x.key());

    let vault_account = TokenAccount::from_account_info(vault)?;

    let seed = [(b"escrow"), maker.key().as_slice(), &[escrow_account.bump]];
    let seeds = &seed[..];
    let escrow_pda = find_program_address(seeds, &crate::ID).0;
    assert_eq!(*escrow.key(), escrow_pda);

    let bump = [escrow_account.bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.key()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);

    // Transfering mint_x from vault to maker_ata_x
    Transfer{
        from: vault,
        to: maker_ata_x,
        authority: escrow,
        amount: vault_account.amount(),
    }.invoke_signed(&[seeds.clone()])?;

    // Closing Vault Account and sending lamport to maker 
    CloseAccount{
        account: vault,
        destination: maker,
        authority: escrow,
    }.invoke_signed(&[seeds])?;

    // Closing Escrow and sending lamports to maker
    unsafe{
        *maker.borrow_mut_lamports_unchecked() += *escrow.borrow_lamports_unchecked();
        *escrow.borrow_mut_lamports_unchecked() = 0;
    }


    Ok(())
}