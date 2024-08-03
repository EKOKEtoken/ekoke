#!/bin/bash

cd "$(dirname "$0")" || exit 1

source ./deploy_functions.sh
source ./did.sh

ADMIN_PRINCIPAL="$(dfx identity get-principal)"
ERC20_BRIDGE_ADDRESS="0xc08e14F47382BCc1dA6c3Ff366018cAb1c77091F"
ERC20_SWAP_FEE="231634000000000"
ERC20_NETWORK="Ethereum"
FALLBACK_CANISTER="$ADMIN_PRINCIPAL"

CANISTER="$1"

if [ -z "$CANISTER" ]; then
  echo "Please provide the canister name as an argument"
  echo "Available canisters:"
  echo "- deferred"
  echo "- ekoke-erc20-swap"
  echo "- ekoke-liquidity-pool"
  echo "- ekoke-reward-pool"
  echo "- marketplace"
  exit 1
fi

set -e
dfx identity use ekoketoken

cd ../

case "$CANISTER" in

  "deferred")
    EKOKE_REWARD_POOL_PRINCIPAL=$(get_arg "ekoke-reward-pool" "$FALLBACK_CANISTER")
    MARKETPLACE_PRINCIPAL=$(get_arg "marketplace" "$FALLBACK_CANISTER")
    EKOKE_LIQUIDITY_POOL_PRINCIPAL=$(get_arg "ekoke-liquidity-pool" "$FALLBACK_CANISTER")
    deploy_deferred "reinstall" "ic" "$DEFERRED_PRINCIPAL" "$EKOKE_REWARD_POOL_PRINCIPAL" "$MARKETPLACE_PRINCIPAL" "$EKOKE_LIQUIDITY_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL"
    ;;
  
  "ekoke-erc20-swap")
    EKOKE_LEDGER_PRINCIPAL=$(get_arg "ekoke-ledger" "$FALLBACK_CANISTER")
    deploy_ekoke_erc20_swap "reinstall" "ic" "$EKOKE_ERC20_SWAP_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LEDGER_PRINCIPAL" "$ERC20_BRIDGE_ADDRESS" "$ERC20_SWAP_FEE" "$ERC20_NETWORK"
    ;;
  
  "ekoke-erc20-swap-frontend")
    deploy_ekoke_erc20_swap_frontend "reinstall" "ic"
    ;;
  
  "ekoke-liquidity-pool")
    DEFERRED_PRINCIPAL=$(get_arg "deferred" "$FALLBACK_CANISTER")
    deploy_ekoke_liquidity_pool "reinstall" "ic" "$EKOKE_LIQUIDITY_POOL_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$ADMIN_PRINCIPAL"
    ;;

  "ekoke-reward-pool")
    EKOKE_LEDGER_PRINCIPAL=$(get_arg "ekoke-ledger" "$FALLBACK_CANISTER")
    DEFERRED_PRINCIPAL=$(get_arg "deferred" "$FALLBACK_CANISTER")
    MARKETPLACE_PRINCIPAL=$(get_arg "marketplace" "$FALLBACK_CANISTER")
    deploy_ekoke_reward_pool "reinstall" "ic" "$EKOKE_REWARD_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LEDGER_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$MARKETPLACE_PRINCIPAL"
    ;;

  "marketplace")
    DEFERRED_PRINCIPAL=$(get_arg "deferred" "$FALLBACK_CANISTER")
    EKOKE_REWARD_POOL_PRINCIPAL=$(get_arg "ekoke-reward-pool" "$FALLBACK_CANISTER")
    EKOKE_LIQUIDITY_POOL_PRINCIPAL=$(get_arg "ekoke-liquidity-pool" "$FALLBACK_CANISTER")
    deploy_marketplace "reinstall" "ic" "$MARKETPLACE_PRINCIPAL" "$DEFERRED_PRINCIPAL" "$EKOKE_REWARD_POOL_PRINCIPAL" "$ADMIN_PRINCIPAL" "$EKOKE_LIQUIDITY_POOL_PRINCIPAL"
    ;;

  *)
    echo "Invalid canister name"
    echo "Available canisters:"
    echo "- deferred"
    echo "- ekoke-erc20-swap"
    echo "- ekoke-liquidity-pool"
    echo "- ekoke-reward-pool"
    echo "- marketplace"
    exit 1
    ;;

esac

set +e

exit 0
