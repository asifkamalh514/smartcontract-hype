use solana_program::pubkey::Pubkey;

pub unsafe fn log_new_client(client_id: u64, order_id: u64, wallet: &Pubkey, time: u32, slot: u64) {
    solana_program::log::sol_log_data(&[
        &[1],
        &client_id.to_le_bytes(),
        &order_id.to_le_bytes(),
        &wallet.to_bytes(),
        &time.to_le_bytes(),
        &slot.to_le_bytes(),
    ]);
}

pub unsafe fn log_new_network(network_id: u32, descriptor: &[u8], time: u32, slot: u64) {
    solana_program::log::sol_log_data(&[
        &[2],
        &network_id.to_le_bytes(),
        descriptor,
        &time.to_le_bytes(),
        &slot.to_le_bytes(),
    ]);
}

pub unsafe fn log_new_token(
    client_id: u64,
    order_id: u64,
    token_id: u64,
    network_id: u32,
    mint: &Pubkey,
    creator: &Pubkey,
    address: &[u8],
    time: u32,
    slot: u64,
) {
    solana_program::log::sol_log_data(&[
        &[3],
        &client_id.to_le_bytes(),
        &order_id.to_le_bytes(),
        &token_id.to_le_bytes(),
        &network_id.to_le_bytes(),
        &mint.to_bytes(),
        &creator.to_bytes(),
        address,
        &time.to_le_bytes(),
        &slot.to_le_bytes(),
    ]);
}

pub unsafe fn log_mint(
    client_id: u64,
    order_id: u64,
    token_id: u64,
    network_id: u32,
    mint: &Pubkey,
    creator: &Pubkey,
    wallet: &Pubkey,
    address: &[u8],
    creation_time: u32,
    supply: u64,
    all_time_trades_count: u64,
    all_time_base_crncy_volume: u64,
    all_time_tokens_volume: u64,
    tokens_amount: u64,
    base_crncy_amount: u64,
    time: u32,
    slot: u64,
) {
    solana_program::log::sol_log_data(&[
        &[4],
        &client_id.to_le_bytes(),
        &order_id.to_le_bytes(),
        &token_id.to_le_bytes(),
        &network_id.to_le_bytes(),
        &mint.to_bytes(),
        &creator.to_bytes(),
        address,
        &supply.to_le_bytes(),
        &creation_time.to_le_bytes(),
        &all_time_trades_count.to_le_bytes(),
        &all_time_base_crncy_volume.to_le_bytes(),
        &all_time_tokens_volume.to_le_bytes(),
        &tokens_amount.to_le_bytes(),
        &base_crncy_amount.to_le_bytes(),
        &time.to_le_bytes(),
        &slot.to_le_bytes(),
        &wallet.to_bytes(),
    ]);
}

pub unsafe fn log_burn(
    client_id: u64,
    order_id: u64,
    token_id: u64,
    network_id: u32,
    mint: &Pubkey,
    creator: &Pubkey,
    wallet: &Pubkey,
    address: &[u8],
    creation_time: u32,
    supply: u64,
    all_time_trades_count: u64,
    all_time_base_crncy_volume: u64,
    all_time_tokens_volume: u64,
    tokens_amount: u64,
    base_crncy_amount: u64,
    time: u32,
    slot: u64,
) {
    solana_program::log::sol_log_data(&[
        &[5],
        &client_id.to_le_bytes(),
        &order_id.to_le_bytes(),
        &token_id.to_le_bytes(),
        &network_id.to_le_bytes(),
        &mint.to_bytes(),
        &creator.to_bytes(),
        address,
        &supply.to_le_bytes(),
        &creation_time.to_le_bytes(),
        &all_time_trades_count.to_le_bytes(),
        &all_time_base_crncy_volume.to_le_bytes(),
        &all_time_tokens_volume.to_le_bytes(),
        &tokens_amount.to_le_bytes(),
        &base_crncy_amount.to_le_bytes(),
        &time.to_le_bytes(),
        &slot.to_le_bytes(),
        &wallet.to_bytes(),
    ]);
}
