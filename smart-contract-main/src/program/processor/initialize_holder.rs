use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    //msg,
    pubkey::Pubkey,
    system_instruction,
    system_program,
    sysvar::rent::Rent,
};
use solana_program::{clock::Clock, sysvar::Sysvar};

pub unsafe fn initialize_holder(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Initialize Holder Account Instruction
    // #1 - Holder Admin Address (Signer, Writable)
    // #2 - New Holder Account (Writable)
    // #3 - Wallet Account
    // #4 - System Program
    // [0] - 0
    // [1..8] - Seed
    // [9] - Bump Seed
    // --------------- Reading Accounts ---------------------

    if accounts.len() != 4 {
        return Err(InvalidAccountsNumber.into());
    }
    if _instruction_data.len() != 10 {
        return Err(InvalidDataLength.into());
    }
    let account_info_iter = &mut accounts.iter();
    let admin = next_account_info(account_info_iter)?;
    let holder_acc = next_account_info(account_info_iter)?;
    let wallet_acc = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    check_holder_admin(admin)?;
    if !system_program::check_id(system_program.key) {
        return Err(InvalidSystemProgramId.into());
    }
    let balance = admin.lamports();
    let rent = &Rent::default();
    let holder_lamports = rent.minimum_balance(HOLDER_ACCOUNT_SIZE);
    if balance < holder_lamports {
        return Err(InsufficientFunds.into());
    }
    let bump_seed = _instruction_data[9];
    let mut seed_length: usize = 0;
    for i in 1..9 {
        if _instruction_data[i] > 0 {
            seed_length += 1;
        } else {
            break;
        }
    }
    let seed = Vec::from_raw_parts(
        (_instruction_data[1..1 + seed_length]).as_ptr() as *mut u8,
        seed_length,
        seed_length,
    );
    let seeds = &[&seed, admin.key.as_ref(), &[bump_seed]];
    let expected_pda = Pubkey::create_program_address(seeds, program_id)?;
    if holder_acc.key != &expected_pda {
        return Err(InvalidNewAccountPDA.into());
    }
    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            holder_acc.key,
            holder_lamports,
            HOLDER_ACCOUNT_SIZE as u64,
            program_id,
        ),
        &[admin.clone(), holder_acc.clone()],
        &[&[&seed, admin.key.as_ref(), &[bump_seed]]],
    )?;
    let clock = Clock::get().unwrap();
    let time = clock.unix_timestamp as u32;
    *(holder_acc.data.borrow().as_ptr() as *mut HolderAccount) = HolderAccount {
        tag: HOLDER_TAG as u32,
        version: 0xFFFFFFFF,
        operators_count: 0,
        time: time,
        slot: clock.slot,
        wallet: *wallet_acc.key,
    };
    Ok(())
}
