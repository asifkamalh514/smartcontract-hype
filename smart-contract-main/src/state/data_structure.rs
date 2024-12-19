use super::log_new_client;
use crate::program::*;
use core::slice::Iter;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    //msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    system_program,
    sysvar::Sysvar,
};
use spl_associated_token_account::*;
use std::str::FromStr;

pub mod token_status {
    pub const NOT_CHECKED: u8 = 0;
    pub const VERIFIED: u8 = 1;
    pub const NOT_VERIFIED: u8 = 2;
}

#[repr(C)]
pub struct TokenAccount {
    pub tag: u32,
    pub version: u32,
    pub id: u64,
    pub mint: Pubkey,
    pub program_address: Pubkey,
    pub creator: Pubkey,
    pub creation_time: u32,
    pub time: u32,
    pub supply: u64,
    pub address: [u8; 24],
    pub network: u32,
    pub reserved: u32,
    pub slot: u64,
    pub all_time_trades_count: u64,
    pub all_time_base_crncy_volume: u128,
    pub all_time_tokens_volume: u128,
    pub status: u64,
}

pub mod token_account_offsets {
    pub const TAG: usize = 0;
    pub const VERSION: usize = 4;
    pub const ID: usize = 8;
    pub const MINT: usize = 16;
    pub const PROGRAM_ADDRESS: usize = 48;
    pub const CREATOR: usize = 80;
    pub const CREATION_TIME: usize = 112;
    pub const TIME: usize = 116;
    pub const SUPPLY: usize = 120;
    pub const ADDRESS: usize = 128;
    pub const NETWORK: usize = 152;
    pub const SLOT: usize = 160;
    pub const ALL_TIME_TRADES_COUNT: usize = 168;
    pub const ALL_TIME_BASE_CRNCY_VOLUME: usize = 176;
    pub const ALL_TIME_TOKENS_VOLUME: usize = 192;
    pub const VALIDATION: usize = 208;
}

#[repr(C)]
pub struct ClientAccount {
    pub tag: u32,
    pub version: u32,
    pub id: u64,
    pub wallet: Pubkey,
    pub all_time_base_crncy_volume: u64,
    pub all_time_tokens_volume: u64,
    pub slot: u64,
    pub time: u32,
    pub tokens_created: u32,
    pub ref_stop: u32,
    pub all_time_trades_count: u32,
    pub nickname: [u8; 32],
    pub ref_address: Pubkey,
    pub ref_paid: u64,
    pub ref_discount: f64,
    pub ref_ratio: f64,
}

pub mod client_account_offsets {
    pub const TAG: usize = 0;
    pub const VERSION: usize = 4;
    pub const ID: usize = 8;
    pub const WALLET: usize = 16;
    pub const ALL_TIME_BASE_CRNCY_VOLUME: usize = 48;
    pub const ALL_TIME_TOKENS_VOLUME: usize = 56;
    pub const SLOT: usize = 64;
    pub const TIME: usize = 72;
    pub const TOKENS_CREATED: usize = 76;
    pub const REF_STOP: usize = 80;
    pub const ALL_TIME_TRADES_COUNT: usize = 84;
    pub const NICKNAME: usize = 88;
    pub const REF_ADDRESS: usize = 120;
    pub const REF_PAID: usize = 152;
    pub const REF_DISCOUNT: usize = 160;
    pub const REF_RATIO: usize = 168;
}

#[repr(C)]
pub struct HolderAccount {
    pub tag: u32,
    pub version: u32,
    pub wallet: Pubkey,
    pub slot: u64,
    pub time: u32,
    pub operators_count: u32,
}

pub mod holder_account_offsets {
    pub const TAG: usize = 0;
    pub const VERSION: usize = 4;
    pub const WALLET: usize = 8;
    pub const SLOT: usize = 40;
    pub const TIME: usize = 48;
    pub const OPERATORS_COUNT: usize = 52;
    pub const OPERATORS_OFFSET: usize = 56;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OperatorRecord {
    pub operator_address: Pubkey,
    pub version: u32,
    pub max_networks_count: u32,
    pub operator_name: [u8; 32],
}

pub mod operator_record_offsets {
    pub const OPERATOR_ADDRESS: usize = 0;
    pub const VERSION: usize = 32;
    pub const OPERATOR_NAME: usize = 36;
    pub const MAX_NETWORKS_COUNT: usize = 68;
}

#[repr(C)]
pub struct RootAccount {
    pub tag: u32,
    pub version: u32,
    pub admin: Pubkey,
    pub fee_wallet: Pubkey,
    pub base_crncy_mint: Pubkey,
    pub base_crncy_program_address: Pubkey,
    pub clients_count: u64,
    pub tokens_count: u64,
    pub fees: u64,
    pub networks_count: u32,
    pub base_crncy_decs_factor: u32,
    pub slot: u64,
    pub time: u32,
    pub decimals: u32,
    pub supply: u64,
    pub tvl: u64,
    pub counter: u64,
    pub all_time_base_crncy_volume: u128,
    pub all_time_tokens_volume: u128,
    pub holder_fees: u64,
    pub init_price: f64,
    pub max_supply: u64,
    pub fee_ratio: f64,
    pub fee_rate: f64,
    pub creation_fee: f64,
    pub max_networks_count: u32,
    pub creation_time: u32,
    pub min_fee: f64,
    pub operator_name: [u8; 32],
    pub ref_duration: u32,
    pub mask: u32,
    pub ref_discount: f64,
    pub ref_ratio: f64,
    pub url_prefix: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkRecord {
    pub max_length: usize,
    pub validator: Pubkey,
    pub descriptor: [u8; 32],
    pub mask: [u8; 64],
}

pub mod network_record_offsets {
    pub const MAX_LENGTH: usize = 0;
    pub const VALIDATOR: usize = 8;
    pub const DESCRIPTOR: usize = 40;
    pub const MASK: usize = 72;
}

pub mod root_account_offsets {
    pub const TAG: usize = 0;
    pub const VERSION: usize = 4;
    pub const ADMIN: usize = 8;
    pub const FEE_WALLET: usize = 40;
    pub const BASE_CRNCY_MINT: usize = 72;
    pub const BASE_CRNCY_PROGRAM_ADDRESS: usize = 104;
    pub const CLIENTS_COUNT: usize = 136;
    pub const TOKENS_COUNT: usize = 144;
    pub const FEES: usize = 152;
    pub const NETWORKS_COUNT: usize = 160;
    pub const BASE_CRNCY_DECS_FACTOR: usize = 164;
    pub const SLOT: usize = 168;
    pub const TIME: usize = 176;
    pub const DECIMALS: usize = 180;
    pub const SUPPLY: usize = 184;
    pub const TVL: usize = 192;
    pub const COUNTER: usize = 200;
    pub const ALL_TIME_BASE_CRNCY_VOLUME: usize = 208;
    pub const ALL_TIME_TOKENS_VOLUME: usize = 224;
    pub const HOLDER_FEES: usize = 240;
    pub const INIT_PRICE: usize = 248;
    pub const MAX_SUPPLY: usize = 256;
    pub const FEE_RATIO: usize = 264;
    pub const FEE_RATE: usize = 272;
    pub const CREATION_FEE: usize = 280;
    pub const MAX_NETWORKS_COUNT: usize = 288;
    pub const CREATION_TIME: usize = 292;
    pub const MIN_FEE: usize = 296;
    pub const OPERATOR_NANE: usize = 304;
    pub const REF_DURATION: usize = 336;
    pub const MASK: usize = 340;
    pub const REF_DISCOUNT: usize = 344;
    pub const REF_RATIO: usize = 352;
    pub const URL_PREFIX: usize = 360;
    pub const NETWORK_RECORDS: usize = 392;
}

pub struct Context<'a, 'info> {
    pub root: *mut RootAccount,
    pub client: *mut ClientAccount,
    pub hype_auth: Pubkey,
    pub hype_bump_seed: u8,
    pub signer: &'a AccountInfo<'info>,
    pub root_acc: &'a AccountInfo<'info>,
    pub client_associated_token_acc: &'a AccountInfo<'info>,
    pub client_associated_hype_acc: &'a AccountInfo<'info>,
    pub token_acc: &'a AccountInfo<'info>,
    pub base_crncy_mint_acc: &'a AccountInfo<'info>,
    pub base_crncy_program_acc: &'a AccountInfo<'info>,
    pub hype_mint_acc: &'a AccountInfo<'info>,
    pub hype_program_acc: &'a AccountInfo<'info>,
    pub hype_auth_acc: &'a AccountInfo<'info>,
    pub token_program_id: &'a AccountInfo<'info>,
    pub token_2022_program_id: &'a AccountInfo<'info>,
    pub ref_acc: &'a AccountInfo<'info>,
    pub ref_associated_token_acc: &'a AccountInfo<'info>,
    pub slot: u64,
    pub time: u32,
}

impl<'a, 'info> Context<'a, 'info> {
    pub unsafe fn new(
        program_id: &Pubkey,
        accounts_iter: &mut Iter<'a, AccountInfo<'info>>,
        instruction_data: &[u8],
        nickname_offset: usize,
        //ref_offset: usize,
    ) -> Result<Self, ProgramError> {
        let signer = next_account_info(accounts_iter)?;
        let client_acc = next_account_info(accounts_iter)?;
        let client_associated_token_acc = next_account_info(accounts_iter)?;
        let client_associated_hype_acc = next_account_info(accounts_iter)?;
        let root_acc = next_account_info(accounts_iter)?;
        let token_acc = next_account_info(accounts_iter)?;
        let base_crncy_mint_acc = next_account_info(accounts_iter)?;
        let base_crncy_program_acc = next_account_info(accounts_iter)?;
        let hype_mint_acc = next_account_info(accounts_iter)?;
        let hype_program_acc = next_account_info(accounts_iter)?;
        let hype_auth_acc = next_account_info(accounts_iter)?;
        let token_program_id = next_account_info(accounts_iter)?;
        let token_2022_program_id = next_account_info(accounts_iter)?;
        let system_program_acc = next_account_info(accounts_iter)?;
        let associated_token_id = next_account_info(accounts_iter)?;
        let ref_acc = next_account_info(accounts_iter)?;
        let ref_associated_token_acc = next_account_info(accounts_iter)?;
        if *ref_acc.key != system_program::ID {
            let ref_expected_address = get_associated_token_address_with_program_id(
                ref_acc.key,
                base_crncy_mint_acc.key,
                token_program_id.key,
            );
            if ref_expected_address != *ref_associated_token_acc.key {
                return Err(InvalidRefAddress.into());
            }
        }
        if *token_program_id.key != spl_token::id() {
            return Err(InvalidTokenProgramId.into());
        }
        if *token_2022_program_id.key != spl_token_2022::id() {
            return Err(InvalidToken2022ProgramId.into());
        }
        if *associated_token_id.key != spl_associated_token_account::id() {
            return Err(InvalidAssociatedTokenId.into());
        }
        if !system_program::check_id(system_program_acc.key) {
            return Err(InvalidSystemProgramId.into());
        }
        if root_acc.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let root = root_acc.data.borrow().as_ptr() as *mut RootAccount;
        if (*root).tag != ROOT_TAG {
            return Err(InvalidAccountTag.into());
        }
        if (*root).base_crncy_mint != *base_crncy_mint_acc.key {
            return Err(InvalidBaseCrncyMint.into());
        }
        if (*root).base_crncy_program_address != *base_crncy_program_acc.key {
            return Err(InvalidBaseCrncyProgramAddress.into());
        }
        let expected_address = get_associated_token_address_with_program_id(
            signer.key,
            base_crncy_mint_acc.key,
            token_program_id.key,
        );
        if expected_address != *client_associated_token_acc.key {
            return Err(InvalidAssociatedTokenAddress.into());
        }
        let expected_hype_address = get_associated_token_address_with_program_id(
            signer.key,
            hype_mint_acc.key,
            token_2022_program_id.key,
        );
        if expected_hype_address != *client_associated_hype_acc.key {
            return Err(InvalidAssociatedTokenAddress.into());
        }
        let clock = Clock::get()?;
        let time = clock.unix_timestamp as u32;
        let slot = clock.slot;
        let client: *mut ClientAccount;
        if client_acc.owner == &system_program::ID {
            let client_seed = get_seed_by_tag((*root).version, CLIENT_TAG as u32);
            let client_bump_seed = check_account(client_acc, signer.key, program_id, &client_seed)?;
            let rent = &Rent::default();
            let client_lamports = rent.minimum_balance(CLIENT_ACCOUNT_SIZE);
            invoke_signed(
                &system_instruction::create_account(
                    signer.key,
                    client_acc.key,
                    client_lamports,
                    CLIENT_ACCOUNT_SIZE as u64,
                    program_id,
                ),
                &[signer.clone(), client_acc.clone()],
                &[&[&client_seed, signer.key.as_ref(), &[client_bump_seed]]],
            )?;
            client = client_acc.data.borrow().as_ptr() as *mut ClientAccount;
            let ref_stop: u32;
            if *ref_acc.key != system_program::ID && ref_acc.lamports() > 0 {
                ref_stop = clock.unix_timestamp as u32 + (*root).ref_duration;
            } else {
                ref_stop = clock.unix_timestamp as u32;
            }
            *client = ClientAccount {
                tag: CLIENT_TAG,
                version: (*root).version,
                wallet: *signer.key,
                all_time_base_crncy_volume: 0,
                all_time_tokens_volume: 0,
                ref_stop: ref_stop,
                ref_paid: 0,
                all_time_trades_count: 0,
                id: (*root).clients_count,
                slot: clock.slot,
                time: clock.unix_timestamp as u32,
                tokens_created: 0,
                nickname: *((instruction_data[nickname_offset..]).as_ptr()
                    as *const [u8; NICKNAME_STRING_LENGTH]),
                ref_address: *ref_acc.key,
                ref_discount: (*root).ref_discount,
                ref_ratio: (*root).ref_ratio,
            };
            log_new_client(
                (*root).clients_count,
                (*root).counter,
                signer.key,
                clock.unix_timestamp as u32,
                clock.slot,
            );
            (*root).clients_count += 1;
            (*root).counter += 1;
        } else {
            if client_acc.owner != program_id {
                return Err(InvalidClientAccount.into());
            }
            client = client_acc.data.borrow().as_ptr() as *mut ClientAccount;
            if (*client).tag != CLIENT_TAG || (*client).wallet != *signer.key {
                return Err(InvalidClientAccount.into());
            }
        }
        let (hype_auth, hype_bump_seed) = Pubkey::find_program_address(&[HYPE_SEED], program_id);
        if hype_auth != *hype_auth_acc.key {
            return Err(InvalidHypeAuthority.into());
        }
        Ok(Context {
            root: root,
            client: client,
            hype_auth: hype_auth,
            hype_bump_seed: hype_bump_seed,
            root_acc: root_acc,
            signer: signer,
            client_associated_token_acc: client_associated_token_acc,
            client_associated_hype_acc: client_associated_hype_acc,
            token_acc: token_acc,
            base_crncy_mint_acc: base_crncy_mint_acc,
            base_crncy_program_acc: base_crncy_program_acc,
            hype_mint_acc: hype_mint_acc,
            hype_program_acc: hype_program_acc,
            hype_auth_acc: hype_auth_acc,
            token_program_id: token_program_id,
            token_2022_program_id: token_2022_program_id,
            ref_acc: ref_acc,
            ref_associated_token_acc: ref_associated_token_acc,
            time: time,
            slot: slot,
        })
    }
}

pub unsafe fn check_holder_account(
    account: &AccountInfo,
    program_id: &Pubkey,
    writable: bool,
) -> ProgramResult {
    if account.owner != program_id
        || account.is_writable != writable
        || *(account.data.borrow().as_ptr() as *const u32) != HOLDER_TAG
    {
        return Err(InvalidHolderAccount.into());
    }
    Ok(())
}

pub fn check_holder_admin(account: &AccountInfo) -> ProgramResult {
    let admin = Pubkey::from_str("5V5zbRbs7wFAu5bE2JgYC3aLXKmfwA1rKfg1cXCsuk1p").unwrap();
    if *account.key != admin || !account.is_writable || !account.is_signer {
        return Err(InvalidHolderAdmin.into());
    }
    Ok(())
}

pub fn check_name(
    name: &[u8],
    mask: &[u8; MASK_STRING_LENGTH],
    max_length: usize,
) -> ProgramResult {
    let mut name_offset = 0;
    while name_offset < ADDRESS_STRING_LENGTH {
        if name_offset > max_length {
            return Err(InvalidAddress.into());
        }
        if name[name_offset] == 0 {
            break;
        }
        let mut mask_offset = 0;
        let mut valid = false;
        while mask_offset < MASK_STRING_LENGTH {
            mask_offset += 1;
            if mask[mask_offset] == 0 {
                break;
            }
            if name[name_offset] == mask[mask_offset] {
                valid = true;
                break;
            }
        }
        if !valid {
            return Err(InvalidAddress.into());
        }
        name_offset += 1;
    }
    Ok(())
}
