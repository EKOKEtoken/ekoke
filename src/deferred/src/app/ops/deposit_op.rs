use did::deferred::{DeferredError, DeferredResult, DepositError};
use did::ID;
use icrc::icrc1::account::Account;
use icrc::IcrcLedgerClient;

use crate::app::configuration::Configuration;
use crate::app::Deferred;
use crate::utils;

pub struct DepositOp;

impl DepositOp {
    /// Take buyer's deposit
    ///
    /// 1. Get ICP fee
    /// 2. Check given allowance to the canister
    /// 3. Transfer the deposit to the canister
    pub async fn take_buyers_deposit(
        contract_id: &ID,
        deposit_account: Account,
        value: u64,
    ) -> DeferredResult<()> {
        let icp_ledger_client = IcrcLedgerClient::new(Configuration::get_icp_ledger_canister());

        let canister_account = Deferred::canister_deposit_account(contract_id);
        let icp_ledger_fee = icp_ledger_client
            .icrc1_fee()
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?;
        let allowance = icp_ledger_client
            .icrc2_allowance(canister_account, deposit_account)
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?;

        // check allowance expiration and value
        if allowance
            .expires_at
            .map(|expiration| expiration < utils::time())
            .unwrap_or_default()
        {
            return Err(DeferredError::Deposit(DepositError::AllowanceExpired));
        }

        let required_allowance = value + icp_ledger_fee;

        // check if allowance is enough
        if allowance.allowance < required_allowance {
            return Err(DeferredError::Deposit(DepositError::AllowanceNotEnough {
                required: required_allowance,
                available: allowance.allowance,
            }));
        }

        icp_ledger_client
            .icrc2_transfer_from(None, deposit_account, canister_account, value.into())
            .await
            .map_err(|(code, msg)| DeferredError::CanisterCall(code, msg))?
            .map_err(|err| DeferredError::Deposit(DepositError::Rejected(err)))?;

        Ok(())
    }
}
