//! # Fly
//!
//! The fly canister serves a ICRC-2 token called $FLY, which is the reward token for Deferred transactions.
//! It is a deflationary token which ...

mod app;
mod constants;
mod inspect;
mod utils;

use app::FlyCanister;
use candid::{candid_method, Nat, Principal};
use did::fly::{FlyInitData, FlyResult, PicoFly, Role, Transaction};
use did::ID;
use ic_cdk_macros::{init, post_upgrade, query, update};
use icrc::icrc::generic_metadata_value::MetadataValue;
use icrc::icrc1::account::Account;
use icrc::icrc1::transfer::TransferArg;
use icrc::icrc1::{self, transfer as icrc1_transfer, Icrc1};

#[init]
pub fn init(data: FlyInitData) {
    FlyCanister::init(data);
}

#[post_upgrade]
pub fn post_upgrade() {
    FlyCanister::post_upgrade();
}

#[update]
#[candid_method(update)]
pub fn get_contract_reward(contract_id: ID, installments: u64) -> FlyResult<PicoFly> {
    FlyCanister::get_contract_reward(contract_id, installments)
}

#[update]
#[candid_method(update)]
pub fn reserve_pool(from: Account, contract_id: ID, picofly_amount: PicoFly) -> FlyResult<PicoFly> {
    FlyCanister::reserve_pool(from, contract_id, picofly_amount)
}

#[update]
#[candid_method(update)]
pub fn admin_set_role(principal: Principal, role: Role) {
    FlyCanister::admin_set_role(principal, role)
}

#[update]
#[candid_method(update)]
pub fn admin_remove_role(principal: Principal, role: Role) -> FlyResult<()> {
    FlyCanister::admin_remove_role(principal, role)
}

#[query]
#[candid_method(query)]
pub fn admin_cycles() -> Nat {
    FlyCanister::admin_cycles()
}

#[update]
#[candid_method(update)]
pub fn admin_burn(amount: PicoFly) -> FlyResult<()> {
    FlyCanister::admin_burn(amount)
}

#[query]
#[candid_method(query)]
pub fn get_transaction(id: u64) -> FlyResult<Transaction> {
    FlyCanister::get_transaction(id)
}

// icrc-1

#[query]
#[candid_method(query)]
pub fn icrc1_name() -> &'static str {
    FlyCanister::icrc1_name()
}

#[query]
#[candid_method(query)]
pub fn icrc1_symbol() -> &'static str {
    FlyCanister::icrc1_symbol()
}

#[query]
#[candid_method(query)]
pub fn icrc1_decimals() -> u8 {
    FlyCanister::icrc1_decimals()
}

#[query]
#[candid_method(query)]
pub fn icrc1_fee() -> Nat {
    FlyCanister::icrc1_fee()
}

#[query]
#[candid_method(query)]
pub fn icrc1_metadata() -> Vec<(String, MetadataValue)> {
    FlyCanister::icrc1_metadata()
}

#[query]
#[candid_method(query)]
pub fn icrc1_total_supply() -> Nat {
    FlyCanister::icrc1_total_supply()
}

#[query]
#[candid_method(query)]
pub fn icrc1_balance_of(account: Account) -> Nat {
    FlyCanister::icrc1_balance_of(account)
}

#[update]
#[candid_method(update)]
pub fn icrc1_transfer(transfer_args: TransferArg) -> Result<Nat, icrc1_transfer::TransferError> {
    FlyCanister::icrc1_transfer(transfer_args)
}

#[query]
#[candid_method(query)]
pub fn icrc1_supported_standards() -> Vec<icrc1::TokenExtension> {
    FlyCanister::icrc1_supported_standards()
}

#[allow(dead_code)]
fn main() {
    // The line below generates did types and service definition from the
    // methods annotated with `candid_method` above. The definition is then
    // obtained with `__export_service()`.
    candid::export_service!();
    std::print!("{}", __export_service());
}
