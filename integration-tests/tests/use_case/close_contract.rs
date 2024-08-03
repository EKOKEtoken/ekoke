use std::time::{SystemTime, UNIX_EPOCH};

use did::deferred::{Buyers, ContractRegistration, ContractType, Deposit, GenericValue, Seller};
use icrc::icrc1::account::Account;
use integration_tests::actor::{admin, alice, bob, bob_account, charlie, charlie_account};
use integration_tests::client::{DeferredClient, IcrcLedgerClient, MarketplaceClient};
use integration_tests::TestEnv;
use pretty_assertions::assert_eq;
use time::OffsetDateTime;

#[test]
#[serial_test::serial]
fn test_as_agent_i_should_close_contract_after_expiration() {
    let env = TestEnv::init();
    let deferred_client = DeferredClient::from(&env);
    let marketplace_client = MarketplaceClient::from(&env);
    let icp_ledger_client = IcrcLedgerClient::new(env.icp_ledger_id, &env);

    // set expiration to tomorrow
    env.pic.set_time(SystemTime::now());
    let expiration = SystemTime::now() + std::time::Duration::from_secs(60 * 60 * 24);
    let after_expiration = expiration + std::time::Duration::from_secs(60 * 60 * 24);
    // format
    let expiration = OffsetDateTime::from_unix_timestamp_nanos(
        expiration.duration_since(UNIX_EPOCH).unwrap().as_nanos() as i128,
    )
    .unwrap();
    let expiration = format!(
        "{}-{:02}-{:02}",
        expiration.year(),
        expiration.month() as u8,
        expiration.day()
    );
    println!("expiration: {}", expiration);

    let registration_data = ContractRegistration {
        r#type: ContractType::Sell,
        sellers: vec![Seller {
            principal: alice(),
            quota: 100,
        }],
        buyers: Buyers {
            principals: vec![bob()],
            deposit_account: bob_account(),
        },
        deposit: Deposit {
            value_fiat: 20_000,
            value_icp: 4_000 * 100_000_000, // 4_000 ICP
        },
        value: 400_000,
        currency: "EUR".to_string(),
        installments: 4,
        properties: vec![(
            "contract:address".to_string(),
            GenericValue::TextContent("via roma 10".to_string()),
        )],
        restricted_properties: vec![],
        expiration: Some(expiration),
    };
    let deposit = registration_data.deposit.clone();
    // value_icp : value_fiat = x : 1
    let icp_value = registration_data.deposit.value_icp / registration_data.deposit.value_fiat;
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

    // let's buy 2 tokens
    let contract = deferred_client.get_contract(&contract_id).unwrap();
    for token in contract.tokens.iter().take(2) {
        // get nft price
        let icp_price = marketplace_client
            .get_token_price_icp(charlie(), token)
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
            .buy_token(charlie(), token, &charlie_account().subaccount)
            .is_ok());
    }

    // get charlie's tokens
    let charlie_tokens = [
        deferred_client.get_token(&0u64.into()).unwrap(),
        deferred_client.get_token(&1u64.into()).unwrap(),
    ];
    println!("icp_value: {}", icp_value);
    let token_value = charlie_tokens.iter().map(|t| t.token.value).sum::<u64>();
    println!("token_value: {}", token_value);
    let charlie_value = fiat_to_icp(&deposit, token_value);
    println!(
        "charlie_value (e8s): {}; ICP {}",
        charlie_value,
        charlie_value / 100_000_000
    );

    // closing contract before expiration should fail
    assert!(deferred_client
        .close_contract(admin(), contract_id.clone())
        .is_err());

    // advance time
    env.pic.set_time(after_expiration);

    // close contract
    let charlie_balance = icp_ledger_client.icrc1_balance_of(charlie_account());
    assert!(deferred_client
        .close_contract(admin(), contract_id.clone())
        .is_ok());
    // verify balance
    let new_balance = icp_ledger_client.icrc1_balance_of(charlie_account());
    let expected_balance = charlie_balance + charlie_value;

    assert_eq!(new_balance, expected_balance);
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
