use crate::program::*;
use solana_program::{
    account_info::AccountInfo,
    //msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub unsafe fn check_new_account(
    account: &AccountInfo,
    pda: &Pubkey,
    program_id: &Pubkey,
    seed: &[u8],
) -> Result<u8, ProgramError> {
    let seeds = &[seed, pda.as_ref()];
    let (expected_pda, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    if account.owner != &solana_program::system_program::id() || expected_pda != *account.key {
        return Err(InvalidAccountKey.into());
    }
    Ok(bump_seed)
}

pub unsafe fn check_account(
    account: &AccountInfo,
    pda: &Pubkey,
    program_id: &Pubkey,
    seed: &[u8],
) -> Result<u8, ProgramError> {
    let seeds = &[seed, pda.as_ref()];
    let (expected_pda, bump_seed) = Pubkey::find_program_address(seeds, program_id);
    if expected_pda != *account.key {
        return Err(InvalidAccountKey.into());
    }
    Ok(bump_seed)
}

pub unsafe fn get_seed_by_tag(version: u32, tag: u32) -> [u8; 8] {
    let mut res = [0; 8];
    res[0..4].copy_from_slice(&version.to_le_bytes());
    res[4..8].copy_from_slice(&tag.to_le_bytes());
    res
}

pub unsafe fn get_token_seed_bytes(version: u32, network: u32, address: &[u8]) -> [u8; 32] {
    let mut res = [0; 32];
    res[0..24].copy_from_slice(&address[0..24]);
    res[24..28].copy_from_slice(&network.to_le_bytes());
    res[28..32].copy_from_slice(&version.to_le_bytes());
    res
}

pub fn get_reserve(init_price: f64, max_supply: u64, supply: u64) -> Result<f64, ProgramError> {
    if max_supply <= supply {
        Err(InvalidSupply.into())
    } else {
        Ok(max_supply as f64 * init_price * supply as f64 / (max_supply - supply) as f64)
    }
}
