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
INITIAL_SUPPLY="8880101010000000000"
SWAP_ACCOUNT="$(account "$ADMIN_PRINCIPAL")"
ERC20_BRIDGE_ADDRESS="0xc08e14F47382BCc1dA6c3Ff366018cAb1c77091F"
ERC20_SWAP_FEE="231634000000000"
EKOKE_MINTING_ACCOUNT="$(account "$ADMIN_PRINCIPAL" "{33;169;149;73;231;146;144;124;94;39;94;84;81;6;141;173;223;77;67;238;141;202;180;135;86;35;26;143;183;113;49;35}")"
ERC20_NETWORK="Sepolia"

dfx stop
dfx start --background

cd ../

deploy_deferred "reinstall" "local" "$DEFERRED_PRINCIPAL" "$EKOKE_REWARD_POOL_PRINCIPAL" "$MARKETPLACE_PRINCIPAL" "$ADMIN_PRINCIPAL"
deploy_ekoke_erc20_swap "reinstall" "local" "$EKOKE_ERC20_SWAP_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LEDGER_PRINCIPAL" "$ERC20_BRIDGE_ADDRESS" "$ERC20_SWAP_FEE" "$ERC20_NETWORK"
deploy_ekoke_liquidity_pool "reinstall" "local" "$EKOKE_LIQUIDITY_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL" "$SWAP_ACCOUNT"
deploy_ekoke_reward_pool "reinstall" "local" "$EKOKE_REWARD_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LEDGER_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$MARKETPLACE_PRINCIPAL"
deploy_marketplace "reinstall" "local" "$MARKETPLACE_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$EKOKE_REWARD_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL" ""$EKOKE_LIQUIDITY_POOL_PRINCIPAL"

dfx stop

exit $RES
