//! # Ekoke Ledger canister
//!
//! The ekoke canister serves a ICRC-2 token called $EKOKE, which is the reward token for Deferred transactions.
//! It is a deflationary token which ...

mod app;
mod http;
mod inspect;
mod utils;

use std::collections::HashMap;

use candid::{candid_method, Nat, Principal};
use did::ekoke::EkokeResult;
use did::ekoke_liquidity_pool::{
    EkokeLiquidityPoolInitData, LiquidityPoolAccounts, LiquidityPoolBalance,
};
use ic_cdk_macros::{init, query, update};
use icrc::icrc1::transfer::TransferError;

use self::app::EkokeLiquidityPoolCanister;

#[init]
pub fn init(data: EkokeLiquidityPoolInitData) {
    EkokeLiquidityPoolCanister::init(data);
}

#[query]
#[candid_method(query)]
pub async fn liquidity_pool_balance() -> EkokeResult<LiquidityPoolBalance> {
    EkokeLiquidityPoolCanister::liquidity_pool_balance().await
}

#[query]
#[candid_method(query)]
pub fn liquidity_pool_accounts() -> LiquidityPoolAccounts {
    EkokeLiquidityPoolCanister::liquidity_pool_accounts()
}

#[update]
#[candid_method(update)]
pub async fn refund_investors(refunds: HashMap<Principal, Nat>) -> Result<(), TransferError> {
    EkokeLiquidityPoolCanister::refund_investors(refunds).await
}

#[query]
#[candid_method(query)]
pub fn admin_cycles() -> Nat {
    EkokeLiquidityPoolCanister::admin_cycles()
}

#[update]
#[candid_method(update)]
pub fn admin_set_icp_ledger_canister(canister_id: Principal) {
    EkokeLiquidityPoolCanister::admin_set_icp_ledger_canister(canister_id)
}

#[update]
#[candid_method(update)]
pub fn admin_set_deferred_canister(canister_id: Principal) {
    EkokeLiquidityPoolCanister::admin_set_deferred_canister(canister_id)
}

#[update]
#[candid_method(update)]
pub fn admin_set_admins(admins: Vec<Principal>) {
    EkokeLiquidityPoolCanister::admin_set_admins(admins)
}

// HTTP endpoint
#[query]
#[candid_method(query)]
pub async fn http_request(req: did::HttpRequest) -> did::HttpResponse {
    http::HttpApi::handle_http_request(req).await
}

#[allow(dead_code)]
fn main() {
    // The line below generates did types and service definition from the
    // methods annotated with `candid_method` above. The definition is then
    // obtained with `__export_service()`.
    candid::export_service!();
    std::print!("{}", __export_service());
}

/// GetRandom fixup to allow getrandom compilation.
/// A getrandom implementation that always fails
///
/// This is a workaround for the fact that the `getrandom` crate does not compile
/// for the `wasm32-unknown-ic` target. This is a dummy implementation that always
/// fails with `Error::UNSUPPORTED`.
pub fn getrandom_always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}

getrandom::register_custom_getrandom!(getrandom_always_fail);
