use crate::program::*;
use crate::state::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    //msg,
    program::invoke,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction,
    system_program,
    sysvar::rent::Rent,
    sysvar::Sysvar,
};

pub unsafe fn initialize_root(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    /*
    [0..4] - 2
    [4..8] - Version
    [8..16] - Fee ratio
    [16..24] - Init price
    [24..32] - Slope
    [32..40] - Fee rate
    [40..48] - Creation fee
    [48..56] - Min fee
    [56..88] - Url prefix
    [88..92] - Mask
    [92..96] - Ref duration
    [96..104] - Ref discount
    [104..112] - Ref ratio
     */
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let signer = next_account_info(accounts_iter)?;
    let holder_acc = next_account_info(accounts_iter)?;
    let root_acc = next_account_info(accounts_iter)?;
    let base_crncy_mint = next_account_info(accounts_iter)?;
    let base_crncy_program_acc = next_account_info(accounts_iter)?;
    let fee_wallet = next_account_info(accounts_iter)?;
    let hype_auth_acc = next_account_info(accounts_iter)?;
    let token_program_id = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let (hype_auth, _bump_seed) = Pubkey::find_program_address(&[HYPE_SEED], program_id);
    if hype_auth != *hype_auth_acc.key {
        return Err(InvalidHypeAuthority.into());
    }
    let old_token = *token_program_id.key == spl_token::id();
    let token_2022 = *token_program_id.key == spl_token_2022::id();
    if !token_2022 && !old_token {
        return Err(InvalidTokenProgramId.into());
    }
    if !system_program::check_id(system_program.key) {
        return Err(InvalidSystemProgramId.into());
    }
    check_holder_account(holder_acc, program_id, false)?;
    let operators_count = *(holder_acc.data.borrow()[holder_account_offsets::OPERATORS_COUNT..]
        .as_ptr() as *const u32);
    let operators: Vec<OperatorRecord>;
    let begin = HOLDER_ACCOUNT_SIZE + (operators_count as usize) * OPERATOR_RECORD_SIZE;
    let operators_ptr = (&holder_acc.data.borrow()[HOLDER_ACCOUNT_SIZE..begin]).as_ptr();
    operators = Vec::from_raw_parts(
        operators_ptr as *mut OperatorRecord,
        operators_count as usize,
        operators_count as usize,
    );
    let mut version: u32 = 0xFFFFFFFF;
    let input_version = *((instruction_data[4..]).as_ptr() as *const u32);
    let mut max_networks_count = 0;
    let mut operator_name = [0; OPERATOR_NAME_STRING_LENGTH];
    for p in operators {
        if p.version == input_version {
            if p.operator_address != *signer.key {
                return Err(InvalidNewOperatorAccount.into());
            }
            version = p.version;
            max_networks_count = p.max_networks_count;
            operator_name = p.operator_name;
            break;
        }
    }

    if version == 0xFFFFFFFF {
        return Err(InvalidNewOperatorAccount.into());
    }
    let rent = Rent::default();
    let spl_lamports = rent.minimum_balance(165);
    invoke(
        &system_instruction::create_account(
            signer.key,
            base_crncy_program_acc.key,
            spl_lamports,
            165,
            token_program_id.key,
        ),
        &[signer.clone(), base_crncy_program_acc.clone()],
    )?;
    let initialize_account_instruction;
    if token_2022 {
        initialize_account_instruction = spl_token_2022::instruction::initialize_account3(
            token_program_id.key,
            base_crncy_program_acc.key,
            base_crncy_mint.key,
            &hype_auth,
        )?;
    } else {
        initialize_account_instruction = spl_token::instruction::initialize_account3(
            token_program_id.key,
            base_crncy_program_acc.key,
            base_crncy_mint.key,
            &hype_auth,
        )?;
    }
    invoke_signed(
        &initialize_account_instruction,
        &[
            token_program_id.clone(),
            base_crncy_mint.clone(),
            base_crncy_program_acc.clone(),
            hype_auth_acc.clone(),
        ],
        &[&[&HYPE_SEED[..], &[_bump_seed]]],
    )?;
    let root_seed = get_seed_by_tag(version, ROOT_TAG);
    let root_bump_seed = check_new_account(root_acc, &hype_auth, program_id, &root_seed)?;
    let root_lamports = rent.minimum_balance(ROOT_ACCOUNT_SIZE);
    invoke_signed(
        &system_instruction::create_account(
            signer.key,
            root_acc.key,
            root_lamports,
            ROOT_ACCOUNT_SIZE as u64,
            program_id,
        ),
        &[signer.clone(), root_acc.clone()],
        &[&[&root_seed, hype_auth_acc.key.as_ref(), &[root_bump_seed]]],
    )?;
    let decs_count = base_crncy_mint.data.borrow()[44] as u32;
    let mut decimals: u32 = 1;
    for _ in 0..decs_count {
        decimals *= 10;
    }
    let clock = Clock::get()?;
    let mask_offset = 56 + NETWORK_STRING_LENGTH;
    let url_prefix_ptr =
        (instruction_data[56..mask_offset]).as_ptr() as *const [u8; URL_PREFIX_STRING_LENGTH];
    *(root_acc.data.borrow().as_ptr() as *mut RootAccount) = RootAccount {
        tag: ROOT_TAG as u32,
        version: version,
        admin: *signer.key,
        fee_wallet: *fee_wallet.key,
        base_crncy_mint: *base_crncy_mint.key,
        base_crncy_program_address: *base_crncy_program_acc.key,
        clients_count: 0,
        tokens_count: 0,
        networks_count: 0,
        base_crncy_decs_factor: decimals,
        slot: clock.slot,
        time: clock.unix_timestamp as u32,
        fees: 0,
        decimals: decs_count,
        all_time_base_crncy_volume: 0,
        all_time_tokens_volume: 0,
        supply: 0,
        tvl: 0,
        counter: 0,
        fee_ratio: *((instruction_data[8..]).as_ptr() as *const f64),
        init_price: *((instruction_data[16..]).as_ptr() as *const f64),
        max_supply: *((instruction_data[24..]).as_ptr() as *const u64),
        fee_rate: *((instruction_data[32..]).as_ptr() as *const f64),
        creation_fee: *((instruction_data[40..]).as_ptr() as *const f64),
        min_fee: *((instruction_data[48..]).as_ptr() as *const f64),
        holder_fees: 0,
        creation_time: clock.unix_timestamp as u32,
        max_networks_count: max_networks_count,
        operator_name: operator_name,
        url_prefix: *url_prefix_ptr,
        mask: *((instruction_data[mask_offset..]).as_ptr() as *const u32),
        ref_duration: *((instruction_data[mask_offset + 4..]).as_ptr() as *const u32),
        ref_discount: *((instruction_data[mask_offset + 8..]).as_ptr() as *const f64),
        ref_ratio: *((instruction_data[mask_offset + 16..]).as_ptr() as *const f64),
    };
    //return Err(solana_program::program_error::ProgramError::Custom(2000));
    Ok(())
}
