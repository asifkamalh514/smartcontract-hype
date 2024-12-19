use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

pub unsafe fn add_network(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let admin = next_account_info(account_info_iter)?;
    let root_acc = next_account_info(account_info_iter)?;
    let validator_acc = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    if !admin.is_signer {
        return Err(AdminSignatureRequired.into());
    }
    if root_acc.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let root = root_acc.data.borrow().as_ptr() as *mut RootAccount;
    if (*root).tag != ROOT_TAG {
        return Err(InvalidAccountTag.into());
    }
    if (*root).admin != *admin.key {
        return Err(InvalidAdmin.into());
    }
    let offset = root_account_offsets::NETWORK_RECORDS
        + (*root).networks_count as usize * NETWORK_RECORD_SIZE;
    let new_size = offset + NETWORK_RECORD_SIZE;
    let clock = Clock::get()?;
    let descriptor_ptr = (instruction_data[8..8 + NETWORK_STRING_LENGTH]).as_ptr()
        as *const [u8; NETWORK_STRING_LENGTH];
    let mask_ptr = (instruction_data
        [8 + NETWORK_STRING_LENGTH..8 + NETWORK_STRING_LENGTH + MASK_STRING_LENGTH])
        .as_ptr() as *const [u8; MASK_STRING_LENGTH];
    log_new_network(
        (*root).networks_count,
        (*descriptor_ptr).as_slice(),
        clock.unix_timestamp as u32,
        clock.slot,
    );
    (*root).networks_count += 1;
    if (*root).max_networks_count > 0 && (*root).networks_count > (*root).max_networks_count {
        return Err(MaxNetworksCountExceeded.into());
    }
    if new_size > root_acc.data_len() {
        let rent = Rent::get()?;
        let new_minimum_balance = rent.minimum_balance(new_size);
        let lamports_diff = new_minimum_balance.saturating_sub(root_acc.lamports());
        if lamports_diff > 0 {
            invoke(
                &system_instruction::transfer(admin.key, root_acc.key, lamports_diff),
                &[admin.clone(), root_acc.clone(), system_program.clone()],
            )?;
        }
        root_acc.realloc(new_size, true)?;
    }
    *(root_acc.data.borrow()[offset..offset + NETWORK_RECORD_SIZE].as_ptr()
        as *mut NetworkRecord) = NetworkRecord {
        descriptor: *descriptor_ptr,
        mask: *mask_ptr,
        max_length: *(instruction_data[8 + NETWORK_STRING_LENGTH + MASK_STRING_LENGTH..].as_ptr()
            as *const usize),
        validator: *validator_acc.key,
    };
    (*root).slot = clock.slot;
    (*root).time = clock.unix_timestamp as u32;

    Ok(())
}
