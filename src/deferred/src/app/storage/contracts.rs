use candid::{Nat, Principal};
use did::deferred::{
    Contract, DeferredError, DeferredResult, RestrictedProperty, Seller, Token, TokenError,
};
use did::{StorableNat, ID};
use dip721_rs::{GenericValue, TokenIdentifier, TokenMetadata};
use itertools::Itertools;

use super::{
    with_contract, with_contract_mut, with_contracts, with_contracts_mut, with_token,
    with_token_mut, with_tokens, with_tokens_mut, TxHistory,
};

pub struct ContractStorage;

impl ContractStorage {
    /// Get next contract id
    pub fn next_contract_id() -> Nat {
        with_contracts(|contracts| Nat::from(contracts.len()))
    }

    /// Get contract by id
    pub fn get_contract(id: &ID) -> Option<Contract> {
        with_contract(id, |contract| Ok(contract.clone())).ok()
    }

    /// Insert contract
    pub fn insert_contract(contract: Contract) -> DeferredResult<()> {
        // check contract existance
        if Self::get_contract(&contract.id).is_some() {
            return Err(DeferredError::Token(TokenError::ContractAlreadyExists(
                contract.id,
            )));
        }

        if contract
            .sellers
            .iter()
            .any(|seller| seller.principal == Principal::anonymous())
        {
            return Err(DeferredError::Token(TokenError::ContractHasNoSeller));
        }

        if contract.installments == 0 {
            return Err(DeferredError::Token(TokenError::ContractHasNoTokens));
        }

        if !contract.tokens.is_empty() {
            return Err(DeferredError::Token(
                TokenError::ContractTokensShouldBeEmpty,
            ));
        }

        // check expiration
        if let Some(expiration) = contract.expiration() {
            let expiraton = expiration?;
            if expiraton < crate::utils::date() {
                return Err(DeferredError::Token(TokenError::BadContractExpiration));
            }
        }

        // check contract props
        if contract
            .properties
            .iter()
            .any(|(key, _)| !key.starts_with("contract:"))
        {
            return Err(DeferredError::Token(TokenError::BadContractProperty));
        }

        with_contracts_mut(|contracts| contracts.insert(contract.id.clone().into(), contract));

        Ok(())
    }

    /// Sign contract and mint tokens
    pub fn sign_contract_and_mint_tokens(
        contract_id: &ID,
        tokens: Vec<Token>,
    ) -> DeferredResult<()> {
        if tokens.is_empty() {
            return Err(DeferredError::Token(TokenError::ContractHasNoTokens));
        }

        // insert tokens in contract
        let token_ids = tokens
            .iter()
            .map(|t| t.id.clone())
            .collect::<Vec<TokenIdentifier>>();

        let sellers = with_contract_mut(contract_id, |contract| {
            if contract.is_signed {
                return Err(DeferredError::Token(TokenError::ContractAlreadySigned(
                    contract.id.clone(),
                )));
            }
            // check if token mismatch
            if contract.installments != tokens.len() as u64 {
                return Err(DeferredError::Token(TokenError::TokensMismatch));
            }

            // sign and set tokens
            contract.is_signed = true;
            contract.tokens = token_ids;

            Ok(contract.sellers.clone())
        })?;

        Self::mint_tokens(contract_id, &sellers, tokens)?;

        Ok(())
    }

    /// Add provided tokens to a contract
    pub fn add_tokens_to_contract(contract_id: &ID, tokens: Vec<Token>) -> DeferredResult<()> {
        // check if tokens is empty
        if tokens.is_empty() {
            return Err(DeferredError::Token(TokenError::ContractHasNoTokens));
        }

        with_contract_mut(contract_id, |contract| {
            // if not signed, return error
            if !contract.is_signed {
                return Err(DeferredError::Token(TokenError::ContractNotSigned(
                    contract.id.clone(),
                )));
            }

            let new_value = contract.value + tokens.iter().map(|t| t.value).sum::<u64>();
            let token_ids = tokens
                .iter()
                .map(|t| t.id.clone())
                .collect::<Vec<TokenIdentifier>>();

            Self::mint_tokens(contract_id, &contract.sellers, tokens)?;

            // update contract value and ids
            contract.value = new_value;
            contract.installments += token_ids.len() as u64;
            contract.tokens.extend(token_ids);

            Ok(())
        })?;
        Ok(())
    }

    fn mint_tokens(contract_id: &ID, sellers: &[Seller], tokens: Vec<Token>) -> DeferredResult<()> {
        with_tokens_mut(|tokens_storage| {
            for token in tokens {
                // check if token already exists
                if tokens_storage.contains_key(&token.id.clone().into()) {
                    return Err(DeferredError::Token(TokenError::TokenAlreadyExists(
                        token.id,
                    )));
                }
                // check if token is associated to the contract
                if &token.contract_id != contract_id {
                    return Err(DeferredError::Token(
                        TokenError::TokenDoesNotBelongToContract(token.id),
                    ));
                }
                // check if token owner is one of the seller
                if sellers
                    .iter()
                    .all(|seller| Some(seller.principal) != token.owner)
                {
                    return Err(DeferredError::Token(TokenError::BadMintTokenOwner(
                        token.id,
                    )));
                }

                // register mint
                TxHistory::register_token_mint(&token);

                tokens_storage.insert(token.id.clone().into(), token);
            }

            Ok(())
        })
    }

    /// Get token by id
    pub fn get_token(id: &TokenIdentifier) -> Option<Token> {
        with_token(id, |token| Ok(token.clone())).ok()
    }

    /// get token metadata
    pub fn get_token_metadata(id: &TokenIdentifier) -> Option<TokenMetadata> {
        let token = Self::get_token(id)?;
        let contract = Self::get_contract(&token.contract_id)?;

        let buyers = contract
            .buyers
            .iter()
            .map(|buyer| {
                (
                    "contract:buyer".to_string(),
                    GenericValue::Principal(*buyer),
                )
            })
            .collect::<Vec<(String, GenericValue)>>();

        let sellers = contract
            .sellers
            .iter()
            .map(|seller| {
                (
                    "contract:seller".to_string(),
                    GenericValue::Principal(seller.principal),
                )
            })
            .collect::<Vec<(String, GenericValue)>>();

        let mut properties = vec![
            (
                "token:contract_id".to_string(),
                GenericValue::TextContent(token.contract_id.to_string()),
            ),
            (
                "token:value".to_string(),
                GenericValue::NatContent(token.value.into()),
            ),
            (
                "token:currency".to_string(),
                GenericValue::TextContent(contract.currency),
            ),
            (
                "token:ekoke_reward".to_string(),
                GenericValue::NatContent(token.ekoke_reward),
            ),
            (
                "contract:buyers".to_string(),
                GenericValue::NestedContent(buyers),
            ),
            (
                "contract:sellers".to_string(),
                GenericValue::NestedContent(sellers),
            ),
        ];
        properties.extend(contract.properties);

        Some(TokenMetadata {
            approved_at: token.approved_at,
            approved_by: token.approved_by,
            burned_at: token.burned_at,
            burned_by: token.burned_by,
            is_burned: token.is_burned,
            minted_at: token.minted_at,
            minted_by: token.minted_by,
            operator: token.operator,
            owner: token.owner,
            properties,
            token_identifier: token.id,
            transferred_at: token.transferred_at,
            transferred_by: token.transferred_by,
        })
    }

    /// get contracts
    pub fn get_signed_contracts() -> Vec<ID> {
        with_contracts(|contracts| {
            contracts
                .iter()
                .filter_map(|(key, contract)| {
                    if contract.is_signed {
                        Some(key.0.clone())
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    /// get contracts
    pub fn get_unsigned_contracts<F>(filter: F) -> Vec<ID>
    where
        F: Fn(&Contract) -> bool,
    {
        with_contracts(|contracts| {
            contracts
                .iter()
                .filter_map(|(key, contract)| {
                    if !contract.is_signed && filter(&contract) {
                        Some(key.0.clone())
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    /// Update contract property
    pub fn update_contract_property(
        contract_id: &ID,
        key: String,
        value: GenericValue,
    ) -> DeferredResult<()> {
        with_contract_mut(contract_id, |contract| {
            let mut found = false;
            for (k, v) in &mut contract.properties {
                if k == &key {
                    *v = value.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                contract.properties.push((key, value));
            }
            Ok(())
        })
    }

    /// Update restricted contract property
    pub fn update_restricted_contract_property(
        contract_id: &ID,
        key: String,
        value: RestrictedProperty,
    ) -> DeferredResult<()> {
        with_contract_mut(contract_id, |contract| {
            let mut found = false;
            for (k, v) in &mut contract.restricted_properties {
                if k == &key {
                    *v = value.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                contract.restricted_properties.push((key, value));
            }
            Ok(())
        })
    }

    /// Update the contract  buyers
    pub fn update_contract_buyers(contract_id: &ID, buyers: Vec<Principal>) -> DeferredResult<()> {
        with_contract_mut(contract_id, |contract| {
            contract.buyers = buyers;
            Ok(())
        })
    }

    /// Update the operator for all token to the new operator canister
    pub fn update_tokens_operator(operator: Principal) -> DeferredResult<()> {
        with_tokens_mut(|tokens| {
            let new_tokens = tokens
                .iter()
                .map(|(id, token)| {
                    let mut token = token.clone();
                    token.operator = Some(operator);
                    (id.clone(), token)
                })
                .collect::<Vec<(StorableNat, Token)>>();
            for (id, token) in new_tokens {
                tokens.insert(id, token);
            }

            Ok(())
        })
    }

    /// Burn token
    pub fn burn_token(token_id: &TokenIdentifier) -> DeferredResult<Nat> {
        let (tx_id, token) = with_token_mut(token_id, |token| {
            // check if burned
            if token.is_burned {
                return Err(DeferredError::Token(TokenError::TokenIsBurned(
                    token_id.clone(),
                )));
            }
            token.is_burned = true;
            token.owner = None;
            token.burned_at = Some(crate::utils::time());
            token.burned_by = Some(crate::utils::caller());

            // register burn
            let tx_id = TxHistory::register_token_burn(token);

            Ok((tx_id, token.clone()))
        })?;

        // reduce contract value
        Self::reduce_contract_value_by(&token.contract_id, token.value)?;

        Ok(tx_id)
    }

    /// Reduce contract value by `decr_by`
    fn reduce_contract_value_by(contract_id: &ID, decr_by: u64) -> DeferredResult<()> {
        with_contract_mut(contract_id, |contract| {
            contract.value -= decr_by;

            Ok(())
        })
    }

    /// Transfer token to provided principal
    pub fn transfer(token_id: &TokenIdentifier, to: Principal) -> DeferredResult<Nat> {
        with_token_mut(token_id, |token| {
            // check if burned
            if token.is_burned {
                return Err(DeferredError::Token(TokenError::TokenIsBurned(
                    token_id.clone(),
                )));
            }
            token.owner = Some(to);
            token.transferred_at = Some(crate::utils::time());
            token.transferred_by = Some(crate::utils::caller());

            // register transfer
            let tx_id = TxHistory::register_transfer(token);

            Ok(tx_id)
        })
    }

    /// Returns the total amount of unique holders of tokens
    pub fn total_unique_holders() -> u64 {
        with_tokens(|tokens| {
            tokens
                .iter()
                .filter_map(|(_, token)| token.owner)
                .unique()
                .count()
        }) as u64
    }

    /// Get tokens owned by a certain principal
    pub fn tokens_by_owner(owner: Principal) -> Vec<TokenIdentifier> {
        with_tokens(|tokens| {
            tokens
                .iter()
                .filter_map(|(id, token)| {
                    if token.owner == Some(owner) && !token.is_burned {
                        Some(id.0.clone())
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    /// Get tokens with operator set to a certain principal
    pub fn tokens_by_operator(operator: Principal) -> Vec<TokenIdentifier> {
        with_tokens(|tokens| {
            tokens
                .iter()
                .filter_map(|(id, token)| {
                    if token.operator == Some(operator) {
                        Some(id.0.clone())
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    /// Returns the total supply of tokens
    pub fn total_supply() -> u64 {
        with_tokens(|tokens| tokens.len())
    }

    #[cfg(test)]
    pub fn mut_token<F>(token_id: &TokenIdentifier, f: F) -> DeferredResult<()>
    where
        F: Fn(&mut Token) -> DeferredResult<()>,
    {
        with_token_mut(token_id, f)
    }

    #[cfg(test)]
    pub fn mut_contract<F>(contract_id: &ID, f: F) -> DeferredResult<()>
    where
        F: Fn(&mut Contract) -> DeferredResult<()>,
    {
        with_contract_mut(contract_id, f)
    }
}

#[cfg(test)]
mod test {

    use candid::Principal;
    use did::deferred::{Deposit, RestrictionLevel, Seller};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::app::test_utils::{mock_contract, mock_token, with_mock_contract, with_mock_token};
    use crate::utils::caller;

    #[test]
    fn test_should_get_next_contract_id() {
        let next_contract_id = ContractStorage::next_contract_id();
        assert_eq!(next_contract_id, Nat::from(0_u64));
    }

    #[test]
    fn test_should_insert_and_get_contract() {
        let seller = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let contract_id = ID::from(1_u64);
        let next_token_id = ContractStorage::total_supply();
        assert_eq!(next_token_id, Nat::from(0_u64));
        let token_1 = with_mock_token(1, 1, |token| {
            token.operator = Some(caller());
            token.owner = Some(caller());
        });
        let token_2 = with_mock_token(2, 1, |token| {
            token.owner = Some(caller());
        });

        let contract = with_mock_contract(1, 2, |contract| {
            contract.sellers = seller;
        });

        assert!(ContractStorage::get_contract(&contract.id).is_none());
        assert!(ContractStorage::insert_contract(contract.clone(),).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1.clone(), token_2.clone()]
        )
        .is_ok());
        assert!(ContractStorage::get_contract(&contract.id).is_some());
        assert!(ContractStorage::get_token(&token_1.id).is_some());
        assert!(ContractStorage::get_token(&token_2.id).is_some());
        assert_eq!(ContractStorage::total_supply(), 2);
        assert_eq!(ContractStorage::tokens_by_owner(caller()).len(), 2);
        assert_eq!(ContractStorage::tokens_by_operator(caller()).len(), 1);
        assert_eq!(ContractStorage::get_signed_contracts(), vec![contract.id]);
    }

    #[test]
    fn test_should_insert_and_get_contract_with_no_buyers() {
        let seller = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let contract_id = ID::from(1_u64);
        let next_token_id = ContractStorage::total_supply();
        assert_eq!(next_token_id, Nat::from(0_u64));
        let token_1 = with_mock_token(1, 1, |token| {
            token.operator = Some(caller());
            token.owner = Some(caller());
        });
        let token_2 = with_mock_token(2, 1, |token| {
            token.owner = Some(caller());
        });

        let contract = with_mock_contract(1, 2, |contract| {
            contract.sellers = seller;
            contract.buyers = vec![];
        });

        assert!(ContractStorage::get_contract(&contract.id).is_none());
        assert!(ContractStorage::insert_contract(contract.clone(),).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1.clone(), token_2.clone()]
        )
        .is_ok());
        assert!(ContractStorage::get_contract(&contract.id).is_some());
        assert!(ContractStorage::get_token(&token_1.id).is_some());
        assert!(ContractStorage::get_token(&token_2.id).is_some());
        assert_eq!(ContractStorage::total_supply(), 2);
        assert_eq!(ContractStorage::tokens_by_owner(caller()).len(), 2);
        assert_eq!(ContractStorage::tokens_by_operator(caller()).len(), 1);
        assert_eq!(ContractStorage::get_signed_contracts(), vec![contract.id]);
    }

    #[test]
    fn test_should_not_allow_duped_contract() {
        let contract = with_mock_contract(1, 2, |contract| {
            contract.sellers = vec![Seller {
                principal: Principal::anonymous(),
                quota: 100,
            }];
            contract.buyers = vec![];
        });

        assert!(ContractStorage::insert_contract(contract).is_err());
    }

    #[test]
    fn test_should_not_allow_contract_with_anonymous_seller() {
        let contract = mock_contract(1, 2);

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());
        assert!(ContractStorage::insert_contract(contract).is_err());
    }

    #[test]
    fn test_should_not_allow_empty_contract() {
        let contract = mock_contract(1, 0);

        assert!(ContractStorage::insert_contract(contract.clone()).is_err());
    }

    #[test]
    fn test_should_not_allow_contract_with_bad_property_name() {
        let contract = with_mock_contract(1, 40, |contract| {
            contract.properties.push((
                "contraaa".to_string(),
                GenericValue::TextContent("Rome".to_string()),
            ));
        });

        assert!(ContractStorage::insert_contract(contract.clone()).is_err());
    }

    #[test]
    fn test_should_not_allow_contract_with_expiration_in_the_past() {
        let contract = with_mock_contract(1, 40, |contract| {
            contract.expiration = Some("2021-01-01".to_string());
        });

        assert!(ContractStorage::insert_contract(contract.clone()).is_err());
    }

    #[test]
    fn test_should_not_allow_duped_token() {
        let contract_id = ID::from(1_u64);
        let token_1 = mock_token(1, 1);
        let token_2 = mock_token(1, 1);
        let contract = mock_contract(1, 2);

        assert!(ContractStorage::insert_contract(contract).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1, token_2]
        )
        .is_err());
    }

    #[test]
    fn test_should_not_allow_token_with_different_contract_id() {
        let token_1 = mock_token(1, 1);
        let token_2 = mock_token(1, 2);

        let contract = mock_contract(1, 2);

        assert!(ContractStorage::insert_contract(contract).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &1_u64.into(),
            vec![token_1, token_2]
        )
        .is_err());
    }

    #[test]
    fn test_should_not_allow_token_owner_different_from_seller() {
        let seller = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let contract_id = ID::from(1_u64);
        let token_1 = mock_token(1, 1);
        let token_2 = with_mock_token(2, 1, |token| {
            token.owner = Some(Principal::anonymous());
        });

        let contract = with_mock_contract(1, 2, |contract| {
            contract.sellers = seller;
        });

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1, token_2]
        )
        .is_err());
    }

    #[test]
    fn test_should_not_allow_mismatching_tokens() {
        let contract_id = ID::from(1_u64);
        let token_1 = mock_token(1, 1);
        let token_2 = mock_token(2, 1);
        let token_3 = mock_token(3, 1);

        let contract = mock_contract(1, 2);

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());

        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1, token_2, token_3]
        )
        .is_err());
    }

    #[test]
    fn test_should_burn_token() {
        let owner = Principal::management_canister();
        let contract_id = ID::from(1_u64);
        let token_1 = with_mock_token(1, 1, |token| {
            token.owner = Some(owner);
            token.value = 1_000;
        });

        let contract = with_mock_contract(1, 1, |contract| {
            contract.sellers = vec![Seller {
                principal: owner,
                quota: 100,
            }];
            contract.value = 250_000;
            contract.initial_value = 250_000;
        });

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1.clone()]
        )
        .is_ok());
        assert!(ContractStorage::burn_token(&token_1.id).is_ok());
        // owner balance is zero
        assert_eq!(ContractStorage::tokens_by_owner(owner).len(), 0);

        assert_eq!(
            ContractStorage::get_token(&token_1.id).unwrap().is_burned,
            true
        );
        assert!(ContractStorage::get_token(&token_1.id)
            .unwrap()
            .burned_at
            .is_some());
        assert!(ContractStorage::get_token(&token_1.id)
            .unwrap()
            .burned_by
            .is_some());
        assert!(ContractStorage::get_token(&token_1.id)
            .unwrap()
            .owner
            .is_none());
        // verify contract value has been decreased
        assert_eq!(
            ContractStorage::get_contract(&contract_id).unwrap().value,
            249_000
        );
        assert_eq!(
            ContractStorage::get_contract(&contract_id)
                .unwrap()
                .initial_value,
            250_000
        );
    }

    #[test]
    fn test_should_transfer_token() {
        let contract_id = ID::from(1_u64);
        let token_1 = with_mock_token(1, 1, |token| {
            token.owner = Some(Principal::management_canister());
        });

        let contract = with_mock_contract(1, 1, |contract| {
            contract.sellers = vec![Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }];
        });

        let new_owner = caller();

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1.clone()]
        )
        .is_ok());
        assert!(ContractStorage::transfer(&token_1.id, new_owner).is_ok());
        assert_eq!(
            ContractStorage::get_token(&token_1.id).unwrap().owner,
            Some(new_owner)
        );
        assert!(ContractStorage::get_token(&token_1.id)
            .unwrap()
            .transferred_at
            .is_some());
        assert!(ContractStorage::get_token(&token_1.id)
            .unwrap()
            .transferred_by
            .is_some());
    }

    #[test]
    fn test_should_return_unique_holders() {
        let seller = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let contract_id = ID::from(1_u64);
        let token_1 = mock_token(1, 1);
        let token_2 = mock_token(2, 1);
        let contract = with_mock_contract(1, 2, |contract| {
            contract.sellers = seller;
        });

        assert!(ContractStorage::insert_contract(contract.clone(),).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1, token_2]
        )
        .is_ok());
        assert_eq!(ContractStorage::total_unique_holders(), 1);
    }

    #[test]
    fn test_should_update_token_operator() {
        let sellers = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let contract_id = ID::from(1_u64);
        let token_1 = mock_token(1, 1);
        let token_2 = mock_token(2, 1);
        let contract = Contract {
            id: contract_id.clone(),
            r#type: did::deferred::ContractType::Financing,
            sellers,
            buyers: vec![Principal::anonymous()],
            tokens: vec![],
            installments: 2,
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
            restricted_properties: vec![],
            agency: None,
            expiration: None,
        };

        assert!(ContractStorage::insert_contract(contract.clone(),).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(
            &contract_id,
            vec![token_1.clone(), token_2]
        )
        .is_ok());
        assert!(ContractStorage::update_tokens_operator(Principal::anonymous()).is_ok());
        assert_eq!(
            ContractStorage::get_token(&token_1.id).unwrap().operator,
            Some(Principal::anonymous())
        );
    }

    #[test]
    fn test_should_update_contract_buyers() {
        let seller =
            Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
                .unwrap();
        let contract_id = ID::from(1_u64);
        let next_token_id = ContractStorage::total_supply();
        assert_eq!(next_token_id, Nat::from(0_u64));
        let token_1 = mock_token(next_token_id, 1);
        let contract = with_mock_contract(1, 1, |contract| {
            contract.sellers = vec![Seller {
                principal: seller,
                quota: 51,
            }];
        });

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());
        assert!(
            ContractStorage::sign_contract_and_mint_tokens(&contract_id, vec![token_1]).is_ok()
        );
        let buyer = seller;
        assert!(ContractStorage::update_contract_buyers(
            &contract_id,
            vec![Principal::anonymous(), buyer]
        )
        .is_ok());
        assert_eq!(
            ContractStorage::get_contract(&contract_id).unwrap().buyers,
            vec![Principal::anonymous(), buyer]
        );
    }

    #[test]
    fn test_should_increment_tokens() {
        let seller = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let contract_id = ID::from(1_u64);
        let next_token_id = ContractStorage::total_supply();
        assert_eq!(next_token_id, Nat::from(0_u64));
        let token_1 = mock_token(next_token_id, 1);
        let contract = with_mock_contract(1, 1, |contract| {
            contract.sellers = seller;
            contract.value = 100;
        });

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());
        assert!(
            ContractStorage::sign_contract_and_mint_tokens(&contract_id, vec![token_1]).is_ok()
        );
        assert_eq!(ContractStorage::total_supply(), 1);
        assert_eq!(ContractStorage::tokens_by_owner(caller()).len(), 1);

        // create new tokens
        let token_2 = mock_token(next_token_id + 1, 1);
        assert!(ContractStorage::add_tokens_to_contract(&contract.id, vec![token_2]).is_ok());
        assert_eq!(ContractStorage::total_supply(), 2);
        assert_eq!(ContractStorage::tokens_by_owner(caller()).len(), 2);
        assert_eq!(
            ContractStorage::get_contract(&contract_id).unwrap().value,
            200
        );
    }

    #[test]
    fn test_should_not_increment_tokens_if_unsigned() {
        let seller = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let next_token_id = ContractStorage::total_supply();
        assert_eq!(next_token_id, Nat::from(0_u64));
        let contract = with_mock_contract(1, 2, |contract| {
            contract.sellers = seller;
        });

        assert!(ContractStorage::insert_contract(contract.clone()).is_ok());

        // create new tokens
        let token_2 = mock_token(next_token_id + 1, 1);
        assert!(ContractStorage::add_tokens_to_contract(&contract.id, vec![token_2]).is_err());
    }

    #[test]
    fn test_should_get_unsigned_contracts() {
        let token = mock_token(1, 1);
        let contract_1 = mock_contract(1, 1);
        assert!(ContractStorage::insert_contract(contract_1).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(&1_u64.into(), vec![token]).is_ok());

        let contract_2 = mock_contract(2, 1);
        assert!(ContractStorage::insert_contract(contract_2).is_ok());

        assert_eq!(ContractStorage::get_signed_contracts().len(), 1);
        assert_eq!(ContractStorage::get_unsigned_contracts(|_| true).len(), 1);

        assert_eq!(
            ContractStorage::get_unsigned_contracts(|contract| contract.id == 1_u64).len(),
            0
        );
    }

    #[test]
    fn test_should_get_token_metadata() {
        let token = mock_token(1, 1);
        let contract = with_mock_contract(1, 1, |contract| {
            contract.properties.push((
                "contract:address".to_string(),
                dip721_rs::GenericValue::TextContent("Rome".to_string()),
            ));
            contract.properties.push((
                "contract:country".to_string(),
                dip721_rs::GenericValue::TextContent("Italy".to_string()),
            ));
        });
        assert!(ContractStorage::insert_contract(contract).is_ok());
        assert!(ContractStorage::sign_contract_and_mint_tokens(&1_u64.into(), vec![token]).is_ok());

        let metadata = ContractStorage::get_token_metadata(&1_u64.into()).unwrap();
        assert_eq!(metadata.token_identifier, 1_u64);
        assert_eq!(metadata.approved_at, None);
        assert_eq!(metadata.approved_by, None);
        assert_eq!(metadata.burned_at, None);
        assert_eq!(metadata.burned_by, None);
        assert_eq!(metadata.is_burned, false);
        assert_eq!(metadata.operator, None);
        assert_eq!(metadata.properties.len(), 9);
        assert_eq!(metadata.transferred_at, None);
        assert_eq!(metadata.transferred_by, None);
    }

    #[test]
    fn test_should_update_contract_property() {
        let contract = with_mock_contract(1, 1, |contract| {
            contract.properties.push((
                "contract:address".to_string(),
                dip721_rs::GenericValue::TextContent("Rome".to_string()),
            ));
            contract.properties.push((
                "contract:country".to_string(),
                dip721_rs::GenericValue::TextContent("Italy".to_string()),
            ));
        });
        assert!(ContractStorage::insert_contract(contract).is_ok());

        assert!(ContractStorage::update_contract_property(
            &1_u64.into(),
            "contract:address".to_string(),
            dip721_rs::GenericValue::TextContent("Milan".to_string())
        )
        .is_ok());
        assert_eq!(
            ContractStorage::get_contract(&1_u64.into())
                .unwrap()
                .properties
                .iter()
                .find(|(k, _)| k == "contract:address")
                .unwrap()
                .1,
            GenericValue::TextContent("Milan".to_string())
        );

        assert!(ContractStorage::update_contract_property(
            &1_u64.into(),
            "contract:addressLong".to_string(),
            dip721_rs::GenericValue::TextContent("Trieste".to_string())
        )
        .is_ok());
        assert_eq!(
            ContractStorage::get_contract(&1_u64.into())
                .unwrap()
                .properties
                .iter()
                .find(|(k, _)| k == "contract:addressLong")
                .unwrap()
                .1,
            GenericValue::TextContent("Trieste".to_string())
        );
    }

    #[test]
    fn test_should_update_restricted_contract_property() {
        let contract = with_mock_contract(1, 1, |contract| {
            contract.restricted_properties.push((
                "contract:address".to_string(),
                RestrictedProperty {
                    access_list: vec![RestrictionLevel::Seller],
                    value: dip721_rs::GenericValue::TextContent("Rome".to_string()),
                },
            ));
        });
        assert!(ContractStorage::insert_contract(contract).is_ok());

        assert!(ContractStorage::update_restricted_contract_property(
            &1_u64.into(),
            "contract:address".to_string(),
            RestrictedProperty {
                access_list: vec![RestrictionLevel::Agent, RestrictionLevel::Seller],
                value: dip721_rs::GenericValue::TextContent("Milan".to_string()),
            },
        )
        .is_ok());
        assert_eq!(
            ContractStorage::get_contract(&1_u64.into())
                .unwrap()
                .restricted_properties
                .iter()
                .find(|(k, _)| k == "contract:address")
                .unwrap()
                .1,
            RestrictedProperty {
                access_list: vec![RestrictionLevel::Agent, RestrictionLevel::Seller],
                value: GenericValue::TextContent("Milan".to_string())
            }
        );

        assert!(ContractStorage::update_restricted_contract_property(
            &1_u64.into(),
            "contract:addressLong".to_string(),
            RestrictedProperty {
                access_list: vec![RestrictionLevel::Agent, RestrictionLevel::Seller],
                value: GenericValue::TextContent("Milan".to_string())
            }
        )
        .is_ok());
        assert_eq!(
            ContractStorage::get_contract(&1_u64.into())
                .unwrap()
                .restricted_properties
                .iter()
                .find(|(k, _)| k == "contract:addressLong")
                .unwrap()
                .1,
            RestrictedProperty {
                access_list: vec![RestrictionLevel::Agent, RestrictionLevel::Seller],
                value: GenericValue::TextContent("Milan".to_string())
            }
        );
    }
}
