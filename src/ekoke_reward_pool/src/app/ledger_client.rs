use candid::Nat;
use did::ekoke::{Ekoke, EkokeError, EkokeResult};
use ic_cdk::api::call::RejectionCode;
use icrc::icrc1::account::Account;
use icrc::IcrcLedgerClient;

use super::configuration::Configuration;
use crate::utils::id;

pub struct LedgerClient;

impl LedgerClient {
    pub async fn canister_balance() -> EkokeResult<Ekoke> {
        Self::client()
            .icrc1_balance_of(Self::canister_account())
            .await
            .map_err(Self::map_err)
    }

    pub async fn transfer(recipient: Account, amount: Ekoke) -> EkokeResult<Nat> {
        Self::client()
            .icrc1_transfer(recipient, amount, None)
            .await
            .map_err(Self::map_err)?
            .map_err(EkokeError::Icrc1Transfer)
    }

    pub async fn transfer_from(from: Account, amount: Ekoke) -> EkokeResult<Nat> {
        Self::client()
            .icrc2_transfer_from(None, from, Self::canister_account(), amount)
            .await
            .map_err(Self::map_err)?
            .map_err(EkokeError::Icrc2Transfer)
    }

    pub async fn allowance(owner: Account) -> EkokeResult<Nat> {
        Self::client()
            .icrc2_allowance(Self::canister_account(), owner)
            .await
            .map_err(Self::map_err)
            .map(|allowance| allowance.allowance)
    }

    #[inline]
    fn canister_account() -> Account {
        Account::from(id())
    }

    #[inline]
    fn client() -> IcrcLedgerClient {
        IcrcLedgerClient::new(Configuration::get_ledger_canister())
    }

    #[inline]
    fn map_err((code, msg): (RejectionCode, String)) -> EkokeError {
        EkokeError::CanisterCall(code, msg)
    }
}
