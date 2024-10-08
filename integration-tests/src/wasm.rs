use std::path::Path;

use candid::{CandidType, Nat};
use icrc::icrc1::account::Account;
use serde::Deserialize;

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct Icrc2InitArgs {
    pub accounts: Vec<(Account, Nat)>,
    pub decimals: u8,
    pub fee: u64,
    pub logo: String,
    pub minting_account: Account,
    pub name: String,
    pub symbol: String,
    pub total_supply: Nat,
}

pub enum Canister {
    Deferred,
    EkokeErc20Swap,
    EkokeLiquidityPool,
    EkokeRewardPool,
    Icrc2,
    Marketplace,
    Xrc,
}

impl Canister {
    pub fn as_path(&self) -> &'static Path {
        match self {
            Canister::Deferred => Path::new("../.dfx/local/canisters/deferred/deferred.wasm"),
            Canister::EkokeErc20Swap => {
                Path::new("../.dfx/local/canisters/ekoke-erc20-swap/ekoke-erc20-swap.wasm")
            }
            Canister::EkokeLiquidityPool => {
                Path::new("../.dfx/local/canisters/ekoke-liquidity-pool/ekoke-liquidity-pool.wasm")
            }
            Canister::EkokeRewardPool => {
                Path::new("../.dfx/local/canisters/ekoke-reward-pool/ekoke-reward-pool.wasm")
            }
            Canister::Marketplace => {
                Path::new("../.dfx/local/canisters/marketplace/marketplace.wasm")
            }
            Canister::Xrc => Path::new("../assets/wasm/xrc-dummy-canister.wasm"),
            Canister::Icrc2 => Path::new("../assets/wasm/icrc2-template-canister.wasm"),
        }
    }
}
