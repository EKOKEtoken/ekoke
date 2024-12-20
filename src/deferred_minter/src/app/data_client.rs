use candid::Principal;
use did::deferred::{
    Agency, Contract, ContractError, DeferredDataResult, DeferredMinterError, DeferredMinterResult,
    GenericValue, Seller,
};
use did::{H160, ID};

use crate::utils::caller;

pub struct DeferredDataClient {
    principal: Principal,
}

impl From<Principal> for DeferredDataClient {
    fn from(principal: Principal) -> Self {
        Self { principal }
    }
}

impl DeferredDataClient {
    pub async fn get_contract(&self, contract_id: &ID) -> DeferredMinterResult<Contract> {
        if cfg!(test) {
            return Ok(Contract {
                id: contract_id.clone(),
                r#type: did::deferred::ContractType::Financing,
                sellers: vec![Seller {
                    address: H160::from_hex_str("0xE46A267b65Ed8CBAeBA9AdC3171063179b642E7A")
                        .unwrap(),
                    quota: 100,
                }],
                buyers: vec![
                    H160::from_hex_str("0xE46A267b65Ed8CBAeBA9AdC3171063179b642E7A").unwrap(),
                ],
                installments: 100,
                value: 250_000,
                deposit: 50_000,
                currency: "EUR".to_string(),
                properties: vec![(
                    "contract:city".to_string(),
                    GenericValue::TextContent("Rome".to_string()),
                )],
                restricted_properties: vec![],
                documents: vec![],
                agency: Some(Agency {
                    name: "Dummy Real estate".to_string(),
                    lat: None,
                    lng: None,
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
                    owner: caller(),
                }),
                expiration: "2078-01-01".to_string(),
                closed: false,
            });
        }

        let (contract,) = ic_cdk::call::<_, (Option<Contract>,)>(
            self.principal,
            "get_contract",
            (contract_id.clone(),),
        )
        .await
        .map_err(|(code, err)| did::deferred::DeferredMinterError::CanisterCall(code, err))?;

        contract.ok_or(DeferredMinterError::Contract(
            ContractError::ContractNotFound(contract_id.clone()),
        ))
    }

    /// Create contract on data canister
    pub async fn create_contract(&self, contract: Contract) -> DeferredMinterResult<()> {
        if cfg!(test) {
            return Ok(());
        }

        let (result,) = ic_cdk::call::<_, (DeferredDataResult<()>,)>(
            self.principal,
            "minter_create_contract",
            (contract,),
        )
        .await
        .map_err(|(code, err)| did::deferred::DeferredMinterError::CanisterCall(code, err))?;

        result.map_err(DeferredMinterError::DataCanister)
    }

    /// Close contract on data canister
    pub async fn close_contract(&self, contract_id: ID) -> DeferredMinterResult<()> {
        if cfg!(test) {
            return Ok(());
        }

        let (result,) = ic_cdk::call::<_, (DeferredDataResult<()>,)>(
            self.principal,
            "minter_close_contract",
            (contract_id,),
        )
        .await
        .map_err(|(code, err)| did::deferred::DeferredMinterError::CanisterCall(code, err))?;

        result.map_err(DeferredMinterError::DataCanister)
    }
}
