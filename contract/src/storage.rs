//! Storage keys and TTL management for Lumiswap Launch.

use soroban_sdk::{contracttype, Env};

/// Time-to-live extension constants (in ledgers).
/// At ~5 seconds per ledger:
/// - INSTANCE_LIFETIME: ~30 days
/// - PERSISTENT_LIFETIME: ~365 days
const INSTANCE_LIFETIME_THRESHOLD: u32 = 518_400; // 30 days
const INSTANCE_BUMP_AMOUNT: u32 = 518_400; // 30 days

const PERSISTENT_LIFETIME_THRESHOLD: u32 = 6_220_800; // 365 days  
const PERSISTENT_BUMP_AMOUNT: u32 = 6_220_800; // 365 days

/// Storage keys for contract data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Contract administrator
    Admin,
    /// Native token (XLM) contract address
    NativeToken,
    /// Fee to create a launch (in stroops)
    CreationFee,
    /// Migration fee in basis points
    MigrationFeeBps,
    /// Launch counter (monotonic ID)
    LaunchCounter,
    /// Launch data by ID
    Launch(u64),
    /// Bonding curve state by launch ID
    Curve(u64),
    /// Trading volume by launch ID (optional analytics)
    Volume(u64),
    /// Unique trader count by launch ID (optional analytics)
    TraderCount(u64),
}

/// Extend the TTL of instance storage.
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Extend the TTL of persistent storage for a specific key.
pub fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}
