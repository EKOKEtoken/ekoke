#[cfg(target_arch = "wasm32")]
use candid::Nat;
use candid::Principal;
#[cfg(not(target_arch = "wasm32"))]
use did::deferred::{Contract, Deposit, Seller, Token};
use did::deferred::{TokenIdentifier, TokenInfo};
#[cfg(target_arch = "wasm32")]
use did::marketplace::MarketplaceError;
use did::marketplace::MarketplaceResult;
#[cfg(target_arch = "wasm32")]
use dip721_rs::NftError;

#[cfg(not(target_arch = "wasm32"))]
use crate::utils::caller;

#[allow(dead_code)]
pub struct DeferredClient {
    deferred_canister: Principal,
}

impl From<Principal> for DeferredClient {
    fn from(deferred_canister: Principal) -> Self {
        Self { deferred_canister }
    }
}

impl DeferredClient {
    pub async fn get_token(
        &self,
        token_id: &TokenIdentifier,
    ) -> MarketplaceResult<Option<TokenInfo>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(Some(TokenInfo {
                token: Token {
                    id: token_id.clone(),
                    contract_id: 1_u64.into(),
                    owner: match token_id {
                        id if id == &candid::Nat::from(2_u64) => Some(caller()),
                        id if id == &candid::Nat::from(3_u64) => None,
                        _ => Some(Principal::management_canister()),
                    },
                    transferred_at: match token_id {
                        id if id == &candid::Nat::from(2_u64) => Some(0),
                        _ => None,
                    },
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
                },
                contract: Contract {
                    id: 1_u64.into(),
                    r#type: did::deferred::ContractType::Financing,
                    sellers: vec![Seller {
                        principal: caller(),
                        quota: 100,
                    }],
                    buyers: match token_id {
                        id if id == &candid::Nat::from(2_u64) => vec![caller()],
                        id if id == &candid::Nat::from(4_u64) => vec![caller()],
                        _ => vec![Principal::management_canister()],
                    },
                    deposit: Deposit {
                        value_fiat: 80_000,
                        value_icp: 100,
                    },
                    tokens: vec![],
                    installments: 4_000,
                    is_signed: false,
                    initial_value: 400_000,
                    value: 400_000,
                    currency: "EUR".to_string(),
                    properties: vec![],
                    restricted_properties: vec![],
                    agency: None,
                    expiration: None,
                },
            }))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let result: (Option<TokenInfo>,) =
                ic_cdk::api::call::call(self.deferred_canister, "get_token", (token_id,))
                    .await
                    .map_err(|(code, err)| MarketplaceError::CanisterCall(code, err))?;
            Ok(result.0)
        }
    }

    #[allow(unused_variables)]
    pub async fn transfer_from(
        &self,
        from: Principal,
        to: Principal,
        token_id: &TokenIdentifier,
    ) -> MarketplaceResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let result: (Result<Nat, NftError>,) = ic_cdk::api::call::call(
                self.deferred_canister,
                "dip721_transfer_from",
                (from, to, token_id),
            )
            .await
            .map_err(|(code, err)| MarketplaceError::CanisterCall(code, err))?;

            result.0?;
            Ok(())
        }
    }

    #[allow(unused_variables)]
    pub async fn burn(&self, token_id: &TokenIdentifier) -> MarketplaceResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let result: (Result<Nat, NftError>,) =
                ic_cdk::api::call::call(self.deferred_canister, "dip721_burn", (token_id,))
                    .await
                    .map_err(|(code, err)| MarketplaceError::CanisterCall(code, err))?;

            result.0?;
            Ok(())
        }
    }
}
