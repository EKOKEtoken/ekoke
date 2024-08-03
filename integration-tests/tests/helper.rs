use candid::Nat;
use icrc::icrc1::account::{Account, Subaccount};
use integration_tests::client::IcrcLedgerClient;
use integration_tests::TestEnv;

pub fn contract_deposit(test_env: &TestEnv, deposit_account: Account, amount: u64) {
    let icp_ledger = IcrcLedgerClient::new(test_env.icp_ledger_id, test_env);
    let fee = icp_ledger.icrc1_fee();
    let amount = amount + fee;

    icp_ledger
        .icrc2_approve(
            deposit_account.owner,
            test_env.deferred_id.into(),
            amount,
            deposit_account.subaccount,
        )
        .expect("contract deposit approve failed");
}

pub fn contract_subaccount(contract_id: &Nat) -> Subaccount {
    let contract_id = contract_id.0.to_bytes_be();
    // if contract id is less than 32 bytes, pad it with zeros
    let contract_id = if contract_id.len() < 32 {
        let mut padded = vec![0; 32 - contract_id.len()];
        padded.extend_from_slice(&contract_id);
        padded
    } else {
        contract_id
    };
    let subaccount: Subaccount = contract_id.try_into().expect("invalid contract id");

    subaccount
}
