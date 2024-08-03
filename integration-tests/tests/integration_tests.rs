mod helper;
mod http;
mod inspect;
mod use_case;

use integration_tests::TestEnv;

#[test]
#[serial_test::serial]
fn test_should_install_canisters() {
    TestEnv::init();
}
