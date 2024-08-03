use did::deferred::{
    Agency, Buyers, Contract, ContractRegistration, ContractType, Deposit, Seller, TokenInfo,
};
use did::ID;
use dip721_rs::GenericValue;
use icrc::icrc1::account::Account;
use integration_tests::actor::{admin, alice, bob};
use integration_tests::client::{DeferredClient, HttpClient};
use integration_tests::TestEnv;

#[test]
#[serial_test::serial]
fn test_http_should_get_contracts() {
    let env = TestEnv::init();
    let contract_id = init_contract(&env);

    let http_client = HttpClient::new(env.deferred_id, &env);
    let contracts: Vec<ID> = http_client.http_request("getContracts", serde_json::json!({}));

    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0], contract_id);

    // get contract by id
    let contract: Contract = http_client.http_request(
        "getContract",
        serde_json::json!({
            "id": contract_id,
        }),
    );
    assert_eq!(contract.id, contract_id);

    // get unexisting contract
    let response = http_client.raw_http_request_response(
        "getContract",
        serde_json::json!({
            "id": 5000_u64,
        }),
    );
    assert_eq!(response.status_code, 404);
}

#[test]
#[serial_test::serial]
fn test_http_should_get_token() {
    let env = TestEnv::init();
    let contract_id = init_contract(&env);

    let http_client = HttpClient::new(env.deferred_id, &env);
    let token_info: TokenInfo = http_client.http_request(
        "getToken",
        serde_json::json!({
            "id": 1,
        }),
    );
    assert_eq!(token_info.token.id, 1u64);
    assert_eq!(token_info.token.contract_id, contract_id);

    // get unexisting token
    let response = http_client.raw_http_request_response(
        "getToken",
        serde_json::json!({
            "id": 5000_u64,
        }),
    );
    assert_eq!(response.status_code, 404);
}

#[test]
#[serial_test::serial]
fn test_http_should_get_agencies() {
    let env = TestEnv::init();
    let deferred_client = DeferredClient::from(&env);

    let agency = Agency {
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
    };

    // give bob an agency
    deferred_client.admin_register_agency(bob(), agency.clone());

    let http_client = HttpClient::new(env.deferred_id, &env);
    let agencies: Vec<Agency> = http_client.http_request("getAgencies", serde_json::json!({}));

    assert_eq!(agencies.len(), 1);
    assert_eq!(agencies[0], agency);
}

fn init_contract(env: &TestEnv) -> ID {
    let deferred_client = DeferredClient::from(env);

    let registration_data = ContractRegistration {
        r#type: ContractType::Sell,
        sellers: vec![Seller {
            principal: bob(),
            quota: 100,
        }],
        buyers: Buyers {
            principals: vec![alice()],
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
        env,
        registration_data.buyers.deposit_account,
        registration_data.deposit.value_icp,
    );
    // call register
    let contract_id = deferred_client
        .register_contract(admin(), registration_data)
        .unwrap();
    assert_eq!(contract_id, 0_u64);

    // sign contract
    let res = deferred_client.sign_contract(contract_id.clone());
    assert!(res.is_ok());

    contract_id
}
