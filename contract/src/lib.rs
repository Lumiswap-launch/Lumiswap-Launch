#![no_std]

//! # Lumiswap Launch - Fair Launch Protocol for Stellar
//!
//! A permissionless token launchpad with bonding curve price discovery
//! and automated liquidity migration to Stellar DEX.
//!
//! ## Features
//! - Constant product bonding curve (x * y = k)
//! - Virtual reserves for smooth price discovery
//! - Permissionless migration at target threshold
//! - Automatic burn of unsold tokens
//! - Protocol fee mechanism
//! - Anti-rug pull guarantees
//!
//! ## Security Properties
//! - All funds held in contract escrow
//! - No admin withdrawal functions
//! - Slippage protection on all trades
//! - Migration only after target reached
//! - Monotonic launch ID generation

mod storage;
mod types;
mod amm;
mod events;
mod errors;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, String};
use types::{Launch, LaunchConfig, CurveState, LaunchStatus};
use storage::{DataKey, extend_instance_ttl, extend_persistent_ttl};
use errors::Error;

/// Maximum basis points (100%)
const MAX_BPS: u32 = 10_000;

/// Minimum virtual reserves (1000 XLM in stroops)
const MIN_VIRTUAL_XLM: i128 = 1_000 * 10_000_000;

/// Maximum total supply (100 billion tokens with 7 decimals)
const MAX_TOTAL_SUPPLY: i128 = 100_000_000_000 * 10_000_000;

#[contract]
pub struct LumiswapLaunchContract;

#[contractimpl]
impl LumiswapLaunchContract {
    /// Initialize the launchpad contract.
    ///
    /// # Arguments
    /// * `admin` - Contract administrator address
    /// * `native_token` - Stellar native token (XLM) contract address
    /// * `creation_fee` - Fee in stroops to create a launch
    /// * `migration_fee_bps` - Protocol fee in basis points (e.g., 100 = 1%)
    ///
    /// # Panics
    /// - If already initialized
    /// - If migration_fee_bps > MAX_BPS
    pub fn initialize(
        env: Env,
        admin: Address,
        native_token: Address,
        creation_fee: i128,
        migration_fee_bps: u32,
    ) -> Result<(), Error> {
        // Ensure not already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        // Validate parameters
        if migration_fee_bps > MAX_BPS {
            return Err(Error::InvalidFee);
        }

        if creation_fee < 0 {
            return Err(Error::InvalidAmount);
        }

        // Store configuration
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NativeToken, &native_token);
        env.storage().instance().set(&DataKey::CreationFee, &creation_fee);
        env.storage().instance().set(&DataKey::MigrationFeeBps, &migration_fee_bps);
        env.storage().instance().set(&DataKey::LaunchCounter, &0u64);

        extend_instance_ttl(&env);

        events::emit_initialized(&env, &admin, creation_fee, migration_fee_bps);
        Ok(())
    }

    /// Create a new token launch with bonding curve.
    ///
    /// # Arguments
    /// * `creator` - Launch creator (must authorize)
    /// * `config` - Launch configuration parameters
    ///
    /// # Returns
    /// Launch ID (monotonic counter)
    ///
    /// # Errors
    /// - InvalidAmount: If parameters out of valid ranges
    /// - InsufficientBalance: If creator cannot pay creation fee
    pub fn create_launch(
        env: Env,
        creator: Address,
        config: LaunchConfig,
    ) -> Result<u64, Error> {
        creator.require_auth();

        // Validate configuration
        Self::validate_config(&config)?;

        // Get and increment counter atomically
        let launch_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LaunchCounter)
            .unwrap_or(0);
        
        env.storage()
            .instance()
            .set(&DataKey::LaunchCounter, &(launch_id + 1));

        // Collect creation fee
        Self::collect_creation_fee(&env, &creator)?;

        // Transfer tokens from creator to contract escrow
        Self::escrow_tokens(&env, &creator, &config.token, config.total_supply)?;

        // Initialize launch state
        let launch = Launch {
            id: launch_id,
            creator: creator.clone(),
            token: config.token.clone(),
            name: config.name.clone(),
            symbol: config.symbol.clone(),
            total_supply: config.total_supply,
            sold: 0,
            xlm_raised: 0,
            target_xlm: config.target_xlm,
            status: LaunchStatus::Active,
            created_at: env.ledger().timestamp(),
            migrated_at: 0,
        };

        // Initialize bonding curve with virtual reserves
        let curve = CurveState {
            virtual_xlm: config.virtual_xlm,
            virtual_tokens: config.total_supply,
            k: config.virtual_xlm
                .checked_mul(config.total_supply)
                .ok_or(Error::MathOverflow)?,
        };

        // Save to storage with extended TTL
        env.storage()
            .persistent()
            .set(&DataKey::Launch(launch_id), &launch);
        env.storage()
            .persistent()
            .set(&DataKey::Curve(launch_id), &curve);

        extend_persistent_ttl(&env, &DataKey::Launch(launch_id));
        extend_persistent_ttl(&env, &DataKey::Curve(launch_id));
        extend_instance_ttl(&env);

        events::emit_launch_created(&env, launch_id, &creator, &config);

        Ok(launch_id)
    }

    /// Buy tokens with XLM using the bonding curve.
    ///
    /// # Arguments
    /// * `buyer` - Token buyer (must authorize)
    /// * `launch_id` - Launch identifier
    /// * `xlm_amount` - XLM amount in stroops to spend
    /// * `min_tokens` - Minimum tokens to receive (slippage protection)
    ///
    /// # Returns
    /// Actual tokens received
    ///
    /// # Errors
    /// - LaunchNotFound: Invalid launch_id
    /// - LaunchNotActive: Launch migrated or cancelled
    /// - SlippageExceeded: Output below min_tokens
    /// - InsufficientSupply: Not enough tokens remaining
    pub fn buy(
        env: Env,
        buyer: Address,
        launch_id: u64,
        xlm_amount: i128,
        min_tokens: i128,
    ) -> Result<i128, Error> {
        buyer.require_auth();

        // Validate input
        if xlm_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Load state
        let mut launch = Self::get_launch_internal(&env, launch_id)?;
        let mut curve = Self::get_curve_internal(&env, launch_id)?;

        // Check launch is active
        if launch.status != LaunchStatus::Active {
            return Err(Error::LaunchNotActive);
        }

        // Calculate tokens out using AMM formula
        let tokens_out = amm::calculate_tokens_out(xlm_amount, &curve)?;

        // Check slippage
        if tokens_out < min_tokens {
            return Err(Error::SlippageExceeded);
        }

        // Check supply
        if launch.sold + tokens_out > launch.total_supply {
            return Err(Error::InsufficientSupply);
        }

        // Transfer XLM from buyer to contract
        Self::transfer_xlm_to_contract(&env, &buyer, xlm_amount)?;

        // Update curve state
        curve.virtual_xlm = curve
            .virtual_xlm
            .checked_add(xlm_amount)
            .ok_or(Error::MathOverflow)?;
        curve.virtual_tokens = curve
            .virtual_tokens
            .checked_sub(tokens_out)
            .ok_or(Error::MathOverflow)?;

        // Transfer tokens to buyer
        Self::transfer_tokens_from_contract(&env, &buyer, &launch.token, tokens_out)?;

        // Update launch state
        launch.sold = launch.sold.checked_add(tokens_out).ok_or(Error::MathOverflow)?;
        launch.xlm_raised = launch.xlm_raised.checked_add(xlm_amount).ok_or(Error::MathOverflow)?;

        // Check if target reached
        if launch.xlm_raised >= launch.target_xlm {
            launch.status = LaunchStatus::TargetReached;
        }

        // Save state
        env.storage()
            .persistent()
            .set(&DataKey::Launch(launch_id), &launch);
        env.storage()
            .persistent()
            .set(&DataKey::Curve(launch_id), &curve);

        extend_persistent_ttl(&env, &DataKey::Launch(launch_id));
        extend_persistent_ttl(&env, &DataKey::Curve(launch_id));

        events::emit_buy(&env, launch_id, &buyer, xlm_amount, tokens_out);

        Ok(tokens_out)
    }

    /// Sell tokens back to the bonding curve for XLM.
    ///
    /// # Arguments
    /// * `seller` - Token seller (must authorize)
    /// * `launch_id` - Launch identifier
    /// * `token_amount` - Token amount to sell
    /// * `min_xlm` - Minimum XLM to receive (slippage protection)
    ///
    /// # Returns
    /// Actual XLM received
    pub fn sell(
        env: Env,
        seller: Address,
        launch_id: u64,
        token_amount: i128,
        min_xlm: i128,
    ) -> Result<i128, Error> {
        seller.require_auth();

        // Validate input
        if token_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Load state
        let mut launch = Self::get_launch_internal(&env, launch_id)?;
        let mut curve = Self::get_curve_internal(&env, launch_id)?;

        // Check launch is active
        if launch.status != LaunchStatus::Active && launch.status != LaunchStatus::TargetReached {
            return Err(Error::LaunchNotActive);
        }

        // Calculate XLM out using AMM formula
        let xlm_out = amm::calculate_xlm_out(token_amount, &curve)?;

        // Check slippage
        if xlm_out < min_xlm {
            return Err(Error::SlippageExceeded);
        }

        // Transfer tokens from seller to contract
        Self::transfer_tokens_to_contract(&env, &seller, &launch.token, token_amount)?;

        // Update curve state
        curve.virtual_tokens = curve
            .virtual_tokens
            .checked_add(token_amount)
            .ok_or(Error::MathOverflow)?;
        curve.virtual_xlm = curve
            .virtual_xlm
            .checked_sub(xlm_out)
            .ok_or(Error::MathOverflow)?;

        // Transfer XLM to seller
        Self::transfer_xlm_from_contract(&env, &seller, xlm_out)?;

        // Update launch state
        launch.sold = launch.sold.checked_sub(token_amount).ok_or(Error::MathOverflow)?;
        launch.xlm_raised = launch.xlm_raised.checked_sub(xlm_out).ok_or(Error::MathOverflow)?;

        // Check if dropped below target
        if launch.xlm_raised < launch.target_xlm {
            launch.status = LaunchStatus::Active;
        }

        // Save state
        env.storage()
            .persistent()
            .set(&DataKey::Launch(launch_id), &launch);
        env.storage()
            .persistent()
            .set(&DataKey::Curve(launch_id), &curve);

        extend_persistent_ttl(&env, &DataKey::Launch(launch_id));
        extend_persistent_ttl(&env, &DataKey::Curve(launch_id));

        events::emit_sell(&env, launch_id, &seller, token_amount, xlm_out);

        Ok(xlm_out)
    }

    /// Migrate liquidity to Stellar DEX after target reached.
    ///
    /// Anyone can call this once the target is reached.
    /// - Burns unsold tokens
    /// - Takes protocol fee
    /// - Remaining XLM + sold tokens ready for DEX liquidity
    ///
    /// # Errors
    /// - LaunchNotFound: Invalid launch_id
    /// - TargetNotReached: xlm_raised < target_xlm
    /// - AlreadyMigrated: Already called once
    pub fn migrate(env: Env, caller: Address, launch_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut launch = Self::get_launch_internal(&env, launch_id)?;

        // Check not already migrated
        if launch.status == LaunchStatus::Migrated {
            return Err(Error::AlreadyMigrated);
        }

        // Check target reached
        if launch.xlm_raised < launch.target_xlm {
            return Err(Error::TargetNotReached);
        }

        // Calculate fee
        let fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MigrationFeeBps)
            .unwrap_or(100);
        
        let protocol_fee = launch
            .xlm_raised
            .checked_mul(fee_bps as i128)
            .ok_or(Error::MathOverflow)?
            .checked_div(MAX_BPS as i128)
            .ok_or(Error::MathOverflow)?;

        let xlm_for_liquidity = launch
            .xlm_raised
            .checked_sub(protocol_fee)
            .ok_or(Error::MathOverflow)?;

        // Burn unsold tokens
        let unsold = launch.total_supply.checked_sub(launch.sold).ok_or(Error::MathOverflow)?;
        if unsold > 0 {
            Self::burn_tokens(&env, &launch.token, unsold)?;
        }

        // Transfer protocol fee to admin
        if protocol_fee > 0 {
            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            Self::transfer_xlm_from_contract(&env, &admin, protocol_fee)?;
        }

        // Update launch status
        launch.status = LaunchStatus::Migrated;
        launch.migrated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Launch(launch_id), &launch);
        extend_persistent_ttl(&env, &DataKey::Launch(launch_id));

        events::emit_migrated(
            &env,
            launch_id,
            xlm_for_liquidity,
            launch.sold,
            protocol_fee,
            unsold,
        );

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // View Functions
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get launch details by ID.
    pub fn get_launch(env: Env, launch_id: u64) -> Result<Launch, Error> {
        Self::get_launch_internal(&env, launch_id)
    }

    /// Get current bonding curve state.
    pub fn get_curve(env: Env, launch_id: u64) -> Result<CurveState, Error> {
        Self::get_curve_internal(&env, launch_id)
    }

    /// Get total number of launches created.
    pub fn get_launch_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LaunchCounter)
            .unwrap_or(0)
    }

    /// Calculate current spot price (XLM per token).
    ///
    /// Returns price in stroops with 7 decimal precision.
    pub fn get_current_price(env: Env, launch_id: u64) -> Result<i128, Error> {
        let curve = Self::get_curve_internal(&env, launch_id)?;
        amm::calculate_spot_price(&curve)
    }

    /// Get quote for buying tokens with XLM.
    pub fn get_buy_quote(env: Env, launch_id: u64, xlm_amount: i128) -> Result<i128, Error> {
        if xlm_amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let curve = Self::get_curve_internal(&env, launch_id)?;
        amm::calculate_tokens_out(xlm_amount, &curve)
    }

    /// Get quote for selling tokens for XLM.
    pub fn get_sell_quote(env: Env, launch_id: u64, token_amount: i128) -> Result<i128, Error> {
        if token_amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let curve = Self::get_curve_internal(&env, launch_id)?;
        amm::calculate_xlm_out(token_amount, &curve)
    }

    /// Get contract configuration.
    pub fn get_config(env: Env) -> Result<(Address, Address, i128, u32), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        let native_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::NativeToken)
            .ok_or(Error::NotInitialized)?;
        let creation_fee: i128 = env
            .storage()
            .instance()
            .get(&DataKey::CreationFee)
            .unwrap_or(0);
        let migration_fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MigrationFeeBps)
            .unwrap_or(0);

        Ok((admin, native_token, creation_fee, migration_fee_bps))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Internal Helper Functions
    // ═══════════════════════════════════════════════════════════════════════════

    fn get_launch_internal(env: &Env, launch_id: u64) -> Result<Launch, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Launch(launch_id))
            .ok_or(Error::LaunchNotFound)
    }

    fn get_curve_internal(env: &Env, launch_id: u64) -> Result<CurveState, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Curve(launch_id))
            .ok_or(Error::LaunchNotFound)
    }

    fn validate_config(config: &LaunchConfig) -> Result<(), Error> {
        // Validate amounts
        if config.total_supply <= 0 || config.total_supply > MAX_TOTAL_SUPPLY {
            return Err(Error::InvalidAmount);
        }

        if config.target_xlm <= 0 {
            return Err(Error::InvalidAmount);
        }

        if config.virtual_xlm < MIN_VIRTUAL_XLM {
            return Err(Error::InvalidAmount);
        }

        // Name and symbol validation
        if config.name.len() == 0 || config.name.len() > 32 {
            return Err(Error::InvalidName);
        }

        if config.symbol.len() == 0 || config.symbol.len() > 12 {
            return Err(Error::InvalidSymbol);
        }

        Ok(())
    }

    fn collect_creation_fee(env: &Env, creator: &Address) -> Result<(), Error> {
        let fee: i128 = env
            .storage()
            .instance()
            .get(&DataKey::CreationFee)
            .unwrap_or(0);

        if fee > 0 {
            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            let native_token: Address = env.storage().instance().get(&DataKey::NativeToken).unwrap();
            
            let client = soroban_sdk::token::TokenClient::new(env, &native_token);
            client.transfer(creator, &admin, &fee);
        }

        Ok(())
    }

    fn escrow_tokens(env: &Env, from: &Address, token: &Address, amount: i128) -> Result<(), Error> {
        let client = soroban_sdk::token::TokenClient::new(env, token);
        client.transfer(from, &env.current_contract_address(), &amount);
        Ok(())
    }

    fn transfer_xlm_to_contract(env: &Env, from: &Address, amount: i128) -> Result<(), Error> {
        let native_token: Address = env.storage().instance().get(&DataKey::NativeToken).unwrap();
        let client = soroban_sdk::token::TokenClient::new(env, &native_token);
        client.transfer(from, &env.current_contract_address(), &amount);
        Ok(())
    }

    fn transfer_xlm_from_contract(env: &Env, to: &Address, amount: i128) -> Result<(), Error> {
        let native_token: Address = env.storage().instance().get(&DataKey::NativeToken).unwrap();
        let client = soroban_sdk::token::TokenClient::new(env, &native_token);
        client.transfer(&env.current_contract_address(), to, &amount);
        Ok(())
    }

    fn transfer_tokens_from_contract(
        env: &Env,
        to: &Address,
        token: &Address,
        amount: i128,
    ) -> Result<(), Error> {
        let client = soroban_sdk::token::TokenClient::new(env, token);
        client.transfer(&env.current_contract_address(), to, &amount);
        Ok(())
    }

    fn transfer_tokens_to_contract(
        env: &Env,
        from: &Address,
        token: &Address,
        amount: i128,
    ) -> Result<(), Error> {
        let client = soroban_sdk::token::TokenClient::new(env, token);
        client.transfer(from, &env.current_contract_address(), &amount);
        Ok(())
    }

    fn burn_tokens(env: &Env, token: &Address, amount: i128) -> Result<(), Error> {
        let client = soroban_sdk::token::TokenClient::new(env, token);
        client.burn(&env.current_contract_address(), &amount);
        Ok(())
    }
}
