use std::cell::RefCell;

use candid::Principal;
use did::deferred::{DeferredError, DeferredResult};
use did::StorablePrincipal;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};

use crate::app::memory::{
    CREATED_AT_MEMORY_ID, EKOKE_REWARD_POOL_CANISTER_MEMORY_ID, ICP_LEDGER_CANISTER_MEMORY_ID,
    LIQUIDITY_POOL_CANISTER_MEMORY_ID, LOGO_MEMORY_ID, MARKETPLACE_CANISTER_MEMORY_ID,
    MEMORY_MANAGER, NAME_MEMORY_ID, SYMBOL_MEMORY_ID, UPGRADED_AT_MEMORY_ID,
};
use crate::constants::{DEFAULT_LOGO, DEFAULT_NAME, DEFAULT_SYMBOL};

thread_local! {
    /// Ekoke Canister principal
    static EKOKE_REWARD_POOL_CANISTER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(EKOKE_REWARD_POOL_CANISTER_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );

    /// Marketplace Canister principal
    static MARKETPLACE_CANISTER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(MARKETPLACE_CANISTER_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );

    /// ICP ledger Canister principal
    static ICP_LEDGER_CANISTER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(ICP_LEDGER_CANISTER_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );

    /// ICP ledger Canister principal
    static LIQUIDITY_POOL_CANISTER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(LIQUIDITY_POOL_CANISTER_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );

    /// Contract logo
    static LOGO: RefCell<StableCell<Option<String>, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(LOGO_MEMORY_ID)), Some(DEFAULT_LOGO.to_string())).unwrap()
    );

    /// Contract name
    static NAME: RefCell<StableCell<Option<String>, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(NAME_MEMORY_ID)), Some(DEFAULT_NAME.to_string())).unwrap()
    );

    /// Contract symbol
    static SYMBOL: RefCell<StableCell<Option<String>, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(SYMBOL_MEMORY_ID)), Some(DEFAULT_SYMBOL.to_string())).unwrap()
    );

    /// Contract creation timestamp
    static CREATED_AT: RefCell<StableCell<u64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(CREATED_AT_MEMORY_ID)), crate::utils::time()).unwrap()
    );

    /// Contract last upgrade timestamp
    static UPGRADED_AT: RefCell<StableCell<Option<u64>, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(UPGRADED_AT_MEMORY_ID)), None).unwrap()
    );

}

pub struct Configuration;

impl Configuration {
    pub fn get_logo() -> Option<String> {
        LOGO.with_borrow(|logo| logo.get().clone())
    }

    pub fn set_logo(logo: String) -> DeferredResult<()> {
        LOGO.with_borrow_mut(|cell| cell.set(Some(logo)))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn get_name() -> Option<String> {
        NAME.with_borrow(|name| name.get().clone())
    }

    pub fn set_name(name: String) -> DeferredResult<()> {
        NAME.with_borrow_mut(|cell| cell.set(Some(name)))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn get_symbol() -> Option<String> {
        SYMBOL.with_borrow(|logo| logo.get().clone())
    }

    pub fn set_symbol(symbol: String) -> DeferredResult<()> {
        SYMBOL
            .with_borrow_mut(|cell| cell.set(Some(symbol)))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn get_created_at() -> u64 {
        CREATED_AT.with_borrow(|cell| *cell.get())
    }

    pub fn get_upgraded_at() -> u64 {
        UPGRADED_AT
            .with_borrow(|cell| *cell.get())
            .unwrap_or(Self::get_created_at())
    }

    pub fn set_upgraded_at() -> DeferredResult<()> {
        UPGRADED_AT
            .with_borrow_mut(|cell| cell.set(Some(crate::utils::time())))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn get_ekoke_reward_pool_canister() -> Principal {
        EKOKE_REWARD_POOL_CANISTER.with_borrow(|cell| *cell.get().as_principal())
    }

    pub fn set_ekoke_reward_pool_canister(canister: Principal) -> DeferredResult<()> {
        EKOKE_REWARD_POOL_CANISTER
            .with_borrow_mut(|cell| cell.set(StorablePrincipal::from(canister)))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn set_marketplace_canister(canister: Principal) -> DeferredResult<()> {
        MARKETPLACE_CANISTER
            .with_borrow_mut(|cell| cell.set(StorablePrincipal::from(canister)))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn get_marketplace_canister() -> Principal {
        MARKETPLACE_CANISTER.with_borrow(|cell| *cell.get().as_principal())
    }

    pub fn set_icp_ledger_canister(canister: Principal) -> DeferredResult<()> {
        ICP_LEDGER_CANISTER
            .with_borrow_mut(|cell| cell.set(StorablePrincipal::from(canister)))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn get_icp_ledger_canister() -> Principal {
        ICP_LEDGER_CANISTER.with_borrow(|cell| *cell.get().as_principal())
    }

    pub fn set_liquidity_pool_canister(canister: Principal) -> DeferredResult<()> {
        LIQUIDITY_POOL_CANISTER
            .with_borrow_mut(|cell| cell.set(StorablePrincipal::from(canister)))
            .map_err(|_| DeferredError::StorageError)?;

        Ok(())
    }

    pub fn get_liquidity_pool_canister() -> Principal {
        LIQUIDITY_POOL_CANISTER.with_borrow(|cell| *cell.get().as_principal())
    }
}

#[cfg(test)]
mod test {

    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_should_get_and_set_logo() {
        assert_eq!(Configuration::get_logo().unwrap().as_str(), DEFAULT_LOGO);
        assert!(Configuration::set_logo("new logo".to_string()).is_ok());
        assert_eq!(Configuration::get_logo().unwrap().as_str(), "new logo");
    }

    #[test]
    fn test_should_get_and_set_name() {
        assert_eq!(Configuration::get_name().unwrap().as_str(), DEFAULT_NAME);
        assert!(Configuration::set_name("new name".to_string()).is_ok());
        assert_eq!(Configuration::get_name().unwrap().as_str(), "new name");
    }

    #[test]
    fn test_should_get_and_set_symbol() {
        assert_eq!(
            Configuration::get_symbol().unwrap().as_str(),
            DEFAULT_SYMBOL
        );
        assert!(Configuration::set_symbol("NFTT".to_string()).is_ok());
        assert_eq!(Configuration::get_symbol().unwrap().as_str(), "NFTT");
    }

    #[test]
    fn test_should_get_created_at() {
        assert!(Configuration::get_created_at() <= crate::utils::time());
    }

    #[test]
    fn test_should_get_and_set_upgraded_at() {
        let last_upgrade = Configuration::get_upgraded_at();
        assert!(Configuration::get_upgraded_at() <= crate::utils::time());
        std::thread::sleep(Duration::from_millis(100));
        assert!(Configuration::set_upgraded_at().is_ok());
        assert!(Configuration::get_upgraded_at() > last_upgrade);
    }

    #[test]
    fn test_should_get_and_set_ekoke_ledger_canister() {
        let principal =
            Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
                .unwrap();
        assert_eq!(
            Configuration::get_ekoke_reward_pool_canister(),
            Principal::anonymous()
        );
        assert!(Configuration::set_ekoke_reward_pool_canister(principal).is_ok());
        assert_eq!(Configuration::get_ekoke_reward_pool_canister(), principal);
    }

    #[test]
    fn test_should_get_and_set_marketplace_canister() {
        let principal =
            Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
                .unwrap();
        assert_eq!(
            Configuration::get_marketplace_canister(),
            Principal::anonymous()
        );
        assert!(Configuration::set_marketplace_canister(principal).is_ok());
        assert_eq!(Configuration::get_marketplace_canister(), principal);
    }

    #[test]
    fn test_should_get_and_set_icp_ledger_canister() {
        let principal =
            Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
                .unwrap();
        assert_eq!(
            Configuration::get_icp_ledger_canister(),
            Principal::anonymous()
        );
        assert!(Configuration::set_icp_ledger_canister(principal).is_ok());
        assert_eq!(Configuration::get_icp_ledger_canister(), principal);
    }

    #[test]
    fn test_should_get_and_set_liquidity_pool_canister() {
        let principal =
            Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
                .unwrap();
        assert_eq!(
            Configuration::get_liquidity_pool_canister(),
            Principal::anonymous()
        );
        assert!(Configuration::set_liquidity_pool_canister(principal).is_ok());
        assert_eq!(Configuration::get_liquidity_pool_canister(), principal);
    }
}
