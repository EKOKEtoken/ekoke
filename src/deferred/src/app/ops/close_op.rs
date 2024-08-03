use std::collections::HashMap;

use candid::{Nat, Principal};
use did::deferred::{
    CloseContractError, Contract, DeferredError, DeferredResult, Deposit, Token, WithdrawError,
};
use did::ID;
use icrc::icrc1::account::{Account, Subaccount};
use icrc::IcrcLedgerClient;

use crate::app::configuration::Configuration;
use crate::app::storage::ContractStorage;
use crate::app::Deferred;
use crate::client::{ekoke_liquidity_pool_client, EkokeLiquidityPoolClient};
use crate::utils;

pub struct CloseOp;

impl CloseOp {
    /// Call for the contract seller to withdraw the buyer deposit in case the contract has been completely paid
    pub async fn withdraw_contract_deposit(
        contract_id: ID,
        withdraw_subaccount: Option<Subaccount>,
    ) -> DeferredResult<()> {
        // check if the contract has been paid
        // get contract
        let contract = ContractStorage::get_contract(&contract_id).ok_or(
            DeferredError::Withdraw(WithdrawError::ContractNotFound(contract_id.clone())),
        )?;

        // check if all the tokens are burned (bought by the contract buyer)
        if contract.tokens.iter().any(|token_id| {
            ContractStorage::get_token(token_id)
                .map(|token| !token.is_burned)
                .unwrap_or_default()
        }) {
            return Err(DeferredError::Withdraw(WithdrawError::ContractNotPaid(
                contract_id,
            )));
        }

        // transfer the deposit to the seller
        let icp_ledger_client = IcrcLedgerClient::new(Configuration::get_icp_ledger_canister());
        // get fee
        let icp_fee = icp_ledger_client
            .icrc1_fee()
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?;

        // get seller quota
        let seller_quota = contract
            .sellers
            .iter()
            .find(|seller| seller.principal == utils::caller())
            .map(|seller| seller.quota)
            .unwrap(); // unwrap is safe because the caller is the seller

        let transfer_amount = (contract.deposit.value_icp.checked_mul(seller_quota as u64))
            .and_then(|value| value.checked_div(100))
            .map(|value| value - icp_fee)
            .ok_or(DeferredError::Withdraw(
                WithdrawError::InvalidTransferAmount(contract.deposit.value_icp, seller_quota),
            ))?;

        let contract_subaccount = Deferred::canister_deposit_account(&contract_id).subaccount;

        icp_ledger_client
            .icrc1_transfer(
                Account {
                    owner: utils::caller(),
                    subaccount: withdraw_subaccount,
                },
                transfer_amount,
                contract_subaccount,
            )
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?
            .map_err(|err| DeferredError::Withdraw(WithdrawError::DepositTransferFailed(err)))?;

        Ok(())
    }

    /// Close a contract which hasn't been completely paid and is expired.
    ///
    /// Only the agency can call this method.
    ///
    /// This method will burn all the tokens and will proportionally refund the NFTs owners, except the contract buyer.
    pub async fn close_contract(contract: Contract) -> DeferredResult<()> {
        let contract_id = contract.id.clone();
        // get all tokens not burned
        let tokens = contract
            .tokens
            .iter()
            .filter_map(ContractStorage::get_token)
            .filter(|token| !token.is_burned && token.owner.is_some())
            .collect::<Vec<_>>();
        // check if the contract has been paid (paid if all the tokens are burned)
        if tokens.is_empty() {
            return Err(DeferredError::CloseContract(
                CloseContractError::ContractPaid(contract_id),
            ));
        }

        // check if the contract is expired
        if let Some(expiration) = contract.expiration() {
            let expiration = expiration?;
            if utils::date() < expiration {
                return Err(DeferredError::CloseContract(
                    CloseContractError::ContractNotExpired(contract_id),
                ));
            }
        }

        // collect all the tokens owners by owned value
        let value_by_owners = Self::third_parties_tokens_value(&tokens, &contract);
        // get refund amounts
        let refund_amounts = Self::refund_by_owners(value_by_owners, &contract.deposit);
        // calc total refund amount
        let total_refund_amount = refund_amounts.values().sum::<u64>();

        // get ICP ledger client
        let icp_ledger_client = IcrcLedgerClient::new(Configuration::get_icp_ledger_canister());
        // get icp fee
        let icp_fee = icp_ledger_client
            .icrc1_fee()
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?;
        // get balance of the liquidity pool canister
        let liquidity_pool_canister = Configuration::get_liquidity_pool_canister();
        let liquidity_pool_balance = icp_ledger_client
            .icrc1_balance_of(Account::from(liquidity_pool_canister))
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?;

        // required balance is the total refund amount plus the icp fee for each refund
        let required_balance = Self::liquidity_pool_required_balance(
            total_refund_amount.into(),
            refund_amounts.len(),
            icp_fee.clone(),
            &contract.deposit,
        );
        if liquidity_pool_balance < required_balance {
            return Err(DeferredError::CloseContract(
                CloseContractError::LiquidityPoolHasNotEnoughIcp {
                    required: required_balance,
                    available: liquidity_pool_balance,
                },
            ));
        }

        // get deposit amount
        let contract_account = Deferred::canister_deposit_account(&contract_id);
        let deposited_amount = icp_ledger_client
            .icrc1_balance_of(contract_account)
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?;
        // send deposit to liquidity pool
        let amount_to_liquidity_pool = deposited_amount - icp_fee;

        icp_ledger_client
            .icrc1_transfer(
                Account::from(liquidity_pool_canister),
                amount_to_liquidity_pool,
                contract_account.subaccount,
            )
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?
            .map_err(|err| {
                DeferredError::CloseContract(CloseContractError::DepositTransferFailed(err))
            })?;

        // call the liquidity pool canister to refund the third parties
        let refund_amounts = refund_amounts
            .into_iter()
            .map(|(principal, amount)| (principal, amount.into()))
            .collect::<HashMap<_, _>>();
        let liquidity_pool_client = ekoke_liquidity_pool_client(liquidity_pool_canister);
        liquidity_pool_client
            .refund_investors(refund_amounts)
            .await?;

        // burn all the tokens
        for token in tokens {
            let _ = ContractStorage::burn_token(&token.id);
        }

        Ok(())
    }

    /// Get the value of the tokens owned by third parties
    fn third_parties_tokens_value(
        tokens: &[Token],
        contract: &Contract,
    ) -> HashMap<Principal, u64> {
        let mut value_by_owners =
            tokens
                .iter()
                .fold(HashMap::<Principal, u64>::new(), |mut acc, token| {
                    *acc.entry(token.owner.unwrap()).or_default() += token.value;
                    acc
                });
        // remove sellers
        for seller in contract.sellers.iter() {
            value_by_owners.remove(&seller.principal);
        }

        value_by_owners
    }

    /// Refund the third parties completely multiplying the value of the tokens by the ICP unit value
    fn refund_by_owners(
        value_by_owners: HashMap<Principal, u64>,
        deposit: &Deposit,
    ) -> HashMap<Principal, u64> {
        value_by_owners.into_iter().fold(
            HashMap::<Principal, u64>::new(),
            |mut acc, (owner, value)| {
                acc.insert(owner, Self::fiat_to_icp(deposit, value));
                acc
            },
        )
    }

    /// Convert the fiat value to ICP value
    fn fiat_to_icp(rate: &Deposit, value: u64) -> u64 {
        // we need to convert the fiat value to ICP value. But ICP value is in e8s
        // so we need to divide the ICP value by 10^8
        let amount = value as f64;
        // get the rate which is in e8s
        let rate = (rate.value_icp / rate.value_fiat / 100) as f64;
        const DECIMALS: f64 = 8.0;

        ((amount * 10_f64.powf(DECIMALS) / rate) * 10_f64.powf(DECIMALS)).round() as u64
    }

    /// Calculate the required balance for the liquidity pool canister to refund the third parties\
    /// This is the required balance BEFORE the deposit is sent to the liquidity pool
    fn liquidity_pool_required_balance(
        total_refund_amount: Nat,
        refund_transfers: usize,
        icp_fee: Nat,
        deposit: &Deposit,
    ) -> Nat {
        let a = total_refund_amount + (icp_fee.clone() * refund_transfers);
        let b = deposit.value_icp + icp_fee;

        if a > b {
            a - b
        } else {
            0u64.into()
        }
    }
}

#[cfg(test)]
mod test {

    use did::deferred::Seller;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::app::test_utils::{alice, bob, charlie, dylan, store_mock_contract_with};

    #[test]
    fn test_should_convert_fiat_to_icp() {
        let deposit = Deposit {
            value_icp: 100 * 100_000_000,
            value_fiat: 10,
        };

        let fiat_amount = 5;
        let icp_value = CloseOp::fiat_to_icp(&deposit, fiat_amount);
        assert_eq!(icp_value, 50 * 10u64.pow(8));
    }

    #[test]
    fn test_should_calc_liquidity_pool_required_balance() {
        let total_refund_amount = 100u64.into();
        let refund_transfers = 2;
        let icp_fee = 10u64.into();
        let deposit = Deposit {
            value_icp: 100,
            value_fiat: 10,
        };

        let required_balance = CloseOp::liquidity_pool_required_balance(
            total_refund_amount,
            refund_transfers,
            icp_fee,
            &deposit,
        );

        assert_eq!(required_balance, 10u64);
    }

    #[test]
    fn test_should_get_third_parties_value() {
        store_mock_contract_with(
            &[1, 2, 3, 4, 5],
            1,
            |contract| {
                contract.buyers = vec![alice()];
                contract.sellers = vec![Seller {
                    principal: bob(),
                    quota: 100,
                }];
                contract.expiration = Some("2050-01-01".to_string());
            },
            |token| {
                token.owner = Some(bob());
                token.value = 10;
            },
        );
        // set expiration to 1970-01-01
        ContractStorage::mut_contract(&1u64.into(), |contract| {
            contract.expiration = Some("1970-01-01".to_string());
            Ok(())
        })
        .expect("failed to update contract");

        // transfer token 1 and 2 to charlie
        ContractStorage::mut_token(&1u64.into(), |token| {
            token.owner = Some(charlie());
            Ok(())
        })
        .expect("failed to update contract");
        ContractStorage::mut_token(&2u64.into(), |token| {
            token.owner = Some(charlie());
            Ok(())
        })
        .expect("failed to update contract");
        // transfer token 3 to dylan
        ContractStorage::mut_token(&3u64.into(), |token| {
            token.owner = Some(dylan());
            Ok(())
        })
        .expect("failed to update contract");
        // transfer token 4 to the buyer
        ContractStorage::mut_token(&4u64.into(), |token| {
            token.owner = Some(alice());
            token.is_burned = true;
            Ok(())
        })
        .expect("failed to update contract");

        let contract = ContractStorage::get_contract(&1u64.into()).expect("contract not found");
        let mut tokens = vec![];
        for token_id in &contract.tokens {
            let token = ContractStorage::get_token(token_id).expect("token not found");
            if token.is_burned {
                continue;
            }
            tokens.push(token);
        }

        let third_parties_value = CloseOp::third_parties_tokens_value(&tokens, &contract);
        println!("{:?}", third_parties_value);
        assert_eq!(third_parties_value.len(), 2);
        assert_eq!(third_parties_value.get(&charlie()).unwrap(), &20);
        assert_eq!(third_parties_value.get(&dylan()).unwrap(), &10);
    }

    #[test]
    fn test_should_get_refund_values() {
        let mut value_by_owners = HashMap::new();
        value_by_owners.insert(charlie(), 20);
        value_by_owners.insert(dylan(), 10);

        let deposit = Deposit {
            value_icp: 100 * 100_000_000,
            value_fiat: 10,
        };

        let refund_amounts = CloseOp::refund_by_owners(value_by_owners, &deposit);
        assert_eq!(refund_amounts.len(), 2);
        assert_eq!(
            refund_amounts.get(&charlie()).unwrap(),
            &(200 * 10u64.pow(8))
        );
        assert_eq!(refund_amounts.get(&dylan()).unwrap(), &(100 * 10u64.pow(8)));
    }

    #[tokio::test]
    async fn test_should_close_contract() {
        store_mock_contract_with(
            &[1, 2, 3, 4, 5],
            1,
            |contract| {
                contract.buyers = vec![alice()];
                contract.sellers = vec![Seller {
                    principal: bob(),
                    quota: 100,
                }];
                contract.deposit = Deposit {
                    value_icp: 100_000_000,
                    value_fiat: 10,
                };
                contract.expiration = Some("2050-01-01".to_string());
            },
            |token| {
                token.owner = Some(bob());
                token.value = 10;
            },
        );

        // set expiration to 1970-01-01
        ContractStorage::mut_contract(&1u64.into(), |contract| {
            contract.expiration = Some("1970-01-01".to_string());
            Ok(())
        })
        .expect("failed to update contract");

        // transfer token 1 and 2 to charlie
        ContractStorage::mut_token(&1u64.into(), |token| {
            token.owner = Some(charlie());
            Ok(())
        })
        .expect("failed to update contract");
        ContractStorage::mut_token(&2u64.into(), |token| {
            token.owner = Some(charlie());
            Ok(())
        })
        .expect("failed to update contract");
        // transfer token 3 to dylan
        ContractStorage::mut_token(&3u64.into(), |token| {
            token.owner = Some(dylan());
            Ok(())
        })
        .expect("failed to update contract");
        // transfer token 4 to the buyer
        ContractStorage::mut_token(&4u64.into(), |token| {
            token.owner = Some(alice());
            Ok(())
        })
        .expect("failed to update contract");

        // close contract
        let contract = ContractStorage::get_contract(&1u64.into()).expect("contract not found");
        assert!(CloseOp::close_contract(contract).await.is_ok());

        // verify all tokens are burned
        let tokens = ContractStorage::get_contract(&1u64.into())
            .expect("contract not found")
            .tokens;
        for token_id in tokens {
            let token = ContractStorage::get_token(&token_id).expect("token not found");
            assert!(token.is_burned);
        }
    }
}
