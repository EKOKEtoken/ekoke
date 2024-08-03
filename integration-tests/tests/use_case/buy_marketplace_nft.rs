use did::deferred::{
    Buyers, ContractRegistration, ContractType, Deposit, GenericValue, Seller, ID,
};
use icrc::icrc1::account::Account;
use integration_tests::actor::{admin, alice, alice_account, bob, charlie, charlie_account};
use integration_tests::client::{DeferredClient, IcrcLedgerClient, MarketplaceClient};
use integration_tests::TestEnv;
use pretty_assertions::{assert_eq, assert_ne};

#[test]
#[serial_test::serial]
fn test_should_buy_marketplace_nft_as_non_contract_buyer() {
    let env = TestEnv::init();
    let contract_id = setup_contract_marketplace(&env);

    let deferred_client = DeferredClient::from(&env);
    let marketplace_client = MarketplaceClient::from(&env);
    let ekoke_ledger_client = IcrcLedgerClient::new(env.ekoke_ledger_id, &env);
    let icp_ledger_client = IcrcLedgerClient::new(env.icp_ledger_id, &env);

    // get initial ekoke balance for charlie
    let initial_balance = ekoke_ledger_client.icrc1_balance_of(charlie_account());

    // get contract by id
    let contract = deferred_client.get_contract(&contract_id).unwrap();
    let token_to_buy = contract.tokens[0].clone();
    // get nft price
    let icp_price = marketplace_client
        .get_token_price_icp(charlie(), &token_to_buy)
        .unwrap();
    assert_ne!(icp_price, 0);

    // approve on icp ledger client a spend for token price to marketplace canister
    let allowance = icp_ledger_client
        .icrc2_approve(
            charlie(),
            Account::from(env.marketplace_id),
            icp_price.into(),
            charlie_account().subaccount,
        )
        .unwrap();
    assert_eq!(icp_price, allowance);
    let allowance =
        icp_ledger_client.icrc2_allowance(charlie_account(), Account::from(env.marketplace_id));
    assert_eq!(icp_price, allowance.allowance);

    // buy token
    assert!(marketplace_client
        .buy_token(charlie(), &token_to_buy, &charlie_account().subaccount)
        .is_ok());

    // verify token owner is charlie
    let token = deferred_client.get_token(&token_to_buy).unwrap().token;
    assert_eq!(token.owner.unwrap(), charlie());

    // verify charlie got the reward
    let final_balance = ekoke_ledger_client.icrc1_balance_of(charlie_account());
    let balance_diff = final_balance - initial_balance + 1_000u64;
    assert_eq!(balance_diff, token.ekoke_reward);
}

#[test]
#[serial_test::serial]
fn test_should_buy_marketplace_nft_as_contract_buyer() {
    let env = TestEnv::init();
    let contract_id = setup_contract_marketplace(&env);

    let deferred_client = DeferredClient::from(&env);
    let marketplace_client = MarketplaceClient::from(&env);
    let ekoke_ledger_client = IcrcLedgerClient::new(env.ekoke_ledger_id, &env);
    let icp_ledger_client = IcrcLedgerClient::new(env.icp_ledger_id, &env);

    // get initial ekoke balance for charlie
    let initial_balance = ekoke_ledger_client.icrc1_balance_of(alice_account());

    // get contract by id
    let contract = deferred_client.get_contract(&contract_id).unwrap();
    let token_to_buy = contract.tokens[0].clone();
    // get nft price
    let icp_price = marketplace_client
        .get_token_price_icp(alice(), &token_to_buy)
        .unwrap();
    assert_ne!(icp_price, 0);

    // approve on icp ledger client a spend for token price to marketplace canister
    assert!(icp_ledger_client
        .icrc2_approve(
            alice(),
            Account::from(env.marketplace_id),
            icp_price.into(),
            alice_account().subaccount
        )
        .is_ok());

    let allowance =
        icp_ledger_client.icrc2_allowance(alice_account(), Account::from(env.marketplace_id));
    assert_eq!(icp_price, allowance.allowance);

    // buy token
    assert!(marketplace_client
        .buy_token(alice(), &token_to_buy, &alice_account().subaccount)
        .is_ok());

    // verify token owner is None
    let token = deferred_client.get_token(&token_to_buy).unwrap().token;
    assert!(token.owner.is_none());
    // should be burned
    assert!(token.is_burned);

    // verify alice got the reward
    let final_balance = ekoke_ledger_client.icrc1_balance_of(alice_account());
    let balance_diff = final_balance - initial_balance + 1_000u64;
    assert_eq!(balance_diff, token.ekoke_reward);
}

fn setup_contract_marketplace(env: &TestEnv) -> ID {
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
        &env,
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
