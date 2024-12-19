use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    //msg,
    program::invoke,
    pubkey::Pubkey,
    system_instruction,
    sysvar::rent::Rent,
    sysvar::Sysvar,
};

use crate::program::*;
use crate::state::*;

pub unsafe fn add_operator(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Add Operator Instruction
    // Accounts
    // #1 - Holder Admin Address (Signer, Writable)
    // #2 - Holder Account (Writavle)
    // #3 - Operator Address
    // #4 - System Program
    // Instruction Data (9 Bytes Length)
    // [0] - 1
    // [1..5] - version
    // --------------- Reading Accounts ---------------------
    if accounts.len() != 4 {
        return Err(InvalidAccountsNumber.into());
    }
    if _instruction_data.len() != 9 + OPERATOR_NAME_STRING_LENGTH {
        return Err(InvalidDataLength.into());
    }
    let accounts_iter = &mut accounts.iter();
    let admin = next_account_info(accounts_iter)?;
    let holder_acc = next_account_info(accounts_iter)?;
    let operator = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    check_holder_admin(admin)?;
    check_holder_account(holder_acc, program_id, true)?;
    let version = *((_instruction_data[1..]).as_ptr() as *const u32);
    let holder = holder_acc.data.borrow()[..].as_ptr() as *mut HolderAccount;
    let new_size =
        HOLDER_ACCOUNT_SIZE + (((*holder).operators_count + 1) as usize) * OPERATOR_RECORD_SIZE;
    let old_size = holder_acc.data_len();
    if old_size < new_size {
        let rent = Rent::default();
        let new_minimum_balance = rent.minimum_balance(new_size);
        let lamports_diff = new_minimum_balance.saturating_sub(holder_acc.lamports());
        invoke(
            &system_instruction::transfer(admin.key, holder_acc.key, lamports_diff),
            &[admin.clone(), holder_acc.clone(), system_program.clone()],
        )?;
        holder_acc.realloc(new_size, true)?;
    }
    let operators: Vec<OperatorRecord>;
    let begin = HOLDER_ACCOUNT_SIZE + ((*holder).operators_count as usize) * OPERATOR_RECORD_SIZE;
    let operators_ptr = (&holder_acc.data.borrow()[HOLDER_ACCOUNT_SIZE..begin]).as_ptr();
    operators = Vec::from_raw_parts(
        operators_ptr as *mut OperatorRecord,
        (*holder).operators_count as usize,
        (*holder).operators_count as usize,
    );
    for p in operators {
        if p.version == version {
            return Err(InvalidNewOperatorAccount.into());
        }
    }
    *((&holder_acc.data.borrow()[begin..]).as_ptr() as *mut OperatorRecord) = OperatorRecord {
        version: version,
        max_networks_count: *((_instruction_data[5..]).as_ptr() as *const u32),
        operator_address: *operator.key,
        operator_name: *((_instruction_data[9..9 + OPERATOR_NAME_STRING_LENGTH]).as_ptr()
            as *const [u8; OPERATOR_NAME_STRING_LENGTH]),
    };
    let clock = Clock::get()?;
    (*holder).operators_count += 1;
    (*holder).time = clock.unix_timestamp as u32;
    (*holder).slot = clock.slot;
    //return Err(solana_program::program_error::ProgramError::Custom(2000));
    Ok(())
}
