use candid::{Encode, Nat, Principal};
use icrc::icrc1::account::Account;
use icrc::icrc2::allowance::{Allowance, AllowanceArgs};
use icrc::icrc2::approve::{ApproveArgs, ApproveError};
use icrc::icrc2::transfer_from::{TransferFromArgs, TransferFromError};

use crate::TestEnv;

pub struct IcrcLedgerClient<'a> {
    principal: Principal,
    env: &'a TestEnv,
}

impl<'a> IcrcLedgerClient<'a> {
    pub fn new(principal: Principal, env: &'a TestEnv) -> Self {
        Self { principal, env }
    }

    pub fn icrc1_fee(&self) -> Nat {
        self.env
            .query(
                self.principal,
                self.principal,
                "icrc1_fee",
                Encode!(&()).unwrap(),
            )
            .unwrap()
    }

    pub fn icrc1_balance_of(&self, account: Account) -> Nat {
        self.env
            .query(
                self.principal,
                account.owner,
                "icrc1_balance_of",
                Encode!(&account).unwrap(),
            )
            .unwrap()
    }

    pub fn icrc2_allowance(&self, account: Account, spender: Account) -> Allowance {
        self.env
            .query(
                self.principal,
                account.owner,
                "icrc2_allowance",
                Encode!(&AllowanceArgs { account, spender }).unwrap(),
            )
            .unwrap()
    }

    pub fn icrc2_approve(
        &self,
        caller: Principal,
        spender: Account,
        amount: Nat,
        from_subaccount: Option<[u8; 32]>,
    ) -> Result<Nat, ApproveError> {
        let args = ApproveArgs {
            spender,
            amount,
            from_subaccount,
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        };
        self.env
            .update(
                self.principal,
                caller,
                "icrc2_approve",
                Encode!(&args).unwrap(),
            )
            .unwrap()
    }

    pub fn icrc2_transfer_from(
        &self,
        caller: Principal,
        from: Account,
        to: Account,
        amount: Nat,
        spender_subaccount: Option<[u8; 32]>,
    ) -> Result<Nat, TransferFromError> {
        let args = TransferFromArgs {
            from,
            to,
            spender_subaccount,
            amount,
            fee: None,
            memo: None,
            created_at_time: None,
        };
        self.env
            .update(
                self.principal,
                caller,
                "icrc2_transfer_from",
                Encode!(&args).unwrap(),
            )
            .unwrap()
    }
}
