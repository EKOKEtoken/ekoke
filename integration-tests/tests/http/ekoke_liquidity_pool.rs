use did::ekoke_liquidity_pool::{LiquidityPoolAccounts, LiquidityPoolBalance};
use integration_tests::client::HttpClient;
use integration_tests::TestEnv;

#[test]
#[ignore]
#[serial_test::serial]
fn test_http_should_get_liquidity_pool_accounts_and_balance() {
    let env = TestEnv::init();

    let http_client = HttpClient::new(env.ekoke_liquidity_pool_id, &env);
    let liquidity_pool_accounts: LiquidityPoolAccounts =
        http_client.http_request("liquidityPoolAccounts", serde_json::json!({}));

    assert_eq!(
        liquidity_pool_accounts.icp.owner,
        env.ekoke_liquidity_pool_id
    );

    let liquidity_pool_balance: LiquidityPoolBalance =
        http_client.http_request("liquidityPoolBalance", serde_json::json!({}));

    assert_eq!(liquidity_pool_balance.icp, 0u64);
}
