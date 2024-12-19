use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub unsafe fn change_token_status(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let signer = next_account_info(account_info_iter)?;
    let root_acc = next_account_info(account_info_iter)?;
    let token_acc = next_account_info(account_info_iter)?;
    if root_acc.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let root = root_acc.data.borrow().as_ptr() as *mut RootAccount;
    if (*root).tag != ROOT_TAG {
        return Err(InvalidAccountTag.into());
    }
    let (hype_auth, _) = Pubkey::find_program_address(&[HYPE_SEED], program_id);
    let token = token_acc.data.borrow()[..].as_ptr() as *mut TokenAccount;
    let token_account_seed =
        get_token_seed_bytes((*root).version, (*token).network, &(*token).address);
    check_account(token_acc, &hype_auth, program_id, &token_account_seed)?;
    let network_record = *(root_acc.data.borrow()
        [ROOT_ACCOUNT_SIZE + (*token).network as usize * NETWORK_RECORD_SIZE..]
        .as_ptr() as *const NetworkRecord);
    if network_record.validator != *signer.key {
        return Err(InvalidValidator.into());
    }
    if (*token).status == token_status::VERIFIED as u64 {
        return Err(TokenAlreadyVerified.into());
    }
    if instruction_data[1] == 1 {
        (*token).status = token_status::VERIFIED as u64;
    } else {
        (*token).status = token_status::NOT_VERIFIED as u64;
    }
    let clock = Clock::get()?;
    (*token).slot = clock.slot;
    (*token).time = clock.unix_timestamp as u32;
    //return Err(solana_program::program_error::ProgramError::Custom(2000));
    Ok(())
}
