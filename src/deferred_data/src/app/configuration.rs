use std::cell::RefCell;

use candid::Principal;
use did::deferred::{DeferredDataError, DeferredDataResult};
use did::{StorableLogSettings, StorablePrincipal};
use ic_log::LogSettingsV2;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};

use crate::app::memory::{
    LOG_SETTINGS_MEMORY_ID, MEMORY_MANAGER, MINTER_MEMORY_ID, OWNER_MEMORY_ID,
};

thread_local! {
    /// Deferred minter memory ID
    static MINTER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(MINTER_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );

    /// Owner memory ID
    static OWNER: RefCell<StableCell<StorablePrincipal, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(OWNER_MEMORY_ID)), Principal::anonymous().into()).unwrap()
    );

    /// log settings
    static LOG_SETTINGS: RefCell<StableCell<StorableLogSettings, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(LOG_SETTINGS_MEMORY_ID)), StorableLogSettings::default()).unwrap()
    );

}

pub struct Configuration;

impl Configuration {
    pub fn get_minter() -> Principal {
        MINTER.with_borrow(|cell| cell.get().0)
    }

    pub fn set_minter(principal: Principal) -> DeferredDataResult<()> {
        MINTER.with_borrow_mut(|cell| {
            cell.set(principal.into())
                .map_err(|_| DeferredDataError::StorageError)
        })?;

        Ok(())
    }

    pub fn get_owner() -> Principal {
        OWNER.with_borrow(|cell| cell.get().0)
    }

    pub fn set_owner(principal: Principal) -> DeferredDataResult<()> {
        OWNER.with_borrow_mut(|cell| {
            cell.set(principal.into())
                .map_err(|_| DeferredDataError::StorageError)
        })?;

        Ok(())
    }

    pub fn set_log_settings(settings: LogSettingsV2) -> DeferredDataResult<()> {
        LOG_SETTINGS.with_borrow_mut(|cell| {
            cell.set(StorableLogSettings(settings))
                .map_err(|_| DeferredDataError::StorageError)
        })?;

        Ok(())
    }

    pub fn get_log_settings() -> LogSettingsV2 {
        LOG_SETTINGS.with_borrow(|cell| cell.get().0.clone())
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_should_get_and_set_minter() {
        let principal =
            Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
                .unwrap();
        assert_eq!(Configuration::get_minter(), Principal::anonymous());
        assert!(Configuration::set_minter(principal).is_ok());
        assert_eq!(Configuration::get_minter(), principal);
    }

    #[test]
    fn test_should_get_and_set_owner() {
        let principal =
            Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
                .unwrap();
        assert_eq!(Configuration::get_owner(), Principal::anonymous());
        assert!(Configuration::set_owner(principal).is_ok());
        assert_eq!(Configuration::get_owner(), principal);
    }

    #[test]
    fn test_should_set_and_get_log_settings() {
        let settings = LogSettingsV2 {
            enable_console: true,
            ..Default::default()
        };
        assert_eq!(Configuration::get_log_settings(), settings);
        assert!(Configuration::set_log_settings(settings.clone()).is_ok());
        assert_eq!(Configuration::get_log_settings(), settings);
    }
}
