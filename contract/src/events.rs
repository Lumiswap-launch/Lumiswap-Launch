//! Event emissions for Lumiswap Launch contract.
//!
//! Events are indexed and queryable via Stellar Horizon API.

use soroban_sdk::{symbol_short, Address, Env, String};
use crate::types::LaunchConfig;

/// Emitted when contract is initialized.
pub fn emit_initialized(
    env: &Env,
    admin: &Address,
    creation_fee: i128,
    migration_fee_bps: u32,
) {
    env.events().publish(
        (symbol_short!("init"),),
        (admin, creation_fee, migration_fee_bps),
    );
}

/// Emitted when a new launch is created.
pub fn emit_launch_created(
    env: &Env,
    launch_id: u64,
    creator: &Address,
    config: &LaunchConfig,
) {
    env.events().publish(
        (symbol_short!("created"), launch_id),
        (
            creator,
            config.token.clone(),
            config.name.clone(),
            config.symbol.clone(),
            config.total_supply,
            config.target_xlm,
        ),
    );
}

/// Emitted when tokens are bought.
pub fn emit_buy(
    env: &Env,
    launch_id: u64,
    buyer: &Address,
    xlm_amount: i128,
    token_amount: i128,
) {
    env.events().publish(
        (symbol_short!("buy"), launch_id),
        (buyer, xlm_amount, token_amount),
    );
}

/// Emitted when tokens are sold.
pub fn emit_sell(
    env: &Env,
    launch_id: u64,
    seller: &Address,
    token_amount: i128,
    xlm_amount: i128,
) {
    env.events().publish(
        (symbol_short!("sell"), launch_id),
        (seller, token_amount, xlm_amount),
    );
}

/// Emitted when launch is migrated to DEX.
pub fn emit_migrated(
    env: &Env,
    launch_id: u64,
    xlm_for_liquidity: i128,
    tokens_for_liquidity: i128,
    protocol_fee: i128,
    tokens_burned: i128,
) {
    env.events().publish(
        (symbol_short!("migrated"), launch_id),
        (
            xlm_for_liquidity,
            tokens_for_liquidity,
            protocol_fee,
            tokens_burned,
        ),
    );
}
