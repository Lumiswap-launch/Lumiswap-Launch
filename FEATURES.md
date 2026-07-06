# Lumiswap Launch - Feature Specification

## Core Features

### 1. Token Launch Creation ✅

**User Story:** As a token creator, I want to launch my token with fair price discovery.

**Features:**
- Permissionless launch creation
- Configurable total supply
- Configurable target XLM amount
- Adjustable virtual reserves for price curve
- Automatic token escrow
- Creation fee payment

**Technical Implementation:**
```rust
pub fn create_launch(
    creator: Address,
    config: LaunchConfig,
) -> Result<u64, Error>
```

**Security:**
- ✅ Authorization check on creator
- ✅ Parameter validation (supply, target, reserves)
- ✅ Token transfer verification
- ✅ Monotonic launch ID generation

---

### 2. Bonding Curve Trading ✅

**User Story:** As a trader, I want to buy/sell tokens at fair algorithmic prices.

**Buy Features:**
- Instant token purchase with XLM
- Real-time price calculation
- Slippage protection
- Quote preview
- Price impact display

**Sell Features:**
- Instant token sale for XLM
- Fair exit liquidity
- Slippage protection
- No lock-up periods

**Technical Implementation:**
```rust
// Buy
pub fn buy(
    buyer: Address,
    launch_id: u64,
    xlm_amount: i128,
    min_tokens: i128,
) -> Result<i128, Error>

// Sell
pub fn sell(
    seller: Address,
    launch_id: u64,
    token_amount: i128,
    min_xlm: i128,
) -> Result<i128, Error>
```

**AMM Formula:**
```
k = x * y (constant product)
tokens_out = y - (k / (x + xlm_in))
xlm_out = x - (k / (y + tokens_in))
```

**Security:**
- ✅ Slippage protection (min_tokens/min_xlm)
- ✅ Overflow protection (checked math)
- ✅ Supply validation
- ✅ Status checks (not migrated)

---

### 3. Automated Migration ✅

**User Story:** As a community member, I want automatic DEX migration when target is reached.

**Features:**
- Permissionless trigger (anyone can call)
- Automatic unsold token burn
- Protocol fee collection
- Liquidity ready for DEX
- Irreversible migration

**Technical Implementation:**
```rust
pub fn migrate(
    caller: Address,
    launch_id: u64,
) -> Result<(), Error>
```

**Migration Process:**
1. Verify target reached: `xlm_raised >= target_xlm`
2. Calculate protocol fee: `fee = xlm_raised * fee_bps / 10000`
3. Burn unsold tokens: `total_supply - sold`
4. Transfer fee to admin
5. Mark as migrated
6. Emit event with liquidity amounts

**Security:**
- ✅ Target verification
- ✅ One-time migration (cannot repeat)
- ✅ Automatic burn (cannot skip)
- ✅ Fee transparency

---

### 4. Price Discovery ✅

**User Story:** As a trader, I want to see current prices and get quotes.

**Features:**
- Real-time spot price
- Buy quote preview
- Sell quote preview
- Price history (via events)
- Price impact calculation

**Technical Implementation:**
```rust
pub fn get_current_price(launch_id: u64) -> Result<i128, Error>
pub fn get_buy_quote(launch_id: u64, xlm_amount: i128) -> Result<i128, Error>
pub fn get_sell_quote(launch_id: u64, token_amount: i128) -> Result<i128, Error>
```

**Price Calculation:**
```rust
price = (virtual_xlm * 10_000_000) / virtual_tokens
```

Returns price in stroops with 7 decimal precision.

---

### 5. Launch Information ✅

**User Story:** As a user, I want to see launch details and status.

**Features:**
- Launch details (creator, token, supply, etc.)
- Current status (Active/TargetReached/Migrated)
- Progress tracking (sold, xlm_raised)
- Curve state (virtual reserves)
- Creation timestamp

**Technical Implementation:**
```rust
pub fn get_launch(launch_id: u64) -> Result<Launch, Error>
pub fn get_curve(launch_id: u64) -> Result<CurveState, Error>
pub fn get_launch_count() -> u64
```

**Data Structure:**
```rust
pub struct Launch {
    id: u64,
    creator: Address,
    token: Address,
    name: String,
    symbol: String,
    total_supply: i128,
    sold: i128,
    xlm_raised: i128,
    target_xlm: i128,
    status: LaunchStatus,
    created_at: u64,
    migrated_at: u64,
}
```

---

## Advanced Features (Planned)

### 6. Multiple Curve Types 🚧

**Goal:** Support different bonding curve shapes.

**Curves:**
- Linear: `price = base_price + (sold * price_increment)`
- Exponential: `price = base_price * e^(sold * growth_rate)`
- Sigmoid: `price = max_price / (1 + e^(-k * (sold - midpoint)))`
- Custom: User-defined formula

**Use Cases:**
- Linear: Predictable price increases
- Exponential: Rewards early buyers more
- Sigmoid: Slow start, fast middle, slow end

### 7. Fee Distribution 🚧

**Goal:** Share protocol revenue with stakeholders.

**Distribution:**
- Protocol treasury: 40%
- Stakers: 40%
- Referrers: 20%

**Features:**
- Automatic distribution on migration
- Staking rewards
- Referral tracking
- Treasury governance

### 8. Launch Templates 🚧

**Goal:** Simplify launch creation with presets.

**Templates:**
- **Fair Launch**: 1M tokens, 50k XLM target
- **Mega Launch**: 100M tokens, 500k XLM target
- **Micro Launch**: 100k tokens, 5k XLM target
- **Custom**: User-defined parameters

### 9. Price Alerts 🚧

**Goal:** Notify users of price changes.

**Features:**
- Target price alerts
- Percentage change alerts
- Volume alerts
- Migration alerts

**Channels:**
- Email
- Telegram bot
- Push notifications
- Discord webhooks

### 10. Portfolio Tracking 🚧

**Goal:** Track user holdings and performance.

**Metrics:**
- Total launches participated
- Total XLM invested
- Total tokens held
- Unrealized P&L
- Realized P&L

---

## Security Features

### 1. Anti-Rug Pull ✅

**Protections:**
- No admin withdrawal function
- Funds locked in contract
- Permissionless migration
- Automatic token burn
- Immutable parameters

### 2. Slippage Protection ✅

**Mechanism:**
```rust
// User specifies minimum acceptable output
buy(xlm_amount, min_tokens)
sell(token_amount, min_xlm)

// Contract enforces
if actual_output < minimum {
    return Error::SlippageExceeded;
}
```

### 3. Overflow Protection ✅

**Implementation:**
```rust
// All arithmetic uses checked operations
let result = a.checked_add(b).ok_or(Error::MathOverflow)?;
let result = a.checked_mul(b).ok_or(Error::MathOverflow)?;
let result = a.checked_div(b).ok_or(Error::DivisionByZero)?;
```

### 4. Access Control ✅

**Checks:**
- Creator authorization on create
- Buyer authorization on buy
- Seller authorization on sell
- Anyone can migrate (if target reached)

### 5. State Integrity ✅

**Guarantees:**
- Monotonic launch IDs
- Immutable curve constant (k)
- Status transitions enforced
- No negative balances

---

## Event System

### Event Types ✅

```rust
// Contract initialized
emit_initialized(admin, creation_fee, migration_fee_bps)

// Launch created
emit_launch_created(launch_id, creator, config)

// Tokens bought
emit_buy(launch_id, buyer, xlm_amount, token_amount)

// Tokens sold
emit_sell(launch_id, seller, token_amount, xlm_amount)

// Launch migrated
emit_migrated(launch_id, xlm_for_liquidity, tokens, fee, burned)
```

### Event Usage

**Indexing:**
- Events indexed by Horizon API
- Queryable via REST
- Real-time subscriptions
- Historical data

**Frontend Integration:**
```typescript
// Listen for new launches
const events = await horizon.operations()
    .forContract(contractId)
    .stream({
        onmessage: (event) => {
            if (event.topic === 'created') {
                // Handle new launch
            }
        }
    });
```

---

## Performance Characteristics

### Gas Costs ⚡

| Operation | Gas | Notes |
|-----------|-----|-------|
| initialize | 50k | One-time |
| create_launch | 500k | Includes token transfer |
| buy | 150k | Per trade |
| sell | 150k | Per trade |
| migrate | 300k | Includes burn |
| get_* | <10k | Read-only |

### Throughput 📊

- **Launches:** 1000+ supported
- **Trades per launch:** Unlimited
- **TPS limit:** Stellar network (~1000 TPS)
- **Latency:** ~5s (ledger close time)

### Storage 💾

| Item | Size | TTL |
|------|------|-----|
| Launch | 200 bytes | 365 days |
| Curve | 100 bytes | 365 days |
| Config | 150 bytes | 30 days |

---

## Integration Points

### 1. Wallet Integration ✅

**Supported:**
- Freighter ✅
- Albedo (planned)
- Ledger (planned)

**Features:**
- Connect wallet
- Sign transactions
- View balances
- Transaction history

### 2. Stellar DEX Integration 🚧

**Migration Target:**
- Create liquidity pool
- Seed with XLM + tokens
- Set initial price
- Enable trading

**Status:** Awaiting Stellar DEX SDK

### 3. Analytics Integration 🚧

**Metrics:**
- Total value locked
- Trading volume
- Active launches
- User statistics
- Price charts

**Providers:**
- Internal analytics
- Dune Analytics
- DefiLlama
- Custom dashboards

### 4. Price Feed Integration 🚧

**Sources:**
- Internal bonding curve
- Stellar DEX price
- External aggregators
- TWAP calculation

---

## User Flows

### Create Launch Flow

```
1. Connect Wallet
   ↓
2. Deploy Token Contract
   ↓
3. Mint Tokens to Creator
   ↓
4. Configure Launch
   - Name & Symbol
   - Total Supply
   - Target XLM
   - Virtual Reserves
   ↓
5. Pay Creation Fee
   ↓
6. Approve Token Transfer
   ↓
7. Create Launch
   ↓
8. Launch Active ✅
```

### Buy Flow

```
1. Connect Wallet
   ↓
2. Browse Launches
   ↓
3. Select Launch
   ↓
4. Enter XLM Amount
   ↓
5. Preview Quote
   - Tokens to receive
   - Price impact
   - Slippage tolerance
   ↓
6. Approve Transaction
   ↓
7. Execute Buy
   ↓
8. Tokens Received ✅
```

### Sell Flow

```
1. Connect Wallet
   ↓
2. View Holdings
   ↓
3. Select Token to Sell
   ↓
4. Enter Token Amount
   ↓
5. Preview Quote
   - XLM to receive
   - Price impact
   - Slippage tolerance
   ↓
6. Approve Transaction
   ↓
7. Execute Sell
   ↓
8. XLM Received ✅
```

### Migration Flow

```
1. Target Reached
   ↓
2. Anyone Can Trigger
   ↓
3. Contract Executes:
   - Calculate fee
   - Burn unsold tokens
   - Transfer fee to admin
   - Mark as migrated
   ↓
4. Liquidity Ready for DEX ✅
```

---

## Feature Comparison

### vs. Traditional IDO

| Feature | Lumiswap | Traditional IDO |
|---------|----------|-----------------|
| Permissionless | ✅ | ❌ (requires approval) |
| Instant liquidity | ✅ | ❌ (manual seeding) |
| Fair price | ✅ | ❌ (arbitrary price) |
| No rug pull | ✅ | ⚠️ (trust required) |
| Automated | ✅ | ❌ (manual process) |

### vs. Pump.fun (Solana)

| Feature | Lumiswap | Pump.fun |
|---------|----------|----------|
| Bonding curve | ✅ | ✅ |
| DEX migration | ✅ | ✅ |
| Token burn | ✅ | ✅ |
| Blockchain | Stellar | Solana |
| Transaction cost | ~$0.00001 | ~$0.001-0.01 |
| Speed | ~5s | ~0.4s |

---

## Roadmap

### Phase 1: MVP ✅ (Current)
- ✅ Core contract
- ✅ Basic trading
- ✅ Migration logic
- ✅ Testing
- ✅ Documentation

### Phase 2: Beta 🚧 (Next 2 weeks)
- 🚧 Frontend UI
- 🚧 Wallet integration
- 🚧 Price charts
- 🚧 Testnet deployment

### Phase 3: Launch (Month 1)
- Security audit
- Gas optimization
- Stellar DEX integration
- Mainnet deployment

### Phase 4: Growth (Month 2-3)
- Advanced curves
- Fee distribution
- Analytics dashboard
- Mobile app

### Phase 5: Ecosystem (Month 4+)
- Governance token
- Protocol revenue sharing
- Cross-chain bridges
- Liquidity mining

---

**Feature Status Legend:**
- ✅ Implemented
- 🚧 In Progress
- 📋 Planned
- ⏳ Future Consideration

**Last Updated:** 2024
