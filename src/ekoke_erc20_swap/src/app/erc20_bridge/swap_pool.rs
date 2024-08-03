use did::ekoke::{Ekoke, EkokeError, EkokeResult};
use icrc::icrc1::account::Account;
use icrc::IcrcLedgerClient;

use crate::app::Configuration;

/// Swap Pool contains the tokens exchanged from ERC20 to EKOKE
pub struct SwapPool;

impl SwapPool {
    /// Deposit $ekoke tokens to the swap pool from the provided account.
    pub async fn deposit(from: Account, amount: Ekoke) -> EkokeResult<()> {
        let ledger_client = IcrcLedgerClient::new(Configuration::get_ledger_canister());
        ledger_client
            .icrc2_transfer_from(None, from, super::Erc20Bridge::swap_account(), amount)
            .await
            .map_err(|(code, msg)| EkokeError::CanisterCall(code, msg))??;

        Ok(())
    }

    /// Withdraw $ekoke tokens from the swap pool to the provided account.
    pub async fn withdraw(to: Account, amount: Ekoke) -> EkokeResult<()> {
        let ledger_client = IcrcLedgerClient::new(Configuration::get_ledger_canister());
        ledger_client
            .icrc1_transfer(to, amount, None)
            .await
            .map_err(|(code, msg)| EkokeError::CanisterCall(code, msg))??;

        Ok(())
    }
}
