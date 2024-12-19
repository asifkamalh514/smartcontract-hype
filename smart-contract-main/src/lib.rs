use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

pub mod program;
pub mod state;
use crate::program::processor::*;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    match _instruction_data[0] {
        0 => unsafe {
            initialize_holder(program_id, accounts, _instruction_data)?;
        },
        1 => unsafe {
            add_operator(program_id, accounts, _instruction_data)?;
        },
        2 => unsafe {
            initialize_root(program_id, accounts, _instruction_data)?;
        },
        3 => unsafe {
            add_network(program_id, accounts, _instruction_data)?;
        },
        4 => unsafe {
            mint(program_id, accounts, _instruction_data)?;
        },
        5 => unsafe {
            burn(program_id, accounts, _instruction_data)?;
        },
        6 => unsafe {
            change_client_data(program_id, accounts, _instruction_data)?;
        },
        7 => unsafe {
            change_token_status(program_id, accounts, _instruction_data)?;
        },
        8 => unsafe {
            withdraw_operator_funds(program_id, accounts)?;
        },
        9 => unsafe {
            withdraw_holder_funds(program_id, accounts)?;
        },
        _ => {}
    }
    Ok(())
}
