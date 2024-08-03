//! # Deferred
//!
//! Deferred is a canister serving a DIP721 NFT contract that allows to create
//! a financial tool to sell any kind of entity (e.g. a house, a car, a boat, etc.) or to get
//! financing from third parties buying the NFTs and getting rewards in $EKOKE tokens

use candid::{candid_method, Nat, Principal};
use did::deferred::{
    Agency, Contract, ContractRegistration, DeferredInitData, DeferredResult, RestrictedProperty,
    Role, TokenInfo,
};
use did::{HttpRequest, HttpResponse, ID};
use dip721_rs::{Dip721 as _, GenericValue, TokenIdentifier};
use ic_cdk_macros::{init, post_upgrade, query, update};

mod app;
mod client;
mod constants;
mod http;
mod inspect;
mod utils;

use app::Deferred;
use icrc::icrc1::account::Subaccount;

#[init]
pub fn init(init_data: DeferredInitData) {
    Deferred::init(init_data);
}

#[post_upgrade]
pub fn post_upgrade() {
    Deferred::post_upgrade();
}

// MHSC api

#[update]
#[candid_method(update)]
pub async fn register_contract(data: ContractRegistration) -> DeferredResult<Nat> {
    Deferred::register_contract(data).await
}

#[update]
#[candid_method(update)]
pub async fn sign_contract(contract_id: ID) -> DeferredResult<()> {
    Deferred::sign_contract(contract_id).await
}

#[update]
#[candid_method(update)]
pub async fn increment_contract_value(
    contract_id: ID,
    value: u64,
    installments: u64,
) -> DeferredResult<()> {
    Deferred::increment_contract_value(contract_id, value, installments).await
}

#[query]
#[candid_method(query)]
pub fn get_token(token_id: TokenIdentifier) -> Option<TokenInfo> {
    Deferred::get_token(&token_id)
}

#[query]
#[candid_method(query)]
pub fn get_contract(id: ID) -> Option<Contract> {
    Deferred::get_contract(&id)
}

#[query]
#[candid_method(query)]
pub fn get_agencies() -> Vec<Agency> {
    Deferred::get_agencies()
}

#[update]
#[candid_method(update)]
pub fn remove_agency(wallet: Principal) -> DeferredResult<()> {
    Deferred::remove_agency(wallet)
}

#[query]
#[candid_method(query)]
pub fn get_signed_contracts() -> Vec<ID> {
    Deferred::get_signed_contracts()
}

#[update]
#[candid_method(update)]
pub fn update_contract_property(
    contract_id: ID,
    key: String,
    value: GenericValue,
) -> DeferredResult<()> {
    Deferred::update_contract_property(contract_id, key, value)
}

#[update]
#[candid_method(update)]
pub fn update_restricted_contract_property(
    contract_id: ID,
    key: String,
    value: RestrictedProperty,
) -> DeferredResult<()> {
    Deferred::update_restricted_contract_property(contract_id, key, value)
}

#[query]
#[candid_method(query)]
pub fn get_restricted_contract_properties(
    contract_id: ID,
) -> Option<Vec<(String, RestrictedProperty)>> {
    Deferred::get_restricted_contract_properties(contract_id)
}

#[query]
#[candid_method(query)]
pub fn get_unsigned_contracts() -> Vec<ID> {
    Deferred::get_unsigned_contracts()
}

#[update]
#[candid_method(update)]
pub fn update_contract_buyers(contract_id: ID, buyers: Vec<Principal>) -> DeferredResult<()> {
    Deferred::update_contract_buyers(contract_id, buyers)
}

#[update]
#[candid_method(update)]
pub async fn withdraw_contract_deposit(
    contract_id: ID,
    withdraw_subaccount: Option<Subaccount>,
) -> DeferredResult<()> {
    Deferred::withdraw_contract_deposit(contract_id, withdraw_subaccount).await
}

#[update]
#[candid_method(update)]
pub async fn close_contract(contract_id: ID) -> DeferredResult<()> {
    Deferred::close_contract(contract_id).await
}

#[update]
#[candid_method(update)]
pub fn admin_set_ekoke_reward_pool_canister(canister_id: Principal) {
    Deferred::admin_set_ekoke_reward_pool_canister(canister_id)
}

#[update]
#[candid_method(update)]
pub fn admin_set_ekoke_liquidity_pool_canister(canister_id: Principal) {
    Deferred::admin_set_ekoke_liquidity_pool_canister(canister_id)
}

#[update]
#[candid_method(update)]
pub fn admin_set_marketplace_canister(canister_id: Principal) {
    Deferred::admin_set_marketplace_canister(canister_id)
}

#[update]
#[candid_method(update)]
pub fn admin_set_role(principal: Principal, role: Role) {
    Deferred::admin_set_role(principal, role)
}

#[update]
#[candid_method(update)]
pub fn admin_remove_role(principal: Principal, role: Role) -> DeferredResult<()> {
    Deferred::admin_remove_role(principal, role)
}

#[update]
#[candid_method(update)]
pub fn admin_register_agency(wallet: Principal, agency: Agency) {
    Deferred::admin_register_agency(wallet, agency)
}

// DIP721

#[query]
#[candid_method(query)]
pub fn dip721_metadata() -> dip721_rs::Metadata {
    Deferred::dip721_metadata()
}

#[query]
#[candid_method(query)]
pub fn dip721_stats() -> dip721_rs::Stats {
    Deferred::dip721_stats()
}

#[query]
#[candid_method(query)]
pub fn dip721_logo() -> Option<String> {
    Deferred::dip721_logo()
}

#[update]
#[candid_method(update)]
pub fn dip721_set_logo(logo: String) {
    Deferred::dip721_set_logo(logo)
}

#[query]
#[candid_method(query)]
pub fn dip721_name() -> Option<String> {
    Deferred::dip721_name()
}

#[update]
#[candid_method(update)]
pub fn dip721_set_name(name: String) {
    Deferred::dip721_set_name(name)
}

#[query]
#[candid_method(query)]
pub fn dip721_symbol() -> Option<String> {
    Deferred::dip721_symbol()
}

#[update]
#[candid_method(update)]
pub fn dip721_set_symbol(symbol: String) {
    Deferred::dip721_set_symbol(symbol)
}

#[query]
#[candid_method(query)]
pub fn dip721_custodians() -> Vec<Principal> {
    Deferred::dip721_custodians()
}

#[update]
#[candid_method(update)]
pub fn dip721_set_custodians(custodians: Vec<Principal>) {
    Deferred::dip721_set_custodians(custodians)
}

#[query]
#[candid_method(query)]
pub fn dip721_cycles() -> Nat {
    Deferred::dip721_cycles()
}

#[query]
#[candid_method(query)]
pub fn dip721_total_unique_holders() -> Nat {
    Deferred::dip721_total_unique_holders()
}

#[query]
#[candid_method(query)]
pub fn dip721_token_metadata(
    token_identifier: dip721_rs::TokenIdentifier,
) -> Result<dip721_rs::TokenMetadata, dip721_rs::NftError> {
    Deferred::dip721_token_metadata(token_identifier)
}

#[query]
#[candid_method(query)]
pub fn dip721_balance_of(owner: Principal) -> Result<Nat, dip721_rs::NftError> {
    Deferred::dip721_balance_of(owner)
}

#[query]
#[candid_method(query)]
pub fn dip721_owner_of(
    token_identifier: dip721_rs::TokenIdentifier,
) -> Result<Option<Principal>, dip721_rs::NftError> {
    Deferred::dip721_owner_of(token_identifier)
}

#[query]
#[candid_method(query)]
pub fn dip721_owner_token_identifiers(
    owner: Principal,
) -> Result<Vec<dip721_rs::TokenIdentifier>, dip721_rs::NftError> {
    Deferred::dip721_owner_token_identifiers(owner)
}

#[query]
#[candid_method(query)]
pub fn dip721_owner_token_metadata(
    owner: Principal,
) -> Result<Vec<dip721_rs::TokenMetadata>, dip721_rs::NftError> {
    Deferred::dip721_owner_token_metadata(owner)
}

#[query]
#[candid_method(query)]
pub fn dip721_operator_of(
    token_identifier: dip721_rs::TokenIdentifier,
) -> Result<Option<Principal>, dip721_rs::NftError> {
    Deferred::dip721_operator_of(token_identifier)
}

#[query]
#[candid_method(query)]
pub fn dip721_operator_token_identifiers(
    operator: Principal,
) -> Result<Vec<dip721_rs::TokenIdentifier>, dip721_rs::NftError> {
    Deferred::dip721_operator_token_identifiers(operator)
}

#[query]
#[candid_method(query)]
pub fn dip721_operator_token_metadata(
    operator: Principal,
) -> Result<Vec<dip721_rs::TokenMetadata>, dip721_rs::NftError> {
    Deferred::dip721_operator_token_metadata(operator)
}

#[query]
#[candid_method(query)]
pub fn dip721_supported_interfaces() -> Vec<dip721_rs::SupportedInterface> {
    Deferred::dip721_supported_interfaces()
}

#[query]
#[candid_method(query)]
pub fn dip721_total_supply() -> Nat {
    Deferred::dip721_total_supply()
}

#[update]
#[candid_method(update)]
pub fn dip721_approve(
    spender: Principal,
    token_identifier: dip721_rs::TokenIdentifier,
) -> Result<dip721_rs::TokenIdentifier, dip721_rs::NftError> {
    Deferred::dip721_approve(spender, token_identifier)
}

#[update]
#[candid_method(update)]
pub fn dip721_set_approval_for_all(
    operator: Principal,
    approved: bool,
) -> Result<dip721_rs::TokenIdentifier, dip721_rs::NftError> {
    Deferred::dip721_set_approval_for_all(operator, approved)
}

#[update]
#[candid_method(update)]
pub fn dip721_is_approved_for_all(
    owner: Principal,
    operator: Principal,
) -> Result<bool, dip721_rs::NftError> {
    Deferred::dip721_is_approved_for_all(owner, operator)
}

#[update]
#[candid_method(update)]
pub async fn dip721_transfer(
    to: Principal,
    token_identifier: dip721_rs::TokenIdentifier,
) -> Result<Nat, dip721_rs::NftError> {
    Deferred::dip721_transfer(to, token_identifier).await
}

#[update]
#[candid_method(update)]
pub async fn dip721_transfer_from(
    from: Principal,
    to: Principal,
    token_identifier: dip721_rs::TokenIdentifier,
) -> Result<Nat, dip721_rs::NftError> {
    Deferred::dip721_transfer_from(from, to, token_identifier).await
}

#[update]
#[candid_method(update)]
pub fn dip721_mint(
    to: Principal,
    token_identifier: dip721_rs::TokenIdentifier,
    properties: Vec<(String, dip721_rs::GenericValue)>,
) -> Result<Nat, dip721_rs::NftError> {
    Deferred::dip721_mint(to, token_identifier, properties)
}

#[update]
#[candid_method(update)]
pub fn dip721_burn(
    token_identifier: dip721_rs::TokenIdentifier,
) -> Result<dip721_rs::TokenIdentifier, dip721_rs::NftError> {
    Deferred::dip721_burn(token_identifier)
}

#[query]
#[candid_method(query)]
pub fn dip721_transaction(tx_id: Nat) -> Result<dip721_rs::TxEvent, dip721_rs::NftError> {
    Deferred::dip721_transaction(tx_id)
}

#[query]
#[candid_method(query)]
pub fn dip721_total_transactions() -> Nat {
    Deferred::dip721_total_transactions()
}

// HTTP endpoint
#[query]
#[candid_method(query)]
pub async fn http_request(req: HttpRequest) -> HttpResponse {
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
