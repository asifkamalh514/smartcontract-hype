use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::AccountInfo,
    borsh1::get_instance_packed_len,
    entrypoint::ProgramResult,
    //msg,
    program::invoke,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    system_program,
};
use spl_associated_token_account::{instruction::create_associated_token_account, *};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::extension::ExtensionType;
use spl_token_metadata_interface::state::TokenMetadata;
use std::convert::TryFrom;

pub unsafe fn mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // [0..4] - Tag
    // [4..8] - Network
    // [8..32] - Address
    // [32..40] - Amount
    // [40..48] - Max Price
    // [48..80] - Nickname
    let ctx = Context::new(program_id, &mut accounts.iter(), instruction_data, 48)?;
    let network = *((instruction_data[4..]).as_ptr() as *const u32);
    if network >= (*ctx.root).networks_count {
        return Err(InvalidNetworkId.into());
    }
    let token_account_seed =
        get_token_seed_bytes((*ctx.root).version, network, &(instruction_data[8..32]));
    let token_account_bump_seed = check_account(
        ctx.token_acc,
        &ctx.hype_auth,
        program_id,
        &token_account_seed,
    )?;
    let token_account: *mut TokenAccount;
    let creation_fee: u64;
    if ctx.token_acc.owner != &solana_program::system_program::id() {
        if ctx.token_acc.owner != program_id {
            return Err(InvalidTokenAccount.into());
        }
        token_account = ctx.token_acc.data.borrow().as_ptr() as *mut TokenAccount;
        if (*token_account).network != network {
            return Err(InvalidNetworkId.into());
        }
        if (*token_account).mint != *ctx.hype_mint_acc.key {
            return Err(InvalidTokenAccount.into());
        }
        creation_fee = 0;
    } else {
        if ctx.hype_mint_acc.owner != &solana_program::system_program::id() {
            return Err(InvalidTokenMint.into());
        }
        let network_record = *((*ctx.root_acc).data.borrow()
            [ROOT_ACCOUNT_SIZE + network as usize * NETWORK_RECORD_SIZE..]
            .as_ptr() as *const NetworkRecord);
        check_name(
            &(instruction_data[8..32]),
            &network_record.mask,
            network_record.max_length,
        )?;
        let rent = Rent::default();
        let token_lamports = rent.minimum_balance(TOKEN_ACCOUNT_SIZE);
        invoke_signed(
            &system_instruction::create_account(
                ctx.signer.key,
                ctx.token_acc.key,
                token_lamports,
                TOKEN_ACCOUNT_SIZE as u64,
                program_id,
            ),
            &[ctx.signer.clone(), ctx.token_acc.clone()],
            &[&[
                &token_account_seed,
                ctx.hype_auth_acc.key.as_ref(),
                &[token_account_bump_seed],
            ]],
        )?;
        token_account = ctx.token_acc.data.borrow().as_ptr() as *mut TokenAccount;
        let address_ptr = (instruction_data[8..8 + ADDRESS_STRING_LENGTH]).as_ptr()
            as *const [u8; ADDRESS_STRING_LENGTH];
        *token_account = TokenAccount {
            tag: TOKEN_TAG,
            version: (*ctx.root).version,
            id: (*ctx.root).tokens_count,
            mint: *ctx.hype_mint_acc.key,
            program_address: *ctx.hype_program_acc.key,
            creator: *ctx.signer.key,
            address: *address_ptr,
            slot: ctx.slot,
            time: ctx.time,
            creation_time: ctx.time,
            supply: 0,
            network: network,
            reserved: 0,
            all_time_trades_count: 0,
            all_time_base_crncy_volume: 0,
            all_time_tokens_volume: 0,
            status: token_status::NOT_CHECKED as u64,
        };
        (*ctx.root).tokens_count += 1;

        let ticker = "Hypemeter".to_string();
        let mut name = "Hypemeter: ".to_string();
        let mut offset = 8;
        for _ in 0..ADDRESS_STRING_LENGTH {
            let c = instruction_data[offset];
            if c == 0 {
                break;
            }
            name.push(char::from_u32_unchecked(c as u32));
            offset += 1;
        }
        name.push_str(" (");
        offset = root_account_offsets::NETWORK_RECORDS
            + NETWORK_RECORD_SIZE * network as usize
            + network_record_offsets::DESCRIPTOR;
        for _ in 0..NETWORK_STRING_LENGTH {
            let c = ctx.root_acc.data.borrow()[offset];
            if c == 0 {
                break;
            }
            name.push(char::from_u32_unchecked(c as u32));
            offset += 1;
        }
        name.push_str(")");
        let uri = "".to_string();
        let update_authority =
            OptionalNonZeroPubkey::try_from(Some(*ctx.hype_auth_acc.key)).unwrap();
        let token_metadata = TokenMetadata {
            name: name,
            symbol: ticker,
            uri: uri,
            update_authority,
            mint: *ctx.hype_mint_acc.key,
            ..Default::default()
        };
        let instance_size = get_instance_packed_len(&token_metadata)?;
        let space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
            ExtensionType::MetadataPointer,
        ])?;
        let hype_mint_account_size = space + instance_size + 4 + 2;
        let hype_lamports = rent.minimum_balance(hype_mint_account_size);
        invoke(
            &system_instruction::create_account(
                ctx.signer.key,
                ctx.hype_mint_acc.key,
                hype_lamports,
                space as u64,
                &spl_token_2022::ID,
            ),
            &[ctx.signer.clone(), ctx.hype_mint_acc.clone()],
        )?;
        let pointer_instruction =
            spl_token_2022::extension::metadata_pointer::instruction::initialize(
                &spl_token_2022::ID,
                ctx.hype_mint_acc.key,
                Some(ctx.hype_auth),
                Some(*ctx.hype_mint_acc.key),
            )?;
        invoke(
            &pointer_instruction,
            &[ctx.hype_mint_acc.clone(), ctx.hype_auth_acc.clone()],
        )?;
        let initialize_mint_instruction = spl_token_2022::instruction::initialize_mint2(
            &spl_token_2022::ID,
            ctx.hype_mint_acc.key,
            &ctx.hype_auth,
            None,
            (*ctx.root).decimals as u8,
        )?;
        invoke_signed(
            &initialize_mint_instruction,
            &[
                ctx.token_program_id.clone(),
                ctx.hype_mint_acc.clone(),
                ctx.hype_auth_acc.clone(),
            ],
            &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
        )?;
        invoke_signed(
            &spl_token_metadata_interface::instruction::initialize(
                &spl_token_2022::ID,
                ctx.hype_mint_acc.key,
                ctx.hype_auth_acc.key,
                ctx.hype_mint_acc.key,
                ctx.hype_auth_acc.key,
                token_metadata.name,
                token_metadata.symbol,
                token_metadata.uri,
            ),
            &[ctx.hype_mint_acc.clone(), ctx.hype_auth_acc.clone()],
            &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
        )?;
        if ctx.hype_mint_acc.data_len() > hype_mint_account_size {
            return Err(InvalidMintSize.into());
        }
        let acc_lamports = rent.minimum_balance(165);
        invoke(
            &system_instruction::create_account(
                ctx.signer.key,
                ctx.hype_program_acc.key,
                acc_lamports,
                165,
                ctx.token_2022_program_id.key,
            ),
            &[ctx.signer.clone(), ctx.hype_program_acc.clone()],
        )?;
        let initialize_account_instruction = spl_token_2022::instruction::initialize_account3(
            ctx.token_2022_program_id.key,
            ctx.hype_program_acc.key,
            ctx.hype_mint_acc.key,
            &ctx.hype_auth,
        )?;
        invoke_signed(
            &initialize_account_instruction,
            &[
                ctx.token_2022_program_id.clone(),
                ctx.hype_mint_acc.clone(),
                ctx.hype_program_acc.clone(),
                ctx.hype_auth_acc.clone(),
            ],
            &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
        )?;
        log_new_token(
            (*ctx.client).id,
            (*ctx.root).counter,
            (*token_account).id,
            network,
            ctx.hype_mint_acc.key,
            ctx.signer.key,
            (*address_ptr).as_slice(),
            ctx.time,
            ctx.slot,
        );
        creation_fee =
            ((*ctx.root).creation_fee * (*ctx.root).base_crncy_decs_factor as f64) as u64;
        (*ctx.client).tokens_created += 1;
    }
    if *ctx.client_associated_hype_acc.owner == system_program::ID {
        invoke(
            &create_associated_token_account(
                ctx.signer.key,
                ctx.signer.key,
                ctx.hype_mint_acc.key,
                ctx.token_2022_program_id.key,
            ),
            &[
                ctx.signer.clone(),
                ctx.client_associated_hype_acc.clone(),
                ctx.hype_mint_acc.clone(),
                ctx.token_2022_program_id.clone(),
            ],
        )?;
    }
    let init_supply = *(ctx.hype_mint_acc.data.borrow()[36..].as_ptr() as *const u64);
    if init_supply != (*token_account).supply {
        return Err(InvalidTokenSupply.into());
    }
    let init_funds = get_reserve((*ctx.root).init_price, (*ctx.root).max_supply, init_supply)?;
    let amount = *((instruction_data[32..]).as_ptr() as *const u64);
    if amount == 0 {
        return Err(TooSmallQuantity.into());
    }
    let max_cost = *((instruction_data[40..]).as_ptr() as *const u64);
    let final_funds = get_reserve(
        (*ctx.root).init_price,
        (*ctx.root).max_supply,
        init_supply + amount,
    )?;
    let cost = final_funds - init_funds;
    let base_crncy_cost = final_funds as u64 - init_funds as u64;
    if max_cost > 0 && base_crncy_cost > max_cost {
        return Err(MaxTradeCostExceeded.into());
    }
    let fees = ((cost * (*ctx.root).fee_rate).max((*ctx.root).min_fee)
        * (*ctx.root).base_crncy_decs_factor as f64) as u64;
    let total_fees = fees + creation_fee;
    let holder_fees = (total_fees as f64 * (1.0 - (*ctx.root).fee_ratio)) as u64;

    let ref_fees: u64;
    let operator_fees: u64;
    if (*ctx.client).ref_stop > ctx.time {
        if (*ctx.client).ref_address != *ctx.ref_acc.key {
            return Err(InvalidRefAddress.into());
        }
        let discounted_fees = (total_fees as f64 * (1.0 - (*ctx.client).ref_discount)) as u64;
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
        operator_fees = total_fees - holder_fees;
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
    let program_payment = base_crncy_cost + operator_fees + holder_fees;
    if *ctx.base_crncy_program_acc.owner == spl_token_2022::id() {
        let transfer_to_taker_ix = spl_token_2022::instruction::transfer_checked(
            &spl_token_2022::id(),
            ctx.client_associated_token_acc.key,
            ctx.base_crncy_mint_acc.key,
            ctx.base_crncy_program_acc.key,
            &ctx.signer.key,
            &[&ctx.signer.key],
            program_payment,
            (*ctx.root).decimals as u8,
        )?;
        invoke(
            &transfer_to_taker_ix,
            &[
                ctx.token_2022_program_id.clone(),
                ctx.client_associated_token_acc.clone(),
                ctx.base_crncy_mint_acc.clone(),
                ctx.base_crncy_program_acc.clone(),
                ctx.signer.clone(),
            ],
        )?;
        if ref_fees > 0 {
            let transfer_to_taker_ix = spl_token_2022::instruction::transfer_checked(
                &spl_token_2022::id(),
                ctx.client_associated_token_acc.key,
                ctx.base_crncy_mint_acc.key,
                ctx.ref_associated_token_acc.key,
                &ctx.signer.key,
                &[&ctx.signer.key],
                ref_fees,
                (*ctx.root).decimals as u8,
            )?;
            invoke(
                &transfer_to_taker_ix,
                &[
                    ctx.token_2022_program_id.clone(),
                    ctx.client_associated_token_acc.clone(),
                    ctx.base_crncy_mint_acc.clone(),
                    ctx.ref_associated_token_acc.clone(),
                    ctx.signer.clone(),
                ],
            )?;
        }
    } else {
        let transfer_to_taker_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            ctx.client_associated_token_acc.key,
            ctx.base_crncy_program_acc.key,
            &ctx.signer.key,
            &[&ctx.signer.key],
            program_payment,
        )?;
        invoke(
            &transfer_to_taker_ix,
            &[
                ctx.client_associated_token_acc.clone(),
                ctx.base_crncy_program_acc.clone(),
                ctx.signer.clone(),
                ctx.token_program_id.clone(),
            ],
        )?;
        if ref_fees > 0 {
            let transfer_to_taker_ix = spl_token::instruction::transfer(
                &spl_token::id(),
                ctx.client_associated_token_acc.key,
                ctx.ref_associated_token_acc.key,
                &ctx.signer.key,
                &[&ctx.signer.key],
                ref_fees,
            )?;
            invoke(
                &transfer_to_taker_ix,
                &[
                    ctx.client_associated_token_acc.clone(),
                    ctx.ref_associated_token_acc.clone(),
                    ctx.signer.clone(),
                    ctx.token_program_id.clone(),
                ],
            )?;
        }
    }
    invoke_signed(
        &spl_token_2022::instruction::mint_to(
            &spl_token_2022::ID,
            ctx.hype_mint_acc.key,
            ctx.client_associated_hype_acc.key,
            ctx.hype_auth_acc.key,
            &[ctx.hype_auth_acc.key],
            amount,
        )?,
        &[
            ctx.hype_mint_acc.clone(),
            ctx.client_associated_hype_acc.clone(),
            ctx.hype_auth_acc.clone(),
        ],
        &[&[&HYPE_SEED[..], &[ctx.hype_bump_seed]]],
    )?;
    (*ctx.root).all_time_base_crncy_volume += base_crncy_cost as u128;
    (*ctx.root).all_time_tokens_volume += amount as u128;
    (*ctx.root).counter += 1;
    (*ctx.root).fees += operator_fees;
    (*ctx.root).holder_fees += holder_fees;
    (*ctx.root).slot = ctx.slot;
    (*ctx.root).time = ctx.time;
    (*ctx.root).supply += amount;
    (*ctx.root).tvl += base_crncy_cost;
    (*ctx.client).all_time_trades_count += 1;
    (*ctx.client).all_time_base_crncy_volume += base_crncy_cost;
    (*ctx.client).all_time_tokens_volume += amount;
    (*ctx.client).slot = ctx.slot;
    (*ctx.client).time = ctx.time;
    (*token_account).all_time_trades_count += 1;
    (*token_account).all_time_base_crncy_volume += base_crncy_cost as u128;
    (*token_account).all_time_tokens_volume += amount as u128;
    (*token_account).supply += amount;
    (*token_account).slot = ctx.slot;
    (*token_account).time = ctx.time;
    log_mint(
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
        base_crncy_cost,
        ctx.time,
        ctx.slot,
    );
    //return Err(solana_program::program_error::ProgramError::Custom(2000));
    Ok(())
}
