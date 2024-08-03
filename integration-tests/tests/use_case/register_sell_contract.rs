use candid::Nat;
use did::deferred::{
    Agency, Buyers, ContractRegistration, ContractType, Deposit, GenericValue, Seller,
};
use icrc::icrc1::account::Account;
use integration_tests::actor::{admin, alice, bob};
use integration_tests::client::DeferredClient;
use integration_tests::TestEnv;
use pretty_assertions::assert_eq;

#[test]
#[serial_test::serial]
fn test_as_seller_i_can_register_a_sell_contract() {
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
        properties: vec![(
            "contract:address".to_string(),
            GenericValue::TextContent("via roma 10".to_string()),
        )],
        restricted_properties: vec![],
        expiration: None,
    };

    let expected_token_value = (registration_data.value - registration_data.deposit.value_fiat)
        / registration_data.installments;
    // approve deposit
    crate::helper::contract_deposit(
        &env,
        registration_data.buyers.deposit_account,
        registration_data.deposit.value_icp,
    );

    // register agency for admin
    let agency = Agency {
        name: "Admin's agency".to_string(),
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
    deferred_client.admin_register_agency(admin(), agency.clone());

    // call register
    let contract_id = deferred_client
        .register_contract(admin(), registration_data)
        .unwrap();
    assert_eq!(contract_id, 0_u64);

    // check unsigned contract and signed contracts
    let unsigned_contracts = deferred_client.get_unsigned_contracts();
    assert_eq!(unsigned_contracts, vec![contract_id.clone()]);
    let signed_contract = deferred_client.get_signed_contracts();
    assert!(signed_contract.is_empty());

    // sign contract
    let res = deferred_client.sign_contract(Nat::from(0_u64));
    assert!(res.is_ok());

    // get contract
    let contract = deferred_client.get_contract(&contract_id).unwrap();
    assert_eq!(contract.agency.unwrap(), agency);

    // check unsigned contract and signed contracts
    let unsigned_contracts = deferred_client.get_unsigned_contracts();
    assert!(unsigned_contracts.is_empty());
    let signed_contract = deferred_client.get_signed_contracts();
    assert_eq!(signed_contract, vec![contract_id.clone()]);

    // verify contract tokens
    // there should be 400_000 / 100 = 4000 tokens
    let total_supply = deferred_client.total_supply();
    assert_eq!(total_supply, 4_000_u64);

    // first half for alice
    let token = deferred_client.token_metadata(Nat::from(0_u64)).unwrap();
    assert_eq!(token.is_burned, false);
    assert_eq!(token.owner.unwrap(), alice());
    assert_eq!(token.operator, Some(env.marketplace_id));
    let token_value = token
        .properties
        .iter()
        .find(|(k, _)| k == "token:value")
        .unwrap()
        .1
        .clone();

    assert_eq!(
        token_value,
        GenericValue::NatContent(expected_token_value.into())
    );

    let token = deferred_client.token_metadata(Nat::from(2000_u64)).unwrap();
    assert_eq!(token.owner.unwrap(), bob());
}
