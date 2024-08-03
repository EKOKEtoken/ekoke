#!/bin/bash

cd "$(dirname "$0")" || exit 1

CANISTER_IDS="../.dfx/local/canister_ids.json"
DEFERRED_PRINCIPAL="$(cat "$CANISTER_IDS" | jq -r '.deferred.local')"
EKOKE_ERC20_SWAP_PRINCIPAL=$(cat "$CANISTER_IDS" | jq -r '."ekoke-erc20-swap".local')
EKOKE_LEDGER_PRINCIPAL=$(cat "$CANISTER_IDS" | jq -r '."sns_ledger".local')
EKOKE_LIQUIDITY_POOL_PRINCIPAL=$(cat "$CANISTER_IDS" | jq -r '."ekoke-liquidity-pool".local')
EKOKE_REWARD_POOL_PRINCIPAL=$(cat "$CANISTER_IDS" | jq -r '."ekoke-reward-pool".local')
MARKETPLACE_PRINCIPAL="$(cat "$CANISTER_IDS" | jq -r '.marketplace.local')"

source ./deploy_functions.sh
source ./did.sh

ADMIN_PRINCIPAL="$(dfx identity get-principal)"
ERC20_BRIDGE_ADDRESS="0xc08e14F47382BCc1dA6c3Ff366018cAb1c77091F"
ERC20_SWAP_FEE="231634000000000"
ERC20_NETWORK="Sepolia"

dfx stop
dfx start --background

cd ../

set -e

deploy_deferred "reinstall" "local" "$DEFERRED_PRINCIPAL" "$EKOKE_REWARD_POOL_PRINCIPAL" "$MARKETPLACE_PRINCIPAL" "$EKOKE_LIQUIDITY_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL"
deploy_ekoke_erc20_swap "reinstall" "local" "$EKOKE_ERC20_SWAP_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LEDGER_PRINCIPAL" "$ERC20_BRIDGE_ADDRESS" "$ERC20_SWAP_FEE" "$ERC20_NETWORK"
deploy_ekoke_liquidity_pool "reinstall" "local" "$EKOKE_LIQUIDITY_POOL_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$ADMIN_PRINCIPAL"
deploy_ekoke_reward_pool "reinstall" "local" "$EKOKE_REWARD_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LEDGER_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$MARKETPLACE_PRINCIPAL"
deploy_marketplace "reinstall" "local" "$MARKETPLACE_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$EKOKE_REWARD_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LIQUIDITY_POOL_PRINCIPAL"

set +e

dfx stop

exit 0
