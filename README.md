# Lumiswap-Launch

A permissionless token launchpad on Stellar with bonding curve price discovery that automatically migrates liquidity to the Stellar native DEX when a market cap target is reached.

---

## Table of Contents

1. [Problem It Solves](#problem-it-solves)
2. [Architecture](#architecture)
3. [Project Structure](#project-structure)
4. [Getting Started](#getting-started)
5. [Contributing](#contributing)

---
## Screenshot
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/a9525b03-01da-4f8a-9a85-97c784357e47" />
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/e9e56e56-c18c-4964-881d-0fce62d92738" />
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/6687817d-823c-464e-be5e-159dd7a5d9f5" />
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/f339120e-da59-4a13-bb07-fd41f65acb74" />
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/5ba265b4-1539-4e5b-83b8-c52161220a53" />


https://lumiswap-launch.lovable.app/

## Problem It Solves

| Gap in the Ecosystem | How Lumiswap-Launch Solves It |
|---|---|
| Token launches on Stellar require manual liquidity provisioning and trusted intermediaries | Bonding curve contract holds all liquidity in escrow; no admin can rug — migration is permissionless and automatic |
| No price discovery mechanism for new tokens before DEX listing | Constant-product AMM (`price = virtual_xlm / virtual_tokens`) sets fair prices from the first buy, with on-chain price history |
| Liquidity fragmentation — teams manually create DEX offers at arbitrary prices | When `xlm_raised ≥ target_xlm`, anyone calls `migrate`; the contract burns unsold tokens and seeds the Stellar DEX at the exact market-clearing price |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser / CLI                        │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Next.js 14 Frontend                      │  │
│  │  TokenGrid  TokenCard  BondingCurveChart  LaunchWizard│  │
│  └────────────────────┬─────────────────────────────────┘  │
│                        │ @stellar/stellar-sdk RPC calls     │
└────────────────────────┼────────────────────────────────────┘
                         │
          ┌──────────────▼──────────────────┐
          │     Soroban Contract            │
          │  (lumenswap-launch)             │
          │                                 │
          │  create_launch()                │
          │  buy()  ──── constant-product   │
          │  sell() ──── AMM math           │
          │  migrate() ─ burns + DEX offer  │
          │  current_price()                │
          └────────┬─────────────┬──────────┘
                   │             │
        ┌──────────▼──┐   ┌──────▼───────────────┐
        │  SAC Token  │   │   Stellar Native DEX  │
        │  (per-launch│   │  (post-migration LP)  │
        │   escrow)   │   └──────────────────────-┘
        └─────────────┘
                   │  Horizon contract_events (polling)
          ┌────────▼────────┐
          │  Telegram Bot   │
          │  (aiogram 3.x)  │
          │  /launches      │
          │  /price <id>    │
          │  alert on new   │
          │  launch/migrate │
          └─────────────────┘
```

**Key design decisions**

- **Constant-product curve** (`k = virtual_xlm × virtual_tokens`) — same math as Uniswap v2 but seeded with virtual reserves so the price curve starts gradual and steepens as supply sells out.
- **Virtual reserves** — initial `virtual_xlm = 30 000 XLM` prevents zero-price exploits at launch without requiring a seed deposit.
- **Permissionless migration** — any wallet can call `migrate` once the target is reached; no admin key needed, eliminating rug vectors.
- **Burn on migrate** — unsold tokens are burned rather than returned to the creator, aligning incentives and preventing post-migration dilution.
- **Migration fee in bps** — a small protocol fee (default 1%) is deducted before LP seeding; configurable at `initialize` time.

---

## Project Structure

```
Lumiswap-Launch/
│
├── contract/                        # Soroban smart contract (Rust)
│   ├── Cargo.toml                   # soroban-sdk dependency, release profile
│   └── src/
│       ├── lib.rs                   # All contract logic: structs, storage, fn impls, events
│       └── test.rs                  # 9-test suite (testutils, migration scenario)
│
├── frontend/                        # Next.js 14 launchpad UI
│   ├── package.json                 # next 14, chart.js, react-chartjs-2, stellar-sdk
│   ├── tailwind.config.ts
│   ├── postcss.config.js
│   └── app/
│       ├── layout.tsx               # Root layout (dark background)
│       ├── globals.css
│       └── page.tsx                 # Hero + "Create Launch" toggle + TokenGrid
│   └── components/
│       ├── TokenGrid.tsx            # Responsive 2/3/4-col grid of launch cards
│       ├── TokenCard.tsx            # Card: progress bar, price, Buy/Sell, chart toggle
│       ├── BondingCurveChart.tsx    # Chart.js line chart: price vs % sold, current-sold marker
│       └── LaunchWizard.tsx         # 3-step wizard: token details → curve params → deploy
│
└── bot/                             # Telegram alert bot (Python)
    ├── bot.py                       # aiogram 3.x: /start /launches /price + Horizon polling
    └── requirements.txt             # aiogram==3.4.1, aiohttp==3.9.3, python-dotenv==1.0.1
```

---

## Getting Started

### Prerequisites

- [Rust + cargo](https://rustup.rs/) with `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (`cargo install stellar-cli`)
- Node.js 18+
- Python 3.11+

---

### 1 — Build & test the contract

```bash
cd contract

# Add wasm target once
rustup target add wasm32-unknown-unknown

# Run tests
cargo test

# Build optimised WASM
cargo build --target wasm32-unknown-unknown --release
```

The compiled artifact is at `target/wasm32-unknown-unknown/release/lumenswap_launch.wasm`.

---

### 2 — Deploy to Testnet

```bash
# Configure testnet identity (one-time)
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet

# Deploy contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/lumenswap_launch.wasm \
  --source deployer \
  --network testnet

# Initialise (replace CONTRACT_ID, ADMIN_ADDRESS)
stellar contract invoke \
  --id $CONTRACT_ID --source deployer --network testnet \
  -- initialize \
  --admin $ADMIN_ADDRESS \
  --creation_fee 10000000 \
  --migration_fee_bps 100
```

---

### 3 — Frontend

```bash
cd frontend
npm install
npm run dev          # http://localhost:3000
```

Set `NEXT_PUBLIC_CONTRACT_ID` and `NEXT_PUBLIC_HORIZON_URL` in `.env.local` to connect to your deployed contract.

---

### 4 — Telegram bot

```bash
cd bot
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt

# Create .env
cat > .env <<EOF
BOT_TOKEN=your_telegram_bot_token
LAUNCHPAD_CONTRACT_ID=your_contract_id
ALERT_CHAT_ID=your_chat_id
HORIZON_URL=https://horizon-testnet.stellar.org
EOF

python bot.py
```

---

## Contributing

### Workflow

1. Fork the repo and create a branch from `main`: `git checkout -b feat/your-feature`
2. Make your changes with tests where applicable
3. Open a PR against `main` — keep the title under 70 characters

### PR Guide

- **One concern per PR.** Bug fixes and features in separate PRs.
- Include a short description of what changed and how it was tested.
- For contract changes, include `cargo test` output in the PR description.
- PRs that break existing tests will not be merged.

### Areas to Contribute

| Area | Ideas |
|---|---|
| Contract | Stellar DEX offer creation in `migrate`, fee distribution, multi-token curve shapes |
| Frontend | Wallet integration (Freighter), live RPC price updates, mobile layout polish |
| Bot | `/portfolio` command, price alert subscriptions, mainnet Horizon support |
| Testing | Fuzz tests for AMM math, end-to-end testnet scripts |

### Code Standards

- **Rust** — `cargo fmt` + `cargo clippy --deny warnings` before committing
- **TypeScript** — ESLint + Prettier; no `any` types
- **Python** — `ruff` for linting; type hints on all functions

### Reporting Issues

Open a GitHub Issue with:
- Environment (OS, toolchain versions)
- Steps to reproduce
- Expected vs actual behaviour
- Relevant logs or error output
