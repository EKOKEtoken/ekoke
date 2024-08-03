//! # Inspect
//!
//! Deferred inspect message handler

use candid::Principal;

use super::configuration::Configuration;
use super::roles::RolesManager;

pub struct Inspect;

impl Inspect {
    /// Returns whether caller is custodian of the canister
    pub fn inspect_is_admin(caller: Principal) -> bool {
        RolesManager::is_admin(caller)
    }

    pub fn inspect_is_deferred_canister(caller: Principal) -> bool {
        caller == Configuration::get_deferred_canister()
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_should_inspect_is_custodian() {
        let caller = Principal::anonymous();
        assert_eq!(Inspect::inspect_is_admin(caller), false);

        let caller = Principal::from_text("aaaaa-aa").unwrap();
        assert_eq!(Inspect::inspect_is_admin(caller), false);

        let caller = Principal::from_text("aaaaa-aa").unwrap();
        assert!(RolesManager::set_admins(vec![caller]).is_ok());
        assert_eq!(Inspect::inspect_is_admin(caller), true);
    }

    #[test]
    fn test_should_inspect_is_deferred_canister() {
        let caller = Principal::anonymous();
        assert_eq!(Inspect::inspect_is_deferred_canister(caller), true);

        let caller = Principal::from_text("aaaaa-aa").unwrap();
        assert_eq!(Inspect::inspect_is_deferred_canister(caller), false);

        let caller = Principal::from_text("aaaaa-aa").unwrap();
        Configuration::set_deferred_canister(caller);
        assert_eq!(Inspect::inspect_is_deferred_canister(caller), true);
    }
}
