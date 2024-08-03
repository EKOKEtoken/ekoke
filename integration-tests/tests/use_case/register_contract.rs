use did::deferred::{Buyers, ContractRegistration, ContractType, Deposit, GenericValue, Seller};
use icrc::icrc1::account::Account;
use integration_tests::actor::{admin, alice, bob};
use integration_tests::client::{DeferredClient, IcrcLedgerClient};
use integration_tests::TestEnv;
use pretty_assertions::assert_eq;

#[test]
#[serial_test::serial]
fn test_as_agency_i_can_register_contract() {
    let env = TestEnv::init();
    let deferred_client = DeferredClient::from(&env);

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
    let deposit_value_icp = registration_data.deposit.value_icp;
    // approve deposit
    crate::helper::contract_deposit(
        &env,
        registration_data.buyers.deposit_account,
        deposit_value_icp,
    );

    // call register
    let contract_id = deferred_client
        .register_contract(admin(), registration_data)
        .unwrap();

    // sign contract
    let res = deferred_client.sign_contract(contract_id.clone());
    assert!(res.is_ok());

    // verify deposit
    let icp_ledger_client = IcrcLedgerClient::new(env.icp_ledger_id, &env);
    let subaccount = crate::helper::contract_subaccount(&contract_id);

    let current_canister_balance = icp_ledger_client.icrc1_balance_of(Account {
        owner: env.deferred_id,
        subaccount: Some(subaccount),
    });
    assert_eq!(current_canister_balance, deposit_value_icp);
}
