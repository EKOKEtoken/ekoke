use did::deferred::{Buyers, ContractRegistration, ContractType, Deposit, GenericValue, Seller};
use icrc::icrc1::account::Account;
use integration_tests::actor::{admin, alice, bob};
use integration_tests::client::DeferredClient;
use integration_tests::TestEnv;
use pretty_assertions::assert_eq;

#[test]
#[serial_test::serial]
fn test_should_update_contract_property() {
    let env = TestEnv::init();
    let deferred_client = DeferredClient::from(&env);

    let registration_data = ContractRegistration {
        r#type: ContractType::Sell,
        sellers: vec![
            Seller {
                principal: alice(),
                quota: 50,
            },
            Seller {
                principal: bob(),
                quota: 50,
            },
        ],
        buyers: Buyers {
            principals: vec![bob()],
            deposit_account: Account::from(alice()),
        },
        deposit: Deposit {
            value_fiat: 20_000,
            value_icp: 100,
        },
        value: 400_000,
        currency: "EUR".to_string(),
        installments: 400_000 / 100,
        properties: vec![
            (
                "contract:address".to_string(),
                GenericValue::TextContent("via roma 10".to_string()),
            ),
            (
                "contract:architect".to_string(),
                GenericValue::TextContent("Gino Valle".to_string()),
            ),
        ],
        restricted_properties: vec![],
        expiration: None,
    };
    // approve deposit
    crate::helper::contract_deposit(
        &env,
        registration_data.buyers.deposit_account,
        registration_data.deposit.value_icp,
    );

    // call register
    let contract_id = deferred_client
        .register_contract(admin(), registration_data)
        .unwrap();

    let res = deferred_client.sign_contract(contract_id.clone());
    assert!(res.is_ok());

    // call update_contract_property
    assert!(deferred_client
        .update_contract_property(
            alice(),
            contract_id,
            "contract:architect".to_string(),
            GenericValue::TextContent("Renzo Piano".to_string())
        )
        .is_ok());

    let token_metadata = deferred_client.token_metadata(0_u64.into()).unwrap();
    let value = token_metadata
        .properties
        .iter()
        .find(|(k, _)| k == "contract:architect")
        .unwrap();
    assert_eq!(
        value.1,
        GenericValue::TextContent("Renzo Piano".to_string())
    );
}
