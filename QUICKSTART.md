# Lumiswap Launch - Quick Start Guide

Get up and running with Lumiswap Launch in under 10 minutes.

---

## ⚡ Prerequisites (2 minutes)

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# 2. Install Stellar CLI
cargo install --locked stellar-cli

# 3. Install Node.js (if not installed)
# Visit: https://nodejs.org/
```

---

## 🚀 Option 1: One-Command Deploy (5 minutes)

```bash
# Clone repository
git clone https://github.com/yourusername/lumiswap-launch
cd lumiswap-launch

# Deploy to testnet (builds, deploys, initializes)
cd scripts
./deploy.sh

# Output will show:
# ✓ Contract deployed
# Contract ID: CA...
# 
# Save this to your .env.local file:
# NEXT_PUBLIC_CONTRACT_ID=CA...
```

**Done!** Contract is live on testnet.

---

## 🎨 Option 2: Run Locally with Frontend (10 minutes)

### Step 1: Deploy Contract (3 min)

```bash
cd contract

# Test
cargo test

# Build
cargo build --target wasm32-unknown-unknown --release

# Deploy
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet

CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
    --source deployer \
    --network testnet)

echo "Contract: $CONTRACT_ID"
```

### Step 2: Initialize Contract (1 min)

```bash
NATIVE_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
ADMIN=$(stellar keys address deployer)

stellar contract invoke \
    --id $CONTRACT_ID \
    --source deployer \
    --network testnet \
    -- initialize \
    --admin $ADMIN \
    --native_token $NATIVE_TOKEN \
    --creation_fee "100000000" \
    --migration_fee_bps "100"
```

### Step 3: Start Frontend (2 min)

```bash
cd ../frontend

# Install dependencies
npm install

# Configure
cat > .env.local <<EOF
NEXT_PUBLIC_CONTRACT_ID=$CONTRACT_ID
NEXT_PUBLIC_NETWORK=testnet
NEXT_PUBLIC_HORIZON_URL=https://horizon-testnet.stellar.org
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_NATIVE_TOKEN=$NATIVE_TOKEN
EOF

# Start dev server
npm run dev
```

**Open** `http://localhost:3000` 🎉

---

## 🧪 Option 3: Just Run Tests (1 minute)

```bash
cd contract
cargo test

# Output:
# running 33 tests
# test amm::tests::test_buy_tokens ... ok
# test test_initialize ... ok
# test test_create_launch ... ok
# ...
# test result: ok. 33 passed; 0 failed
```

---

## 📖 Quick Examples

### Create a Launch (CLI)

```bash
# 1. Deploy a test token
TOKEN_ID=$(stellar contract asset deploy \
    --asset TEST:$(stellar keys address creator) \
    --source creator \
    --network testnet)

# 2. Mint tokens to creator
stellar contract invoke \
    --id $TOKEN_ID \
    --source creator \
    --network testnet \
    -- mint \
    --to $(stellar keys address creator) \
    --amount "10000000000000"

# 3. Create launch
stellar contract invoke \
    --id $CONTRACT_ID \
    --source creator \
    --network testnet \
    -- create_launch \
    --creator $(stellar keys address creator) \
    --config '{
        "token": "'$TOKEN_ID'",
        "name": "My Token",
        "symbol": "MTK",
        "total_supply": "10000000000000",
        "target_xlm": "500000000000",
        "virtual_xlm": "300000000000"
    }'

# Output: 0 (launch ID)
```

### Buy Tokens (CLI)

```bash
stellar contract invoke \
    --id $CONTRACT_ID \
    --source buyer \
    --network testnet \
    -- buy \
    --buyer $(stellar keys address buyer) \
    --launch_id 0 \
    --xlm_amount "10000000000" \
    --min_tokens "0"

# Output: 123456789 (tokens received)
```

### Get Current Price (CLI)

```bash
stellar contract invoke \
    --id $CONTRACT_ID \
    --source anyone \
    --network testnet \
    -- get_current_price \
    --launch_id 0

# Output: 300000 (0.03 XLM per token in stroops)
```

---

## 🔧 Troubleshooting

### "Command not found: stellar"

```bash
# Reinstall Stellar CLI
cargo install --locked stellar-cli --force
```

### "Build failed"

```bash
# Ensure wasm target installed
rustup target add wasm32-unknown-unknown

# Clean and rebuild
cargo clean
cargo build --target wasm32-unknown-unknown --release
```

### "Account not found"

```bash
# Fund your account
stellar keys fund YOUR_KEY_NAME --network testnet
```

### "Contract not found"

```bash
# Verify contract ID
echo $CONTRACT_ID

# Re-deploy if needed
stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
    --source deployer \
    --network testnet
```

---

## 📚 Next Steps

- **Read the Docs**: [README.md](README.md)
- **Understand Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
- **Deploy to Mainnet**: [DEPLOYMENT.md](DEPLOYMENT.md)
- **Contribute**: [CONTRIBUTING.md](CONTRIBUTING.md)

---

## 🆘 Get Help

- **GitHub Issues**: Report bugs
- **Discord**: [Join community](https://discord.gg/lumiswap)
- **Docs**: [docs.lumiswap.io](https://docs.lumiswap.io)
- **Email**: help@lumiswap.io

---

**That's it!** You're ready to build on Lumiswap Launch. 🚀
