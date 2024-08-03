use candid::{Encode, Nat, Principal};
use did::deferred::{Agency, Contract, ContractRegistration, DeferredResult, TokenInfo};
use did::ID;
use dip721_rs::{GenericValue, NftError, TokenIdentifier, TokenMetadata};
use icrc::icrc1::account::Subaccount;

use crate::actor::{admin, alice};
use crate::TestEnv;

pub struct DeferredClient<'a> {
    pub env: &'a TestEnv,
}

impl<'a> From<&'a TestEnv> for DeferredClient<'a> {
    fn from(env: &'a TestEnv) -> Self {
        Self::new(env)
    }
}

impl<'a> DeferredClient<'a> {
    pub fn new(env: &'a TestEnv) -> Self {
        Self { env }
    }

    pub fn register_contract(
        &self,
        caller: Principal,
        data: ContractRegistration,
    ) -> DeferredResult<ID> {
        let contract_id: DeferredResult<ID> = self
            .env
            .update(
                self.env.deferred_id,
                caller,
                "register_contract",
                Encode!(&data).unwrap(),
            )
            .unwrap();

        contract_id
    }

    pub fn sign_contract(&self, id: Nat) -> DeferredResult<()> {
        let res: DeferredResult<()> = self
            .env
            .update(
                self.env.deferred_id,
                admin(),
                "sign_contract",
                Encode!(&id).unwrap(),
            )
            .unwrap();

        res
    }

    pub fn withdraw_contract_deposit(
        &self,
        caller: Principal,
        contract_id: ID,
        subaccount: Option<Subaccount>,
    ) -> DeferredResult<()> {
        let res: DeferredResult<()> = self
            .env
            .update(
                self.env.deferred_id,
                caller,
                "withdraw_contract_deposit",
                Encode!(&contract_id, &subaccount).unwrap(),
            )
            .unwrap();

        res
    }

    pub fn close_contract(&self, caller: Principal, contract_id: ID) -> DeferredResult<()> {
        let res: DeferredResult<()> = self
            .env
            .update(
                self.env.deferred_id,
                caller,
                "close_contract",
                Encode!(&contract_id).unwrap(),
            )
            .unwrap();

        res
    }

    pub fn update_contract_buyers(
        &self,
        caller: Principal,
        contract_id: ID,
        buyers: Vec<Principal>,
    ) -> DeferredResult<()> {
        let res: DeferredResult<()> = self
            .env
            .update(
                self.env.deferred_id,
                caller,
                "update_contract_buyers",
                Encode!(&contract_id, &buyers).unwrap(),
            )
            .unwrap();

        res
    }

    pub fn increment_contract_value(
        &self,
        caller: Principal,
        id: ID,
        value: u64,
        installments: u64,
    ) -> DeferredResult<()> {
        self.env
            .update(
                self.env.deferred_id,
                caller,
                "increment_contract_value",
                Encode!(&id, &value, &installments).unwrap(),
            )
            .unwrap()
    }

    pub fn update_contract_property(
        &self,
        caller: Principal,
        id: ID,
        key: String,
        property: GenericValue,
    ) -> DeferredResult<()> {
        self.env
            .update(
                self.env.deferred_id,
                caller,
                "update_contract_property",
                Encode!(&id, &key, &property).unwrap(),
            )
            .unwrap()
    }

    pub fn get_signed_contracts(&self) -> Vec<ID> {
        let signed_contract: Vec<ID> = self
            .env
            .query(
                self.env.deferred_id,
                admin(),
                "get_signed_contracts",
                Encode!(&()).unwrap(),
            )
            .unwrap();

        signed_contract
    }

    pub fn get_contract(&self, contract_id: &ID) -> Option<Contract> {
        self.env
            .query(
                self.env.deferred_id,
                admin(),
                "get_contract",
                Encode!(contract_id).unwrap(),
            )
            .unwrap()
    }

    pub fn get_token(&self, token_id: &TokenIdentifier) -> Option<TokenInfo> {
        self.env
            .query(
                self.env.deferred_id,
                admin(),
                "get_token",
                Encode!(token_id).unwrap(),
            )
            .unwrap()
    }

    pub fn total_supply(&self) -> Nat {
        let total_supply: Nat = self
            .env
            .query(
                self.env.deferred_id,
                admin(),
                "dip721_total_supply",
                Encode!(&()).unwrap(),
            )
            .unwrap();

        total_supply
    }

    pub fn token_metadata(&self, token_id: Nat) -> Result<TokenMetadata, NftError> {
        let token: Result<TokenMetadata, NftError> = self
            .env
            .query(
                self.env.deferred_id,
                alice(),
                "dip721_token_metadata",
                Encode!(&token_id).unwrap(),
            )
            .unwrap();

        token
    }

    pub fn get_unsigned_contracts(&self) -> Vec<ID> {
        let unsigned_contracts: Vec<ID> = self
            .env
            .query(
                self.env.deferred_id,
                admin(),
                "get_unsigned_contracts",
                Encode!(&()).unwrap(),
            )
            .unwrap();

        unsigned_contracts
    }

    pub fn set_custodians(&self, principals: Vec<Principal>) {
        self.env
            .update::<()>(
                self.env.deferred_id,
                admin(),
                "dip721_set_custodians",
                Encode!(&principals).unwrap(),
            )
            .unwrap();
    }

    pub fn transfer_from(
        &self,
        caller: Principal,
        from: Principal,
        to: Principal,
        id: ID,
    ) -> Result<ID, NftError> {
        self.env
            .update(
                self.env.deferred_id,
                caller,
                "dip721_transfer_from",
                Encode!(&from, &to, &id).unwrap(),
            )
            .unwrap()
    }

    pub fn admin_register_agency(&self, wallet: Principal, agency: Agency) {
        let _: () = self
            .env
            .update(
                self.env.deferred_id,
                admin(),
                "admin_register_agency",
                Encode!(&wallet, &agency).unwrap(),
            )
            .unwrap();
    }

    pub fn remove_agency(&self, wallet: Principal) -> DeferredResult<()> {
        self.env
            .update(
                self.env.deferred_id,
                wallet,
                "remove_agency",
                Encode!(&wallet).unwrap(),
            )
            .unwrap()
    }
}
