# Deployment Guide

Complete guide for deploying Lumiswap Launch contract to Stellar Testnet and Mainnet.

## 📋 Prerequisites

### Required Tools

```bash
# 1. Rust toolchain with wasm target
rustup target add wasm32-unknown-unknown

# 2. Stellar CLI (latest version)
cargo install --locked stellar-cli

# 3. wasm-opt (optional but recommended)
# Ubuntu/Debian:
sudo apt-get install binaryen

# macOS:
brew install binaryen
```

### Verify Installation

```bash
stellar --version  # Should be 21.0.0 or higher
rustc --version    # Should be 1.74.0 or higher
wasm-opt --version # Optional
```

---

## 🧪 Testnet Deployment

### Step 1: Build Contract

```bash
cd contract

# Build optimized WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize with wasm-opt (optional)
wasm-opt -Oz \
    target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
    -o target/wasm32-unknown-unknown/release/lumiswap_launch_optimized.wasm
```

**Expected output:**
- Unoptimized: ~150-200 KB
- Optimized: ~80-120 KB

### Step 2: Create Deployer Identity

```bash
# Generate new keypair
stellar keys generate deployer --network testnet

# Get address
stellar keys address deployer

# Fund account from Friendbot
stellar keys fund deployer --network testnet
```

**Save your deployer address** - you'll need it for initialization.

### Step 3: Deploy Contract

```bash
CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
    --source deployer \
    --network testnet \
    2>&1 | tail -n 1)

echo "Contract deployed: $CONTRACT_ID"
```

**Save the CONTRACT_ID** - this is your contract address.

### Step 4: Initialize Contract

```bash
# Testnet native XLM token address
NATIVE_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

# Get deployer address for admin
ADMIN=$(stellar keys address deployer)

# Initialize
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

**Parameters explained:**
- `creation_fee`: 100000000 stroops = 10 XLM
- `migration_fee_bps`: 100 = 1% (basis points)

### Step 5: Verify Deployment

```bash
# Get contract configuration
stellar contract invoke \
    --id $CONTRACT_ID \
    --source deployer \
    --network testnet \
    -- get_config

# Get launch count (should be 0)
stellar contract invoke \
    --id $CONTRACT_ID \
    --source deployer \
    --network testnet \
    -- get_launch_count
```

### Step 6: Configure Frontend

```bash
cd ../frontend

# Create environment file
cat > .env.local <<EOF
NEXT_PUBLIC_NETWORK=testnet
NEXT_PUBLIC_HORIZON_URL=https://horizon-testnet.stellar.org
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_CONTRACT_ID=$CONTRACT_ID
NEXT_PUBLIC_NATIVE_TOKEN=$NATIVE_TOKEN
NEXT_PUBLIC_NETWORK_PASSPHRASE=Test SDF Network ; September 2015
EOF

# Install and run
npm install
npm run dev
```

---

## 🚀 Mainnet Deployment

### Step 1: Prepare Mainnet Account

```bash
# Generate mainnet identity
stellar keys generate mainnet-deployer --network mainnet

# Get address
MAINNET_ADDRESS=$(stellar keys address mainnet-deployer)
echo "Fund this address with XLM: $MAINNET_ADDRESS"
```

**⚠️ Important:**
- Fund account with at least 100 XLM
- Keep private key secure (backup seed phrase)
- Consider using a hardware wallet for production

### Step 2: Security Checklist

Before mainnet deployment, verify:

- [ ] Contract audited by reputable security firm
- [ ] All tests passing: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Deployed and tested on testnet
- [ ] Admin key secured (hardware wallet or multi-sig)
- [ ] Emergency response plan in place
- [ ] Documentation complete
- [ ] Frontend tested with mainnet contract
- [ ] Team reviewed and approved

### Step 3: Build for Mainnet

```bash
cd contract

# Clean build
cargo clean

# Build with optimizations
cargo build --target wasm32-unknown-unknown --release

# Optimize
wasm-opt -Oz \
    target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
    -o target/wasm32-unknown-unknown/release/lumiswap_launch_mainnet.wasm

# Verify size
ls -lh target/wasm32-unknown-unknown/release/lumiswap_launch_mainnet.wasm
```

### Step 4: Deploy to Mainnet

```bash
# Deploy
MAINNET_CONTRACT=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/lumiswap_launch_mainnet.wasm \
    --source mainnet-deployer \
    --network mainnet \
    2>&1 | tail -n 1)

echo "⚠️  SAVE THIS CONTRACT ID: $MAINNET_CONTRACT"
```

**⚠️ Critical:** Save the contract ID immediately. This cannot be recovered if lost.

### Step 5: Initialize Mainnet Contract

```bash
# Mainnet native XLM token address
MAINNET_NATIVE="CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA"

# Set admin (use multi-sig or hardware wallet address)
MAINNET_ADMIN="YOUR_SECURE_ADMIN_ADDRESS"

# Initialize
stellar contract invoke \
    --id $MAINNET_CONTRACT \
    --source mainnet-deployer \
    --network mainnet \
    -- initialize \
    --admin $MAINNET_ADMIN \
    --native_token $MAINNET_NATIVE \
    --creation_fee "100000000" \
    --migration_fee_bps "100"
```

### Step 6: Verify Mainnet Deployment

```bash
# Verify configuration
stellar contract invoke \
    --id $MAINNET_CONTRACT \
    --source mainnet-deployer \
    --network mainnet \
    -- get_config

# Test with small launch (recommended)
# Create a test token and launch with minimal values
```

### Step 7: Production Frontend Configuration

```bash
cd ../frontend

# Create production environment
cat > .env.production <<EOF
NEXT_PUBLIC_NETWORK=mainnet
NEXT_PUBLIC_HORIZON_URL=https://horizon.stellar.org
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-mainnet.stellar.org
NEXT_PUBLIC_CONTRACT_ID=$MAINNET_CONTRACT
NEXT_PUBLIC_NATIVE_TOKEN=$MAINNET_NATIVE
NEXT_PUBLIC_NETWORK_PASSPHRASE=Public Global Stellar Network ; September 2015
EOF

# Build for production
npm run build

# Deploy to hosting (Vercel, Netlify, etc.)
```

---

## 🔧 Advanced Deployment Options

### Using Deployment Script

```bash
cd scripts

# Testnet deployment
./deploy.sh

# Mainnet deployment (requires confirmation)
NETWORK=mainnet DEPLOYER_KEY=mainnet-deployer ./deploy.sh
```

### Custom Parameters

```bash
# Higher creation fee (100 XLM)
CREATION_FEE="1000000000" ./deploy.sh

# Higher migration fee (2.5%)
MIGRATION_FEE_BPS="250" ./deploy.sh

# Different admin
ADMIN_ADDRESS="GA..." ./deploy.sh
```

### Multi-Signature Admin

For production, use multi-sig admin:

```bash
# Create multi-sig account (example: 2-of-3)
# 1. Create account
# 2. Add signers
# 3. Set thresholds
# 4. Use multi-sig address as admin

# See: https://developers.stellar.org/docs/encyclopedia/signatures-multisig
```

---

## 📊 Post-Deployment

### Monitoring

```bash
# Watch contract events
stellar contract events \
    --id $CONTRACT_ID \
    --network testnet \
    --start-ledger recent

# Monitor Horizon
curl "https://horizon-testnet.stellar.org/contracts/$CONTRACT_ID"
```

### Testing First Launch

```bash
# 1. Deploy test token
TOKEN_ID=$(stellar contract asset deploy \
    --asset TEST:$CREATOR_ADDRESS \
    --source creator \
    --network testnet)

# 2. Mint tokens to creator
stellar contract invoke \
    --id $TOKEN_ID \
    --source creator \
    --network testnet \
    -- mint \
    --to $CREATOR_ADDRESS \
    --amount "10000000000000"

# 3. Create launch
stellar contract invoke \
    --id $CONTRACT_ID \
    --source creator \
    --network testnet \
    -- create_launch \
    --creator $CREATOR_ADDRESS \
    --config '{
        "token": "'$TOKEN_ID'",
        "name": "Test Token",
        "symbol": "TEST",
        "total_supply": "10000000000000",
        "target_xlm": "500000000000",
        "virtual_xlm": "300000000000"
    }'
```

### Health Checks

1. **Contract State**
   ```bash
   stellar contract invoke --id $CONTRACT_ID --network testnet -- get_launch_count
   ```

2. **Admin Access**
   ```bash
   stellar contract invoke --id $CONTRACT_ID --network testnet -- get_config
   ```

3. **Price Queries**
   ```bash
   stellar contract invoke --id $CONTRACT_ID --network testnet -- get_current_price --launch_id 0
   ```

---

## 🔐 Security Best Practices

### Key Management

1. **Never commit private keys**
   ```bash
   # Add to .gitignore
   echo "*.key" >> .gitignore
   echo ".stellar-keys" >> .gitignore
   ```

2. **Use hardware wallets for mainnet**
   - Ledger Nano S/X
   - Use with Freighter wallet

3. **Backup seed phrases securely**
   - Store offline in safe location
   - Never share or store digitally

### Admin Controls

1. **Use multi-signature for admin**
   - Require 2-of-3 or 3-of-5 signatures
   - Distribute keys to trusted team members

2. **Separate deployment and admin keys**
   - Deployer key only for deployment
   - Admin key for contract management

3. **Monitor admin operations**
   - Set up alerts for admin function calls
   - Regular security audits

---

## 🐛 Troubleshooting

### Build Errors

```bash
# Clear cache and rebuild
cargo clean
rm -rf target/
cargo build --target wasm32-unknown-unknown --release
```

### Deployment Fails

```bash
# Check account balance
stellar account --address $DEPLOYER_ADDRESS

# Increase fee if needed
stellar contract deploy --wasm file.wasm --source deployer --network testnet --fee 10000
```

### Simulation Errors

```bash
# Get detailed error
stellar contract invoke --id $CONTRACT_ID --source test --network testnet -- function_name --arg value 2>&1 | grep -A 10 "Error"
```

### Contract Not Found

```bash
# Verify contract exists
curl "https://horizon-testnet.stellar.org/contracts/$CONTRACT_ID"

# Re-deploy if needed
```

---

## 📝 Deployment Checklist

### Pre-Deployment
- [ ] All tests pass
- [ ] Contract optimized
- [ ] Security review complete
- [ ] Documentation updated
- [ ] Backup keys secured

### Deployment
- [ ] Contract built and optimized
- [ ] Deployer account funded
- [ ] Contract deployed
- [ ] Contract initialized
- [ ] Configuration verified

### Post-Deployment
- [ ] Contract ID saved
- [ ] Frontend configured
- [ ] Test launch successful
- [ ] Monitoring set up
- [ ] Team notified

---

## 📞 Support

- **GitHub Issues**: Bug reports and feature requests
- **Discord**: Live help from community
- **Email**: deploy@lumiswap.io

---

**Next Steps:** [Integration Guide](INTEGRATION.md) | [API Reference](API.md)
