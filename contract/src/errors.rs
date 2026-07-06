//! Error types for Lumiswap Launch contract.

use soroban_sdk::contracterror;

/// Contract errors with descriptive codes.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Initialization errors (1-9)
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Contract not initialized
    NotInitialized = 2,

    // Validation errors (10-29)
    /// Invalid amount (negative or zero)
    InvalidAmount = 10,
    /// Invalid fee (exceeds maximum)
    InvalidFee = 11,
    /// Invalid name (empty or too long)
    InvalidName = 12,
    /// Invalid symbol (empty or too long)
    InvalidSymbol = 13,
    
    // Launch errors (30-49)
    /// Launch not found
    LaunchNotFound = 30,
    /// Launch not active
    LaunchNotActive = 31,
    /// Target not reached yet
    TargetNotReached = 32,
    /// Already migrated
    AlreadyMigrated = 33,
    /// Insufficient supply remaining
    InsufficientSupply = 34,

    // Trading errors (50-69)
    /// Slippage tolerance exceeded
    SlippageExceeded = 50,
    /// Insufficient balance
    InsufficientBalance = 51,

    // Math errors (70-79)
    /// Arithmetic overflow
    MathOverflow = 70,
    /// Division by zero
    DivisionByZero = 71,

    // Authorization errors (80-89)
    /// Unauthorized access
    Unauthorized = 80,
}
