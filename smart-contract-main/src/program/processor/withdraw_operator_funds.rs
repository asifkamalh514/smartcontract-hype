use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    //msg,
    program::invoke,
    program::invoke_signed,
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::{instruction::create_associated_token_account, *};

pub unsafe fn withdraw_operator_funds(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin = next_account_info(accounts_iter)?;
    let root_acc = next_account_info(accounts_iter)?;
    let base_crncy_mint = next_account_info(accounts_iter)?;
    let base_crncy_program_acc = next_account_info(accounts_iter)?;
    let fee_wallet = next_account_info(accounts_iter)?;
    let associated_token_acc = next_account_info(accounts_iter)?;
    let token_program_id = next_account_info(accounts_iter)?;
    let hype_auth_acc = next_account_info(accounts_iter)?;
    let root = root_acc.data.borrow().as_ptr() as *mut RootAccount;
    if (*root).tag != ROOT_TAG {
        return Err(InvalidAccountTag.into());
    }
    if (*root).admin != *admin.key {
        return Err(InvalidAdmin.into());
    }
    if (*root).fee_wallet != *fee_wallet.key {
        return Err(InvalidFeeWallet.into());
    }
    if *associated_token_acc.owner == system_program::ID {
        invoke(
            &create_associated_token_account(
                admin.key,
                fee_wallet.key,
                base_crncy_mint.key,
                token_program_id.key,
            ),
            &[
                admin.clone(),
                associated_token_acc.clone(),
                fee_wallet.clone(),
                base_crncy_mint.clone(),
                token_program_id.clone(),
            ],
        )?;
    }
    let (hype_auth, hype_bump_seed) = Pubkey::find_program_address(&[HYPE_SEED], program_id);
    if hype_auth != *hype_auth_acc.key {
        return Err(InvalidHypeAuthority.into());
    }
    if (*root).fees > 0 {
        let transfer_to_taker_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            base_crncy_program_acc.key,
            associated_token_acc.key,
            &hype_auth,
            &[&hype_auth],
            (*root).fees,
        )?;
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                base_crncy_program_acc.clone(),
                associated_token_acc.clone(),
                hype_auth_acc.clone(),
                token_program_id.clone(),
            ],
            &[&[&HYPE_SEED[..], &[hype_bump_seed]]],
        )?;
    }
    (*root).fees = 0;
    Ok(())
}
