//! Core data types for Lumiswap Launch protocol.

use soroban_sdk::{contracttype, Address, String};

/// Launch configuration parameters provided during creation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchConfig {
    /// Token contract address
    pub token: Address,
    /// Token name (max 32 chars)
    pub name: String,
    /// Token symbol (max 12 chars)
    pub symbol: String,
    /// Total token supply to sell
    pub total_supply: i128,
    /// Target XLM amount to raise (in stroops)
    pub target_xlm: i128,
    /// Initial virtual XLM reserves for bonding curve
    pub virtual_xlm: i128,
}

/// Launch state tracking sales and status.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Launch {
    /// Unique launch identifier
    pub id: u64,
    /// Launch creator address
    pub creator: Address,
    /// Token contract address
    pub token: Address,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Total token supply
    pub total_supply: i128,
    /// Tokens sold so far
    pub sold: i128,
    /// XLM raised so far (in stroops)
    pub xlm_raised: i128,
    /// Target XLM to raise (in stroops)
    pub target_xlm: i128,
    /// Current launch status
    pub status: LaunchStatus,
    /// Creation timestamp
    pub created_at: u64,
    /// Migration timestamp (0 if not migrated)
    pub migrated_at: u64,
}

/// Bonding curve state using constant product formula (x * y = k).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurveState {
    /// Virtual XLM reserve
    pub virtual_xlm: i128,
    /// Virtual token reserve
    pub virtual_tokens: i128,
    /// Constant product (k = x * y)
    pub k: i128,
}

/// Launch lifecycle status.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LaunchStatus {
    /// Launch is active, trading enabled
    Active = 0,
    /// Target XLM reached, ready for migration
    TargetReached = 1,
    /// Migrated to DEX, trading disabled
    Migrated = 2,
}

/// Statistics for a launch (computed view).
#[contracttype]
#[derive(Clone, Debug)]
pub struct LaunchStats {
    /// Launch ID
    pub id: u64,
    /// Percentage of tokens sold (0-10000 basis points)
    pub sold_percentage: u32,
    /// Percentage of target reached (0-10000 basis points)
    pub target_percentage: u32,
    /// Current price in stroops per token
    pub current_price: i128,
    /// Market cap in XLM (sold_tokens * price)
    pub market_cap: i128,
    /// Number of unique traders
    pub trader_count: u32,
}
