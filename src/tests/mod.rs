#[cfg(test)]
mod tests {

    use mollusk_svm::{program, result::Check, Mollusk};
    use solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        program_option::COption,
        program_pack::Pack,
        pubkey::{self, Pubkey}, rent::Rent, sysvar::Sysvar,
    };
    use spl_token::state::AccountState;

    const ID: Pubkey = pubkey::Pubkey::new_from_array(five8_const::decode_32_const(
        "22222222222222222222222222222222222222222222",
    ));

    #[test]
    fn test_make() {
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_escrow");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        let maker = Pubkey::new_from_array([0x02; 32]);
        let maker_account = AccountSharedData::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        let (escrow, escrow_bump) =
            solana_sdk::pubkey::Pubkey::find_program_address(&[(b"escrow"), maker.as_ref()], &ID);
        let escrow_account = AccountSharedData::new(0, 0, &system_program);

        let mint_x = Pubkey::new_from_array([0x03; 32]);
        let mut mint_x_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_x_account.data_as_mut_slice(),
        )
        .unwrap();

        let mint_y = Pubkey::new_from_array([0x04; 32]);
        let mut mint_y_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_y_account.data_as_mut_slice(),
        )
        .unwrap();

        let maker_ata = Pubkey::new_from_array([0x05; 32]);
        let mut maker_ata_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: maker,
                amount: 100_000_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            maker_ata_account.data_as_mut_slice(),
        )
        .unwrap();

        let vault = Pubkey::new_from_array([0x06; 32]);
        let mut vault_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: escrow,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        )
        .unwrap();

        let data = [
            vec![0],
            vec![escrow_bump],
            1_000_000u64.to_le_bytes().to_vec(),
            1_000_000u64.to_le_bytes().to_vec(),
        ]
        .concat();

        let instruction = Instruction::new_with_bytes(
            ID,
            &data,
            vec![
                AccountMeta::new(maker, true),
                AccountMeta::new_readonly(mint_x, false),
                AccountMeta::new_readonly(mint_y, false),
                AccountMeta::new(maker_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(escrow, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(token_program, false),
            ],
        );

        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (maker, maker_account),
                (mint_x, mint_x_account),
                (mint_y, mint_y_account),
                (maker_ata, maker_ata_account),
                (vault, vault_account),
                (escrow, escrow_account),
                (system_program, system_account),
                (token_program, token_account),
            ],
            &[
                Check::success(),
            ],
        );
    }

    #[test]
    fn test_take() {
        // Create a Mollusk test harness with our escrow program.
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_escrow");

        // Set up the system program account.
        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        // Add the SPL token program to the harness.
        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        // Create a token program account.
        let (token_program, token_account) = (
            spl_token::ID,
            mollusk_svm::program::create_program_account_loader_v3(&spl_token::ID),
        );

        // Define maker and taker keys.
        let maker = Pubkey::new_from_array([0x02; 32]);
        let taker = Pubkey::new_from_array([0x08; 32]);

        // Create maker and taker accounts.
        let maker_account = AccountSharedData::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
        let taker_account = AccountSharedData::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        // Define mint keys.
        let mint_x = Pubkey::new_from_array([0x03; 32]);
        let mint_y = Pubkey::new_from_array([0x04; 32]);

        // Create and initialize mint_x account.
        let mut mint_x_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_x_account.data_as_mut_slice(),
        )
        .unwrap();

        // Create and initialize mint_y account.
        let mut mint_y_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_y_account.data_as_mut_slice(),
        )
        .unwrap();

        // Define associated token account keys.
        let taker_ata_x = Pubkey::new_from_array([0x09; 32]);
        let taker_ata_y = Pubkey::new_from_array([0x0A; 32]);
        let maker_ata_y = Pubkey::new_from_array([0x0B; 32]);

        // Create taker's ATA for token X (starts with 0 tokens).
        let mut taker_ata_x_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: taker,
                amount: 0,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            taker_ata_x_account.data_as_mut_slice(),
        )
        .unwrap();

        // Create taker's ATA for token Y (starts with sufficient tokens).
        let mut taker_ata_y_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_y,
                owner: taker,
                amount: 1_000_000,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            taker_ata_y_account.data_as_mut_slice(),
        )
        .unwrap();

        // Create maker's ATA for token Y (initially 0 tokens).
        let mut maker_ata_y_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_y,
                owner: maker,
                amount: 0,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            maker_ata_y_account.data_as_mut_slice(),
        )
        .unwrap();

        // Derive the escrow PDA and bump using seeds ["escrow", maker.as_ref()].
        let (escrow, escrow_bump) = Pubkey::find_program_address(&[b"escrow", maker.as_ref()], &ID);

        let vault = Pubkey::new_from_array([0x06; 32]);
        // Create the vault account with the escrow PDA as its owner.
        let mut vault_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: escrow, // Set the vault owner immediately.
                amount: 2_000_000, // For example, the vault holds 2,000,000 tokens.
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        )
        .unwrap();

        // Create the escrow account with enough space (assume Escrow::LEN is 105 bytes).
        let mut escrow_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(105),
            105,
            &system_program,
        );
        // Write the escrow state as a concatenated vector.
        let escrow_state_data = [
            maker.as_ref().to_vec(),
            mint_x.as_ref().to_vec(),
            mint_y.as_ref().to_vec(),
            1_000_000u64.to_le_bytes().to_vec(), // escrow.amount
            vec![escrow_bump],
        ]
        .concat();
        escrow_account
            .data_as_mut_slice()
            .copy_from_slice(&escrow_state_data);

        // Construct the "take" instruction.
        // Expected accounts order:
        // taker, maker, mint_x, mint_y, taker_ata_x, taker_ata_y, maker_ata_y, vault, escrow, token_program, system_program
        let instruction = Instruction::new_with_bytes(
            ID,
            &[], // No extra instruction data needed.
            vec![
                AccountMeta::new(taker, true),
                AccountMeta::new(maker, false),
                AccountMeta::new_readonly(mint_x, false),
                AccountMeta::new_readonly(mint_y, false),
                AccountMeta::new(taker_ata_x, false),
                AccountMeta::new(taker_ata_y, false),
                AccountMeta::new(maker_ata_y, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(escrow, true),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        );

        // Execute and validate the "take" instruction.
        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (taker, taker_account),
                (maker, maker_account),
                (mint_x, mint_x_account),
                (mint_y, mint_y_account),
                (taker_ata_x, taker_ata_x_account),
                (taker_ata_y, taker_ata_y_account),
                (maker_ata_y, maker_ata_y_account),
                (vault, vault_account),
                (escrow, escrow_account),
                (system_program, system_account),
                (token_program, token_account),
            ],
            &[Check::success()],
        );
    }


    #[test]
    fn test_refund() {
        // Create a Mollusk test harness with our escrow program.
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_escrow");

        // Set up the system program account.
        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        // Add the SPL token program to the harness.
        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        // Create a token program account.
        let (token_program, token_account) = (
            spl_token::ID,
            mollusk_svm::program::create_program_account_loader_v3(&spl_token::ID),
        );

        // Define maker key.
        let maker = Pubkey::new_from_array([0x02; 32]);

        // Create maker account with some lamports.
        let maker_account = AccountSharedData::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        // Define mint keys.
        let mint_x = Pubkey::new_from_array([0x03; 32]);
        let mint_y = Pubkey::new_from_array([0x04; 32]);

        // Create and initialize mint_x account.
        let mut mint_x_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_x_account.data_as_mut_slice(),
        )
        .unwrap();

        // Create and initialize mint_y account.
        let mut mint_y_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_y_account.data_as_mut_slice(),
        )
        .unwrap();

        // Define maker's associated token account for token X (destination for refund).
        let maker_ata_x = Pubkey::new_from_array([0x05; 32]);
        let mut maker_ata_x_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        // Initialize as a token account with mint_x and owned by maker, starting with 0 tokens.
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: maker,
                amount: 0,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            maker_ata_x_account.data_as_mut_slice(),
        )
        .unwrap();

        // Derive the escrow PDA and bump using seeds ["escrow", maker.as_ref()].
        let (escrow, escrow_bump) = Pubkey::find_program_address(&[b"escrow", maker.as_ref()], &ID);

        // Create the vault account with the escrow PDA as its owner.
        let vault = Pubkey::new_from_array([0x06; 32]);
        let mut vault_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        // For example, the vault holds 2,000,000 tokens of mint_x.
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: mint_x,
                owner: escrow,
                amount: 2_000_000,
                delegate: COption::None,
                state: spl_token::state::AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        )
        .unwrap();

        // Create the escrow account with enough space (assume Escrow::LEN is 105 bytes).
        let mut escrow_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(105),
            105,
            &system_program,
        );
        // Build escrow state data as a concatenated vector (maker, mint_x, mint_y, amount, bump).
        let escrow_state_data = [
            maker.as_ref().to_vec(),
            mint_x.as_ref().to_vec(),
            mint_y.as_ref().to_vec(),
            1_000_000u64.to_le_bytes().to_vec(), // escrow.amount
            vec![escrow_bump],
        ]
        .concat();
        escrow_account
            .data_as_mut_slice()
            .copy_from_slice(&escrow_state_data);

        // Construct the "refund" instruction.
        // Expected accounts order:
        // maker, mint_x, maker_ata_x, vault, escrow, token_program, system_program
        let instruction = Instruction::new_with_bytes(
            ID,
            &[], // No extra instruction data is required.
            vec![
                AccountMeta::new(maker, true),
                AccountMeta::new_readonly(mint_x, false),
                AccountMeta::new(maker_ata_x, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(escrow, true),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(system_program, false),
            ],
        );

        // Execute and validate the "refund" instruction.
        mollusk.process_and_validate_instruction(
            &instruction,
            &vec![
                (maker, maker_account),
                (mint_x, mint_x_account),
                (maker_ata_x, maker_ata_x_account),
                (vault, vault_account),
                (escrow, escrow_account),
                (system_program, system_account),
                (token_program, token_account),
            ],
            &[Check::success()],
        );
    }

}
