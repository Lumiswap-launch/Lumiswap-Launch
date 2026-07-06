#!/bin/bash

# Lumiswap Launch - Testnet Deployment Script
# This script builds and deploys the Lumiswap Launch contract to Stellar Testnet

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  Lumiswap Launch - Deployment Script${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Configuration
NETWORK="${NETWORK:-testnet}"
DEPLOYER_KEY="${DEPLOYER_KEY:-deployer}"
ADMIN_ADDRESS="${ADMIN_ADDRESS:-}"
CREATION_FEE="${CREATION_FEE:-100000000}" # 10 XLM default
MIGRATION_FEE_BPS="${MIGRATION_FEE_BPS:-100}" # 1% default

# Check if stellar CLI is installed
if ! command -v stellar &> /dev/null; then
    echo -e "${RED}Error: stellar CLI not found${NC}"
    echo "Install from: https://developers.stellar.org/docs/tools/stellar-cli"
    exit 1
fi

# Check if we're in the contract directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${YELLOW}Changing to contract directory...${NC}"
    cd contract
fi

echo -e "${YELLOW}[1/6] Building contract...${NC}"
stellar contract build

if [ ! -f "target/wasm32-unknown-unknown/release/lumiswap_launch.wasm" ]; then
    echo -e "${RED}Error: Build failed - WASM file not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Contract built successfully${NC}"
echo ""

echo -e "${YELLOW}[2/6] Optimizing WASM...${NC}"
if command -v wasm-opt &> /dev/null; then
    wasm-opt -Oz target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
        -o target/wasm32-unknown-unknown/release/lumiswap_launch_optimized.wasm
    WASM_FILE="target/wasm32-unknown-unknown/release/lumiswap_launch_optimized.wasm"
    echo -e "${GREEN}✓ WASM optimized${NC}"
else
    echo -e "${YELLOW}⚠ wasm-opt not found, using unoptimized WASM${NC}"
    echo "  Install wasm-opt from: https://github.com/WebAssembly/binaryen"
    WASM_FILE="target/wasm32-unknown-unknown/release/lumiswap_launch.wasm"
fi

WASM_SIZE=$(wc -c < "$WASM_FILE" | tr -d ' ')
echo "  WASM size: ${WASM_SIZE} bytes"
echo ""

echo -e "${YELLOW}[3/6] Setting up deployer identity...${NC}"
if ! stellar keys ls | grep -q "$DEPLOYER_KEY"; then
    echo "  Generating new identity: $DEPLOYER_KEY"
    stellar keys generate "$DEPLOYER_KEY" --network "$NETWORK"
else
    echo "  Using existing identity: $DEPLOYER_KEY"
fi

DEPLOYER_ADDRESS=$(stellar keys address "$DEPLOYER_KEY")
echo "  Deployer address: $DEPLOYER_ADDRESS"
echo ""

echo -e "${YELLOW}[4/6] Funding deployer account...${NC}"
if [ "$NETWORK" = "testnet" ]; then
    echo "  Requesting testnet XLM from Friendbot..."
    stellar keys fund "$DEPLOYER_KEY" --network "$NETWORK"
    echo -e "${GREEN}✓ Account funded${NC}"
else
    echo -e "${YELLOW}⚠ Mainnet detected - ensure account has sufficient XLM${NC}"
fi
echo ""

echo -e "${YELLOW}[5/6] Deploying contract...${NC}"
CONTRACT_ID=$(stellar contract deploy \
    --wasm "$WASM_FILE" \
    --source "$DEPLOYER_KEY" \
    --network "$NETWORK" \
    2>&1 | tail -n 1)

if [ -z "$CONTRACT_ID" ]; then
    echo -e "${RED}Error: Deployment failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Contract deployed${NC}"
echo "  Contract ID: $CONTRACT_ID"
echo ""

# Get native XLM token address for the network
if [ "$NETWORK" = "testnet" ]; then
    NATIVE_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
else
    NATIVE_TOKEN="CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA"
fi

echo -e "${YELLOW}[6/6] Initializing contract...${NC}"

# Use deployer as admin if not specified
if [ -z "$ADMIN_ADDRESS" ]; then
    ADMIN_ADDRESS="$DEPLOYER_ADDRESS"
    echo "  Using deployer as admin: $ADMIN_ADDRESS"
fi

stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$DEPLOYER_KEY" \
    --network "$NETWORK" \
    -- initialize \
    --admin "$ADMIN_ADDRESS" \
    --native_token "$NATIVE_TOKEN" \
    --creation_fee "$CREATION_FEE" \
    --migration_fee_bps "$MIGRATION_FEE_BPS"

echo -e "${GREEN}✓ Contract initialized${NC}"
echo ""

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  Deployment Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Contract ID: $CONTRACT_ID"
echo "Network: $NETWORK"
echo "Admin: $ADMIN_ADDRESS"
echo "Creation Fee: $CREATION_FEE stroops"
echo "Migration Fee: $MIGRATION_FEE_BPS bps"
echo ""
echo "Save these details to your .env file:"
echo ""
echo "NEXT_PUBLIC_CONTRACT_ID=$CONTRACT_ID"
echo "NEXT_PUBLIC_NETWORK=$NETWORK"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Update frontend/.env.local with the contract ID"
echo "2. Run 'cd ../frontend && npm install && npm run dev'"
echo "3. Connect your wallet and create a test launch"
echo ""
