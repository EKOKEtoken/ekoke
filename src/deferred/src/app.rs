//! # Deferred
//!
//! API for Deferred

mod configuration;
mod inspect;
mod memory;
mod minter;
mod ops;
mod roles;
pub mod storage;
#[cfg(test)]
mod test_utils;

use async_trait::async_trait;
use candid::{Nat, Principal};
use configuration::Configuration;
use did::deferred::{
    Agency, CloseContractError, Contract, ContractRegistration, DeferredError, DeferredInitData,
    DeferredResult, RestrictedContractProperties, RestrictedProperty, RestrictionLevel, Role,
    TokenError, TokenInfo,
};
use did::ID;
use dip721_rs::{
    Dip721, GenericValue, Metadata, NftError, Stats, SupportedInterface, TokenIdentifier,
    TokenMetadata, TxEvent,
};
use icrc::icrc1::account::{Account, Subaccount};
use ops::{CloseOp, DepositOp};

pub use self::inspect::Inspect;
use self::minter::Minter;
use self::roles::RolesManager;
use self::storage::{Agents, ContractStorage, TxHistory};
use crate::utils::{self, caller};

#[derive(Default)]
/// Deferred canister API
pub struct Deferred;

impl Deferred {
    /// On init set custodians and canisters ids
    pub fn init(init_data: DeferredInitData) {
        RolesManager::set_custodians(init_data.custodians).expect("storage error");
        Configuration::set_ekoke_reward_pool_canister(init_data.ekoke_reward_pool_canister)
            .expect("storage error");
        Configuration::set_marketplace_canister(init_data.marketplace_canister)
            .expect("storage error");
        Configuration::set_icp_ledger_canister(init_data.icp_ledger_canister)
            .expect("storage error");
        Configuration::set_liquidity_pool_canister(init_data.liquidity_pool_canister)
            .expect("storage error");
    }

    /// Task to execute on post upgrade
    pub fn post_upgrade() {
        // update upgraded at timestamp
        if let Err(err) = Configuration::set_upgraded_at() {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// get token and contract info by token identifier
    pub fn get_token(token_identifier: &TokenIdentifier) -> Option<TokenInfo> {
        let token = ContractStorage::get_token(token_identifier)?;
        let contract = ContractStorage::get_contract(&token.contract_id)?;

        Some(TokenInfo { token, contract })
    }

    /// get contract by id
    pub fn get_contract(id: &ID) -> Option<Contract> {
        ContractStorage::get_contract(id)
    }

    /// get contracts ids
    pub fn get_signed_contracts() -> Vec<ID> {
        ContractStorage::get_signed_contracts()
    }

    /// get agencies
    pub fn get_agencies() -> Vec<Agency> {
        Agents::get_agencies()
    }

    /// Remove agency by wallet.
    ///
    /// Only a custodian can call this method or the caller must be the owner of the agency
    pub fn remove_agency(wallet: Principal) -> DeferredResult<()> {
        if !Inspect::inspect_remove_agency(caller()) {
            ic_cdk::trap("Unauthorized");
        }

        Agents::remove_agency(wallet);
        // remove role
        RolesManager::remove_role(wallet, Role::Agent)
    }

    /// get unsigned contracts for agent.
    /// If called by admin return them all
    pub fn get_unsigned_contracts() -> Vec<ID> {
        let is_custodian = RolesManager::is_custodian(caller());
        if !is_custodian && !Inspect::inspect_is_agent(caller()) {
            ic_cdk::trap("Unauthorized");
        }

        let agency = Agents::get_agency_by_wallet(caller());

        ContractStorage::get_unsigned_contracts(|contract| {
            if is_custodian {
                true
            } else {
                contract.agency == agency
            }
        })
    }

    /// Update contract buyers. Only the seller can call this method.
    pub fn update_contract_buyers(contract_id: ID, buyers: Vec<Principal>) -> DeferredResult<()> {
        Inspect::inspect_is_seller(caller(), contract_id.clone())?;
        ContractStorage::update_contract_buyers(&contract_id, buyers)
    }

    /// Update a contract property. Only custodian,seller,agent can call this function
    pub fn update_contract_property(
        contract_id: ID,
        key: String,
        value: GenericValue,
    ) -> DeferredResult<()> {
        Inspect::inspect_update_contract_property(caller(), &contract_id, &key)?;
        ContractStorage::update_contract_property(&contract_id, key, value)
    }

    /// Update a restricted contract property. Only custodian,seller,agent can call this function
    pub fn update_restricted_contract_property(
        contract_id: ID,
        key: String,
        value: RestrictedProperty,
    ) -> DeferredResult<()> {
        Inspect::inspect_update_contract_property(caller(), &contract_id, &key)?;
        ContractStorage::update_restricted_contract_property(&contract_id, key, value)
    }

    /// Get all the restricted contract properties based on the contract id and the caller access
    pub fn get_restricted_contract_properties(
        contract_id: ID,
    ) -> Option<RestrictedContractProperties> {
        let contract = ContractStorage::get_contract(&contract_id)?;
        let mut caller_access_levels = vec![];
        let caller = caller();

        if contract.buyers.contains(&caller) {
            caller_access_levels.push(RestrictionLevel::Buyer);
        }
        if contract
            .sellers
            .iter()
            .any(|seller| seller.principal == caller)
        {
            caller_access_levels.push(RestrictionLevel::Seller);
        }
        if Agents::get_agency_by_wallet(caller) == contract.agency {
            caller_access_levels.push(RestrictionLevel::Agent);
        }

        Some(
            contract
                .restricted_properties
                .into_iter()
                .filter(|(_, prop)| {
                    prop.access_list
                        .iter()
                        .any(|level| caller_access_levels.contains(level))
                })
                .collect(),
        )
    }

    /// Increment contract value. Only an admin or the associated agency can call this method
    pub async fn increment_contract_value(
        contract_id: ID,
        incr_by: u64,
        installments: u64,
    ) -> DeferredResult<()> {
        let contract_sellers =
            Inspect::inspect_increment_contract_value(caller(), contract_id.clone())?.sellers;

        // mint new tokens
        let (tokens, _) =
            Minter::mint(&contract_id, contract_sellers, installments, incr_by).await?;

        // update contract
        ContractStorage::add_tokens_to_contract(&contract_id, tokens)
    }

    /// Register contract inside of the canister.
    /// Only a custodian can call this method.
    ///
    /// Returns the contract id
    pub async fn register_contract(data: ContractRegistration) -> DeferredResult<ID> {
        Inspect::inspect_register_contract(
            caller(),
            data.value,
            &data.deposit,
            &data.sellers,
            &data.buyers,
            data.installments,
            data.expiration.as_deref(),
        )?;

        let next_contract_id = ContractStorage::next_contract_id();

        // take buyer's deposit
        if data.deposit.value_icp > 0 {
            DepositOp::take_buyers_deposit(
                &next_contract_id,
                data.buyers.deposit_account,
                data.deposit.value_icp,
            )
            .await?;
        }

        // make contract
        let contract = Contract {
            id: next_contract_id.clone(),
            buyers: data.buyers.principals,
            currency: data.currency,
            initial_value: data.value,
            properties: data.properties,
            restricted_properties: data.restricted_properties,
            installments: data.installments,
            is_signed: false,
            r#type: data.r#type,
            sellers: data.sellers,
            tokens: vec![],
            value: data.value,
            deposit: data.deposit,
            expiration: data.expiration,
            agency: Agents::get_agency_by_wallet(caller()),
        };

        // register contract
        ContractStorage::insert_contract(contract)?;

        Ok(next_contract_id)
    }

    /// Sign contract and mint tokens
    pub async fn sign_contract(contract_id: ID) -> DeferredResult<()> {
        if !Inspect::inspect_sign_contract(caller(), &contract_id) {
            ic_cdk::trap("Unauthorized");
        }

        let contract = match ContractStorage::get_contract(&contract_id) {
            Some(contract) => contract,
            None => {
                return Err(DeferredError::Token(TokenError::ContractNotFound(
                    contract_id,
                )))
            }
        };

        let installments_value = contract.installments_value();

        // mint new tokens
        let (tokens, _) = Minter::mint(
            &contract_id,
            contract.sellers,
            contract.installments,
            installments_value,
        )
        .await?;

        // update contract
        ContractStorage::sign_contract_and_mint_tokens(&contract_id, tokens)
    }

    /// Call for the contract seller to withdraw the buyer deposit in case the contract has been completely paid
    pub async fn withdraw_contract_deposit(
        contract_id: ID,
        withdraw_subaccount: Option<Subaccount>,
    ) -> DeferredResult<()> {
        Inspect::inspect_is_seller(caller(), contract_id.clone())?;

        CloseOp::withdraw_contract_deposit(contract_id, withdraw_subaccount).await
    }

    /// Close a contract which hasn't been completely paid and is expired.
    ///
    /// Only the agency can call this method.
    ///
    /// This method will burn all the tokens and will proportionally refund the NFTs owners, except the contract buyer.
    pub async fn close_contract(contract_id: ID) -> DeferredResult<()> {
        if Inspect::inspect_is_agent_for_contract(caller(), &contract_id).is_err()
            && !Inspect::inspect_is_custodian(caller())
        {
            ic_cdk::trap("Unauthorized");
        }
        let contract = ContractStorage::get_contract(&contract_id).ok_or_else(|| {
            DeferredError::CloseContract(CloseContractError::ContractNotFound(contract_id.clone()))
        })?;

        CloseOp::close_contract(contract).await
    }

    /// Update marketplace canister id and update the operator for all the tokens
    pub fn admin_set_marketplace_canister(canister: Principal) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }

        if let Err(err) = Configuration::set_marketplace_canister(canister) {
            ic_cdk::trap(&err.to_string());
        }

        // update tokens
        if let Err(err) = ContractStorage::update_tokens_operator(canister) {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// Update liquidity pool canister id
    pub fn admin_set_ekoke_liquidity_pool_canister(canister: Principal) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }

        if let Err(err) = Configuration::set_liquidity_pool_canister(canister) {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// Update ekoke reward pool canister id
    pub fn admin_set_ekoke_reward_pool_canister(canister: Principal) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }

        if let Err(err) = Configuration::set_ekoke_reward_pool_canister(canister) {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// Insert agency into the storage
    pub fn admin_register_agency(wallet: Principal, agency: Agency) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }

        Agents::insert_agency(wallet, agency);
        // give role to the agent
        if !RolesManager::is_custodian(wallet) {
            RolesManager::give_role(wallet, Role::Agent);
        }
    }

    /// Give role to the provied principal
    pub fn admin_set_role(principal: Principal, role: Role) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }
        RolesManager::give_role(principal, role);
    }

    /// Remove role from principal.
    ///
    /// Fails if trying to remove the only custodian of the canister
    pub fn admin_remove_role(principal: Principal, role: Role) -> DeferredResult<()> {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }

        RolesManager::remove_role(principal, role)
    }

    /// Canister subaccount for contract deposit
    fn contract_deposit_subaccount(contract_id: &ID) -> Subaccount {
        let contract_id = contract_id.0.to_bytes_be();
        // if contract id is less than 32 bytes, pad it with zeros
        let contract_id = if contract_id.len() < 32 {
            let mut padded = vec![0; 32 - contract_id.len()];
            padded.extend_from_slice(&contract_id);
            padded
        } else {
            contract_id
        };
        let subaccount: Subaccount = contract_id.try_into().expect("invalid contract id");

        subaccount
    }

    /// Canister ICRC account
    fn canister_deposit_account(contract_id: &ID) -> Account {
        Account {
            owner: utils::id(),
            subaccount: Some(Self::contract_deposit_subaccount(contract_id)),
        }
    }
}

#[async_trait]
impl Dip721 for Deferred {
    /// Returns the Metadata of the NFT canister which includes custodians, logo, name, symbol.
    fn dip721_metadata() -> Metadata {
        Metadata {
            created_at: Configuration::get_created_at(),
            custodians: Self::dip721_custodians(),
            logo: Self::dip721_logo(),
            name: Self::dip721_name(),
            symbol: Self::dip721_symbol(),
            upgraded_at: Configuration::get_upgraded_at(),
        }
    }

    /// Returns the Stats of the NFT canister which includes cycles, totalSupply, totalTransactions, totalUniqueHolders.
    fn dip721_stats() -> Stats {
        Stats {
            cycles: Self::dip721_cycles(),
            total_supply: Self::dip721_total_supply(),
            total_transactions: Self::dip721_total_transactions(),
            total_unique_holders: Self::dip721_total_unique_holders(),
        }
    }

    /// Returns the logo of the NFT contract as Base64 encoded text.
    fn dip721_logo() -> Option<String> {
        Configuration::get_logo()
    }

    /// Sets the logo of the NFT canister. Base64 encoded text is recommended.
    /// Caller must be the custodian of NFT canister.
    fn dip721_set_logo(logo: String) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }
        if let Err(err) = Configuration::set_logo(logo) {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// Returns the name of the NFT canister.
    fn dip721_name() -> Option<String> {
        Configuration::get_name()
    }

    /// Sets the name of the NFT contract.
    /// Caller must be the custodian of NFT canister.
    fn dip721_set_name(name: String) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }
        if let Err(err) = Configuration::set_name(name) {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// Returns the symbol of the NFT contract.
    fn dip721_symbol() -> Option<String> {
        Configuration::get_symbol()
    }

    /// Set symbol
    /// Caller must be the custodian of NFT canister.
    fn dip721_set_symbol(symbol: String) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }
        if let Err(err) = Configuration::set_symbol(symbol) {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// Returns a list of the canister custodians
    fn dip721_custodians() -> Vec<Principal> {
        RolesManager::get_custodians()
    }

    /// Set canister custodians
    /// Caller must be the custodian of NFT canister.
    fn dip721_set_custodians(custodians: Vec<Principal>) {
        if !Inspect::inspect_is_custodian(caller()) {
            ic_cdk::trap("Unauthorized");
        }
        if let Err(err) = RolesManager::set_custodians(custodians) {
            ic_cdk::trap(&err.to_string());
        }
    }

    /// Returns canister cycles
    fn dip721_cycles() -> Nat {
        crate::utils::cycles()
    }

    /// Returns total unique holders of tokens
    fn dip721_total_unique_holders() -> Nat {
        ContractStorage::total_unique_holders().into()
    }

    /// Returns metadata for token
    fn dip721_token_metadata(token_identifier: TokenIdentifier) -> Result<TokenMetadata, NftError> {
        ContractStorage::get_token_metadata(&token_identifier).ok_or(NftError::TokenNotFound)
    }

    /// Returns the count of NFTs owned by user.
    /// If the user does not own any NFTs, returns an error containing NftError.
    fn dip721_balance_of(owner: Principal) -> Result<Nat, NftError> {
        match ContractStorage::tokens_by_owner(owner) {
            tokens if tokens.is_empty() => Err(NftError::OwnerNotFound),
            tokens => Ok(tokens.len().into()),
        }
    }

    /// Returns the owner of the token.
    /// Returns an error containing NftError if token_identifier is invalid.
    fn dip721_owner_of(token_identifier: TokenIdentifier) -> Result<Option<Principal>, NftError> {
        match ContractStorage::get_token(&token_identifier).map(|token| token.owner) {
            Some(owner) => Ok(owner),
            None => Err(NftError::TokenNotFound),
        }
    }

    /// Returns the list of the token_identifier of the NFT associated with owner.
    /// Returns an error containing NftError if principal is invalid.
    fn dip721_owner_token_identifiers(owner: Principal) -> Result<Vec<TokenIdentifier>, NftError> {
        match ContractStorage::tokens_by_owner(owner) {
            tokens if tokens.is_empty() => Err(NftError::OwnerNotFound),
            tokens => Ok(tokens),
        }
    }

    /// Returns the list of the token_metadata of the NFT associated with owner.
    /// Returns an error containing NftError if principal is invalid.
    fn dip721_owner_token_metadata(owner: Principal) -> Result<Vec<TokenMetadata>, NftError> {
        let tokens = Self::dip721_owner_token_identifiers(owner)?;
        let mut metadata = Vec::with_capacity(tokens.len());
        for token in tokens {
            metadata.push(Self::dip721_token_metadata(token)?);
        }

        if metadata.is_empty() {
            return Err(NftError::OwnerNotFound);
        }

        Ok(metadata)
    }

    /// Returns the Principal of the operator of the NFT associated with token_identifier.
    fn dip721_operator_of(
        token_identifier: TokenIdentifier,
    ) -> Result<Option<Principal>, NftError> {
        match ContractStorage::get_token(&token_identifier) {
            Some(token) => Ok(token.operator),
            None => Err(NftError::TokenNotFound),
        }
    }

    /// Returns the list of the token_identifier of the NFT associated with operator.
    fn dip721_operator_token_identifiers(
        operator: Principal,
    ) -> Result<Vec<TokenIdentifier>, NftError> {
        match ContractStorage::tokens_by_operator(operator) {
            tokens if tokens.is_empty() => Err(NftError::OperatorNotFound),
            tokens => Ok(tokens),
        }
    }

    /// Returns the list of the token_metadata of the NFT associated with operator.
    fn dip721_operator_token_metadata(operator: Principal) -> Result<Vec<TokenMetadata>, NftError> {
        let tokens = Self::dip721_operator_token_identifiers(operator)?;
        let mut metadata = Vec::with_capacity(tokens.len());
        for token in tokens {
            metadata.push(Self::dip721_token_metadata(token)?);
        }

        if metadata.is_empty() {
            return Err(NftError::OperatorNotFound);
        }

        Ok(metadata)
    }

    /// Returns the list of the interfaces supported by this canister
    fn dip721_supported_interfaces() -> Vec<SupportedInterface> {
        vec![
            SupportedInterface::Burn,
            SupportedInterface::TransactionHistory,
        ]
    }

    /// Returns the total supply of the NFT.
    /// NFTs that are minted and later burned explicitly or sent to the zero address should also count towards totalSupply.
    fn dip721_total_supply() -> Nat {
        ContractStorage::total_supply().into()
    }

    // Calling approve grants the operator the ability to make update calls to the specificied token_identifier.
    // Approvals given by the approve function are independent from approvals given by the setApprovalForAll.
    //
    // If the approval goes through, returns a nat that represents the CAP History transaction ID that can be used at the transaction method.
    /// Interface: approval
    fn dip721_approve(
        _operator: Principal,
        _token_identifier: TokenIdentifier,
    ) -> Result<Nat, NftError> {
        Err(NftError::Other("Not implemented".to_string()))
    }

    /// Enable or disable an operator to manage all of the tokens for the caller of this function. The contract allows multiple operators per owner.
    /// Approvals granted by the approve function are independent from the approvals granted by setApprovalForAll function.
    /// If the approval goes through, returns a nat that represents the CAP History transaction ID that can be used at the transaction method.
    /// Interface: approval
    fn dip721_set_approval_for_all(_operator: Principal, _approved: bool) -> Result<Nat, NftError> {
        Err(NftError::Other("Not implemented".to_string()))
    }

    /// Returns true if the given operator is an approved operator for all the tokens owned by the caller through the use of the setApprovalForAll method, returns false otherwise.
    /// Interface: approval
    fn dip721_is_approved_for_all(
        _owner: Principal,
        _operator: Principal,
    ) -> Result<bool, NftError> {
        Err(NftError::Other("Not implemented".to_string()))
    }

    /// Sends the callers nft token_identifier to `to`` and returns a nat that represents a
    /// transaction id that can be used at the transaction method.
    async fn dip721_transfer(
        to: Principal,
        token_identifier: TokenIdentifier,
    ) -> Result<Nat, NftError> {
        Self::dip721_transfer_from(caller(), to, token_identifier).await
    }

    /// Caller of this method is able to transfer the NFT token_identifier that is in from's balance to to's balance
    /// if the caller is an approved operator to do so.
    ///
    /// If the transfer goes through, returns a nat that represents the CAP History transaction ID
    /// that can be used at the transaction method.
    async fn dip721_transfer_from(
        owner: Principal,
        to: Principal,
        token_identifier: TokenIdentifier,
    ) -> Result<Nat, NftError> {
        let token = Inspect::inspect_transfer_from(caller(), &token_identifier)?;
        // verify that from owner is the same as the token's
        if token.owner != Some(owner) {
            return Err(NftError::OwnerNotFound);
        }
        // verify that owner is not the same as to
        if token.owner == Some(to) {
            return Err(NftError::SelfTransfer);
        }

        // transfer token to the new owner
        let tx_id = match ContractStorage::transfer(&token_identifier, to) {
            Ok(tx_id) => tx_id,
            Err(DeferredError::Token(TokenError::TokenNotFound(_))) => {
                return Err(NftError::TokenNotFound)
            }
            Err(_) => return Err(NftError::UnauthorizedOperator),
        };

        Ok(tx_id)
    }

    fn dip721_mint(
        _to: Principal,
        _token_identifier: TokenIdentifier,
        _properties: Vec<(String, GenericValue)>,
    ) -> Result<Nat, NftError> {
        Err(NftError::Other("Not implemented".to_string()))
    }

    /// Burn an NFT identified by token_identifier. Calling burn on a token sets the owner to None and
    /// will no longer be useable.
    /// Burned tokens do still count towards totalSupply.
    /// Implementations are encouraged to only allow burning by the owner of the token_identifier.
    ///
    /// The burn will also reduce the contract value by the token value
    fn dip721_burn(token_identifier: TokenIdentifier) -> Result<Nat, NftError> {
        Inspect::inspect_burn(caller(), &token_identifier)?;

        match ContractStorage::burn_token(&token_identifier) {
            Ok(tx_id) => Ok(tx_id),
            Err(DeferredError::Token(TokenError::TokenNotFound(_))) => Err(NftError::TokenNotFound),
            Err(_) => Err(NftError::UnauthorizedOperator),
        }
    }

    /// Returns the TxEvent that corresponds with tx_id.
    /// If there is no TxEvent that corresponds with the tx_id entered, returns a NftError.TxNotFound.
    fn dip721_transaction(tx_id: Nat) -> Result<TxEvent, NftError> {
        match TxHistory::get_transaction_by_id(tx_id) {
            Some(ev) => Ok(ev),
            None => Err(NftError::TxNotFound),
        }
    }

    /// Returns a nat that represents the total number of transactions that have occurred on the NFT canister.
    fn dip721_total_transactions() -> Nat {
        TxHistory::count().into()
    }
}

#[cfg(test)]
mod test {

    use std::time::Duration;
    use std::u64;

    use did::deferred::{Buyers, Deposit, Seller};
    use pretty_assertions::{assert_eq, assert_ne};
    use test_utils::{alice, bob_account};

    use self::test_utils::{bob, mock_agency};
    use super::test_utils::store_mock_contract;
    use super::*;
    use crate::app::test_utils::{mock_token, store_mock_contract_with};
    use crate::constants::{DEFAULT_LOGO, DEFAULT_NAME, DEFAULT_SYMBOL};

    #[test]
    fn test_should_init_canister() {
        Deferred::init(DeferredInitData {
            custodians: vec![caller()],
            ekoke_reward_pool_canister: caller(),
            icp_ledger_canister: caller(),
            liquidity_pool_canister: caller(),
            marketplace_canister: caller(),
        });

        assert_eq!(Deferred::dip721_custodians(), vec![caller()]);
        assert_eq!(Configuration::get_ekoke_reward_pool_canister(), caller());
        assert_eq!(Configuration::get_marketplace_canister(), caller());
    }

    #[test]
    fn test_should_set_upgrade_time_on_post_upgrade() {
        init_canister();
        let metadata = Deferred::dip721_metadata();
        assert!(metadata.upgraded_at == metadata.created_at);
        std::thread::sleep(Duration::from_millis(100));
        Deferred::post_upgrade();
        let metadata = Deferred::dip721_metadata();
        assert!(metadata.upgraded_at > metadata.created_at);
    }

    #[test]
    fn test_should_get_token_info() {
        init_canister();
        store_mock_contract(&[1, 2], 1);

        let token_info = Deferred::get_token(&2_u64.into()).unwrap();
        assert_eq!(token_info.token.id, Nat::from(2_u64));
        assert_eq!(token_info.contract.id, Nat::from(1_u64));
    }

    #[test]
    fn test_should_get_contract() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        assert_eq!(
            Deferred::get_contract(&1_u64.into()).unwrap().id,
            Nat::from(1_u64)
        );
    }

    #[test]
    fn test_should_get_signed_contracts() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        store_mock_contract(&[3, 4], 2);
        assert_eq!(
            Deferred::get_signed_contracts(),
            vec![Nat::from(1_u64), Nat::from(2_u64)]
        );
    }

    #[tokio::test]
    async fn test_should_register_and_sign_contract() {
        init_canister();
        let contract = ContractRegistration {
            buyers: Buyers {
                principals: vec![caller()],
                deposit_account: bob_account(),
            },
            currency: "EUR".to_string(),
            installments: 10,
            properties: vec![],
            restricted_properties: vec![],
            deposit: Deposit {
                value_fiat: 20,
                value_icp: 100,
            },
            r#type: did::deferred::ContractType::Financing,
            sellers: vec![Seller {
                principal: caller(),
                quota: 100,
            }],
            value: 100,
            expiration: Some("2048-01-01".to_string()),
        };

        assert_eq!(Deferred::register_contract(contract).await.unwrap(), 0_u64);
        assert_eq!(Deferred::dip721_total_supply(), Nat::from(0_u64));
        assert_eq!(Deferred::get_unsigned_contracts(), vec![Nat::from(0_u64)]);

        // sign and get total supply
        assert!(Deferred::sign_contract(0_u64.into()).await.is_ok());
        assert_eq!(Deferred::get_signed_contracts(), vec![Nat::from(0_u64)]);
        assert_eq!(Deferred::dip721_total_supply(), Nat::from(10_u64));

        // verify installment value is 8
        let token = ContractStorage::get_token(&0_u64.into()).unwrap();
        assert_eq!(token.value, 8);
    }

    #[tokio::test]
    async fn test_should_increment_contract_value() {
        init_canister();
        let contract = ContractRegistration {
            buyers: Buyers {
                principals: vec![caller()],
                deposit_account: bob_account(),
            },
            currency: "EUR".to_string(),
            installments: 10,
            properties: vec![],
            restricted_properties: vec![],
            deposit: Deposit {
                value_fiat: 20,
                value_icp: 100,
            },
            r#type: did::deferred::ContractType::Financing,
            sellers: vec![Seller {
                principal: caller(),
                quota: 100,
            }],
            value: 100,
            expiration: Some("2048-01-01".to_string()),
        };
        assert_eq!(Deferred::register_contract(contract).await.unwrap(), 0_u64);
        assert!(Deferred::sign_contract(0_u64.into()).await.is_ok());

        // increment value
        assert!(Deferred::increment_contract_value(0_u64.into(), 50, 10)
            .await
            .is_ok());
        assert_eq!(Deferred::dip721_total_supply(), Nat::from(20_u64));
    }

    #[tokio::test]
    async fn test_should_withdraw_contract_deposit() {
        init_canister();

        let contract_id = 1;
        test_utils::store_mock_contract_with(
            &[1, 2, 3],
            contract_id,
            |contract| {
                contract.value = 400_000;
                contract.deposit = Deposit {
                    value_fiat: 10000,
                    value_icp: 100_000_000,
                };
                contract.sellers = vec![
                    Seller {
                        principal: caller(),
                        quota: 50,
                    },
                    Seller {
                        principal: bob(),
                        quota: 50,
                    },
                ];
            },
            |token| {
                token.is_burned = true;
                token.owner = Some(caller());
            },
        );

        // withdraw deposit
        assert!(
            Deferred::withdraw_contract_deposit(contract_id.into(), None)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_should_not_withdraw_contract_deposit_if_not_burned_yet() {
        init_canister();

        let contract_id = 1;
        test_utils::store_mock_contract_with(
            &[1, 2, 3],
            contract_id,
            |contract| {
                contract.deposit = Deposit {
                    value_fiat: 10000,
                    value_icp: 100_000_000,
                };
                contract.sellers = vec![
                    Seller {
                        principal: caller(),
                        quota: 50,
                    },
                    Seller {
                        principal: bob(),
                        quota: 50,
                    },
                ];
            },
            |token| token.is_burned = false,
        );

        // withdraw deposit
        assert!(
            Deferred::withdraw_contract_deposit(contract_id.into(), None)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_should_not_withdraw_contract_deposit_if_not_seller() {
        init_canister();

        let contract_id = 1;
        test_utils::store_mock_contract_with(
            &[1, 2, 3],
            contract_id,
            |contract| {
                contract.deposit = Deposit {
                    value_fiat: 10000,
                    value_icp: 100,
                };
                contract.sellers = vec![
                    Seller {
                        principal: alice(),
                        quota: 50,
                    },
                    Seller {
                        principal: bob(),
                        quota: 50,
                    },
                ];
            },
            |token| {
                token.is_burned = true;
                token.owner = Some(alice());
            },
        );

        // withdraw deposit
        assert!(
            Deferred::withdraw_contract_deposit(contract_id.into(), None)
                .await
                .is_err()
        );
    }

    #[test]
    fn test_should_update_contract_buyers() {
        init_canister();
        store_mock_contract_with(
            &[1, 2],
            1,
            |contract| {
                contract.buyers = vec![caller(), Principal::management_canister()];
            },
            |_| {},
        );
        assert!(Deferred::update_contract_buyers(
            1_u64.into(),
            vec![Principal::management_canister(), caller()]
        )
        .is_ok());
        assert_eq!(
            Deferred::get_contract(&1_u64.into()).unwrap().buyers,
            vec![Principal::management_canister(), caller()]
        );
    }

    #[test]
    fn test_should_set_role() {
        init_canister();
        let principal = Principal::management_canister();
        let role = Role::Custodian;
        Deferred::admin_set_role(principal, role);
        assert!(RolesManager::is_custodian(principal));
    }

    #[test]
    fn test_should_remove_role() {
        init_canister();
        let principal = Principal::management_canister();
        let role = Role::Custodian;
        Deferred::admin_set_role(principal, role);
        assert!(RolesManager::is_custodian(principal));
        Deferred::admin_remove_role(principal, role).unwrap();
        assert!(!RolesManager::is_custodian(principal));
    }

    #[test]
    fn test_should_get_metadata() {
        init_canister();
        let metadata = Deferred::dip721_metadata();
        assert_eq!(metadata.custodians, vec![caller()]);
        assert_eq!(metadata.logo.as_deref(), Some(DEFAULT_LOGO));
        assert_eq!(metadata.name.as_deref(), Some(DEFAULT_NAME));
        assert_eq!(metadata.symbol.as_deref(), Some(DEFAULT_SYMBOL));
    }

    #[test]
    fn test_should_get_stats() {
        init_canister();
        let stats = Deferred::dip721_stats();
        assert_eq!(stats.cycles, crate::utils::cycles());
        assert_eq!(stats.total_supply, 0_u64);
        assert_eq!(stats.total_transactions, 0_u64);
        assert_eq!(stats.total_unique_holders, 0_u64);
    }

    #[test]
    fn test_should_set_logo() {
        init_canister();
        let logo = "logo";
        Deferred::dip721_set_logo(logo.to_string());
        assert_eq!(Deferred::dip721_logo().as_deref(), Some(logo));
    }

    #[test]
    fn test_should_set_name() {
        init_canister();
        let name = "name";
        Deferred::dip721_set_name(name.to_string());
        assert_eq!(Deferred::dip721_name().as_deref(), Some(name));
    }

    #[test]
    fn test_should_set_symbol() {
        init_canister();
        let symbol = "symbol";
        Deferred::dip721_set_symbol(symbol.to_string());
        assert_eq!(Deferred::dip721_symbol().as_deref(), Some(symbol));
    }

    #[test]
    fn test_should_set_custodians() {
        init_canister();
        let custodians = vec![caller(), Principal::management_canister()];
        Deferred::dip721_set_custodians(custodians.clone());
        assert_eq!(Deferred::dip721_custodians().len(), custodians.len());
    }

    #[test]
    fn test_should_get_cycles() {
        init_canister();
        assert_eq!(Deferred::dip721_cycles(), crate::utils::cycles());
    }

    #[test]
    fn test_should_get_unique_holders() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        assert_eq!(Deferred::dip721_total_unique_holders(), Nat::from(1_u64));
    }

    #[test]
    fn test_should_get_token_metadata() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        let metadata = Deferred::dip721_token_metadata(1_u64.into()).unwrap();
        assert_eq!(metadata.owner, Some(caller()));
        assert_eq!(metadata.token_identifier, Nat::from(1_u64));

        // unexisting token
        assert!(Deferred::dip721_token_metadata(5_u64.into()).is_err());
    }

    #[test]
    fn test_should_get_balance_of() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        assert_eq!(
            Deferred::dip721_balance_of(caller()).unwrap(),
            Nat::from(2_u64)
        );
        assert!(Deferred::dip721_balance_of(Principal::management_canister()).is_err());
    }

    #[test]
    fn test_should_get_owner_of() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        assert_eq!(
            Deferred::dip721_owner_of(1_u64.into()).unwrap(),
            Some(caller())
        );
        assert!(Deferred::dip721_owner_of(5_u64.into()).is_err());
    }

    #[test]
    fn test_should_get_owner_token_identifiers() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        assert_eq!(
            Deferred::dip721_owner_token_identifiers(caller()).unwrap(),
            vec![Nat::from(1_u64), Nat::from(2_u64)]
        );
        assert!(
            Deferred::dip721_owner_token_identifiers(Principal::management_canister()).is_err()
        );
    }

    #[test]
    fn test_should_get_owner_token_metadata() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        let metadata = Deferred::dip721_owner_token_metadata(caller()).unwrap();
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata[0].owner, Some(caller()));
        assert_eq!(metadata[0].token_identifier, Nat::from(1_u64));
        assert_eq!(metadata[1].owner, Some(caller()));
        assert_eq!(metadata[1].token_identifier, Nat::from(2_u64));

        // unexisting owner
        assert!(Deferred::dip721_owner_token_metadata(Principal::management_canister()).is_err());
    }

    #[test]
    fn test_should_get_operator_of() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        assert_eq!(Deferred::dip721_operator_of(1_u64.into()).unwrap(), None);
        store_mock_contract_with(
            &[3],
            2,
            |_| {},
            |token| token.operator = Some(Principal::management_canister()),
        );

        assert_eq!(
            Deferred::dip721_operator_of(3_u64.into()).unwrap(),
            Some(Principal::management_canister())
        );

        assert!(Deferred::dip721_operator_of(5_u64.into()).is_err());
    }

    #[test]
    fn test_should_get_operator_token_identifiers() {
        init_canister();
        // no owner
        store_mock_contract_with(
            &[1, 2],
            1,
            |_| {},
            |token| {
                token.operator = None;
            },
        );
        assert!(Deferred::dip721_operator_token_identifiers(caller()).is_err());

        // with operator
        store_mock_contract_with(
            &[3, 4],
            2,
            |_| {},
            |token| token.operator = Some(Principal::management_canister()),
        );
        assert_eq!(
            Deferred::dip721_operator_token_identifiers(Principal::management_canister()).unwrap(),
            vec![Nat::from(3_u64), Nat::from(4_u64)]
        );
        assert!(Deferred::dip721_operator_of(5_u64.into()).is_err());
    }

    #[test]
    fn test_should_get_operator_token_metadata() {
        init_canister();
        // no owner
        store_mock_contract_with(
            &[1, 2],
            1,
            |_| {},
            |token| {
                token.operator = None;
            },
        );
        assert!(Deferred::dip721_operator_token_metadata(caller()).is_err());

        // with operator
        store_mock_contract_with(
            &[3, 4],
            2,
            |_| {},
            |token| token.operator = Some(Principal::management_canister()),
        );
        let metadata =
            Deferred::dip721_operator_token_metadata(Principal::management_canister()).unwrap();
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata[0].owner, Some(caller()));
        assert_eq!(metadata[0].token_identifier, Nat::from(3_u64));
        assert_eq!(metadata[1].owner, Some(caller()));
        assert_eq!(metadata[1].token_identifier, Nat::from(4_u64));

        assert!(Deferred::dip721_operator_of(5_u64.into()).is_err());
    }

    #[test]
    fn test_should_get_supported_interfaces() {
        init_canister();
        assert_eq!(
            Deferred::dip721_supported_interfaces(),
            vec![
                SupportedInterface::Burn,
                SupportedInterface::TransactionHistory
            ]
        );
    }

    #[test]
    fn test_should_get_total_supply() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        store_mock_contract(&[3, 4], 2);
        assert_eq!(Deferred::dip721_total_supply(), Nat::from(4_u64));
    }

    #[tokio::test]
    async fn test_should_transfer() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        // self transfer
        assert!(Deferred::dip721_transfer(caller(), 1_u64.into())
            .await
            .is_err());

        // transfer
        assert!(
            Deferred::dip721_transfer(Principal::management_canister(), 1_u64.into())
                .await
                .is_ok()
        );
        assert_eq!(
            Deferred::dip721_balance_of(caller()).unwrap(),
            Nat::from(1_u64)
        );
        assert_eq!(
            Deferred::dip721_balance_of(Principal::management_canister()).unwrap(),
            Nat::from(1_u64)
        );
        // transfer unexisting
        assert!(
            Deferred::dip721_transfer(Principal::management_canister(), 5_u64.into())
                .await
                .is_err()
        );
    }

    #[test]
    fn test_should_burn() {
        init_canister();
        store_mock_contract(&[1, 2], 1);
        assert!(Deferred::dip721_burn(1_u64.into()).is_ok());
        assert_eq!(
            Deferred::dip721_balance_of(caller()).unwrap(),
            Nat::from(1_u64)
        );

        assert!(Deferred::dip721_burn(5_u64.into()).is_err());
    }

    #[test]
    fn test_should_get_tx() {
        assert!(Deferred::dip721_transaction(Nat::from(1_u64)).is_err());
        let id = TxHistory::register_token_mint(&mock_token(1, 1));
        assert!(Deferred::dip721_transaction(id).is_ok());
    }

    #[test]
    fn test_should_get_total_transactions() {
        assert_eq!(Deferred::dip721_total_transactions(), Nat::from(0_u64));
        let _ = TxHistory::register_token_mint(&mock_token(1, 1));
        assert_eq!(Deferred::dip721_total_transactions(), Nat::from(1_u64));
    }

    #[test]
    fn test_should_register_agency() {
        init_canister();
        let wallet = bob();
        let agency = mock_agency();

        Deferred::admin_register_agency(wallet, agency.clone());
        // check if has role
        assert!(RolesManager::is_agent(wallet));
        // check if is in the storage
        assert_eq!(Agents::get_agency_by_wallet(wallet), Some(agency));
    }

    #[tokio::test]
    async fn test_should_set_and_get_restricted_property() {
        init_canister();
        let contract = ContractRegistration {
            buyers: Buyers {
                principals: vec![caller()],
                deposit_account: bob_account(),
            },
            currency: "EUR".to_string(),
            installments: 10,
            properties: vec![],
            restricted_properties: vec![],
            deposit: Deposit {
                value_fiat: 10,
                value_icp: 100,
            },
            r#type: did::deferred::ContractType::Financing,
            sellers: vec![Seller {
                principal: caller(),
                quota: 100,
            }],
            value: 100,
            expiration: Some("2048-01-01".to_string()),
        };

        assert_eq!(Deferred::register_contract(contract).await.unwrap(), 0_u64);
        assert_eq!(Deferred::dip721_total_supply(), Nat::from(0_u64));
        assert_eq!(Deferred::get_unsigned_contracts(), vec![Nat::from(0_u64)]);
        assert!(Deferred::sign_contract(0_u64.into()).await.is_ok());

        assert!(Deferred::update_restricted_contract_property(
            0_u64.into(),
            "contract:secret".to_string(),
            RestrictedProperty {
                value: GenericValue::TextContent("secret".to_string()),
                access_list: vec![RestrictionLevel::Buyer],
            }
        )
        .is_ok());
        // get restricted properties
        let restricted_properties =
            Deferred::get_restricted_contract_properties(0_u64.into()).unwrap();

        assert_eq!(restricted_properties.len(), 1);
    }

    #[test]
    fn test_should_make_subaccount_from_contract_id() {
        let subaccount_a = Deferred::contract_deposit_subaccount(&ID::from(1u64));
        let subaccount_b = Deferred::contract_deposit_subaccount(&ID::from(2u64));
        let subaccount_c = Deferred::contract_deposit_subaccount(&ID::from(u64::MAX));

        assert_ne!(subaccount_a, subaccount_b);
        assert_ne!(subaccount_a, subaccount_c);
    }

    fn init_canister() {
        Deferred::init(DeferredInitData {
            custodians: vec![caller()],
            icp_ledger_canister: caller(),
            ekoke_reward_pool_canister: caller(),
            liquidity_pool_canister: caller(),
            marketplace_canister: caller(),
        });
    }
}
