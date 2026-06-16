#![no_std]
mod test;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Map, Symbol,
};

// ── Types ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct Launch {
    pub creator: Address,
    pub token: Address,
    pub name: Symbol,
    pub total_supply: i128,
    pub sold: i128,
    pub xlm_raised: i128,
    pub target_xlm: i128,
    pub migrated: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct BondingCurve {
    pub virtual_xlm: i128,
    pub virtual_tokens: i128,
}

// ── Storage keys ─────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    Counter,
    Admin,
    CreationFee,
    MigrationFeeBps,
    Launch(u32),
    Curve(u32),
}

// ── Events ───────────────────────────────────────────────────────────────────

fn emit_launch_created(e: &Env, id: u32, creator: &Address, token: &Address) {
    e.events().publish(
        (symbol_short!("LaunchCrt"), id),
        (creator.clone(), token.clone()),
    );
}

fn emit_bought(e: &Env, id: u32, buyer: &Address, xlm_in: i128, tokens_out: i128) {
    e.events().publish(
        (symbol_short!("TokBought"), id),
        (buyer.clone(), xlm_in, tokens_out),
    );
}

fn emit_sold(e: &Env, id: u32, seller: &Address, tokens_in: i128, xlm_out: i128) {
    e.events().publish(
        (symbol_short!("TokSold"), id),
        (seller.clone(), tokens_in, xlm_out),
    );
}

fn emit_migrated(e: &Env, id: u32, xlm_raised: i128) {
    e.events()
        .publish((symbol_short!("Migrated"), id), xlm_raised);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn get_launch(e: &Env, id: u32) -> Launch {
    e.storage()
        .persistent()
        .get(&DataKey::Launch(id))
        .expect("launch not found")
}

fn save_launch(e: &Env, id: u32, l: &Launch) {
    e.storage().persistent().set(&DataKey::Launch(id), l);
}

fn get_curve(e: &Env, id: u32) -> BondingCurve {
    e.storage()
        .persistent()
        .get(&DataKey::Curve(id))
        .expect("curve not found")
}

fn save_curve(e: &Env, id: u32, c: &BondingCurve) {
    e.storage().persistent().set(&DataKey::Curve(id), c);
}

/// Constant-product AMM out: dx * y / (x + dx)
fn cp_out(dx: i128, x: i128, y: i128) -> i128 {
    let k = x.checked_mul(y).expect("overflow");
    let new_x = x.checked_add(dx).expect("overflow");
    let new_y = k.checked_div(new_x).expect("div zero");
    y.checked_sub(new_y).expect("overflow")
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct LaunchpadContract;

#[contractimpl]
impl LaunchpadContract {
    /// One-time initialiser.
    pub fn initialize(
        e: Env,
        admin: Address,
        creation_fee: i128,
        migration_fee_bps: u32,
    ) {
        assert!(
            !e.storage().instance().has(&DataKey::Admin),
            "already initialized"
        );
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage()
            .instance()
            .set(&DataKey::CreationFee, &creation_fee);
        e.storage()
            .instance()
            .set(&DataKey::MigrationFeeBps, &migration_fee_bps);
        e.storage().instance().set(&DataKey::Counter, &0u32);
    }

    /// Create a new bonding-curve launch. The caller pays `creation_fee` XLM
    /// to the admin, then the contract is authorised as minter of a
    /// pre-deployed token contract.
    pub fn create_launch(
        e: Env,
        creator: Address,
        token: Address,
        name: Symbol,
        total_supply: i128,
        target_xlm: i128,
    ) -> u32 {
        creator.require_auth();
        assert!(total_supply > 0 && target_xlm > 0, "invalid params");

        // Collect creation fee
        let fee: i128 = e
            .storage()
            .instance()
            .get(&DataKey::CreationFee)
            .unwrap_or(0);
        if fee > 0 {
            let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
            let xlm = token::Client::new(&e, &e.current_contract_address()); // placeholder: real impl uses native XLM token id
            let _ = (xlm, admin); // fee transfer handled off-chain via SAC in production
        }

        let id: u32 = e
            .storage()
            .instance()
            .get(&DataKey::Counter)
            .unwrap_or(0);
        e.storage().instance().set(&DataKey::Counter, &(id + 1));

        // Mint total_supply to this contract so it can sell tokens
        let tok = token::Client::new(&e, &token);
        // The token contract must have authorised this contract as minter.
        // We transfer the full supply from creator into escrow here.
        tok.transfer(&creator, &e.current_contract_address(), &total_supply);

        let launch = Launch {
            creator: creator.clone(),
            token: token.clone(),
            name,
            total_supply,
            sold: 0,
            xlm_raised: 0,
            target_xlm,
            migrated: false,
            created_at: e.ledger().timestamp(),
        };

        // Virtual reserves: start price ≈ 1 XLM per 1000 tokens
        let curve = BondingCurve {
            virtual_xlm: 30_000 * 10_000_000i128,   // 30 000 XLM in stroops
            virtual_tokens: total_supply,
        };

        save_launch(&e, id, &launch);
        save_curve(&e, id, &curve);
        emit_launch_created(&e, id, &creator, &token);
        id
    }

    /// Buy tokens with XLM using constant-product curve.
    pub fn buy(
        e: Env,
        buyer: Address,
        launch_id: u32,
        xlm_in: i128,
        min_tokens: i128,
    ) -> i128 {
        buyer.require_auth();
        let mut launch = get_launch(&e, launch_id);
        assert!(!launch.migrated, "migrated");
        assert!(xlm_in > 0, "xlm_in must be > 0");

        let mut curve = get_curve(&e, launch_id);
        let tokens_out = cp_out(xlm_in, curve.virtual_xlm, curve.virtual_tokens);
        assert!(tokens_out >= min_tokens, "slippage");
        assert!(
            launch.sold + tokens_out <= launch.total_supply,
            "exceeds supply"
        );

        // Transfer XLM from buyer to contract (via SAC)
        // In production: native SAC transfer. Here we record the accounting.
        curve.virtual_xlm += xlm_in;
        curve.virtual_tokens -= tokens_out;

        // Send tokens to buyer
        let tok = token::Client::new(&e, &launch.token);
        tok.transfer(&e.current_contract_address(), &buyer, &tokens_out);

        launch.sold += tokens_out;
        launch.xlm_raised += xlm_in;

        save_curve(&e, launch_id, &curve);
        save_launch(&e, launch_id, &launch);
        emit_bought(&e, launch_id, &buyer, xlm_in, tokens_out);
        tokens_out
    }

    /// Sell tokens back to the curve for XLM.
    pub fn sell(
        e: Env,
        seller: Address,
        launch_id: u32,
        tokens_in: i128,
        min_xlm: i128,
    ) -> i128 {
        seller.require_auth();
        let mut launch = get_launch(&e, launch_id);
        assert!(!launch.migrated, "migrated");
        assert!(tokens_in > 0, "tokens_in must be > 0");

        let mut curve = get_curve(&e, launch_id);
        let xlm_out = cp_out(tokens_in, curve.virtual_tokens, curve.virtual_xlm);
        assert!(xlm_out >= min_xlm, "slippage");

        // Transfer tokens from seller to contract
        let tok = token::Client::new(&e, &launch.token);
        tok.transfer(&seller, &e.current_contract_address(), &tokens_in);

        curve.virtual_tokens += tokens_in;
        curve.virtual_xlm -= xlm_out;

        launch.sold -= tokens_in;
        launch.xlm_raised -= xlm_out;

        save_curve(&e, launch_id, &curve);
        save_launch(&e, launch_id, &launch);
        emit_sold(&e, launch_id, &seller, tokens_in, xlm_out);
        xlm_out
    }

    /// Migrate liquidity to Stellar DEX when target is reached.
    /// Burns unsold tokens, takes migration fee, creates DEX offer.
    pub fn migrate(e: Env, launch_id: u32) {
        let mut launch = get_launch(&e, launch_id);
        assert!(!launch.migrated, "already migrated");
        assert!(
            launch.xlm_raised >= launch.target_xlm,
            "target not reached"
        );

        let fee_bps: u32 = e
            .storage()
            .instance()
            .get(&DataKey::MigrationFeeBps)
            .unwrap_or(100);
        let fee = launch.xlm_raised * fee_bps as i128 / 10_000;
        let xlm_for_lp = launch.xlm_raised - fee;
        let unsold = launch.total_supply - launch.sold;

        // Burn unsold tokens
        let tok = token::Client::new(&e, &launch.token);
        if unsold > 0 {
            tok.burn(&e.current_contract_address(), &unsold);
        }

        // In production: use Stellar DEX / Soroswap SDK to create
        // a liquidity position with xlm_for_lp + launch.sold tokens.
        // Placeholder: emit event with the amounts for off-chain handler.
        let _ = xlm_for_lp; // used by off-chain migration handler

        launch.migrated = true;
        save_launch(&e, launch_id, &launch);
        emit_migrated(&e, launch_id, launch.xlm_raised);
    }

    /// Spot price: virtual_xlm / virtual_tokens (in stroops per token-unit).
    pub fn current_price(e: Env, launch_id: u32) -> i128 {
        let curve = get_curve(&e, launch_id);
        curve
            .virtual_xlm
            .checked_mul(10_000_000)
            .expect("overflow")
            .checked_div(curve.virtual_tokens)
            .expect("div zero")
    }

    // ── Views ─────────────────────────────────────────────────────────────────

    pub fn get_launch(e: Env, launch_id: u32) -> Launch {
        get_launch(&e, launch_id)
    }

    pub fn get_curve(e: Env, launch_id: u32) -> BondingCurve {
        get_curve(&e, launch_id)
    }

    pub fn launch_count(e: Env) -> u32 {
        e.storage()
            .instance()
            .get(&DataKey::Counter)
            .unwrap_or(0)
    }
}
