//! # Inspect
//!
//! Deferred inspect message handler

use candid::{Nat, Principal};
use did::deferred::{
    Buyers, Contract, DeferredError, DeferredResult, Deposit, Seller, Token, TokenError,
};
use did::ID;
use dip721_rs::NftError;

use super::roles::RolesManager;
use super::storage::{Agents, ContractStorage};

pub struct Inspect;

impl Inspect {
    /// Returns whether caller is custodian of the canister
    pub fn inspect_is_custodian(caller: Principal) -> bool {
        RolesManager::is_custodian(caller)
    }

    pub fn inspect_is_agent(caller: Principal) -> bool {
        RolesManager::is_agent(caller)
    }

    /// Inspect whether can sign contract
    pub fn inspect_sign_contract(caller: Principal, contract_id: &ID) -> bool {
        Inspect::inspect_is_custodian(caller)
            || Inspect::inspect_is_agent_for_contract(caller, contract_id).is_ok()
    }

    /// Returns whether caller is agent of the canister and agent for the contracts
    pub fn inspect_is_agent_for_contract(
        caller: Principal,
        contract_id: &ID,
    ) -> DeferredResult<Contract> {
        if !RolesManager::is_agent(caller) {
            return Err(DeferredError::Unauthorized);
        }

        let contract = match ContractStorage::get_contract(contract_id) {
            Some(contract) => contract,
            None => return Err(DeferredError::Unauthorized),
        };

        let agency = match contract.agency.as_ref() {
            Some(agency) => agency,
            None => return Err(DeferredError::Unauthorized),
        };

        if Agents::get_agency_by_wallet(caller)
            .as_ref()
            .map(|assoc_agency| agency == assoc_agency)
            .unwrap_or_default()
        {
            Ok(contract)
        } else {
            Err(DeferredError::Unauthorized)
        }
    }

    /// Returns whether caller is owner or operator of the token
    pub fn inspect_is_owner_or_operator(
        caller: Principal,
        token_identifier: &Nat,
    ) -> Result<Token, NftError> {
        let token = match ContractStorage::get_token(token_identifier) {
            Some(token) => token,
            None => return Err(NftError::TokenNotFound),
        };

        let owner = match token.owner {
            Some(owner) => owner,
            None => return Err(NftError::UnauthorizedOwner),
        };

        if caller != owner && Some(caller) != token.operator {
            return Err(NftError::UnauthorizedOperator);
        }

        Ok(token)
    }

    /// Inspect whether the caller is owner or operator of the token and the token is not burned.
    pub fn inspect_transfer_from(
        caller: Principal,
        token_identifier: &Nat,
    ) -> Result<Token, NftError> {
        let token = Self::inspect_is_owner_or_operator(caller, token_identifier)?;
        if token.is_burned {
            return Err(NftError::ExistedNFT);
        }

        Ok(token)
    }

    /// Inspect burn, allow burn only if:
    /// - caller is owner or operator
    /// - token is owned by a buyer or a seller.
    pub fn inspect_burn(caller: Principal, token_identifier: &Nat) -> Result<(), NftError> {
        let token = match ContractStorage::get_token(token_identifier) {
            Some(token) => token,
            None => return Err(NftError::TokenNotFound),
        };
        let contract = match ContractStorage::get_contract(&token.contract_id) {
            Some(contract) => contract,
            None => return Err(NftError::TokenNotFound),
        };
        let owner = match token.owner {
            Some(owner) => owner,
            None => return Err(NftError::UnauthorizedOwner),
        };

        if !contract.buyers.contains(&owner) && !contract.is_seller(&caller) {
            return Err(NftError::Other(
                "owner is not nor a buyer nor the seller".to_string(),
            ));
        }
        if caller != owner && Some(caller) != token.operator {
            return Err(NftError::UnauthorizedOperator);
        }

        Ok(())
    }

    /// Inspect update contract property:
    ///
    /// - caller must be one of custodian,seller,agent
    /// - contract must exist
    /// - key must start with "contract:"
    pub fn inspect_update_contract_property(
        caller: Principal,
        id: &ID,
        key: &str,
    ) -> DeferredResult<()> {
        if !key.starts_with("contract:") {
            return Err(DeferredError::Token(TokenError::BadContractProperty));
        }
        let contract = match ContractStorage::get_contract(id) {
            Some(contract) => contract,
            None => {
                return Err(DeferredError::Token(TokenError::ContractNotFound(
                    id.clone(),
                )))
            }
        };

        if !Self::inspect_is_custodian(caller)
            && Self::inspect_is_agent_for_contract(caller, id).is_err()
            && !contract.is_seller(&caller)
        {
            Err(DeferredError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Inspect register contract parameters:
    ///
    /// - caller must be custodian or agent
    /// - value must be multiple of installments
    /// - must have sellers
    /// - cannot be expired
    pub fn inspect_register_contract(
        caller: Principal,
        value: u64,
        deposit: &Deposit,
        sellers: &[Seller],
        buyers: &Buyers,
        installments: u64,
        expiration: Option<&str>,
    ) -> DeferredResult<()> {
        if !Self::inspect_is_custodian(caller) && !Self::inspect_is_agent(caller) {
            return Err(DeferredError::Unauthorized);
        }

        if sellers.is_empty()
            || sellers
                .iter()
                .any(|seller| seller.principal == Principal::anonymous())
        {
            return Err(DeferredError::Token(TokenError::ContractHasNoSeller));
        }

        if buyers.principals.is_empty()
            || buyers
                .principals
                .iter()
                .any(|buyer| buyer == &Principal::anonymous())
        {
            return Err(DeferredError::Token(TokenError::ContractHasNoBuyer));
        }

        if buyers.deposit_account.owner == Principal::anonymous() {
            return Err(DeferredError::Token(TokenError::BadBuyerDepositAccount));
        }

        if value < deposit.value_fiat {
            return Err(DeferredError::Token(
                TokenError::ContractValueIsLessThanDeposit,
            ));
        }
        let installments_value = value - deposit.value_fiat;

        // verify value must be multiple of installments
        if installments_value % installments != 0 {
            return Err(DeferredError::Token(
                TokenError::ContractValueIsNotMultipleOfInstallments,
            ));
        }

        let total_quota = sellers.iter().map(|seller| seller.quota).sum::<u8>();
        if total_quota != 100 {
            return Err(DeferredError::Token(
                TokenError::ContractSellerQuotaIsNot100,
            ));
        }

        if let Some(expiration) = expiration {
            let format = time::macros::format_description!("[year]-[month]-[day]");
            match time::Date::parse(expiration, format) {
                Ok(expiration) => {
                    if expiration < crate::utils::date() {
                        return Err(DeferredError::Token(TokenError::BadContractExpiration));
                    }
                }
                Err(_) => {
                    return Err(DeferredError::Token(TokenError::BadContractExpiration));
                }
            }
        }

        Ok(())
    }

    pub fn inspect_is_seller(caller: Principal, contract: ID) -> DeferredResult<Contract> {
        let contract = match ContractStorage::get_contract(&contract) {
            Some(contract) => contract,
            None => return Err(DeferredError::Token(TokenError::ContractNotFound(contract))),
        };

        if contract.is_seller(&caller) {
            Ok(contract)
        } else {
            Err(DeferredError::Unauthorized)
        }
    }

    pub fn inspect_increment_contract_value(
        caller: Principal,
        contract: ID,
    ) -> DeferredResult<Contract> {
        if !Self::inspect_is_custodian(caller)
            && Self::inspect_is_agent_for_contract(caller, &contract).is_err()
        {
            return Err(DeferredError::Unauthorized);
        }

        let contract = match ContractStorage::get_contract(&contract) {
            Some(contract) => contract,
            None => return Err(DeferredError::Token(TokenError::ContractNotFound(contract))),
        };
        // check if is signed
        if !contract.is_signed {
            return Err(DeferredError::Token(TokenError::ContractNotSigned(
                contract.id,
            )));
        }

        Ok(contract)
    }

    /// Inspect whether caller is custodian or owner of the agency
    pub fn inspect_remove_agency(caller: Principal) -> bool {
        RolesManager::is_custodian(caller) || Agents::get_agency_by_wallet(caller).is_some()
    }
}

#[cfg(test)]
mod test {

    use did::deferred::{Role, Seller};
    use icrc::icrc1::account::Account;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::app::test_utils::{self, alice, bob, bob_account, mock_agency};
    use crate::utils::caller;

    #[test]
    fn test_should_inspect_is_custodian() {
        let caller = Principal::anonymous();
        assert_eq!(Inspect::inspect_is_custodian(caller), false);

        let caller = Principal::from_text("aaaaa-aa").unwrap();
        assert_eq!(Inspect::inspect_is_custodian(caller), false);

        let caller = Principal::from_text("aaaaa-aa").unwrap();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert_eq!(Inspect::inspect_is_custodian(caller), true);
    }

    #[test]
    fn test_should_inspect_is_agent_for_contract() {
        let caller = Principal::management_canister();
        let contract_id = 1;
        test_utils::store_mock_contract_with(
            &[1],
            contract_id,
            |contract| {
                contract.sellers = vec![Seller {
                    principal: caller,
                    quota: 100,
                }];
            },
            |token| {
                token.owner = Some(caller);
                token.operator = None;
            },
        );
        assert!(Inspect::inspect_is_agent_for_contract(caller, &contract_id.into()).is_err());

        // is agent, but doesn't own the contract
        RolesManager::give_role(alice(), Role::Agent);
        assert!(Inspect::inspect_is_agent_for_contract(alice(), &contract_id.into()).is_err());

        let agency = test_utils::mock_agency();
        Agents::insert_agency(alice(), agency.clone());

        let contract_id = 2;
        test_utils::store_mock_contract_with(
            &[2],
            contract_id,
            |contract| {
                contract.agency = Some(agency);
                contract.sellers = vec![Seller {
                    principal: alice(),
                    quota: 100,
                }];
            },
            |token| {
                token.owner = Some(alice());
                token.operator = None;
            },
        );

        assert!(Inspect::inspect_is_agent_for_contract(alice(), &contract_id.into()).is_ok());
    }

    #[test]
    fn test_should_inspect_is_agent() {
        let caller = Principal::anonymous();
        assert_eq!(Inspect::inspect_is_agent(caller), false);

        // is agent, but doesn't own the contract
        RolesManager::give_role(alice(), Role::Agent);
        assert_eq!(Inspect::inspect_is_agent(alice()), true);
    }

    #[test]
    fn test_should_is_owner_or_operator() {
        let caller = caller();
        test_utils::store_mock_contract_with(
            &[1],
            1,
            |_| {},
            |token| {
                token.owner = Some(caller);
                token.operator = None;
            },
        );
        assert!(Inspect::inspect_is_owner_or_operator(caller, &1_u64.into()).is_ok());

        // with operator
        test_utils::store_mock_contract_with(
            &[2],
            2,
            |_| {},
            |token| {
                token.operator = Some(caller);
            },
        );
        assert!(ContractStorage::transfer(&2_u64.into(), Principal::management_canister()).is_ok());
        assert!(Inspect::inspect_is_owner_or_operator(caller, &2_u64.into()).is_ok());

        // no operator, no owner
        test_utils::store_mock_contract_with(
            &[3],
            3,
            |_| {},
            |token| {
                token.operator = Some(Principal::management_canister());
            },
        );
        assert!(ContractStorage::transfer(&3_u64.into(), Principal::management_canister()).is_ok());
        assert!(Inspect::inspect_is_owner_or_operator(caller, &3_u64.into()).is_err());
    }

    #[test]
    fn test_should_inspect_transfer_from() {
        let caller = caller();
        test_utils::store_mock_contract_with(
            &[1],
            1,
            |_| {},
            |token| {
                token.owner = Some(caller);
                token.operator = None;
            },
        );
        assert!(Inspect::inspect_transfer_from(caller, &1_u64.into()).is_ok());

        // with operator
        test_utils::store_mock_contract_with(
            &[2],
            2,
            |_| {},
            |token| {
                token.operator = Some(caller);
            },
        );
        assert!(ContractStorage::transfer(&2_u64.into(), Principal::management_canister()).is_ok());
        assert!(Inspect::inspect_transfer_from(caller, &2_u64.into()).is_ok());

        // no operator, no owner
        test_utils::store_mock_contract_with(
            &[3],
            3,
            |_| {},
            |token| {
                token.operator = Some(Principal::management_canister());
            },
        );
        assert!(ContractStorage::transfer(&3_u64.into(), Principal::management_canister()).is_ok());
        assert!(Inspect::inspect_transfer_from(caller, &3_u64.into()).is_err());

        test_utils::store_mock_contract_with(
            &[4],
            4,
            |_| {},
            |token| {
                token.owner = Some(caller);
                token.operator = None;
            },
        );
        assert!(ContractStorage::burn_token(&4_u64.into()).is_ok());
        assert!(Inspect::inspect_transfer_from(caller, &4_u64.into()).is_err());
    }

    #[test]
    fn test_should_inspect_burn() {
        let caller = caller();
        // caller is owner and token is owned by buyer
        test_utils::store_mock_contract_with(
            &[1],
            1,
            |contract| {
                contract.buyers = vec![Principal::management_canister()];
            },
            |token| {
                token.owner = Some(caller);
                token.operator = None;
            },
        );
        assert!(Inspect::inspect_burn(caller, &1_u64.into()).is_ok());
        // caller is operator and token is owned by buyer
        test_utils::store_mock_contract_with(
            &[2],
            2,
            |contract| {
                contract.buyers = vec![Principal::management_canister()];
            },
            |token| {
                token.operator = Some(caller);
            },
        );
        assert!(ContractStorage::transfer(&2_u64.into(), Principal::management_canister()).is_ok());
        assert!(Inspect::inspect_burn(caller, &2_u64.into()).is_ok());
        // caller is owner and token is owned by buyer
        test_utils::store_mock_contract_with(
            &[3],
            3,
            |contract| {
                contract.sellers = vec![Seller {
                    principal: Principal::management_canister(),
                    quota: 100,
                }];
                contract.buyers = vec![caller];
            },
            |token| {
                token.owner = Some(Principal::management_canister());
                token.operator = None;
            },
        );
        assert!(ContractStorage::transfer(&2_u64.into(), caller).is_ok());
        assert!(Inspect::inspect_burn(caller, &1_u64.into()).is_ok());
        // caller is operator and token is owned by buyer
        test_utils::store_mock_contract_with(
            &[4],
            4,
            |contract| {
                contract.sellers = vec![Seller {
                    principal: Principal::management_canister(),
                    quota: 100,
                }];
                contract.buyers = vec![caller];
            },
            |token| {
                token.owner = Some(Principal::management_canister());
                token.operator = Some(caller);
            },
        );
        assert!(ContractStorage::transfer(&4_u64.into(), caller).is_ok());
        assert!(Inspect::inspect_burn(caller, &4_u64.into()).is_ok());
        // caller is not owner nor operator
        test_utils::store_mock_contract_with(
            &[5],
            5,
            |contract| {
                contract.buyers = vec![Principal::management_canister()];
            },
            |token| {
                token.operator = None;
            },
        );
        assert!(ContractStorage::transfer(&5_u64.into(), Principal::management_canister()).is_ok());
        assert!(Inspect::inspect_burn(caller, &5_u64.into()).is_err());
        // caller is owner, but owner is a third party
        test_utils::store_mock_contract_with(
            &[6],
            6,
            |contract| {
                contract.sellers = vec![Seller {
                    principal: Principal::management_canister(),
                    quota: 100,
                }];
                contract.buyers = vec![Principal::management_canister()];
            },
            |token| {
                token.owner = Some(Principal::management_canister());
                token.operator = None;
            },
        );
        assert!(ContractStorage::transfer(&6_u64.into(), caller).is_ok());
        assert!(Inspect::inspect_burn(caller, &6_u64.into()).is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_caller_is_not_custodian() {
        // caller is not custodian
        let caller = Principal::from_text("aaaaa-aa").unwrap();
        assert!(RolesManager::set_custodians(vec![crate::utils::caller()]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_value_is_not_multiple_of_installments() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            110,
            &Deposit {
                value_fiat: 25,
                value_icp: 10,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_caller_is_not_agent() {
        // caller is not agent
        let caller = Principal::from_text("aaaaa-aa").unwrap();
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_custodian() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_ok());
    }

    #[test]
    fn test_should_inspect_contract_register_if_seller_is_anonymous() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::anonymous(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_sellers_is_empty() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_value_is_less_than_deposit() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 200,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_buyer_is_anonymous() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![Principal::anonymous()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_buyers_is_empty() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_buyer_deposit_account_is_invalid() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: Account::from(Principal::anonymous()),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_quota_is_not_100() {
        let caller = crate::utils::caller();
        RolesManager::give_role(caller, Role::Agent);
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[
                Seller {
                    principal: Principal::management_canister(),
                    quota: 20,
                },
                Seller {
                    principal: caller,
                    quota: 40,
                }
            ],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_contract_register_if_agent() {
        let caller = crate::utils::caller();
        RolesManager::give_role(caller, Role::Agent);
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            None,
        )
        .is_ok());
    }

    #[test]
    fn test_should_inspect_contract_register_if_expired() {
        let caller = crate::utils::caller();
        RolesManager::give_role(caller, Role::Agent);
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            Some("2078-01-01"),
        )
        .is_ok());
        assert!(Inspect::inspect_register_contract(
            caller,
            100,
            &Deposit {
                value_fiat: 25,
                value_icp: 25,
            },
            &[Seller {
                principal: Principal::management_canister(),
                quota: 100,
            }],
            &Buyers {
                principals: vec![bob()],
                deposit_account: bob_account(),
            },
            25,
            Some("2018-01-01"),
        )
        .is_err());
    }

    #[test]
    fn test_should_inspect_caller_is_contract_seller() {
        let caller = crate::utils::caller();
        test_utils::store_mock_contract_with(
            &[6],
            1,
            |_| {},
            |token| {
                token.owner = Some(caller);
            },
        );
        assert!(Inspect::inspect_is_seller(caller, 1_u64.into()).is_ok());
        assert!(
            Inspect::inspect_is_seller(Principal::management_canister(), 1_u64.into()).is_err()
        );
        // unexisting contract
        assert!(Inspect::inspect_is_seller(caller, 2_u64.into()).is_err());
    }

    #[test]
    fn test_should_inspect_increment_contract_value() {
        let caller = crate::app::test_utils::alice();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());
        // set agent
        let agent = crate::app::test_utils::bob();
        let agency = mock_agency();
        Agents::insert_agency(agent, agency);
        RolesManager::give_role(agent, Role::Agent);
        let contract = test_utils::with_mock_contract(0, 1, |_| {});
        assert!(ContractStorage::insert_contract(contract).is_ok());
        let tokens = vec![test_utils::mock_token(0, 0)];
        assert!(Inspect::inspect_increment_contract_value(caller, 0_u64.into()).is_err());
        // sign contract
        assert!(ContractStorage::sign_contract_and_mint_tokens(&0_u64.into(), tokens).is_ok());
        assert!(Inspect::inspect_increment_contract_value(caller, 0_u64.into()).is_ok());
        assert!(Inspect::inspect_increment_contract_value(agent, 0_u64.into()).is_ok());
        // not seller
        assert!(Inspect::inspect_increment_contract_value(
            Principal::management_canister(),
            1_u64.into()
        )
        .is_err());
        // unexisting contract
        assert!(Inspect::inspect_increment_contract_value(caller, 2_u64.into()).is_err());
    }

    #[test]
    fn test_should_inspect_update_contract_property() {
        let caller = crate::utils::caller();
        test_utils::store_mock_contract_with(
            &[6],
            1,
            |_| {},
            |token| {
                token.owner = Some(caller);
            },
        );
        assert!(Inspect::inspect_update_contract_property(
            caller,
            &1_u64.into(),
            "contract:address"
        )
        .is_ok());
        assert!(
            Inspect::inspect_update_contract_property(caller, &1_u64.into(), "foobar").is_err()
        );
        assert!(Inspect::inspect_update_contract_property(
            Principal::management_canister(),
            &1_u64.into(),
            "contract:address"
        )
        .is_err());
        // unexisting contract
        assert!(Inspect::inspect_update_contract_property(
            caller,
            &2_u64.into(),
            "contract:address"
        )
        .is_err());
        // admin
        assert!(RolesManager::set_custodians(vec![Principal::management_canister()]).is_ok());
        assert!(Inspect::inspect_update_contract_property(
            Principal::management_canister(),
            &1_u64.into(),
            "contract:address"
        )
        .is_ok());
    }

    #[test]
    fn test_should_inspect_whether_to_remove_agency() {
        let caller = crate::utils::caller();
        assert!(RolesManager::set_custodians(vec![caller]).is_ok());

        // register agency
        Agents::insert_agency(bob(), test_utils::mock_agency());
        assert!(Inspect::inspect_remove_agency(caller));
        assert!(Inspect::inspect_remove_agency(bob()));
        assert!(!Inspect::inspect_remove_agency(
            Principal::management_canister()
        ));
    }

    #[test]
    fn test_should_inspect_sign_contract() {
        let admin = alice();
        assert!(RolesManager::set_custodians(vec![admin]).is_ok());
        // set agent
        let agent = bob();
        let agency = mock_agency();
        let other_agent = Principal::management_canister();
        Agents::insert_agency(agent, agency.clone());
        RolesManager::give_role(agent, Role::Agent);

        let mut other_agency = agency.clone();
        other_agency.name = "other".to_string();
        Agents::insert_agency(other_agent, other_agency.clone());
        RolesManager::give_role(other_agent, Role::Agent);

        let contract_id = 0;
        let contract = test_utils::with_mock_contract(contract_id, 1, |contract| {
            contract.agency = Some(agency);
        });
        assert!(ContractStorage::insert_contract(contract).is_ok());

        // admin
        assert!(Inspect::inspect_sign_contract(admin, &contract_id.into()));

        // agent
        assert_eq!(
            Inspect::inspect_sign_contract(other_agent, &contract_id.into()),
            false
        );
    }
}
