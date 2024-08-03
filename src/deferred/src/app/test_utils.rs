use candid::Principal;
use did::deferred::{
    Agency, Contract, Deposit, RestrictedProperty, RestrictionLevel, Seller, Token,
};
use did::ID;
use dip721_rs::TokenIdentifier;
use icrc::icrc1::account::Account;

use super::storage::ContractStorage;
use crate::utils::caller;

pub fn mock_token(id: u64, contract_id: u64) -> Token {
    Token {
        id: TokenIdentifier::from(id),
        contract_id: ID::from(contract_id),
        owner: Some(caller()),
        transferred_at: None,
        transferred_by: None,
        approved_at: None,
        approved_by: None,
        ekoke_reward: 4000_u64.into(),
        burned_at: None,
        burned_by: None,
        minted_at: 0,
        value: 100,
        operator: None,
        is_burned: false,
        minted_by: Principal::anonymous(),
    }
}

pub fn mock_contract(id: u64, installments: u64) -> Contract {
    Contract {
        id: id.into(),
        r#type: did::deferred::ContractType::Financing,
        sellers: vec![Seller {
            principal: caller(),
            quota: 100,
        }],
        buyers: vec![Principal::management_canister()],
        tokens: vec![],
        installments,
        is_signed: false,
        initial_value: 250_000,
        value: 250_000,
        deposit: Deposit {
            value_fiat: 50_000,
            value_icp: 100,
        },
        currency: "EUR".to_string(),
        properties: vec![(
            "contract:city".to_string(),
            dip721_rs::GenericValue::TextContent("Rome".to_string()),
        )],
        restricted_properties: vec![(
            "contract:seller_address".to_string(),
            RestrictedProperty {
                access_list: vec![RestrictionLevel::Agent, RestrictionLevel::Seller],
                value: dip721_rs::GenericValue::TextContent("Via Roma 123".to_string()),
            },
        )],
        agency: Some(mock_agency()),
        expiration: None,
    }
}

pub fn mock_agency() -> Agency {
    Agency {
        name: "Dummy Real estate".to_string(),
        address: "Via Delle Botteghe Scure".to_string(),
        city: "Rome".to_string(),
        region: "Lazio".to_string(),
        zip_code: "00100".to_string(),
        country: "Italy".to_string(),
        continent: did::deferred::Continent::Europe,
        email: "email".to_string(),
        website: "website".to_string(),
        mobile: "mobile".to_string(),
        vat: "vat".to_string(),
        agent: "agent".to_string(),
        logo: None,
    }
}

pub fn store_mock_contract(token_ids: &[u64], contract_id: u64) {
    store_mock_contract_with(token_ids, contract_id, |_| {}, |_| {})
}

pub fn store_mock_contract_with<F, F2>(
    token_ids: &[u64],
    contract_id: u64,
    contract_fn: F,
    token_fn: F2,
) where
    F: FnOnce(&mut Contract),
    F2: FnOnce(&mut Token) + Copy,
{
    let mut tokens = Vec::new();
    for id in token_ids {
        let mut token = mock_token(*id, contract_id);
        token_fn(&mut token);
        tokens.push(token);
    }

    let mut contract = mock_contract(contract_id, token_ids.len() as u64);
    contract_fn(&mut contract);

    if let Err(err) = ContractStorage::insert_contract(contract) {
        panic!("{err}");
    }
    if let Err(err) = ContractStorage::sign_contract_and_mint_tokens(&contract_id.into(), tokens) {
        panic!("{err}");
    }
}

pub fn with_mock_token<F>(id: u64, contract_id: u64, f: F) -> Token
where
    F: FnOnce(&mut Token),
{
    let mut token = mock_token(id, contract_id);
    f(&mut token);
    token
}

pub fn with_mock_contract<F>(id: u64, installments: u64, f: F) -> Contract
where
    F: FnOnce(&mut Contract),
{
    let mut contract = mock_contract(id, installments);
    f(&mut contract);
    contract
}

pub fn alice() -> Principal {
    Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap()
}

pub fn bob() -> Principal {
    Principal::from_text("bs5l3-6b3zu-dpqyj-p2x4a-jyg4k-goneb-afof2-y5d62-skt67-3756q-dqe").unwrap()
}

pub fn bob_account() -> Account {
    Account::from(bob())
}

pub fn charlie() -> Principal {
    Principal::from_text("vuwfz-pyaaa-aaaal-ai5da-cai").unwrap()
}

pub fn dylan() -> Principal {
    Principal::from_text("vtxdn-caaaa-aaaal-ai5dq-cai").unwrap()
}
