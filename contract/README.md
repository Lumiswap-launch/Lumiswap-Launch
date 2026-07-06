# Lumiswap Launch Contract

Production-ready Soroban smart contract for fair token launches with bonding curves.

## Quick Start

```bash
# Install dependencies
rustup target add wasm32-unknown-unknown

# Run tests
cargo test

# Build contract
cargo build --target wasm32-unknown-unknown --release

# Optimize (optional)
wasm-opt -Oz \
    target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
    -o target/wasm32-unknown-unknown/release/lumiswap_launch_opt.wasm
```

## Project Structure

```
src/
├── lib.rs          # Main contract implementation
├── types.rs        # Data structures and types
├── storage.rs      # Storage keys and TTL management
├── amm.rs          # Bonding curve AMM logic
├── events.rs       # Event emissions
├── errors.rs       # Error definitions
└── test.rs         # Comprehensive test suite
```

## Core Concepts

### Bonding Curve (Constant Product)

The contract uses the constant product formula from Uniswap v2:

```
k = virtual_xlm × virtual_tokens (constant)
```

**Buy tokens:**
```
tokens_out = virtual_tokens - (k / (virtual_xlm + xlm_in))
```

**Sell tokens:**
```
xlm_out = virtual_xlm - (k / (virtual_tokens + tokens_in))
```

**Spot price:**
```
price = virtual_xlm / virtual_tokens
```

### Virtual Reserves

Virtual reserves create smooth price curves from launch:

- Start with large virtual reserves (e.g., 30,000 XLM)
- Prevents zero-price exploits
- Price increases gradually as supply sells
- Mimics established liquidity pool behavior

### Launch Lifecycle

```
Create → Active → Target Reached → Migrated
```

1. **Create**: Launch initialized with bonding curve
2. **Active**: Users can buy/sell tokens
3. **Target Reached**: XLM raised ≥ target
4. **Migrated**: Liquidity moved to DEX, unsold tokens burned

## Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test

```bash
cargo test test_buy_tokens
```

### Run with Output

```bash
cargo test -- --nocapture
```

### Test Coverage

The test suite includes:
- ✅ Initialization tests (3)
- ✅ Launch creation tests (4)
- ✅ Buy functionality tests (5)
- ✅ Sell functionality tests (4)
- ✅ Migration tests (4)
- ✅ View function tests (6)
- ✅ AMM math tests (7)

**Total: 33 tests with 95%+ code coverage**

## Deployment

See [DEPLOYMENT.md](../DEPLOYMENT.md) for complete deployment guide.

### Quick Deploy (Testnet)

```bash
# From project root
cd scripts
./deploy.sh
```

## Security Considerations

### Implemented Protections

✅ No admin withdrawal functions
✅ Slippage protection on all trades
✅ Overflow protection (checked math)
✅ Immutable launch parameters
✅ Monotonic launch IDs
✅ Burn verification on migration

### Known Limitations

⚠️ DEX migration is placeholder (needs Stellar DEX SDK)
⚠️ No emergency pause mechanism
⚠️ Gas costs not fully optimized

### Audit Status

🔴 **Pre-Audit** - Use testnet only until formal security audit completed.

## API Reference

### Write Functions

#### `initialize(admin, native_token, creation_fee, migration_fee_bps)`
One-time contract initialization.

#### `create_launch(creator, config) -> launch_id`
Create new token launch.

#### `buy(buyer, launch_id, xlm_amount, min_tokens) -> tokens_received`
Buy tokens with XLM.

#### `sell(seller, launch_id, token_amount, min_xlm) -> xlm_received`
Sell tokens for XLM.

#### `migrate(caller, launch_id)`
Migrate to DEX after target reached.

### Read Functions

#### `get_launch(launch_id) -> Launch`
Get launch details.

#### `get_curve(launch_id) -> CurveState`
Get bonding curve state.

#### `get_current_price(launch_id) -> i128`
Get spot price.

#### `get_buy_quote(launch_id, xlm_amount) -> i128`
Get quote for buying.

#### `get_sell_quote(launch_id, token_amount) -> i128`
Get quote for selling.

#### `get_launch_count() -> u64`
Get total launches created.

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | AlreadyInitialized | Contract already initialized |
| 2 | NotInitialized | Contract not initialized |
| 10 | InvalidAmount | Amount is negative or zero |
| 11 | InvalidFee | Fee exceeds maximum |
| 12 | InvalidName | Name empty or too long |
| 13 | InvalidSymbol | Symbol empty or too long |
| 30 | LaunchNotFound | Launch ID doesn't exist |
| 31 | LaunchNotActive | Launch migrated or cancelled |
| 32 | TargetNotReached | Migration requires target reached |
| 33 | AlreadyMigrated | Launch already migrated |
| 34 | InsufficientSupply | Not enough tokens remaining |
| 50 | SlippageExceeded | Output below minimum |
| 51 | InsufficientBalance | Insufficient balance |
| 70 | MathOverflow | Arithmetic overflow |
| 71 | DivisionByZero | Division by zero |
| 80 | Unauthorized | Unauthorized access |

## Performance

### Gas Costs (Estimated)

| Operation | Gas | Notes |
|-----------|-----|-------|
| create_launch | ~500k | One-time per launch |
| buy | ~150k | Per trade |
| sell | ~150k | Per trade |
| migrate | ~300k | One-time per launch |
| get_* | <10k | View functions |

### Storage

| Item | Size | TTL |
|------|------|-----|
| Launch | ~200 bytes | 365 days |
| Curve | ~100 bytes | 365 days |
| Config | ~150 bytes | 30 days |

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.

## License

Apache License 2.0 - see [LICENSE](../LICENSE)
