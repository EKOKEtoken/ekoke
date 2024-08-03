use candid::{Nat, Principal};
use ic_cdk::api::call::CallResult;
use icrc_ledger_types::icrc1::account::Subaccount;
use icrc_ledger_types::icrc1::transfer::TransferError;
use icrc_ledger_types::icrc2::allowance::Allowance;
use icrc_ledger_types::icrc2::approve::ApproveError;
use icrc_ledger_types::icrc2::transfer_from::TransferFromError;

use crate::icrc1::account::Account;

/// Icrc ledger client
pub struct IcrcLedgerClient {
    principal: Principal,
}

impl From<Principal> for IcrcLedgerClient {
    fn from(principal: Principal) -> Self {
        Self::new(principal)
    }
}

impl IcrcLedgerClient {
    /// Create new icrc ledger client
    pub fn new(principal: Principal) -> Self {
        Self { principal }
    }

    pub fn principal(&self) -> Principal {
        self.principal
    }

    pub async fn icrc1_fee(&self) -> CallResult<Nat> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(1_000u64.into())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let result: (Nat,) = ic_cdk::call(self.principal, "icrc1_fee", ()).await?;

            Ok(result.0)
        }
    }

    /// Get account balance
    #[allow(unused_variables)]
    pub async fn icrc1_balance_of(&self, account: Account) -> CallResult<Nat> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(888_010_101_000_000_u64.into())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let result: (Nat,) =
                ic_cdk::call(self.principal, "icrc1_balance_of", (account,)).await?;

            Ok(result.0)
        }
    }

    /// Transfer tokens from `from` account to `to` account
    #[allow(unused_variables)]
    pub async fn icrc1_transfer(
        &self,
        to: Account,
        amount: Nat,
        from_subaccount: Option<Subaccount>,
    ) -> CallResult<Result<Nat, TransferError>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(Ok(amount))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let args = crate::icrc1::transfer::TransferArg {
                to,
                from_subaccount,
                fee: None,
                created_at_time: None,
                memo: None,
                amount,
            };
            let result: (Result<Nat, TransferError>,) =
                ic_cdk::call(self.principal, "icrc1_transfer", (args,)).await?;

            Ok(result.0)
        }
    }

    /// Get tokens allowance for account
    #[allow(unused_variables)]
    pub async fn icrc2_allowance(
        &self,
        spender: Account,
        account: Account,
    ) -> CallResult<Allowance> {
        #[cfg(not(target_arch = "wasm32"))]
        match account.subaccount {
            None => Ok(Allowance {
                allowance: 888_010_101_000_000_u64.into(),
                expires_at: None,
            }),
            Some(subaccount) if subaccount == [2; 32] => Ok(Allowance {
                allowance: 5_000_000_000_u64.into(),
                expires_at: Some(0),
            }),
            Some(subaccount) if subaccount == [3; 32] => Ok(Allowance {
                allowance: 0u64.into(),
                expires_at: Some(0),
            }),
            Some(_) => Ok(Allowance {
                allowance: 5_000_000_u64.into(),
                expires_at: None,
            }),
        }
        #[cfg(target_arch = "wasm32")]
        {
            let args = crate::icrc2::allowance::AllowanceArgs { spender, account };
            let allowance: (Allowance,) =
                ic_cdk::call(self.principal, "icrc2_allowance", (args,)).await?;

            Ok(allowance.0)
        }
    }

    /// Transfer tokens from `from` account to `to` account
    #[allow(unused_variables)]
    pub async fn icrc2_transfer_from(
        &self,
        spender_subaccount: Option<[u8; 32]>,
        from: Account,
        to: Account,
        amount: Nat,
    ) -> CallResult<Result<Nat, TransferFromError>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(Ok(amount))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let args = crate::icrc2::transfer_from::TransferFromArgs {
                spender_subaccount,
                from,
                to,
                amount,
                fee: None,
                memo: None,
                created_at_time: None,
            };
            let result: (Result<Nat, TransferFromError>,) =
                ic_cdk::call(self.principal, "icrc2_transfer_from", (args,)).await?;

            Ok(result.0)
        }
    }

    /// Approve tokens transfer
    #[allow(unused_variables)]
    pub async fn icrc2_approve(
        &self,
        spender: Account,
        amount: Nat,
        from_subaccount: Option<[u8; 32]>,
    ) -> CallResult<Result<Nat, ApproveError>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(Ok(amount))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let args = icrc_ledger_types::icrc2::approve::ApproveArgs {
                spender,
                amount,
                from_subaccount,
                expected_allowance: None,
                expires_at: None,
                fee: None,
                memo: None,
                created_at_time: None,
            };
            let result: (Result<Nat, ApproveError>,) =
                ic_cdk::call(self.principal, "icrc2_approve", (args,)).await?;

            Ok(result.0)
        }
    }
}
