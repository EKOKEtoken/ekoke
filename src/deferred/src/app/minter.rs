use candid::{Nat, Principal};
use did::deferred::{DeferredResult, Seller, Token};
use did::ID;

use super::configuration::Configuration;
use super::storage::ContractStorage;
use crate::client::{ekoke_reward_pool_client, EkokeRewardPoolClient};
use crate::utils::caller;

pub struct Minter;

impl Minter {
    /// Mint tokens for a contract
    ///
    /// NOTE: Returns vec because hashmap is not ordered
    pub async fn mint(
        contract_id: &ID,
        sellers: Vec<Seller>,
        installments: u64,
        contract_value: u64,
    ) -> DeferredResult<(Vec<Token>, Vec<Nat>)> {
        // get reward for contract
        let ekoke_reward =
            ekoke_reward_pool_client(Configuration::get_ekoke_reward_pool_canister())
                .get_contract_reward(contract_id.clone(), installments)
                .await?;

        // make tokens
        let mut tokens = Vec::with_capacity(installments as usize);
        let mut tokens_ids = Vec::with_capacity(installments as usize);
        let token_value: u64 = contract_value / installments;
        let marketplace_canister = Configuration::get_marketplace_canister();

        for (seller, token_id) in Self::get_sellers_quota(&sellers, installments) {
            tokens.push(Token {
                approved_at: Some(crate::utils::time()),
                approved_by: Some(caller()),
                burned_at: None,
                burned_by: None,
                contract_id: contract_id.clone(),
                id: token_id.into(),
                is_burned: false,
                minted_at: crate::utils::time(),
                minted_by: caller(),
                operator: Some(marketplace_canister), // * the operator must be the marketplace canister
                owner: Some(*seller),
                transferred_at: None,
                transferred_by: None,
                ekoke_reward: ekoke_reward.clone(),
                value: token_value,
            });
            tokens_ids.push(token_id.into());
        }

        Ok((tokens, tokens_ids))
    }

    /// Get the iterator for the association between sellers and installments to (seller, token_id)
    /// in percentage to their quota
    fn get_sellers_quota(sellers: &[Seller], installments: u64) -> Vec<(&'_ Principal, u64)> {
        let mut next_token_id = ContractStorage::total_supply();
        // for each seller, calculate the number of tokens to mint
        let mut tokens_to_mint: Vec<(&Principal, u64)> = Vec::with_capacity(sellers.len());
        for seller in sellers {
            let tokens = (seller.quota as u64 * installments) / 100;
            tokens_to_mint.push((&seller.principal, tokens));
        }
        // generate the association between seller and token_id
        let mut quotas: Vec<(&Principal, u64)> = Vec::with_capacity(installments as usize);
        for (seller, tokens) in tokens_to_mint {
            for token_id in next_token_id..tokens + next_token_id {
                quotas.push((seller, token_id));
            }
            next_token_id += tokens;
        }
        // push leftovers to the one with the highest quota
        let leftovers = installments - quotas.len() as u64;
        if leftovers > 0 {
            let seller_with_higer_quota = quotas
                .iter()
                .max_by(|(_, quota_a), (_, quota_b)| quota_a.cmp(quota_b))
                .unwrap()
                .0;

            for token_id in next_token_id..leftovers + next_token_id {
                quotas.push((seller_with_higer_quota, token_id));
            }
        }

        quotas
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_should_mint_token_with_single_seller() {
        let contract_id = ID::from(1_u64);
        let sellers = vec![Seller {
            principal: caller(),
            quota: 100,
        }];
        let installments = 3;
        let contract_value = 120;

        let result = Minter::mint(&contract_id, sellers, installments, contract_value).await;
        assert!(result.is_ok());
        let (tokens, tokens_ids) = result.unwrap();
        assert_eq!(tokens.len(), installments as usize);
        assert_eq!(tokens_ids.len(), installments as usize);
        assert_eq!(tokens[0].id, 0_u64);
        assert_eq!(tokens[1].id, 1_u64);
        assert_eq!(tokens[2].id, 2_u64);
        assert_eq!(tokens[0].value, 40);
        assert_eq!(tokens[1].value, 40);
        assert_eq!(tokens[2].value, 40);
    }

    #[tokio::test]
    async fn test_should_mint_tokens_with_multiple_owners_and_same_quota() {
        let contract_id = ID::from(1_u64);
        let sellers = vec![
            Seller {
                principal: caller(),
                quota: 50,
            },
            Seller {
                principal: Principal::management_canister(),
                quota: 50,
            },
        ];
        let installments = 10;
        let contract_value = 100;

        let result = Minter::mint(&contract_id, sellers, installments, contract_value).await;
        assert!(result.is_ok());
        let (tokens, _) = result.unwrap();

        let caller_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == caller())
            .collect::<Vec<&Token>>();
        assert_eq!(caller_tokens.len(), installments as usize / 2);
        assert_eq!(caller_tokens[0].id, 0_u64);
        assert_eq!(caller_tokens[1].id, 1_u64);

        let management_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == Principal::management_canister())
            .collect::<Vec<&Token>>();
        assert_eq!(caller_tokens.len(), installments as usize / 2);

        assert_eq!(management_tokens[0].id, 5_u64);
        assert_eq!(management_tokens[1].id, 6_u64);
    }

    #[tokio::test]
    async fn test_should_mint_tokens_with_different_quotas() {
        let contract_id = ID::from(1_u64);
        let sellers = vec![
            Seller {
                principal: caller(),
                quota: 50,
            },
            Seller {
                principal: Principal::management_canister(),
                quota: 30,
            },
            Seller {
                principal: Principal::anonymous(),
                quota: 20,
            },
        ];
        let installments = 10;
        let contract_value = 100;

        let result = Minter::mint(&contract_id, sellers, installments, contract_value).await;
        assert!(result.is_ok());
        let (tokens, _) = result.unwrap();

        let caller_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == caller())
            .collect::<Vec<&Token>>();
        assert_eq!(caller_tokens.len(), 5);
        assert_eq!(caller_tokens[0].id, 0_u64);
        assert_eq!(caller_tokens[1].id, 1_u64);

        let management_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == Principal::management_canister())
            .collect::<Vec<&Token>>();

        assert_eq!(management_tokens.len(), 3);
        assert_eq!(management_tokens[0].id, 5_u64);
        assert_eq!(management_tokens[1].id, 6_u64);

        let anonymous_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == Principal::anonymous())
            .collect::<Vec<&Token>>();

        assert_eq!(anonymous_tokens.len(), 2);
        assert_eq!(anonymous_tokens[0].id, 8_u64);
        assert_eq!(anonymous_tokens[1].id, 9_u64);
    }

    #[tokio::test]
    async fn test_should_mint_tokens_with_leftovers() {
        // theorically this should not happen, but we need to handle this case
        let contract_id = ID::from(1_u64);
        let sellers = vec![
            Seller {
                principal: caller(),
                quota: 33,
            },
            Seller {
                principal: Principal::management_canister(),
                quota: 33,
            },
            Seller {
                principal: Principal::anonymous(),
                quota: 33,
            },
        ];
        let installments = 10;
        let contract_value = 100;

        let result = Minter::mint(&contract_id, sellers, installments, contract_value).await;
        assert!(result.is_ok());
        let (tokens, _) = result.unwrap();
        assert_eq!(tokens.len(), 10);

        let caller_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == caller())
            .collect::<Vec<&Token>>();
        assert_eq!(caller_tokens.len(), 3);
        assert_eq!(caller_tokens[0].id, 0_u64);
        assert_eq!(caller_tokens[1].id, 1_u64);
        assert_eq!(caller_tokens[2].id, 2_u64);

        let management_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == Principal::management_canister())
            .collect::<Vec<&Token>>();
        assert_eq!(management_tokens.len(), 3);

        assert_eq!(management_tokens[0].id, 3_u64);
        assert_eq!(management_tokens[1].id, 4_u64);

        let anonymous_tokens = tokens
            .iter()
            .filter(|token| token.owner.unwrap() == Principal::anonymous())
            .collect::<Vec<&Token>>();
        assert_eq!(anonymous_tokens.len(), 4);

        assert_eq!(anonymous_tokens[0].id, 6_u64);
        assert_eq!(anonymous_tokens[1].id, 7_u64);
        assert_eq!(anonymous_tokens[3].id, 9_u64);
    }
}
