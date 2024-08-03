use did::deferred::{Buyers, ContractRegistration, ContractType, Deposit, GenericValue, Seller};
use icrc::icrc1::account::Account;
use integration_tests::actor::{admin, alice, bob, charlie, charlie_account};
use integration_tests::client::{DeferredClient, IcrcLedgerClient, MarketplaceClient};
use integration_tests::TestEnv;
use pretty_assertions::assert_eq;

#[test]
#[serial_test::serial]
fn test_as_seller_i_should_withdraw_contract_deposit_after_being_paid() {
    let env = TestEnv::init();
    let deferred_client = DeferredClient::from(&env);
    let marketplace_client = MarketplaceClient::from(&env);
    let icp_ledger_client = IcrcLedgerClient::new(env.icp_ledger_id, &env);

    let registration_data = ContractRegistration {
        r#type: ContractType::Sell,
        sellers: vec![
            Seller {
                principal: alice(),
                quota: 60,
            },
            Seller {
                principal: bob(),
                quota: 40,
            },
        ],
        buyers: Buyers {
            principals: vec![charlie()],
            deposit_account: Account::from(charlie()),
        },
        deposit: Deposit {
            value_fiat: 20_000,
            value_icp: 4_000 * 100_000_000, // 4_000 ICP
        },
        value: 400_000,
        currency: "EUR".to_string(),
        installments: 2,
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

    // we need to buy all the contracts :(
    let contract = deferred_client.get_contract(&contract_id).unwrap();
    for token in contract.tokens {
        // get nft price
        let icp_price = marketplace_client
            .get_token_price_icp(charlie(), &token)
            .unwrap();
        // approve on icp ledger client a spend for token price to marketplace canister
        icp_ledger_client
            .icrc2_approve(
                charlie(),
                Account::from(env.marketplace_id),
                icp_price.into(),
                charlie_account().subaccount,
            )
            .unwrap();
        assert!(marketplace_client
            .buy_token(charlie(), &token, &charlie_account().subaccount)
            .is_ok());
    }

    // get fee
    let fee = icp_ledger_client.icrc1_fee();
    // withdraw deposit
    let alice_balance = icp_ledger_client.icrc1_balance_of(Account::from(alice()));
    assert!(deferred_client
        .withdraw_contract_deposit(alice(), contract_id.clone(), None)
        .is_ok());
    // verify balance
    let new_balance = icp_ledger_client.icrc1_balance_of(Account::from(alice()));
    let expected_balance = alice_balance + (deposit_value_icp * 60 / 100) - fee.clone();
    let diff = expected_balance.clone() - new_balance.clone();
    println!("diff: {:?}", diff);
    assert_eq!(new_balance, expected_balance);

    // withdraw deposit for bob
    let bob_balance = icp_ledger_client.icrc1_balance_of(Account::from(bob()));
    assert!(deferred_client
        .withdraw_contract_deposit(bob(), contract_id.clone(), None)
        .is_ok());
    // verify balance
    let new_balance = icp_ledger_client.icrc1_balance_of(Account::from(bob()));
    let expected_balance = bob_balance + (deposit_value_icp * 40 / 100) - fee;
    assert_eq!(new_balance, expected_balance);
}
