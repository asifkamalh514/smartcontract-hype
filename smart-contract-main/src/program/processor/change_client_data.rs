use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub unsafe fn change_client_data(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let signer = next_account_info(account_info_iter)?;
    let client_acc = next_account_info(account_info_iter)?;
    if client_acc.owner != program_id {
        return Err(InvalidClientAccount.into());
    }
    let client = client_acc.data.borrow().as_ptr() as *mut ClientAccount;
    if (*client).tag != CLIENT_TAG || (*client).wallet != *signer.key {
        return Err(InvalidClientAccount.into());
    }
    (*client).nickname = *((instruction_data[8..]).as_ptr() as *const [u8; NICKNAME_STRING_LENGTH]);
    let clock = Clock::get()?;
    (*client).slot = clock.slot;
    (*client).time = clock.unix_timestamp as u32;
    Ok(())
}
