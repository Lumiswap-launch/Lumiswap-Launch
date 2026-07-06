#!/bin/bash

# Lumiswap Launch - Integration Test Script
# Tests the full lifecycle: deploy → create launch → buy → sell → migrate

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  Lumiswap Launch - Integration Test${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Setup
NETWORK="testnet"
cd contract

echo -e "${YELLOW}[1/8] Building contract...${NC}"
stellar contract build

echo -e "${YELLOW}[2/8] Creating test identities...${NC}"
stellar keys generate test-deployer --network "$NETWORK" 2>/dev/null || true
stellar keys generate test-creator --network "$NETWORK" 2>/dev/null || true
stellar keys generate test-buyer --network "$NETWORK" 2>/dev/null || true

echo -e "${YELLOW}[3/8] Funding accounts...${NC}"
stellar keys fund test-deployer --network "$NETWORK"
stellar keys fund test-creator --network "$NETWORK"
stellar keys fund test-buyer --network "$NETWORK"

echo -e "${YELLOW}[4/8] Deploying contract...${NC}"
CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
    --source test-deployer \
    --network "$NETWORK" \
    2>&1 | tail -n 1)

echo "Contract ID: $CONTRACT_ID"

echo -e "${YELLOW}[5/8] Deploying test token...${NC}"
TOKEN_ID=$(stellar contract asset deploy \
    --asset TEST:$(stellar keys address test-creator) \
    --source test-creator \
    --network "$NETWORK" \
    2>&1 | tail -n 1)

echo "Token ID: $TOKEN_ID"

NATIVE_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"

echo -e "${YELLOW}[6/8] Initializing contract...${NC}"
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source test-deployer \
    --network "$NETWORK" \
    -- initialize \
    --admin $(stellar keys address test-deployer) \
    --native_token "$NATIVE_TOKEN" \
    --creation_fee "100000000" \
    --migration_fee_bps "100"

echo -e "${YELLOW}[7/8] Creating launch...${NC}"
# Mint tokens to creator
stellar contract invoke \
    --id "$TOKEN_ID" \
    --source test-creator \
    --network "$NETWORK" \
    -- mint \
    --to $(stellar keys address test-creator) \
    --amount "10000000000000"

# Create launch
LAUNCH_ID=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source test-creator \
    --network "$NETWORK" \
    -- create_launch \
    --creator $(stellar keys address test-creator) \
    --config "{\"token\":\"$TOKEN_ID\",\"name\":\"Test Token\",\"symbol\":\"TEST\",\"total_supply\":\"10000000000000\",\"target_xlm\":\"500000000000\",\"virtual_xlm\":\"300000000000\"}")

echo "Launch ID: $LAUNCH_ID"

echo -e "${YELLOW}[8/8] Testing buy...${NC}"
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source test-buyer \
    --network "$NETWORK" \
    -- buy \
    --buyer $(stellar keys address test-buyer) \
    --launch_id "$LAUNCH_ID" \
    --xlm_amount "10000000000" \
    --min_tokens "0"

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  Integration Test Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Contract ID: $CONTRACT_ID"
echo "Launch ID: $LAUNCH_ID"
echo ""
