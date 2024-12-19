use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    //msg,
    program::invoke,
    program::invoke_signed,
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::{instruction::create_associated_token_account, *};
pub unsafe fn burn(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // [0..8] - 3
    // [8..16] - Amount
    // [16..24] - Min cash out
    let ctx = Context::new(program_id, &mut accounts.iter(), instruction_data, 24)?;
    let token_account = ctx.token_acc.data.borrow().as_ptr() as *mut TokenAccount;
    if (*token_account).mint != *ctx.hype_mint_acc.key {
        return Err(InvalidTokenAccount.into());
    }
    if ctx.token_acc.owner != program_id {
        return Err(InvalidTokenAccount.into());
    }
    if *ctx.client_associated_token_acc.owner == system_program::ID {
        invoke(
            &create_associated_token_account(
                ctx.signer.key,
                ctx.signer.key,
                ctx.base_crncy_mint_acc.key,
                ctx.token_program_id.key,
            ),
            &[
                ctx.signer.clone(),
                ctx.client_associated_token_acc.clone(),
                ctx.base_crncy_mint_acc.clone(),
                ctx.token_program_id.clone(),
            ],
        )?;
    }
    let init_supply = *(ctx.hype_mint_acc.data.borrow()[36..].as_ptr() as *const u64);
    if init_supply != (*token_account).supply {
        return Err(InvalidTokenSupply.into());
    }
    let init_funds = get_reserve((*ctx.root).init_price, (*ctx.root).max_supply, init_supply)?;
    let amount = *((instruction_data[8..]).as_ptr() as *const u64);
    if amount == 0 {
        return Err(TooSmallQuantity.into());
    }
    if amount > init_supply {
        return Err(TooBigQuantity.into());
    }
    let min_cashout = *((instruction_data[16..]).as_ptr() as *const u64);
    let final_funds = get_reserve(
        (*ctx.root).init_price,
        (*ctx.root).max_supply,
        init_supply - amount,
    )?;
    let cashout = init_funds - final_funds;
    let base_crncy_cashout = init_funds as u64 - final_funds as u64;
    if min_cashout > 0 && base_crncy_cashout < min_cashout {
        return Err(MaxTradeCostExceeded.into());
    }
    let fees = ((cashout * (*ctx.root).fee_rate).max((*ctx.root).min_fee)
        * (*ctx.root).base_crncy_decs_factor as f64) as u64;

    let holder_fees = (fees as f64 * (1.0 - (*ctx.root).fee_ratio)) as u64;
    let ref_fees: u64;
    let operator_fees: u64;
    if (*ctx.client).ref_stop > ctx.time {
        if (*ctx.client).ref_address != *ctx.ref_acc.key {
            return Err(InvalidRefAddress.into());
        }
        let discounted_fees = (fees as f64 * (1.0 - (*ctx.client).ref_discount)) as u64;
        let rest_of_fees: u64;
        if discounted_fees > holder_fees {
            rest_of_fees = discounted_fees - holder_fees;
        } else {
            rest_of_fees = 0;
        }
        if rest_of_fees > 0 {
            ref_fees = (rest_of_fees as f64 * (*ctx.client).ref_ratio) as u64;
            operator_fees = rest_of_fees - ref_fees;
        } else {
            ref_fees = 0;
            operator_fees = 0;
        }
    } else {
        ref_fees = 0;
        operator_fees = fees - holder_fees;
    }
    if ref_fees > 0 {
        (*ctx.client).ref_paid += ref_fees;
        if *ctx.ref_associated_token_acc.owner == system_program::ID {
            invoke(
                &create_associated_token_account(
                    ctx.signer.key,
                    ctx.ref_acc.key,
                    ctx.base_crncy_mint_acc.key,
                    ctx.token_program_id.key,
                ),
                &[
                    ctx.signer.clone(),
                    ctx.ref_associated_token_acc.clone(),
                    ctx.ref_acc.clone(),
                    ctx.base_crncy_mint_acc.clone(),
                    ctx.token_program_id.clone(),
                ],
            )?;
        }
    }
    let final_payment = base_crncy_cashout - fees;
    if *ctx.base_crncy_program_acc.owner == spl_token_2022::id() {
        let transfer_to_taker_ix = spl_token_2022::instruction::transfer_checked(
            &spl_token_2022::id(),
            ctx.base_crncy_program_acc.key,
            ctx.base_crncy_mint_acc.key,
            ctx.client_associated_token_acc.key,
            &ctx.hype_auth,
            &[&ctx.hype_auth],
            final_payment,
            (*ctx.root).decimals as u8,
        )?;
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                ctx.token_2022_program_id.clone(),
                ctx.base_crncy_program_acc.clone(),
                ctx.base_crncy_mint_acc.clone(),
                ctx.client_associated_token_acc.clone(),
                ctx.hype_auth_acc.clone(),
            ],
            &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
        )?;
        if ref_fees > 0 {
            let transfer_to_taker_ix = spl_token_2022::instruction::transfer_checked(
                &spl_token_2022::id(),
                ctx.base_crncy_program_acc.key,
                ctx.base_crncy_mint_acc.key,
                ctx.ref_associated_token_acc.key,
                &ctx.hype_auth,
                &[&ctx.hype_auth],
                ref_fees,
                (*ctx.root).decimals as u8,
            )?;
            invoke_signed(
                &transfer_to_taker_ix,
                &[
                    ctx.token_2022_program_id.clone(),
                    ctx.base_crncy_program_acc.clone(),
                    ctx.base_crncy_mint_acc.clone(),
                    ctx.ref_associated_token_acc.clone(),
                    ctx.hype_auth_acc.clone(),
                ],
                &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
            )?;
        }
    } else {
        let transfer_to_taker_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            ctx.base_crncy_program_acc.key,
            ctx.client_associated_token_acc.key,
            &ctx.hype_auth,
            &[&ctx.hype_auth],
            final_payment,
        )?;
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                ctx.base_crncy_program_acc.clone(),
                ctx.client_associated_token_acc.clone(),
                ctx.hype_auth_acc.clone(),
                ctx.token_program_id.clone(),
            ],
            &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
        )?;
        if ref_fees > 0 {
            let transfer_to_taker_ix = spl_token::instruction::transfer(
                &spl_token::id(),
                ctx.base_crncy_program_acc.key,
                ctx.ref_associated_token_acc.key,
                &ctx.hype_auth,
                &[&ctx.hype_auth],
                ref_fees,
            )?;
            invoke_signed(
                &transfer_to_taker_ix,
                &[
                    ctx.base_crncy_program_acc.clone(),
                    ctx.ref_associated_token_acc.clone(),
                    ctx.hype_auth_acc.clone(),
                    ctx.token_program_id.clone(),
                ],
                &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
            )?;
        }
    }
    let transfer_to_taker_ix = spl_token_2022::instruction::transfer_checked(
        &spl_token_2022::id(),
        ctx.client_associated_hype_acc.key,
        ctx.hype_mint_acc.key,
        ctx.hype_program_acc.key,
        &ctx.signer.key,
        &[&ctx.signer.key],
        amount,
        (*ctx.root).decimals as u8,
    )?;
    invoke(
        &transfer_to_taker_ix,
        &[
            ctx.token_2022_program_id.clone(),
            ctx.client_associated_hype_acc.clone(),
            ctx.hype_mint_acc.clone(),
            ctx.hype_program_acc.clone(),
            ctx.signer.clone(),
        ],
    )?;
    invoke_signed(
        &spl_token_2022::instruction::burn(
            &spl_token_2022::ID,
            ctx.hype_program_acc.key,
            ctx.hype_mint_acc.key,
            ctx.hype_auth_acc.key,
            &[ctx.hype_auth_acc.key],
            amount as u64,
        )?,
        &[
            ctx.hype_mint_acc.clone(),
            ctx.hype_program_acc.clone(),
            ctx.hype_auth_acc.clone(),
        ],
        &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
    )?;
    (*ctx.root).all_time_base_crncy_volume += base_crncy_cashout as u128;
    (*ctx.root).all_time_tokens_volume += amount as u128;
    (*ctx.root).counter += 1;
    (*ctx.root).fees += operator_fees;
    (*ctx.root).holder_fees += holder_fees;
    (*ctx.root).slot = ctx.slot;
    (*ctx.root).time = ctx.time;
    if (*ctx.root).supply < amount {
        return Err(InvalidTotalSupply.into());
    }
    (*ctx.root).supply -= amount;
    if (*ctx.root).tvl < base_crncy_cashout {
        return Err(InvalidTVL.into());
    }
    (*ctx.root).tvl -= base_crncy_cashout;
    (*ctx.client).all_time_trades_count += 1;
    (*ctx.client).all_time_base_crncy_volume += base_crncy_cashout;
    (*ctx.client).all_time_tokens_volume += amount;
    (*ctx.client).slot = ctx.slot;
    (*ctx.client).time = ctx.time;
    (*token_account).all_time_trades_count += 1;
    (*token_account).all_time_base_crncy_volume += base_crncy_cashout as u128;
    (*token_account).all_time_tokens_volume += amount as u128;
    (*token_account).supply -= amount;
    (*token_account).slot = ctx.slot;
    (*token_account).time = ctx.time;
    log_burn(
        (*ctx.client).id,
        (*ctx.root).counter,
        (*token_account).id,
        (*token_account).network,
        &(*token_account).mint,
        &(*token_account).creator,
        ctx.signer.key,
        &(*token_account).address,
        (*token_account).creation_time,
        (*token_account).supply,
        (*token_account).all_time_trades_count,
        (*token_account).all_time_base_crncy_volume as u64,
        (*token_account).all_time_tokens_volume as u64,
        amount,
        base_crncy_cashout,
        ctx.time,
        ctx.slot,
    );
    Ok(())
}
