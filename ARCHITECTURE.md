# Lumiswap Launch - Architecture Documentation

## System Overview

Lumiswap Launch is a production-ready decentralized token launchpad built on Stellar's Soroban smart contract platform. It implements a fair launch mechanism using bonding curves for price discovery and automated liquidity migration to Stellar DEX.

---

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                   Presentation Layer                        │
│              (Next.js 15 + TypeScript)                      │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                  Integration Layer                          │
│         (Stellar SDK + Freighter Wallet)                    │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                   Protocol Layer                            │
│                  (Soroban RPC/Horizon)                      │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                  Smart Contract Layer                       │
│                (Lumiswap Launch Contract)                   │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                   Settlement Layer                          │
│            (Stellar Blockchain + SAC Tokens)                │
└─────────────────────────────────────────────────────────────┘
```

---

## Smart Contract Architecture

### Module Structure

```
contract/src/
├── lib.rs          # Main contract logic & public interface
├── types.rs        # Data structures and enums
├── storage.rs      # Storage keys and TTL management
├── amm.rs          # AMM math (bonding curve)
├── events.rs       # Event emissions
├── errors.rs       # Error definitions
└── test.rs         # Comprehensive test suite
```

### Core Components

#### 1. Storage Layer (`storage.rs`)

**Purpose:** Manage persistent and instance storage with TTL.

```rust
pub enum DataKey {
    Admin,              // Instance: Contract admin
    NativeToken,        // Instance: XLM token address
    CreationFee,        // Instance: Launch creation fee
    MigrationFeeBps,    // Instance: Migration fee in bps
    LaunchCounter,      // Instance: Monotonic launch ID
    Launch(u64),        // Persistent: Launch data
    Curve(u64),         // Persistent: Curve state
}
```

**TTL Strategy:**
- Instance storage: 30 days (config data)
- Persistent storage: 365 days (launch data)
- Auto-extend on access

#### 2. Data Types (`types.rs`)

**Launch State:**
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

**Bonding Curve:**
```rust
pub struct CurveState {
    virtual_xlm: i128,      // Current XLM reserve
    virtual_tokens: i128,   // Current token reserve
    k: i128,                // Constant product
}
```

**Launch Status:**
```rust
pub enum LaunchStatus {
    Active,          // Trading enabled
    TargetReached,   // Ready for migration
    Migrated,        // Completed
}
```

#### 3. AMM Logic (`amm.rs`)

**Constant Product Formula:**
```
k = x * y (constant)
where x = virtual_xlm, y = virtual_tokens
```

**Buy Calculation:**
```rust
fn calculate_tokens_out(xlm_in: i128, curve: &CurveState) -> Result<i128> {
    let new_xlm = curve.virtual_xlm + xlm_in;
    let new_tokens = curve.k / new_xlm;
    let tokens_out = curve.virtual_tokens - new_tokens;
    Ok(tokens_out)
}
```

**Sell Calculation:**
```rust
fn calculate_xlm_out(tokens_in: i128, curve: &CurveState) -> Result<i128> {
    let new_tokens = curve.virtual_tokens + tokens_in;
    let new_xlm = curve.k / new_tokens;
    let xlm_out = curve.virtual_xlm - new_xlm;
    Ok(xlm_out)
}
```

**Price Calculation:**
```rust
fn calculate_spot_price(curve: &CurveState) -> Result<i128> {
    let price = (curve.virtual_xlm * 10_000_000) / curve.virtual_tokens;
    Ok(price)
}
```

#### 4. Event System (`events.rs`)

**Event Types:**
- `init`: Contract initialized
- `created`: Launch created
- `buy`: Tokens purchased
- `sell`: Tokens sold
- `migrated`: Launch migrated to DEX

**Event Structure:**
```rust
env.events().publish(
    (topic, launch_id),
    (data1, data2, ...)
);
```

Events are indexed by Horizon and queryable via REST API.

#### 5. Error Handling (`errors.rs`)

**Error Categories:**
- Initialization (1-9)
- Validation (10-29)
- Launch (30-49)
- Trading (50-69)
- Math (70-79)
- Authorization (80-89)

All functions return `Result<T, Error>` for explicit error handling.

---

## Frontend Architecture

### Technology Stack

- **Framework:** Next.js 15 (App Router)
- **Language:** TypeScript (strict mode)
- **Styling:** Tailwind CSS
- **State:** Zustand
- **Blockchain:** Stellar SDK v13
- **Wallet:** Freighter API

### Directory Structure

```
frontend/
├── app/
│   ├── layout.tsx         # Root layout
│   ├── page.tsx           # Home page
│   └── globals.css        # Global styles
├── components/
│   ├── TokenGrid.tsx      # Launch grid
│   ├── TokenCard.tsx      # Launch card
│   ├── LaunchWizard.tsx   # Create launch flow
│   └── BondingCurveChart.tsx  # Price chart
├── lib/
│   ├── stellar.ts         # Stellar utilities
│   ├── contract-client.ts # Contract wrapper
│   └── hooks/             # React hooks
└── public/                # Static assets
```

### Contract Client Layer

**Type-Safe Wrapper:**
```typescript
class LumiswapContractClient {
    constructor(contractId: string)
    
    async createLaunch(creator: string, config: LaunchConfig): Promise<bigint>
    async buy(buyer: string, launchId: bigint, xlmAmount: bigint): Promise<bigint>
    async sell(seller: string, launchId: bigint, tokenAmount: bigint): Promise<bigint>
    async migrate(caller: string, launchId: bigint): Promise<void>
    
    async getLaunch(launchId: bigint): Promise<Launch>
    async getCurrentPrice(launchId: bigint): Promise<bigint>
    async getBuyQuote(launchId: bigint, xlmAmount: bigint): Promise<bigint>
}
```

### Wallet Integration

**Freighter Integration:**
```typescript
// Check installation
const installed = await isFreighterInstalled();

// Connect wallet
const publicKey = await connectFreighter();

// Sign and submit transaction
const result = await signAndSubmitWithFreighter(transaction);
```

---

## Data Flow

### Create Launch Flow

```
User Input
    │
    ├─> Validate Config
    │
    ├─> Build Transaction
    │   ├─> create_launch() call
    │   └─> Transfer tokens to contract
    │
    ├─> Sign with Freighter
    │
    ├─> Submit to Stellar
    │
    └─> Contract Execution
        ├─> Validate parameters
        ├─> Collect creation fee
        ├─> Escrow tokens
        ├─> Initialize curve
        ├─> Save state
        └─> Emit event
```

### Buy Flow

```
User Input (XLM amount)
    │
    ├─> Get Quote
    │   └─> calculate_tokens_out()
    │
    ├─> Set Slippage (min_tokens)
    │
    ├─> Build Transaction
    │   └─> buy() call
    │
    ├─> Sign with Freighter
    │
    ├─> Submit to Stellar
    │
    └─> Contract Execution
        ├─> Validate launch active
        ├─> Calculate tokens out
        ├─> Check slippage
        ├─> Transfer XLM to contract
        ├─> Transfer tokens to buyer
        ├─> Update curve state
        ├─> Update launch state
        └─> Emit buy event
```

### Migration Flow

```
Target Reached
    │
    ├─> Any User Calls migrate()
    │
    ├─> Contract Execution
    │   ├─> Verify target reached
    │   ├─> Calculate fee
    │   ├─> Burn unsold tokens
    │   ├─> Transfer fee to admin
    │   ├─> Mark as migrated
    │   └─> Emit migrated event
    │
    └─> Liquidity Ready for DEX
        └─> (xlm_raised - fee) + sold_tokens
```

---

## Security Architecture

### Threat Model

**Threats Addressed:**
1. ✅ Rug pulls (funds locked in contract)
2. ✅ Price manipulation (slippage protection)
3. ✅ Arithmetic attacks (checked math)
4. ✅ Reentrancy (Soroban architecture prevents)
5. ✅ Front-running (fair ordering via Stellar consensus)

**Threats Not Addressed:**
1. ⚠️ Malicious token contracts
2. ⚠️ MEV/sandwich attacks (partial)
3. ⚠️ Phishing attacks (client-side)

### Security Mechanisms

**1. Access Control:**
- No admin withdrawal functions
- Authorization checks on all state-changing functions
- Immutable launch parameters

**2. Input Validation:**
- Range checks on all amounts
- Name/symbol length validation
- Fee limits (max 100%)

**3. Arithmetic Safety:**
- All math uses `checked_*` operations
- Explicit error handling
- No unchecked casts

**4. Slippage Protection:**
- `min_tokens` on buy
- `min_xlm` on sell
- User-defined tolerance

**5. State Integrity:**
- Monotonic launch IDs
- Immutable curve constant (k)
- Status transitions enforced

---

## Performance Characteristics

### Gas Costs (Estimated)

| Operation | Gas Cost | Notes |
|-----------|----------|-------|
| initialize | ~50k | One-time |
| create_launch | ~500k | Includes token transfer |
| buy | ~150k | Per trade |
| sell | ~150k | Per trade |
| migrate | ~300k | Includes burn |
| get_* | <10k | View functions |

### Storage Costs

| Item | Size | Cost (XLM) |
|------|------|------------|
| Launch | 200 bytes | ~0.1 |
| Curve | 100 bytes | ~0.05 |
| Config | 150 bytes | ~0.075 |

### Scalability

**Throughput:**
- 1000+ launches supported
- Limited by Stellar ledger close time (~5s)
- Parallel launch operations possible

**Storage:**
- Linear growth with launches
- TTL management prevents unbounded growth
- Archival possible for migrated launches

---

## Deployment Architecture

### Testnet

```
Developer
    │
    ├─> Build WASM
    ├─> Deploy to Testnet
    ├─> Initialize Contract
    └─> Configure Frontend
        └─> Deploy to Vercel
```

### Mainnet

```
Security Audit
    │
    ├─> Build Optimized WASM
    ├─> Multi-sig Admin Setup
    ├─> Deploy to Mainnet
    ├─> Initialize Contract
    ├─> Verify Configuration
    └─> Production Frontend
        └─> Deploy with monitoring
```

---

## Future Enhancements

### Smart Contract

1. **Configurable Curves**
   - Linear bonding curves
   - Exponential curves
   - Custom formulas per launch

2. **Fee Distribution**
   - Protocol revenue sharing
   - Referral rewards
   - Staking rewards

3. **Emergency Controls**
   - Pausable trades
   - Emergency withdraw (time-locked)
   - Upgrade mechanism

### Frontend

1. **Analytics Dashboard**
   - Launch performance metrics
   - Price history charts
   - Volume tracking

2. **Advanced Features**
   - Limit orders
   - Portfolio tracking
   - Price alerts

3. **Mobile App**
   - Native iOS/Android
   - Push notifications
   - QR code payments

---

## Monitoring & Observability

### Contract Events

Monitor via Horizon:
```bash
curl "https://horizon-testnet.stellar.org/contracts/$CONTRACT_ID/events"
```

### Metrics to Track

- Launch creation rate
- Total value locked
- Trading volume
- Migration success rate
- Average hold time
- Price volatility

### Alerting

- Failed transactions
- Unusual trading patterns
- Low liquidity warnings
- Migration eligibility

---

## References

- [Stellar Documentation](https://developers.stellar.org/)
- [Soroban Docs](https://soroban.stellar.org/)
- [Uniswap V2 Whitepaper](https://uniswap.org/whitepaper.pdf)
- [Bonding Curves Explained](https://blog.relevant.community/bonding-curves-in-depth-intuition-parametrization-d3905a681e0a)

---

**Last Updated:** 2024
**Version:** 1.0.0
