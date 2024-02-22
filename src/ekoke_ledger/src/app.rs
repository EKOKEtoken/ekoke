//! # App
//!
//! API implementation for deferred canister

mod archive_canister;
mod balance;
mod configuration;
mod inspect;
mod liquidity_pool;
mod memory;
mod pool;
mod reward;
mod roles;
mod spend_allowance;
#[cfg(test)]
mod test_utils;

use async_trait::async_trait;
use candid::{Nat, Principal};
use did::ekoke::{
    AllowanceError, BalanceError, EkokeError, EkokeInitData, EkokeResult, LiquidityPoolAccounts,
    LiquidityPoolBalance, PicoEkoke, PoolError, Role,
};
use did::ekoke_index::{Approve, Transaction, Transfer};
use did::ID;
use icrc::icrc::generic_metadata_value::MetadataValue;
use icrc::icrc1::account::Account;
use icrc::icrc1::{self, transfer as icrc1_transfer, Icrc1};
use icrc::icrc2::{self, Icrc2};

use self::archive_canister::ArchiveCanister;
use self::balance::Balance;
use self::configuration::Configuration;
pub use self::inspect::Inspect;
use self::liquidity_pool::LiquidityPool;
use self::pool::Pool;
use self::reward::Reward;
use self::roles::RolesManager;
use self::spend_allowance::SpendAllowance;
use crate::constants::{ICRC1_DECIMALS, ICRC1_FEE, ICRC1_LOGO, ICRC1_NAME, ICRC1_SYMBOL};
use crate::utils::{self, caller, time};

pub struct EkokeCanister;

impl EkokeCanister {
    /// Init ekoke canister
    pub fn init(data: EkokeInitData) {
        // Set minting account
        Configuration::set_minting_account(data.minting_account);
        // set swap account
        Configuration::set_swap_account(data.swap_account);
        // set canisters
        Configuration::set_xrc_canister(data.xrc_canister);
        Configuration::set_ckbtc_canister(data.ckbtc_canister);
        Configuration::set_icp_ledger_canister(data.icp_ledger_canister);
        Configuration::set_archive_canister(data.archive_canister);
        // init liquidity pool
        LiquidityPool::init();
        // set roles
        if let Err(err) = RolesManager::set_admins(data.admins) {
            ic_cdk::trap(&format!("Error setting admins: {}", err));
        }
        // Set deferred canister
        RolesManager::give_role(data.deferred_canister, Role::DeferredCanister);
        // set marketplace canister
        RolesManager::give_role(data.marketplace_canister, Role::MarketplaceCanister);
        // init balances
        Balance::init_balances(data.total_supply, data.initial_balances);
        // set timers
        Self::set_timers();
    }

    pub fn post_upgrade() {
        Self::set_timers();
    }

    /// Set application timers
    fn set_timers() {
        #[cfg(target_family = "wasm")]
        async fn swap_icp_to_btc_timer() {
            let xrc_principal = Configuration::get_xrc_canister();
            let _ = LiquidityPool::swap_icp_to_btc(xrc_principal).await;
        }

        // Expired spend allowance timers
        #[cfg(target_family = "wasm")]
        ic_cdk_timers::set_timer_interval(
            crate::constants::SPEND_ALLOWANCE_EXPIRED_ALLOWANCE_TIMER_INTERVAL,
            SpendAllowance::remove_expired_allowance,
        );
        // Liquidity pool ICP -> BTC swap timer
        #[cfg(target_family = "wasm")]
        ic_cdk_timers::set_timer_interval(crate::constants::LIQUIDITY_POOL_SWAP_INTERVAL, || {
            ic_cdk::spawn(swap_icp_to_btc_timer());
        });
    }

    /// Reserve a pool for the provided contract ID with the provided amount of $picoEkoke tokens.
    ///
    /// The tokens are withdrawned from the from's wallet.
    /// Obviously `from` wallet must be owned by the caller.
    pub async fn reserve_pool(
        contract_id: ID,
        picoekoke_amount: PicoEkoke,
        from_subaccount: Option<[u8; 32]>,
    ) -> EkokeResult<PicoEkoke> {
        let from_account = Account {
            owner: utils::caller(),
            subaccount: from_subaccount,
        };

        Pool::reserve(&contract_id, from_account, picoekoke_amount).await
    }

    /// Get liquidity pool balance from the different ledgers
    pub async fn liquidity_pool_balance() -> EkokeResult<LiquidityPoolBalance> {
        LiquidityPool::balance().await
    }

    /// Get liquidity pool accounts
    pub fn liquidity_pool_accounts() -> LiquidityPoolAccounts {
        LiquidityPool::accounts()
    }

    /// Send reward to buyer reducing the balance from the pool associated to the contract, for the value of picoEkoke
    pub async fn send_reward(
        contract_id: ID,
        picoekoke: PicoEkoke,
        buyer: Account,
    ) -> EkokeResult<()> {
        if !Inspect::inspect_is_marketplace_canister(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }

        if !Inspect::inspect_pool_exists(&contract_id) {
            return Err(EkokeError::Pool(PoolError::PoolNotFound(contract_id)));
        }

        Pool::withdraw_tokens(&contract_id, buyer, picoekoke).await?;

        Ok(())
    }

    /// Get contract reward.
    ///
    /// This method can be called only by the deferred canister.
    ///
    /// If a pool is already reserved for the provided contract ID, the reserved amount will be returned.
    /// Otherwise, the provided amount will be reserved from canister wallet, if possible and returned.
    ///
    /// If the canister wallet doesn't have enough tokens to reserve `InsufficientBalance` error is returned
    pub async fn get_contract_reward(contract_id: ID, installments: u64) -> EkokeResult<PicoEkoke> {
        if !Inspect::inspect_is_deferred_canister(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }

        Reward::get_contract_reward(contract_id, installments).await
    }

    // # admin methods

    /// Set role to the provided principal
    pub fn admin_set_role(principal: Principal, role: Role) {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        RolesManager::give_role(principal, role)
    }

    /// Remove role from the provided principal
    pub fn admin_remove_role(principal: Principal, role: Role) -> EkokeResult<()> {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        RolesManager::remove_role(principal, role)
    }

    /// Returns cycles
    pub fn admin_cycles() -> Nat {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        utils::cycles()
    }

    /// Burn an arbitrary amount of tokens.
    /// This method transfers `amount` tokens from the canister account to the minting account.
    pub fn admin_burn(amount: PicoEkoke) -> EkokeResult<()> {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        Balance::burn(amount)
    }

    /// Set swap account
    pub fn admin_set_swap_account(account: Account) {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        Configuration::set_swap_account(account);
    }

    /// Set xrc canister
    pub fn admin_set_xrc_canister(canister_id: Principal) {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        Configuration::set_xrc_canister(canister_id);
    }

    /// Set ckbtc canister
    pub fn admin_set_ckbtc_canister(canister_id: Principal) {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        Configuration::set_ckbtc_canister(canister_id);
    }

    /// Set icp ledger canister
    pub fn admin_set_icp_ledger_canister(canister_id: Principal) {
        if !Inspect::inspect_is_admin(utils::caller()) {
            ic_cdk::trap("Unauthorized");
        }
        Configuration::set_icp_ledger_canister(canister_id);
    }
}

#[async_trait]
impl Icrc1 for EkokeCanister {
    fn icrc1_name() -> &'static str {
        ICRC1_NAME
    }

    fn icrc1_symbol() -> &'static str {
        ICRC1_SYMBOL
    }

    fn icrc1_decimals() -> u8 {
        ICRC1_DECIMALS
    }

    fn icrc1_fee() -> Nat {
        ICRC1_FEE.into()
    }

    fn icrc1_metadata() -> Vec<(String, MetadataValue)> {
        vec![
            (
                "icrc1:symbol".to_string(),
                MetadataValue::from(ICRC1_SYMBOL),
            ),
            ("icrc1:name".to_string(), MetadataValue::from(ICRC1_NAME)),
            (
                "icrc1:decimals".to_string(),
                MetadataValue::from(Nat::from(ICRC1_DECIMALS)),
            ),
            (
                "icrc1:fee".to_string(),
                MetadataValue::from(Nat::from(ICRC1_FEE)),
            ),
            ("icrc1:logo".to_string(), MetadataValue::from(ICRC1_LOGO)),
        ]
    }

    fn icrc1_total_supply() -> Nat {
        Balance::total_supply()
    }

    fn icrc1_minting_account() -> Account {
        Configuration::get_minting_account()
    }

    fn icrc1_balance_of(account: Account) -> Nat {
        Balance::balance_of(account).unwrap_or_default()
    }

    async fn icrc1_transfer(
        transfer_args: icrc1_transfer::TransferArg,
    ) -> Result<Nat, icrc1_transfer::TransferError> {
        // get fee and check if fee is at least ICRC1_FEE
        Inspect::inspect_transfer(&transfer_args)?;
        let fee = transfer_args.fee.clone().unwrap_or(ICRC1_FEE.into());

        // get from account
        let from_account = Account {
            owner: utils::caller(),
            subaccount: transfer_args.from_subaccount,
        };

        // check if it is a burn
        if transfer_args.to == Self::icrc1_minting_account() {
            Balance::transfer_wno_fees(from_account, transfer_args.to, transfer_args.amount.clone())
        } else {
            // make transfer
            Balance::transfer(
                from_account,
                transfer_args.to,
                transfer_args.amount.clone(),
                fee.clone(),
            )
        }
        .map_err(|err| match err {
            EkokeError::Balance(BalanceError::InsufficientBalance) => {
                icrc1_transfer::TransferError::InsufficientFunds {
                    balance: Self::icrc1_balance_of(from_account),
                }
            }
            _ => icrc1_transfer::TransferError::GenericError {
                error_code: Nat::from(3_u64),
                message: err.to_string(),
            },
        })?;

        // register transaction
        let tx = Transaction {
            kind: "transfer".to_string(),
            mint: None,
            burn: None,
            transfer: Some(Transfer {
                from: from_account,
                to: transfer_args.to,
                amount: transfer_args.amount,
                fee: transfer_args.fee,
                memo: transfer_args.memo,
                created_at_time: transfer_args.created_at_time,
                spender: None,
            }),
            approve: None,
            timestamp: time(),
        };
        ArchiveCanister::commit(tx)
            .await
            .map_err(|_| icrc1_transfer::TransferError::GenericError {
                error_code: Nat::from(4_u64),
                message: "failed to register transaction".to_string(),
            })
            .map(Nat::from)
    }

    fn icrc1_supported_standards() -> Vec<icrc1::TokenExtension> {
        vec![
            icrc1::TokenExtension::icrc1(),
            icrc1::TokenExtension::icrc2(),
        ]
    }
}

#[async_trait]
impl Icrc2 for EkokeCanister {
    async fn icrc2_approve(
        args: icrc2::approve::ApproveArgs,
    ) -> Result<Nat, icrc2::approve::ApproveError> {
        Inspect::inspect_icrc2_approve(caller(), &args)?;

        let caller_account = Account {
            owner: caller(),
            subaccount: args.from_subaccount,
        };

        let current_allowance = SpendAllowance::get_allowance(caller_account, args.spender).0;

        // pay fee
        let fee = args.fee.clone().unwrap_or(ICRC1_FEE.into());
        Balance::transfer_wno_fees(caller_account, Configuration::get_minting_account(), fee)
            .map_err(|_| icrc2::approve::ApproveError::InsufficientFunds {
                balance: Self::icrc1_balance_of(caller_account),
            })?;

        // approve spend
        let amount = match SpendAllowance::approve_spend(caller(), args.clone()) {
            Ok(amount) => Ok(amount),
            Err(EkokeError::Allowance(AllowanceError::AllowanceChanged)) => {
                Err(icrc2::approve::ApproveError::AllowanceChanged { current_allowance })
            }
            Err(EkokeError::Allowance(AllowanceError::BadExpiration)) => {
                Err(icrc2::approve::ApproveError::TooOld)
            }
            Err(err) => Err(icrc2::approve::ApproveError::GenericError {
                error_code: 0_u64.into(),
                message: err.to_string(),
            }),
        }?;

        // register transaction
        let tx = Transaction {
            kind: "approve".to_string(),
            mint: None,
            burn: None,
            transfer: None,
            approve: Some(Approve {
                amount,
                from: caller_account,
                spender: Some(args.spender),
                expected_allowance: args.expected_allowance,
                expires_at: args.expires_at,
                memo: args.memo,
                created_at_time: args.created_at_time,
                fee: args.fee,
            }),
            timestamp: time(),
        };
        ArchiveCanister::commit(tx)
            .await
            .map_err(|_| icrc2::approve::ApproveError::GenericError {
                error_code: Nat::from(4_u64),
                message: "failed to register transaction".to_string(),
            })
            .map(Nat::from)
    }

    async fn icrc2_transfer_from(
        args: icrc2::transfer_from::TransferFromArgs,
    ) -> Result<Nat, icrc2::transfer_from::TransferFromError> {
        Inspect::inspect_icrc2_transfer_from(&args)?;

        // check if owner has enough balance + fee
        let owner_balance = Self::icrc1_balance_of(args.from);
        let fee = args.fee.clone().unwrap_or(ICRC1_FEE.into());
        let total_amount = args.amount.clone() + fee.clone();
        if owner_balance < total_amount {
            return Err(icrc2::transfer_from::TransferFromError::InsufficientFunds {
                balance: owner_balance,
            });
        }

        // check if spender has fee
        let spender = Account {
            owner: caller(),
            subaccount: args.spender_subaccount,
        };

        // check allowance
        let (allowance, expires_at) = SpendAllowance::get_allowance(args.from, spender);
        if allowance < total_amount {
            return Err(
                icrc2::transfer_from::TransferFromError::InsufficientAllowance { allowance },
            );
        }

        // check if has expired
        if expires_at.is_some() && expires_at.unwrap() < utils::time() {
            return Err(icrc2::transfer_from::TransferFromError::TooOld);
        }

        // spend allowance
        match SpendAllowance::spend_allowance(
            caller(),
            args.from,
            total_amount.clone(),
            args.spender_subaccount,
        ) {
            Ok(()) => Ok(()),
            Err(EkokeError::Allowance(AllowanceError::InsufficientFunds)) => {
                Err(icrc2::transfer_from::TransferFromError::InsufficientAllowance { allowance })
            }
            Err(EkokeError::Allowance(AllowanceError::AllowanceExpired)) => {
                Err(icrc2::transfer_from::TransferFromError::TooOld)
            }
            Err(e) => Err(icrc2::transfer_from::TransferFromError::GenericError {
                error_code: 0_u64.into(),
                message: e.to_string(),
            }),
        }?;

        // pay fee (the fee is paid by from account, not by spender account)
        Balance::transfer_wno_fees(args.from, Configuration::get_minting_account(), fee.clone())
            .map_err(
                |_| icrc2::transfer_from::TransferFromError::InsufficientFunds {
                    balance: Self::icrc1_balance_of(args.from),
                },
            )?;

        // transfer from `from` balance to `to` balance
        Balance::transfer_wno_fees(args.from, args.to, args.amount.clone()).map_err(|_| {
            icrc2::transfer_from::TransferFromError::InsufficientFunds {
                balance: Self::icrc1_balance_of(args.from),
            }
        })?;

        // register transaction
        let tx = Transaction {
            kind: "transfer".to_string(),
            mint: None,
            burn: None,
            transfer: Some(Transfer {
                from: args.from,
                to: args.to,
                amount: total_amount,
                fee: args.fee,
                memo: args.memo,
                spender: Some(Account {
                    owner: caller(),
                    subaccount: args.spender_subaccount,
                }),
                created_at_time: args.created_at_time,
            }),
            approve: None,
            timestamp: time(),
        };
        ArchiveCanister::commit(tx)
            .await
            .map_err(|_| icrc2::transfer_from::TransferFromError::GenericError {
                error_code: Nat::from(4_u64),
                message: "failed to register transaction".to_string(),
            })
            .map(Nat::from)
    }

    fn icrc2_allowance(args: icrc2::allowance::AllowanceArgs) -> icrc2::allowance::Allowance {
        let (allowance, expires_at) = SpendAllowance::get_allowance(args.account, args.spender);
        icrc2::allowance::Allowance {
            allowance,
            expires_at,
        }
    }
}

#[cfg(test)]
mod test {

    use std::str::FromStr as _;

    use icrc::icrc1::transfer::TransferArg;
    use icrc::icrc2::allowance::{Allowance, AllowanceArgs};
    use icrc::icrc2::approve::ApproveArgs;
    use icrc::icrc2::transfer_from::TransferFromArgs;
    use pretty_assertions::{assert_eq, assert_ne};

    use super::test_utils::{alice_account, bob_account, caller_account, ekoke_to_picoekoke};
    use super::*;
    use crate::app::test_utils::bob;
    use crate::constants::ICRC1_TX_TIME_SKID;
    use crate::utils::caller;

    #[tokio::test]
    async fn test_should_init_canister() {
        init_canister();

        assert_ne!(
            Configuration::get_minting_account().owner,
            Principal::anonymous()
        );
        assert_eq!(RolesManager::get_admins(), vec![caller()]);
        assert!(RolesManager::has_role(caller(), Role::DeferredCanister));
        // init balance
        assert_eq!(
            Balance::balance_of(alice_account()).unwrap(),
            ekoke_to_picoekoke(50_000)
        );
        assert_eq!(
            Balance::balance_of(bob_account()).unwrap(),
            ekoke_to_picoekoke(50_000)
        );
        assert_eq!(
            Balance::balance_of(caller_account()).unwrap(),
            ekoke_to_picoekoke(100_000)
        );
        // supply
        assert_eq!(
            Balance::balance_of(Balance::canister_wallet_account()).unwrap(),
            ekoke_to_picoekoke(8_688_888)
        );

        // liquidity pool
        assert_eq!(LiquidityPool::accounts().ckbtc.owner, utils::id());
        assert!(LiquidityPool::accounts().ckbtc.subaccount.is_none());

        // swap account
        assert_eq!(Configuration::get_swap_account(), bob_account());

        // check canisters
        assert_eq!(Configuration::get_xrc_canister(), caller());
        assert_eq!(Configuration::get_ckbtc_canister(), caller());
        assert_eq!(Configuration::get_icp_ledger_canister(), caller());
        assert_eq!(Configuration::get_archive_canister(), caller());
    }

    #[tokio::test]
    async fn test_should_reserve_pool() {
        init_canister();
        let contract_id = 1_u64.into();
        let picoekoke_amount: Nat = 1000_u64.into();

        let result = EkokeCanister::reserve_pool(
            contract_id,
            picoekoke_amount.clone(),
            test_utils::caller_account().subaccount,
        )
        .await;

        assert_eq!(result, Ok(picoekoke_amount));
    }

    #[tokio::test]
    async fn test_should_not_allow_reserve_pool() {
        init_canister();
        let contract_id = 1_u64.into();
        let picoekoke_amount = 1000_u64.into();

        assert!(EkokeCanister::reserve_pool(
            contract_id,
            picoekoke_amount,
            test_utils::bob_account().subaccount,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn test_should_send_reward() {
        init_canister();
        let contract_id: ID = 1_u64.into();

        let picoekoke_amount: Nat = 1000_u64.into();

        let result = EkokeCanister::reserve_pool(
            contract_id.clone(),
            picoekoke_amount.clone(),
            test_utils::caller_account().subaccount,
        )
        .await;

        assert_eq!(result, Ok(picoekoke_amount));

        // send reward to bob
        assert!(
            EkokeCanister::send_reward(contract_id, 500_u64.into(), bob_account())
                .await
                .is_ok()
        );
        assert_eq!(
            Balance::balance_of(bob_account()).unwrap(),
            ekoke_to_picoekoke(50_000) + 500_u64
        );
    }

    #[tokio::test]
    async fn test_should_not_send_reward() {
        init_canister();
        let contract_id: ID = 1_u64.into();

        let picoekoke_amount: Nat = 1000_u64.into();

        let result = EkokeCanister::reserve_pool(
            contract_id.clone(),
            picoekoke_amount.clone(),
            test_utils::caller_account().subaccount,
        )
        .await;

        assert_eq!(result, Ok(picoekoke_amount));

        // send reward to bob
        assert!(
            EkokeCanister::send_reward(contract_id, 5000_u64.into(), bob_account())
                .await
                .is_err()
        );
        assert!(
            EkokeCanister::send_reward(2_u64.into(), 500_u64.into(), bob_account())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_should_set_role() {
        init_canister();
        let principal = Principal::management_canister();
        let role = Role::Admin;
        EkokeCanister::admin_set_role(principal, role);
        assert!(RolesManager::is_admin(principal));
    }

    #[tokio::test]
    async fn test_should_remove_role() {
        init_canister();
        let principal = Principal::management_canister();
        let role = Role::Admin;
        EkokeCanister::admin_set_role(principal, role);
        assert!(RolesManager::is_admin(principal));
        EkokeCanister::admin_remove_role(principal, role).unwrap();
        assert!(!RolesManager::is_admin(principal));
    }

    #[tokio::test]
    async fn test_should_get_cycles() {
        init_canister();
        assert_eq!(EkokeCanister::admin_cycles(), utils::cycles());
    }

    #[tokio::test]
    async fn test_should_burn() {
        init_canister();
        let canister_balance = Balance::canister_balance();
        let amount = ekoke_to_picoekoke(1000);
        assert!(EkokeCanister::admin_burn(amount.clone()).is_ok());
        assert_eq!(Balance::canister_balance(), canister_balance - amount);
        assert_eq!(
            Balance::total_supply(),
            ekoke_to_picoekoke(8_888_888 - 1000)
        );
    }

    #[tokio::test]
    async fn test_should_get_name() {
        init_canister();
        assert_eq!(EkokeCanister::icrc1_name(), ICRC1_NAME);
    }

    #[tokio::test]
    async fn test_should_get_symbol() {
        init_canister();
        assert_eq!(EkokeCanister::icrc1_symbol(), ICRC1_SYMBOL);
    }

    #[tokio::test]
    async fn test_should_get_decimals() {
        init_canister();
        assert_eq!(EkokeCanister::icrc1_decimals(), ICRC1_DECIMALS);
    }

    #[tokio::test]
    async fn test_should_get_fee() {
        init_canister();
        assert_eq!(EkokeCanister::icrc1_fee(), Nat::from(ICRC1_FEE));
    }

    #[tokio::test]
    async fn test_should_get_metadata() {
        init_canister();
        let metadata = EkokeCanister::icrc1_metadata();
        assert_eq!(metadata.len(), 5);
        assert_eq!(
            metadata.get(0).unwrap(),
            &(
                "icrc1:symbol".to_string(),
                MetadataValue::from(ICRC1_SYMBOL)
            )
        );
        assert_eq!(
            metadata.get(1).unwrap(),
            &("icrc1:name".to_string(), MetadataValue::from(ICRC1_NAME))
        );
        assert_eq!(
            metadata.get(2).unwrap(),
            &(
                "icrc1:decimals".to_string(),
                MetadataValue::from(Nat::from(ICRC1_DECIMALS))
            )
        );
        assert_eq!(
            metadata.get(3).unwrap(),
            &(
                "icrc1:fee".to_string(),
                MetadataValue::from(Nat::from(ICRC1_FEE))
            )
        );
        assert_eq!(
            metadata.get(4).unwrap(),
            &("icrc1:logo".to_string(), MetadataValue::from(ICRC1_LOGO))
        );
    }

    #[tokio::test]
    async fn test_should_get_total_supply() {
        init_canister();
        assert_eq!(
            EkokeCanister::icrc1_total_supply(),
            Nat::from(ekoke_to_picoekoke(8_888_888))
        );
    }

    #[tokio::test]
    async fn test_should_get_minting_account() {
        init_canister();
        assert_eq!(
            EkokeCanister::icrc1_minting_account(),
            Configuration::get_minting_account()
        );
    }

    #[test]
    fn test_should_set_xrc_canister() {
        init_canister();
        let canister_id = Principal::from_str("aaaaa-aa").unwrap();
        EkokeCanister::admin_set_xrc_canister(canister_id);
        assert_eq!(Configuration::get_xrc_canister(), canister_id);
    }

    #[test]
    fn test_should_set_ckbtc_canister() {
        init_canister();
        let canister_id = Principal::from_str("aaaaa-aa").unwrap();
        EkokeCanister::admin_set_ckbtc_canister(canister_id);
        assert_eq!(Configuration::get_ckbtc_canister(), canister_id);
    }

    #[test]
    fn test_should_set_icp_ledger_canister() {
        init_canister();
        let canister_id = Principal::from_str("aaaaa-aa").unwrap();
        EkokeCanister::admin_set_icp_ledger_canister(canister_id);
        assert_eq!(Configuration::get_icp_ledger_canister(), canister_id);
    }

    #[tokio::test]
    async fn test_should_get_balance_of() {
        init_canister();
        assert_eq!(
            EkokeCanister::icrc1_balance_of(alice_account()),
            Nat::from(ekoke_to_picoekoke(50_000))
        );
        assert_eq!(
            EkokeCanister::icrc1_balance_of(bob_account()),
            Nat::from(ekoke_to_picoekoke(50_000))
        );
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            Nat::from(ekoke_to_picoekoke(100_000))
        );
        assert_eq!(
            EkokeCanister::icrc1_balance_of(Account {
                owner: utils::id(),
                subaccount: Some(utils::random_subaccount().await),
            }),
            Nat::from(0_u64)
        );
    }

    #[tokio::test]
    async fn test_should_transfer() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(EkokeCanister::icrc1_transfer(transfer_args).await.is_ok());
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            Nat::from(ekoke_to_picoekoke(90_000) - ICRC1_FEE)
        );
        assert_eq!(
            EkokeCanister::icrc1_balance_of(bob_account()),
            Nat::from(ekoke_to_picoekoke(60_000))
        );
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_bad_time() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(0),
            memo: None,
        };
        assert!(matches!(
            EkokeCanister::icrc1_transfer(transfer_args)
                .await
                .unwrap_err(),
            icrc1_transfer::TransferError::TooOld { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_old_time() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time() - (ICRC1_TX_TIME_SKID.as_nanos() as u64 * 2)),
            memo: None,
        };
        assert!(matches!(
            EkokeCanister::icrc1_transfer(transfer_args)
                .await
                .unwrap_err(),
            icrc1_transfer::TransferError::TooOld { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_time_in_future() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time() + (ICRC1_TX_TIME_SKID.as_nanos() as u64 * 2)),
            memo: None,
        };
        assert!(matches!(
            EkokeCanister::icrc1_transfer(transfer_args)
                .await
                .unwrap_err(),
            icrc1_transfer::TransferError::CreatedInFuture { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_bad_fee() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ICRC1_FEE / 2)),
            created_at_time: Some(utils::time()),
            memo: None,
        };

        assert!(matches!(
            EkokeCanister::icrc1_transfer(transfer_args)
                .await
                .unwrap_err(),
            icrc1_transfer::TransferError::BadFee { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_transfer_with_null_fee() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(EkokeCanister::icrc1_transfer(transfer_args).await.is_ok());
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            Nat::from(ekoke_to_picoekoke(90_000) - ICRC1_FEE)
        );
    }

    #[tokio::test]
    async fn test_should_transfer_with_higher_fee() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ICRC1_FEE * 2)),
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(EkokeCanister::icrc1_transfer(transfer_args).await.is_ok());
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            Nat::from(ekoke_to_picoekoke(90_000) - (ICRC1_FEE * 2))
        );
    }

    #[tokio::test]
    async fn test_should_not_allow_bad_memo() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: Some("9888".as_bytes().to_vec().into()),
        };

        assert!(matches!(
            EkokeCanister::icrc1_transfer(transfer_args)
                .await
                .unwrap_err(),
            icrc1_transfer::TransferError::GenericError { .. }
        ));

        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: Some("988898889888988898889888988898889888988898889888988898889888988898889888988898889888988898889888".as_bytes().to_vec().into()),
        };

        assert!(matches!(
            EkokeCanister::icrc1_transfer(transfer_args)
                .await
                .unwrap_err(),
            icrc1_transfer::TransferError::GenericError { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_transfer_with_memo() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time()),
            memo: Some(
                "293458234690283506958436839246024563"
                    .to_string()
                    .as_bytes()
                    .to_vec()
                    .into(),
            ),
        };
        assert!(EkokeCanister::icrc1_transfer(transfer_args).await.is_ok());
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            Nat::from(ekoke_to_picoekoke(90_000) - ICRC1_FEE)
        );
        assert_eq!(
            EkokeCanister::icrc1_balance_of(bob_account()),
            Nat::from(ekoke_to_picoekoke(60_000))
        );
    }

    #[tokio::test]
    async fn test_should_burn_from_transfer() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: EkokeCanister::icrc1_minting_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(EkokeCanister::icrc1_transfer(transfer_args).await.is_ok());
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            Nat::from(ekoke_to_picoekoke(90_000))
        );
        assert_eq!(
            EkokeCanister::icrc1_total_supply(),
            Nat::from(ekoke_to_picoekoke(8_888_888 - 10_000))
        );
    }

    #[tokio::test]
    async fn test_should_get_supported_extensions() {
        init_canister();
        let extensions = EkokeCanister::icrc1_supported_standards();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            extensions.get(0).unwrap().name,
            icrc1::TokenExtension::icrc1().name
        );
        assert_eq!(
            extensions.get(1).unwrap().name,
            icrc1::TokenExtension::icrc2().name
        );
    }

    #[tokio::test]
    async fn test_should_approve_spending() {
        init_canister();
        let approval_args = ApproveArgs {
            from_subaccount: caller_account().subaccount,
            spender: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: None,
            expires_at: None,
            expected_allowance: None,
            memo: None,
            created_at_time: None,
        };

        assert!(EkokeCanister::icrc2_approve(approval_args).await.is_ok());
        // check allowance
        assert_eq!(
            EkokeCanister::icrc2_allowance(AllowanceArgs {
                account: caller_account(),
                spender: bob_account(),
            }),
            Allowance {
                allowance: Nat::from(ekoke_to_picoekoke(10_000)),
                expires_at: None,
            }
        );
        // check we have paid fee
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            ekoke_to_picoekoke(100_000) - ICRC1_FEE
        );
    }

    #[tokio::test]
    async fn test_should_not_approve_spending_if_we_cannot_pay_fee() {
        init_canister();
        let approval_args = ApproveArgs {
            from_subaccount: caller_account().subaccount,
            spender: bob_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: Some(Nat::from(ekoke_to_picoekoke(110_000))),
            expires_at: None,
            expected_allowance: None,
            memo: None,
            created_at_time: None,
        };

        assert!(EkokeCanister::icrc2_approve(approval_args).await.is_err());
    }

    #[tokio::test]
    async fn test_should_spend_approved_amount() {
        init_canister();
        let fee = Nat::from(ICRC1_FEE);
        let approval_args = ApproveArgs {
            from_subaccount: bob_account().subaccount,
            spender: caller_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000) + fee.clone()),
            fee: None,
            expires_at: None,
            expected_allowance: None,
            memo: None,
            created_at_time: None,
        };
        assert!(SpendAllowance::approve_spend(bob(), approval_args).is_ok());
        assert_eq!(
            EkokeCanister::icrc2_allowance(AllowanceArgs {
                account: bob_account(),
                spender: caller_account(),
            }),
            Allowance {
                allowance: Nat::from(ekoke_to_picoekoke(10_000) + fee.clone()),
                expires_at: None,
            }
        );

        // spend
        assert!(EkokeCanister::icrc2_transfer_from(TransferFromArgs {
            spender_subaccount: caller_account().subaccount,
            from: bob_account(),
            to: alice_account(),
            amount: Nat::from(ekoke_to_picoekoke(10_000)),
            fee: None,
            memo: None,
            created_at_time: None,
        })
        .await
        .is_ok());
        // verify balance
        assert_eq!(
            EkokeCanister::icrc1_balance_of(bob_account()),
            Nat::from(ekoke_to_picoekoke(40_000) - fee)
        );
        assert_eq!(
            EkokeCanister::icrc1_balance_of(alice_account()),
            Nat::from(ekoke_to_picoekoke(60_000))
        );
        assert_eq!(
            EkokeCanister::icrc1_balance_of(caller_account()),
            Nat::from(ekoke_to_picoekoke(100_000))
        );
        // verify allowance
        assert_eq!(
            EkokeCanister::icrc2_allowance(AllowanceArgs {
                account: bob_account(),
                spender: caller_account(),
            }),
            Allowance {
                allowance: Nat::from(ekoke_to_picoekoke(0)),
                expires_at: None,
            }
        );
    }

    fn init_canister() {
        let data = EkokeInitData {
            admins: vec![caller()],
            total_supply: ekoke_to_picoekoke(8_888_888),
            deferred_canister: caller(),
            marketplace_canister: caller(),
            archive_canister: caller(),
            swap_account: bob_account(),
            minting_account: test_utils::minting_account(),
            initial_balances: vec![
                (alice_account(), ekoke_to_picoekoke(50_000)),
                (bob_account(), ekoke_to_picoekoke(50_000)),
                (caller_account(), ekoke_to_picoekoke(100_000)),
            ],
            xrc_canister: caller(),
            ckbtc_canister: caller(),
            icp_ledger_canister: caller(),
        };
        EkokeCanister::init(data);
    }
}
