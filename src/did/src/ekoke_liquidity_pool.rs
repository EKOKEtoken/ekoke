//! Types associated to the "Ekoke" canister

mod liquidity_pool;

use candid::{CandidType, Deserialize, Principal};

pub use self::liquidity_pool::{LiquidityPoolAccounts, LiquidityPoolBalance};

/// These are the arguments which are taken by the ekoke liquidity pool canister on init
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct EkokeLiquidityPoolInitData {
    pub admins: Vec<Principal>,
    /// ICP ledger canister id
    pub icp_ledger_canister: Principal,
    /// Deferred canister id
    pub deferred_canister_id: Principal,
}
