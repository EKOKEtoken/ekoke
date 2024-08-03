//! Types associated to the "Deferred" canister

mod canister;
mod contract;
mod error;

pub type DeferredResult<T> = Result<T, DeferredError>;

pub use self::canister::{DeferredInitData, Role, Roles, StorableTxEvent};
pub use self::contract::{
    Agency, Buyers, Continent, Contract, ContractProperties, ContractRegistration, ContractType,
    Deposit, GenericValue, RestrictedContractProperties, RestrictedProperty, RestrictionLevel,
    Seller, Token, TokenIdentifier, TokenInfo, ID,
};
pub use self::error::{
    CloseContractError, ConfigurationError, DeferredError, DepositError, TokenError, WithdrawError,
};

#[cfg(test)]
mod test {

    use candid::{Decode, Encode, Principal};
    use dip721_rs::{GenericValue, TokenIdentifier, TxEvent};
    use ic_stable_structures::Storable as _;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::ID;

    #[test]
    fn test_should_encode_token() {
        let token = Token {
            id: TokenIdentifier::from(1_u64),
            contract_id: ID::from(1_u64),
            owner: Some(
                Principal::from_text(
                    "zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae",
                )
                .unwrap(),
            ),
            ekoke_reward: 4_000_u64.into(),
            transferred_at: None,
            transferred_by: None,
            approved_at: None,
            approved_by: None,
            burned_at: None,
            burned_by: None,
            minted_at: 0,
            minted_by: Principal::anonymous(),
            operator: None,
            value: 100,
            is_burned: false,
        };
        let data = Encode!(&token).unwrap();
        let decoded_token = Decode!(&data, Token).unwrap();

        assert_eq!(token.id, decoded_token.id);
        assert_eq!(token.contract_id, decoded_token.contract_id);
        assert_eq!(token.owner, decoded_token.owner);
        assert_eq!(token.value, decoded_token.value);
        assert_eq!(token.ekoke_reward, decoded_token.ekoke_reward);
    }

    #[test]
    fn test_should_encode_contract() {
        let contract = Contract {
            id: ID::from(1_u64),
            r#type: ContractType::Sell,
            sellers: vec![
                Seller {
                    principal: Principal::from_text(
                        "zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae",
                    )
                    .unwrap(),
                    quota: 51,
                },
                Seller {
                    principal: Principal::management_canister(),
                    quota: 49,
                },
            ],
            buyers: vec![
                Principal::anonymous(),
                Principal::from_text(
                    "zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae",
                )
                .unwrap(),
            ],
            installments: 2,
            is_signed: true,
            tokens: vec![TokenIdentifier::from(1_u64), TokenIdentifier::from(2_u64)],
            initial_value: 250_000,
            value: 250_000,
            deposit: Deposit {
                value_fiat: 50_000,
                value_icp: 100,
            },
            currency: "EUR".to_string(),
            properties: vec![(
                "Rome".to_string(),
                GenericValue::TextContent("Rome".to_string()),
            )],
            restricted_properties: vec![(
                "Secret".to_string(),
                RestrictedProperty {
                    access_list: vec![RestrictionLevel::Agent],
                    value: GenericValue::TextContent("Secret".to_string()),
                },
            )],
            agency: Some(Agency {
                name: "Agency".to_string(),
                address: "Address".to_string(),
                city: "City".to_string(),
                region: "Region".to_string(),
                zip_code: "Zip".to_string(),
                country: "Country".to_string(),
                continent: Continent::Europe,
                email: "Email".to_string(),
                website: "Website".to_string(),
                mobile: "Mobile".to_string(),
                vat: "VAT".to_string(),
                agent: "Agent".to_string(),
                logo: None,
            }),
            expiration: Some("2040-01-01".to_string()),
        };
        let data = Encode!(&contract).unwrap();
        let decoded_contract = Decode!(&data, Contract).unwrap();

        assert_eq!(contract.id, decoded_contract.id);
        assert_eq!(contract.sellers, decoded_contract.sellers);
        assert_eq!(contract.buyers, decoded_contract.buyers);
        assert_eq!(contract.tokens, decoded_contract.tokens);
        assert_eq!(contract.properties, decoded_contract.properties);
        assert_eq!(contract.value, decoded_contract.value);
        assert_eq!(contract.initial_value, decoded_contract.initial_value);
        assert_eq!(contract.currency, decoded_contract.currency);
        assert_eq!(contract.installments, decoded_contract.installments);
        assert_eq!(contract.is_signed, decoded_contract.is_signed);
        assert_eq!(contract.agency, decoded_contract.agency);
    }

    #[test]
    fn test_should_encode_tx_event() {
        let tx_event: StorableTxEvent = TxEvent {
            caller: Principal::from_text(
                "zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae",
            )
            .unwrap(),
            details: vec![],
            operation: "mint".to_string(),
            time: 0,
        }
        .into();

        let data = tx_event.to_bytes();
        let decoded_tx_event = StorableTxEvent::from_bytes(data);
        assert_eq!(tx_event.0, decoded_tx_event.0);
    }

    #[test]
    fn test_should_encode_role() {
        let role: Roles = vec![Role::Agent, Role::Custodian].into();

        let data = role.to_bytes();
        let decoded_role = Roles::from_bytes(data);
        assert_eq!(role, decoded_role);
    }
}
