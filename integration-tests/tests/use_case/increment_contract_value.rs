use did::deferred::{
    Agency, Buyers, ContractRegistration, ContractType, Deposit, GenericValue, Seller,
};
use icrc::icrc1::account::Account;
use integration_tests::actor::{alice, bob};
use integration_tests::client::DeferredClient;
use integration_tests::TestEnv;
use pretty_assertions::assert_eq;

#[test]
#[serial_test::serial]
fn test_as_seller_i_can_set_the_contract_buyers() {
    let env = TestEnv::init();
    let deferred_client = DeferredClient::from(&env);

    let agent = bob();
    // give bob an agency
    deferred_client.admin_register_agency(
        agent,
        Agency {
            name: "Bob's agency".to_string(),
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
        },
    );

    let registration_data = ContractRegistration {
        r#type: ContractType::Sell,
        sellers: vec![Seller {
            principal: alice(),
            quota: 100,
        }],
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
        properties: vec![(
            "contract:address".to_string(),
            GenericValue::TextContent("via roma 10".to_string()),
        )],
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
        .register_contract(agent, registration_data)
        .unwrap();

    // sign contract
    let res = deferred_client.sign_contract(contract_id.clone());
    assert!(res.is_ok());

    // increment contract value
    assert!(deferred_client
        .increment_contract_value(agent, contract_id, 100_000, 1_000)
        .is_ok());

    // verify new value and supply
    assert_eq!(deferred_client.total_supply(), 5_000_u64);
}
