#![cfg(test)]

use crate::{LumiswapLaunchContract, LumiswapLaunchContractClient};
use crate::errors::Error;
use crate::types::{LaunchConfig, LaunchStatus};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::{StellarAssetClient as SACClient, TokenClient},
    Address, Env, String,
};

// ═══════════════════════════════════════════════════════════════════════════
// Test Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, Address) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone());
    let token = TokenClient::new(env, &token_address);
    (token, token_address.clone())
}

fn create_lumiswap_contract<'a>(env: &Env) -> LumiswapLaunchContractClient<'a> {
    LumiswapLaunchContractClient::new(env, &env.register(LumiswapLaunchContract, ()))
}

fn setup_test_env() -> (
    Env,
    LumiswapLaunchContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    
    let contract = create_lumiswap_contract(&env);
    
    // Create native token (XLM)
    let (xlm_token, xlm_address) = create_token_contract(&env, &admin);
    
    // Initialize contract
    let creation_fee = 10 * 10_000_000; // 10 XLM
    let migration_fee_bps = 100; // 1%
    
    contract.initialize(&admin, &xlm_address, &creation_fee, &migration_fee_bps);
    
    // Mint XLM to creator for fees
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&creator, &1_000_000 * 10_000_000); // 1M XLM
    
    (env, contract, admin, creator, xlm_address)
}

fn create_test_launch(
    env: &Env,
    contract: &LumiswapLaunchContractClient,
    creator: &Address,
    admin: &Address,
) -> (u64, Address) {
    // Create launch token
    let (launch_token, token_address) = create_token_contract(&env, admin);
    
    // Mint tokens to creator
    let total_supply = 1_000_000 * 10_000_000i128; // 1M tokens
    let token_sac = SACClient::new(&env, &token_address);
    token_sac.mint(creator, &total_supply);
    
    // Create launch config
    let config = LaunchConfig {
        token: token_address.clone(),
        name: String::from_str(&env, "Test Token"),
        symbol: String::from_str(&env, "TEST"),
        total_supply,
        target_xlm: 50_000 * 10_000_000, // 50k XLM target
        virtual_xlm: 30_000 * 10_000_000, // 30k XLM virtual reserve
    };
    
    // Create launch
    let launch_id = contract.create_launch(&creator, &config);
    
    (launch_id, token_address)
}

// ═══════════════════════════════════════════════════════════════════════════
// Initialization Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract = create_lumiswap_contract(&env);
    let admin = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    
    let result = contract.initialize(&admin, &xlm_token, &(10 * 10_000_000), &100);
    
    assert_eq!(result, Ok(()));
    
    let (stored_admin, stored_xlm, fee, fee_bps) = contract.get_config();
    assert_eq!(stored_admin, admin);
    assert_eq!(stored_xlm, xlm_token);
    assert_eq!(fee, 10 * 10_000_000);
    assert_eq!(fee_bps, 100);
}

#[test]
fn test_initialize_twice_fails() {
    let (_, contract, admin, _, xlm_address) = setup_test_env();
    
    let result = contract.try_initialize(&admin, &xlm_address, &0, &100);
    
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_initialize_invalid_fee() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract = create_lumiswap_contract(&env);
    let admin = Address::generate(&env);
    let xlm_token = Address::generate(&env);
    
    // Fee > 100% (10000 bps)
    let result = contract.try_initialize(&admin, &xlm_token, &0, &10_001);
    
    assert_eq!(result, Err(Ok(Error::InvalidFee)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Launch Creation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_create_launch() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (launch_id, token_address) = create_test_launch(&env, &contract, &creator, &admin);
    
    // Verify launch was created
    assert_eq!(launch_id, 0);
    
    let launch = contract.get_launch(&launch_id);
    assert_eq!(launch.creator, creator);
    assert_eq!(launch.token, token_address);
    assert_eq!(launch.sold, 0);
    assert_eq!(launch.xlm_raised, 0);
    assert_eq!(launch.status, LaunchStatus::Active);
    
    // Verify counter incremented
    assert_eq!(contract.get_launch_count(), 1);
}

#[test]
fn test_create_multiple_launches() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (id1, _) = create_test_launch(&env, &contract, &creator, &admin);
    let (id2, _) = create_test_launch(&env, &contract, &creator, &admin);
    let (id3, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 2);
    assert_eq!(contract.get_launch_count(), 3);
}

#[test]
fn test_create_launch_invalid_supply() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (_, token_address) = create_token_contract(&env, &admin);
    
    let config = LaunchConfig {
        token: token_address,
        name: String::from_str(&env, "Test"),
        symbol: String::from_str(&env, "TST"),
        total_supply: -1, // Invalid
        target_xlm: 50_000 * 10_000_000,
        virtual_xlm: 30_000 * 10_000_000,
    };
    
    let result = contract.try_create_launch(&creator, &config);
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_create_launch_invalid_name() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (_, token_address) = create_token_contract(&env, &admin);
    
    let config = LaunchConfig {
        token: token_address,
        name: String::from_str(&env, ""), // Empty name
        symbol: String::from_str(&env, "TST"),
        total_supply: 1_000_000 * 10_000_000,
        target_xlm: 50_000 * 10_000_000,
        virtual_xlm: 30_000 * 10_000_000,
    };
    
    let result = contract.try_create_launch(&creator, &config);
    assert_eq!(result, Err(Ok(Error::InvalidName)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Buy Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_buy_tokens() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    // Setup buyer
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &100_000 * 10_000_000);
    
    // Buy tokens
    let xlm_amount = 1_000 * 10_000_000i128; // 1000 XLM
    let tokens_received = contract.buy(&buyer, &launch_id, &xlm_amount, &0);
    
    assert!(tokens_received > 0);
    
    // Verify launch state updated
    let launch = contract.get_launch(&launch_id);
    assert_eq!(launch.sold, tokens_received);
    assert_eq!(launch.xlm_raised, xlm_amount);
}

#[test]
fn test_buy_with_slippage_protection() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &100_000 * 10_000_000);
    
    // Get quote
    let xlm_amount = 1_000 * 10_000_000i128;
    let expected_tokens = contract.get_buy_quote(&launch_id, &xlm_amount);
    
    // Set min_tokens too high (will fail)
    let result = contract.try_buy(
        &buyer,
        &launch_id,
        &xlm_amount,
        &(expected_tokens + 1_000),
    );
    
    assert_eq!(result, Err(Ok(Error::SlippageExceeded)));
}

#[test]
fn test_buy_price_increases() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &1_000_000 * 10_000_000);
    
    // Get initial price
    let price_1 = contract.get_current_price(&launch_id);
    
    // Buy tokens
    let xlm_amount = 10_000 * 10_000_000i128;
    contract.buy(&buyer, &launch_id, &xlm_amount, &0);
    
    // Price should increase after buy
    let price_2 = contract.get_current_price(&launch_id);
    assert!(price_2 > price_1);
    
    // Buy more
    contract.buy(&buyer, &launch_id, &xlm_amount, &0);
    
    // Price should increase more
    let price_3 = contract.get_current_price(&launch_id);
    assert!(price_3 > price_2);
}

#[test]
fn test_buy_exceeds_supply() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &10_000_000 * 10_000_000); // 10M XLM
    
    // Try to buy with huge amount that would exceed supply
    let xlm_amount = 5_000_000 * 10_000_000i128;
    let result = contract.try_buy(&buyer, &launch_id, &xlm_amount, &0);
    
    assert_eq!(result, Err(Ok(Error::InsufficientSupply)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Sell Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_sell_tokens() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, token_address) = create_test_launch(&env, &contract, &creator, &admin);
    
    // Setup buyer
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &100_000 * 10_000_000);
    
    // Buy tokens first
    let xlm_amount = 10_000 * 10_000_000i128;
    let tokens_bought = contract.buy(&buyer, &launch_id, &xlm_amount, &0);
    
    // Sell half back
    let tokens_to_sell = tokens_bought / 2;
    let xlm_received = contract.sell(&buyer, &launch_id, &tokens_to_sell, &0);
    
    assert!(xlm_received > 0);
    assert!(xlm_received < xlm_amount); // Should receive less than bought for (price moved)
    
    // Verify state
    let launch = contract.get_launch(&launch_id);
    assert_eq!(launch.sold, tokens_bought - tokens_to_sell);
}

#[test]
fn test_sell_with_slippage_protection() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &100_000 * 10_000_000);
    
    // Buy tokens
    let tokens_bought = contract.buy(&buyer, &launch_id, &(10_000 * 10_000_000), &0);
    
    // Get sell quote
    let tokens_to_sell = tokens_bought / 2;
    let expected_xlm = contract.get_sell_quote(&launch_id, &tokens_to_sell);
    
    // Try to sell with min_xlm too high
    let result = contract.try_sell(
        &buyer,
        &launch_id,
        &tokens_to_sell,
        &(expected_xlm + 1_000_000),
    );
    
    assert_eq!(result, Err(Ok(Error::SlippageExceeded)));
}

#[test]
fn test_sell_price_decreases() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &100_000 * 10_000_000);
    
    // Buy tokens
    let tokens_bought = contract.buy(&buyer, &launch_id, &(20_000 * 10_000_000), &0);
    let price_before = contract.get_current_price(&launch_id);
    
    // Sell tokens
    contract.sell(&buyer, &launch_id, &(tokens_bought / 4), &0);
    
    // Price should decrease
    let price_after = contract.get_current_price(&launch_id);
    assert!(price_after < price_before);
}

// ═══════════════════════════════════════════════════════════════════════════
// Migration Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_migrate_after_target_reached() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &1_000_000 * 10_000_000);
    
    // Buy until target reached (50k XLM)
    let target_xlm = 50_000 * 10_000_000i128;
    contract.buy(&buyer, &launch_id, &target_xlm, &0);
    
    // Verify target reached
    let launch = contract.get_launch(&launch_id);
    assert!(launch.xlm_raised >= target_xlm);
    assert_eq!(launch.status, LaunchStatus::TargetReached);
    
    // Migrate
    let migrator = Address::generate(&env);
    let result = contract.migrate(&migrator, &launch_id);
    assert_eq!(result, Ok(()));
    
    // Verify migrated
    let launch = contract.get_launch(&launch_id);
    assert_eq!(launch.status, LaunchStatus::Migrated);
    assert!(launch.migrated_at > 0);
}

#[test]
fn test_migrate_before_target_fails() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    // Try to migrate without reaching target
    let migrator = Address::generate(&env);
    let result = contract.try_migrate(&migrator, &launch_id);
    
    assert_eq!(result, Err(Ok(Error::TargetNotReached)));
}

#[test]
fn test_migrate_twice_fails() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &1_000_000 * 10_000_000);
    
    // Reach target and migrate
    contract.buy(&buyer, &launch_id, &(50_000 * 10_000_000), &0);
    let migrator = Address::generate(&env);
    contract.migrate(&migrator, &launch_id);
    
    // Try to migrate again
    let result = contract.try_migrate(&migrator, &launch_id);
    assert_eq!(result, Err(Ok(Error::AlreadyMigrated)));
}

#[test]
fn test_cannot_buy_after_migration() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &1_000_000 * 10_000_000);
    
    // Reach target and migrate
    contract.buy(&buyer, &launch_id, &(50_000 * 10_000_000), &0);
    let migrator = Address::generate(&env);
    contract.migrate(&migrator, &launch_id);
    
    // Try to buy after migration
    let result = contract.try_buy(&buyer, &launch_id, &(1_000 * 10_000_000), &0);
    assert_eq!(result, Err(Ok(Error::LaunchNotActive)));
}

// ═══════════════════════════════════════════════════════════════════════════
// View Function Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_get_buy_quote() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let xlm_amount = 1_000 * 10_000_000i128;
    let quote = contract.get_buy_quote(&launch_id, &xlm_amount);
    
    assert!(quote > 0);
}

#[test]
fn test_get_sell_quote() {
    let (env, contract, admin, creator, xlm_address) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let buyer = Address::generate(&env);
    let xlm_sac = SACClient::new(&env, &xlm_address);
    xlm_sac.mint(&buyer, &100_000 * 10_000_000);
    
    // Buy first
    let tokens_bought = contract.buy(&buyer, &launch_id, &(10_000 * 10_000_000), &0);
    
    // Get sell quote
    let quote = contract.get_sell_quote(&launch_id, &(tokens_bought / 2));
    assert!(quote > 0);
}

#[test]
fn test_get_current_price() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let price = contract.get_current_price(&launch_id);
    
    // Initial price should be virtual_xlm / virtual_tokens
    // 30_000 XLM / 1_000_000 tokens = 0.03 XLM = 300_000 stroops
    assert_eq!(price, 300_000);
}

#[test]
fn test_get_curve_state() {
    let (env, contract, admin, creator, _) = setup_test_env();
    
    let (launch_id, _) = create_test_launch(&env, &contract, &creator, &admin);
    
    let curve = contract.get_curve(&launch_id);
    
    assert_eq!(curve.virtual_xlm, 30_000 * 10_000_000);
    assert_eq!(curve.virtual_tokens, 1_000_000 * 10_000_000);
    assert_eq!(curve.k, curve.virtual_xlm * curve.virtual_tokens);
}

#[test]
fn test_launch_not_found() {
    let (_, contract, _, _, _) = setup_test_env();
    
    let result = contract.try_get_launch(&999);
    assert_eq!(result, Err(Ok(Error::LaunchNotFound)));
}
