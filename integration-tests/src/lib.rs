//! # Integration tests
//!
//! This is a test module that is compiled as a separate crate for testing

pub mod actor;
pub mod client;
mod wasm;

use std::io::Read as _;
use std::path::PathBuf;
use std::vec;

use candid::{CandidType, Decode, Encode, Nat, Principal};
use did::deferred::DeferredInitData;
use did::ekoke::Ekoke;
use did::ekoke_erc20_swap::{EkokeErc20SwapInitData, EthNetwork};
use did::ekoke_liquidity_pool::EkokeLiquidityPoolInitData;
use did::ekoke_reward_pool::EkokeRewardPoolInitData;
use did::marketplace::MarketplaceInitData;
use did::H160;
use icrc::icrc1::account::Account;
use pocket_ic::common::rest::SubnetConfigSet;
use pocket_ic::{PocketIc, WasmResult};
use serde::de::DeserializeOwned;
use xrc::{Asset, AssetClass, ExchangeRate, ExchangeRateMetadata};

use self::wasm::Canister;
use crate::wasm::Icrc2InitArgs;

const DEFAULT_CYCLES: u128 = 2_000_000_000_000_000;

/// Test environment
pub struct TestEnv {
    pub pic: PocketIc,
    pub cketh_ledger_id: Principal,
    pub cketh_minter_id: Principal,
    pub deferred_id: Principal,
    pub ekoke_erc20_swap_id: Principal,
    pub ekoke_ledger_id: Principal,
    pub ekoke_liquidity_pool_id: Principal,
    pub ekoke_reward_pool_id: Principal,
    pub icp_ledger_id: Principal,
    pub marketplace_id: Principal,
    pub xrc_id: Principal,
}

impl TestEnv {
    pub fn query<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> anyhow::Result<R>
    where
        R: DeserializeOwned + CandidType,
    {
        let result = match self.pic.query_call(canister, caller, method, payload) {
            Ok(result) => result,
            Err(e) => anyhow::bail!("Error calling {}: {:?}", method, e),
        };
        let reply = match result {
            WasmResult::Reply(r) => r,
            WasmResult::Reject(r) => anyhow::bail!("{} was rejected: {:?}", method, r),
        };
        let ret_type = Decode!(&reply, R)?;

        Ok(ret_type)
    }

    pub fn update<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> anyhow::Result<R>
    where
        R: DeserializeOwned + CandidType,
    {
        let result = match self.pic.update_call(canister, caller, method, payload) {
            Ok(result) => result,
            Err(e) => anyhow::bail!("Error calling {}: {:?}", method, e),
        };

        let reply = match result {
            WasmResult::Reply(r) => r,
            WasmResult::Reject(r) => anyhow::bail!("{} was rejected: {:?}", method, r),
        };
        let ret_type = Decode!(&reply, R)?;

        Ok(ret_type)
    }

    /// Install the canisters needed for the tests
    pub fn init() -> TestEnv {
        let config = SubnetConfigSet {
            nns: true,
            sns: true,
            application: 1,
            ..Default::default()
        };
        let pic = PocketIc::from_config(config);

        // create canisters
        let icp_ledger_id = pic.create_canister();
        let cketh_ledger_id = pic.create_canister();
        let cketh_minter_id = pic.create_canister();
        let xrc_id = pic.create_canister();
        let deferred_id = pic.create_canister();
        let ekoke_erc20_swap_id = pic.create_canister();
        let ekoke_ledger_id = pic.create_canister();
        let ekoke_liquidity_pool_id = pic.create_canister();
        let ekoke_reward_pool_id = pic.create_canister();
        let marketplace_id = pic.create_canister();

        // install deferred canister
        Self::install_icrc2(
            &pic,
            icp_ledger_id,
            "ICP",
            "Internet Computer",
            8,
            vec![(
                Account::from(ekoke_liquidity_pool_id),
                1_000_000_000 * 100_000_000,
            )],
        );
        Self::install_icrc2(&pic, cketh_ledger_id, "ckETH", "ckETH", 18, vec![]);
        Self::install_deferred(
            &pic,
            deferred_id,
            ekoke_reward_pool_id,
            marketplace_id,
            icp_ledger_id,
            ekoke_liquidity_pool_id,
        );
        Self::install_xrc(&pic, xrc_id);
        Self::install_ekoke_erc20_swap(
            &pic,
            ekoke_erc20_swap_id,
            cketh_ledger_id,
            cketh_minter_id,
            ekoke_ledger_id,
        );
        Self::install_ekoke_ledger(&pic, ekoke_ledger_id, ekoke_reward_pool_id);
        Self::install_ekoke_liquidity_pool(
            &pic,
            ekoke_liquidity_pool_id,
            icp_ledger_id,
            deferred_id,
        );
        Self::install_ekoke_reward_pool(
            &pic,
            ekoke_reward_pool_id,
            deferred_id,
            ekoke_ledger_id,
            marketplace_id,
        );
        Self::install_marketplace(
            &pic,
            marketplace_id,
            deferred_id,
            ekoke_reward_pool_id,
            ekoke_liquidity_pool_id,
            xrc_id,
            icp_ledger_id,
        );

        TestEnv {
            pic,
            cketh_ledger_id,
            cketh_minter_id,
            deferred_id,
            icp_ledger_id,
            ekoke_erc20_swap_id,
            ekoke_reward_pool_id,
            ekoke_ledger_id,
            ekoke_liquidity_pool_id,
            marketplace_id,
            xrc_id,
        }
    }

    fn install_xrc(pic: &PocketIc, xrc_id: Principal) {
        pic.add_cycles(xrc_id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::Xrc);
        let eur = Asset {
            symbol: "EUR".to_string(),
            class: AssetClass::FiatCurrency,
        };
        let icp = Asset {
            symbol: "ICP".to_string(),
            class: AssetClass::Cryptocurrency,
        };
        let btc = Asset {
            symbol: "BTC".to_string(),
            class: AssetClass::Cryptocurrency,
        };

        let eur_icp = ExchangeRate {
            base_asset: eur,
            quote_asset: icp.clone(),
            rate: 813000000,
            timestamp: 0,
            metadata: ExchangeRateMetadata {
                decimals: 8,
                base_asset_num_queried_sources: 0,
                base_asset_num_received_rates: 0,
                quote_asset_num_queried_sources: 0,
                quote_asset_num_received_rates: 0,
                standard_deviation: 0,
                forex_timestamp: None,
            },
        };
        let icp_btc = ExchangeRate {
            base_asset: icp.clone(),
            quote_asset: btc,
            rate: 2162,
            timestamp: 0,
            metadata: ExchangeRateMetadata {
                decimals: 8,
                base_asset_num_queried_sources: 0,
                base_asset_num_received_rates: 0,
                quote_asset_num_queried_sources: 0,
                quote_asset_num_received_rates: 0,
                standard_deviation: 0,
                forex_timestamp: None,
            },
        };

        let init_arg = client::XrcxInitArgs {
            rates: vec![eur_icp, icp_btc],
        };
        let init_arg = Encode!(&init_arg).unwrap();

        pic.install_canister(xrc_id, wasm_bytes, init_arg, None);
    }

    fn install_icrc2(
        pic: &PocketIc,
        id: Principal,
        symbol: &str,
        name: &str,
        decimals: u8,
        initial_balances: Vec<(Account, u64)>,
    ) {
        let initial_balances = vec![
            (actor::alice_account(), 1_000_000_000_000_000_u64),
            (actor::bob_account(), 1_000_000_000_000_000_u64),
            (actor::charlie_account(), 1_000_000_000_000_000_u64),
        ]
        .into_iter()
        .chain(initial_balances)
        .map(|(account, balance)| (account, Nat::from(balance)))
        .collect::<Vec<_>>();

        pic.add_cycles(id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::Icrc2);
        let init_arg = Encode!(&Icrc2InitArgs {
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimals,
            fee: 10,
            logo: "https://ic0.app/img/logo.png".to_string(),
            minting_account: actor::minting_account(),
            total_supply: Nat::from(1_000_000_000_000_000_000_u64),
            accounts: initial_balances,
        })
        .unwrap();

        pic.install_canister(id, wasm_bytes, init_arg, None);
    }

    fn install_deferred(
        pic: &PocketIc,
        deferred_id: Principal,
        ekoke_reward_pool_id: Principal,
        marketplace_id: Principal,
        icp_ledger_canister: Principal,
        liquidity_pool_id: Principal,
    ) {
        pic.add_cycles(deferred_id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::Deferred);

        let init_arg = DeferredInitData {
            custodians: vec![actor::admin()],
            ekoke_reward_pool_canister: ekoke_reward_pool_id,
            icp_ledger_canister,
            liquidity_pool_canister: liquidity_pool_id,
            marketplace_canister: marketplace_id,
        };
        let init_arg = Encode!(&init_arg).unwrap();

        pic.install_canister(deferred_id, wasm_bytes, init_arg, None);
    }

    fn install_ekoke_erc20_swap(
        pic: &PocketIc,
        ekoke_erc20_swap_id: Principal,
        cketh_ledger_canister: Principal,
        cketh_minter_canister: Principal,
        ledger_id: Principal,
    ) {
        pic.add_cycles(ekoke_erc20_swap_id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::EkokeErc20Swap);

        let init_arg = EkokeErc20SwapInitData {
            admins: vec![actor::admin()],
            cketh_ledger_canister,
            cketh_minter_canister,
            ledger_id,
            erc20_bridge_address: H160::from_hex_str("0x2CE04Fd64DB0372F6fb4B7a542f0F9196feE5663")
                .unwrap(),
            erc20_gas_price: 39_000_000_000_u64, // 39 gwei
            erc20_network: EthNetwork::Sepolia,
        };

        let init_arg = Encode!(&init_arg).unwrap();

        pic.install_canister(ekoke_erc20_swap_id, wasm_bytes, init_arg, None);
    }

    fn install_ekoke_liquidity_pool(
        pic: &PocketIc,
        ekoke_liquidity_pool_id: Principal,
        icp_ledger_canister: Principal,
        deferred_id: Principal,
    ) {
        pic.add_cycles(ekoke_liquidity_pool_id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::EkokeLiquidityPool);

        let init_arg = EkokeLiquidityPoolInitData {
            admins: vec![actor::admin()],
            icp_ledger_canister,
            deferred_canister_id: deferred_id,
        };
        let init_arg = Encode!(&init_arg).unwrap();

        pic.install_canister(ekoke_liquidity_pool_id, wasm_bytes, init_arg, None);
    }

    fn install_ekoke_reward_pool(
        pic: &PocketIc,
        ekoke_reward_pool_id: Principal,
        deferred_id: Principal,
        ledger_id: Principal,
        marketplace_id: Principal,
    ) {
        pic.add_cycles(ekoke_reward_pool_id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::EkokeRewardPool);

        let init_arg = EkokeRewardPoolInitData {
            admins: vec![actor::admin()],
            deferred_canister: deferred_id,
            ledger_canister: ledger_id,
            marketplace_canister: marketplace_id,
        };
        let init_arg = Encode!(&init_arg).unwrap();

        pic.install_canister(ekoke_reward_pool_id, wasm_bytes, init_arg, None);
    }

    fn install_ekoke_ledger(
        pic: &PocketIc,
        ekoke_ledger_id: Principal,
        ekoke_reward_pool: Principal,
    ) {
        let total_supply = 888_010_101_000_000_u64;
        let alice_balance = ekoke_to_e8s(50_000);
        let reward_pool_balance =
            total_supply - alice_balance.clone() - alice_balance.clone() - alice_balance.clone();
        let ekoke_reward_pool_account = Account::from(ekoke_reward_pool);

        pic.add_cycles(ekoke_ledger_id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::Icrc2);
        let init_arg = Encode!(&Icrc2InitArgs {
            name: "EKOKE".to_string(),
            symbol: "EKOKE".to_string(),
            decimals: 8,
            fee: 1_000,
            logo: "https://ic0.app/img/logo.png".to_string(),
            minting_account: actor::minting_account(),
            total_supply: total_supply.into(),
            accounts: vec![
                (actor::alice_account(), alice_balance.clone()),
                (actor::bob_account(), alice_balance.clone()),
                (actor::charlie_account(), alice_balance.clone()),
                (ekoke_reward_pool_account, reward_pool_balance),
            ],
        })
        .unwrap();

        pic.install_canister(ekoke_ledger_id, wasm_bytes, init_arg, None);
    }

    fn install_marketplace(
        pic: &PocketIc,
        marketplace_id: Principal,
        deferred_id: Principal,
        ekoke_reward_pool_id: Principal,
        ekoke_liquidity_pool_id: Principal,
        xrc_canister: Principal,
        icp_ledger_canister: Principal,
    ) {
        pic.add_cycles(marketplace_id, DEFAULT_CYCLES);
        let wasm_bytes = Self::load_wasm(Canister::Marketplace);

        let init_arg = MarketplaceInitData {
            admins: vec![actor::admin()],
            deferred_canister: deferred_id,
            ekoke_reward_pool_canister: ekoke_reward_pool_id,
            ekoke_liquidity_pool_canister: ekoke_liquidity_pool_id,
            xrc_canister,
            icp_ledger_canister,
        };
        let init_arg = Encode!(&init_arg).unwrap();

        pic.install_canister(marketplace_id, wasm_bytes, init_arg, None);
    }

    fn load_wasm(canister: Canister) -> Vec<u8> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(canister.as_path());

        let mut file = std::fs::File::open(path).unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();

        wasm_bytes
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        // NOTE: execute test one by one
        for tempdir in std::fs::read_dir(std::path::Path::new("/tmp")).unwrap() {
            let tempdir = tempdir.unwrap();
            if tempdir.file_name().to_string_lossy().starts_with(".tmp") {
                std::fs::remove_dir_all(tempdir.path()).unwrap();
            }
        }
    }
}

pub fn ekoke_to_e8s(amount: u64) -> Ekoke {
    let amount = Nat::from(amount);
    let multiplier = Nat::from(100_000_000_u64);
    amount * multiplier
}
