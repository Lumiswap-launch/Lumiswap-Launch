#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, Symbol,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn setup() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, LaunchpadContract);
    (env, admin, contract_id)
}

/// Deploy a Stellar Asset Contract (SAC) and return its address plus an admin client.
fn deploy_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    token_id.address()
}

fn mint(env: &Env, token: &Address, admin: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
    let _ = admin; // kept for clarity
}

fn balance(env: &Env, token: &Address, who: &Address) -> i128 {
    TokenClient::new(env, token).balance(who)
}

// cp_out mirrors the contract helper so we can assert exact values in tests.
fn cp_out(dx: i128, x: i128, y: i128) -> i128 {
    let new_x = x + dx;
    y - (x * y) / new_x
}

// ── 1. initialize ─────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &1_000_000i128, &100u32);
    assert_eq!(client.launch_count(), 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_double_panics() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &1_000_000i128, &100u32);
    client.initialize(&admin, &0i128, &0u32);
}

// ── 2. create_launch ──────────────────────────────────────────────────────────

#[test]
fn test_create_launch() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &0i128, &100u32);

    let creator = Address::generate(&env);
    let token = deploy_token(&env, &creator);
    let supply = 1_000_000_000i128;
    mint(&env, &token, &creator, &creator, supply);

    let id = client.create_launch(
        &creator,
        &token,
        &Symbol::new(&env, "TEST"),
        &supply,
        &(30_000 * 10_000_000i128),
    );

    assert_eq!(id, 0);
    assert_eq!(client.launch_count(), 1);

    // Full supply should now be held by the contract
    assert_eq!(balance(&env, &token, &contract_id), supply);
    assert_eq!(balance(&env, &token, &creator), 0);

    let launch = client.get_launch(&id);
    assert_eq!(launch.total_supply, supply);
    assert_eq!(launch.sold, 0);
    assert!(!launch.migrated);
}

// ── 3. buy ────────────────────────────────────────────────────────────────────

#[test]
fn test_buy_correct_tokens_and_curve() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &0i128, &100u32);

    let creator = Address::generate(&env);
    let token = deploy_token(&env, &creator);
    let supply = 1_000_000_000i128;
    mint(&env, &token, &creator, &creator, supply);

    let id = client.create_launch(
        &creator,
        &token,
        &Symbol::new(&env, "TEST"),
        &supply,
        &(300_000 * 10_000_000i128),
    );

    // Mint XLM-equivalent token to buyer and into contract so transfers work.
    // The contract's buy() transfers tokens *out* to buyer; XLM accounting is
    // recorded internally (no SAC XLM transfer in this implementation).
    let buyer = Address::generate(&env);
    let xlm_in = 1_000 * 10_000_000i128; // 1 000 XLM in stroops

    let virtual_xlm = 30_000 * 10_000_000i128;
    let virtual_tokens = supply;
    let expected_out = cp_out(xlm_in, virtual_xlm, virtual_tokens);

    let tokens_out = client.buy(&buyer, &id, &xlm_in, &1i128);

    assert_eq!(tokens_out, expected_out);
    assert_eq!(balance(&env, &token, &buyer), expected_out);

    let curve = client.get_curve(&id);
    assert_eq!(curve.virtual_xlm, virtual_xlm + xlm_in);
    assert_eq!(curve.virtual_tokens, virtual_tokens - expected_out);

    let launch = client.get_launch(&id);
    assert_eq!(launch.sold, expected_out);
    assert_eq!(launch.xlm_raised, xlm_in);
}

#[test]
#[should_panic(expected = "slippage")]
fn test_buy_slippage_check() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &0i128, &100u32);

    let creator = Address::generate(&env);
    let token = deploy_token(&env, &creator);
    let supply = 1_000_000_000i128;
    mint(&env, &token, &creator, &creator, supply);

    let id = client.create_launch(
        &creator,
        &token,
        &Symbol::new(&env, "TEST"),
        &supply,
        &(300_000 * 10_000_000i128),
    );

    // min_tokens set absurdly high → should fail slippage check
    client.buy(&Address::generate(&env), &id, &1_000_000i128, &i128::MAX);
}

// ── 4. sell ───────────────────────────────────────────────────────────────────

#[test]
fn test_sell_correct_xlm_and_curve() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &0i128, &100u32);

    let creator = Address::generate(&env);
    let token = deploy_token(&env, &creator);
    let supply = 1_000_000_000i128;
    mint(&env, &token, &creator, &creator, supply);

    let target = 300_000 * 10_000_000i128;
    let id = client.create_launch(
        &creator,
        &token,
        &Symbol::new(&env, "TEST"),
        &supply,
        &target,
    );

    // Buy some tokens first
    let buyer = Address::generate(&env);
    let xlm_in = 10_000 * 10_000_000i128;
    let tokens_bought = client.buy(&buyer, &id, &xlm_in, &1i128);

    // Capture curve state after buy
    let curve_after_buy = client.get_curve(&id);

    // Sell half back
    let tokens_to_sell = tokens_bought / 2;
    let expected_xlm = cp_out(tokens_to_sell, curve_after_buy.virtual_tokens, curve_after_buy.virtual_xlm);

    let xlm_out = client.sell(&buyer, &id, &tokens_to_sell, &1i128);

    assert_eq!(xlm_out, expected_xlm);
    assert_eq!(balance(&env, &token, &buyer), tokens_bought - tokens_to_sell);

    let launch = client.get_launch(&id);
    assert_eq!(launch.sold, tokens_bought - tokens_to_sell);
    assert_eq!(launch.xlm_raised, xlm_in - expected_xlm);
}

// ── 5. Migration scenario ─────────────────────────────────────────────────────

#[test]
fn test_full_migration() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &0i128, &100u32);

    let creator = Address::generate(&env);
    let token = deploy_token(&env, &creator);
    // Use a small supply so we hit the target quickly
    let supply = 10_000_000i128;
    let target = 100 * 10_000_000i128; // 100 XLM target
    mint(&env, &token, &creator, &creator, supply);

    let id = client.create_launch(
        &creator,
        &token,
        &Symbol::new(&env, "TEST"),
        &supply,
        &target,
    );

    // Buy repeatedly until xlm_raised >= target
    let buyer = Address::generate(&env);
    let xlm_per_buy = 50 * 10_000_000i128;
    loop {
        let launch = client.get_launch(&id);
        if launch.xlm_raised >= target {
            break;
        }
        let remaining = launch.total_supply - launch.sold;
        if remaining == 0 {
            break;
        }
        client.buy(&buyer, &id, &xlm_per_buy, &0i128);
    }

    let launch_before = client.get_launch(&id);
    assert!(launch_before.xlm_raised >= target, "target not reached before migrate");

    let unsold_before = launch_before.total_supply - launch_before.sold;

    // Anyone can call migrate
    let anyone = Address::generate(&env);
    client.migrate(&id);

    let launch_after = client.get_launch(&id);
    assert!(launch_after.migrated);

    // Unsold tokens should be burned (contract balance decreases by unsold)
    let contract_token_balance = balance(&env, &token, &contract_id);
    // After migrate: contract held (total_supply - sold) unsold tokens which got burned,
    // plus 0 remaining sold tokens (all transferred to buyer). Contract balance = 0.
    assert_eq!(
        contract_token_balance,
        0,
        "all tokens accounted for: sold to buyers or burned"
    );
    let _ = (unsold_before, anyone);
}

// ── 6. buy/sell after migration reverts ───────────────────────────────────────

#[test]
#[should_panic(expected = "migrated")]
fn test_buy_after_migration_panics() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &0i128, &100u32);

    let creator = Address::generate(&env);
    let token = deploy_token(&env, &creator);
    let supply = 10_000_000i128;
    let target = 100 * 10_000_000i128;
    mint(&env, &token, &creator, &creator, supply);

    let id = client.create_launch(
        &creator,
        &token,
        &Symbol::new(&env, "TEST"),
        &supply,
        &target,
    );

    let buyer = Address::generate(&env);
    // Buy enough to exceed target
    client.buy(&buyer, &id, &(150 * 10_000_000i128), &0i128);
    client.migrate(&id);

    // This must panic with "migrated"
    client.buy(&buyer, &id, &1_000i128, &0i128);
}

#[test]
#[should_panic(expected = "migrated")]
fn test_sell_after_migration_panics() {
    let (env, admin, contract_id) = setup();
    let client = LaunchpadContractClient::new(&env, &contract_id);
    client.initialize(&admin, &0i128, &100u32);

    let creator = Address::generate(&env);
    let token = deploy_token(&env, &creator);
    let supply = 10_000_000i128;
    let target = 100 * 10_000_000i128;
    mint(&env, &token, &creator, &creator, supply);

    let id = client.create_launch(
        &creator,
        &token,
        &Symbol::new(&env, "TEST"),
        &supply,
        &target,
    );

    let buyer = Address::generate(&env);
    let tokens = client.buy(&buyer, &id, &(150 * 10_000_000i128), &0i128);
    client.migrate(&id);

    // This must panic with "migrated"
    client.sell(&buyer, &id, &tokens, &0i128);
}
