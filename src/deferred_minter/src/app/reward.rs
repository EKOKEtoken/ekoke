//! # Reward
//!
//! This module defines functions to calculate Deferred contracts rewards.

use std::cell::RefCell;

use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};

use crate::app::memory::{
    AVIDITY_MEMORY_ID, CPM_MEMORY_ID, LAST_CPM_MEMORY_ID, LAST_MONTH_MEMORY_ID, MEMORY_MANAGER,
    NEXT_HALVING_MEMORY_ID, RMC_MEMORY_ID,
};
use crate::utils::time;

/// Initial "reward multiplier coefficient" value
const INITIAL_RMC: f64 = 0.0000042;
/// Initial "avidity" value
const INITIAL_AVIDITY: f64 = 1.0;
/// Minimum reward
const MIN_REWARD: u128 = 1_000;
/// Base token price
const BASE_TOKEN_PRICE: u128 = 100;

thread_local! {
    /// RMC
    static RMC: RefCell<StableCell<f64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(RMC_MEMORY_ID)),
        INITIAL_RMC).unwrap()
    );

    /// Next halving time
    static NEXT_HALVING: RefCell<StableCell<u64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(NEXT_HALVING_MEMORY_ID)),
        Reward::next_rmc_halving()).unwrap()
    );

    /// Avidity
    static AVIDITY: RefCell<StableCell<f64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(AVIDITY_MEMORY_ID)),
            INITIAL_AVIDITY
        ).unwrap()
    );

    /// Contracts per month
    static CPM: RefCell<StableCell<u64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(CPM_MEMORY_ID)),
            0
        ).unwrap()
    );

    /// Contracts per month
    static LAST_CPM: RefCell<StableCell<u64, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(LAST_CPM_MEMORY_ID)),
            0
        ).unwrap()
    );

    static LAST_MONTH: RefCell<StableCell<u8, VirtualMemory<DefaultMemoryImpl>>> =
    RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(LAST_MONTH_MEMORY_ID)),
        crate::utils::date().month() as u8
    ).unwrap()
);

}

pub struct Reward;

impl Reward {
    /// Calculate reward for the provided contract ID and installments.
    ///
    /// Formula is: (RMC * Avidity * Remaining Supply * Token Price) / 100
    ///
    /// Returns None if unable to reserve enough tokens.
    pub fn get_contract_reward(
        installments: u64,
        remaining_supply: u128,
        token_price: u64,
    ) -> Option<u128> {
        // check if we need to halve the RMC
        if Self::should_halve_rmc() {
            Self::halve_rmc();
        }
        // check if we need to adjust the avidity
        if Self::should_adjust_avidity() {
            Self::adjust_avidity();
        }
        // calculate the reward
        let avidity = AVIDITY.with_borrow(|avidity| *avidity.get());
        let rmc = RMC.with_borrow(|rmc| *rmc.get());

        let remaining_supply_f64 = remaining_supply as f64;
        let reward = match remaining_supply_f64 * rmc * avidity {
            res if res < MIN_REWARD as f64 => MIN_REWARD,
            res => res as u128,
        };

        // calculate the final reward based on the token price
        // reward : BASE_TOKEN_PRICE = x : token_price
        let reward = (reward * token_price as u128).div_ceil(BASE_TOKEN_PRICE);

        // check if canister has enough tokens to pay the reward
        let pool_value = reward * installments as u128;
        if pool_value > remaining_supply {
            return None;
        }

        // increment CPM
        CPM.with_borrow_mut(|cpm| {
            cpm.set(*cpm.get() + 1).unwrap();
        });

        // return reward
        Some(reward)
    }

    /// Get the next RMC halving time. 4 years from now.
    #[inline]
    fn next_rmc_halving() -> u64 {
        // time in 4 years (is nanoseconds)
        time() + (60 * 60 * 24 * 365 * 4 * 1_000_000_000)
    }

    /// Check if it is time to halve the RMC.
    /// If RMC value is below 2e-9, it will never halve.
    fn should_halve_rmc() -> bool {
        // check if RMC value less than 2e-9
        if RMC.with_borrow(|rmc| *rmc.get() < 2e-12) {
            return false;
        }
        // check time
        time() >= NEXT_HALVING.with_borrow(|halving| *halving.get())
    }

    /// Halve the RMC, update its value and update the next halving time.
    fn halve_rmc() {
        RMC.with_borrow_mut(|rmc| {
            rmc.set(*rmc.get() / 2.0).unwrap();
        });
        NEXT_HALVING.with_borrow_mut(|halving| {
            halving.set(Self::next_rmc_halving()).unwrap();
        })
    }

    /// Check if current month is different from the last month.
    fn should_adjust_avidity() -> bool {
        let last_month = LAST_MONTH.with_borrow(|last_month| *last_month.get());
        let current_month = crate::utils::date().month() as u8;
        // check time
        current_month != last_month
    }

    /// Adjust the Avidity value and reset CPM
    fn adjust_avidity() {
        let cpm = CPM.with_borrow(|cpm| *cpm.get());
        let last_cpm = LAST_CPM.with_borrow(|last_cpm| *last_cpm.get());
        let avidity = AVIDITY.with_borrow(|avidity| *avidity.get());

        // calculate avidity
        let new_avidity = if cpm > last_cpm {
            avidity - 0.1
        } else {
            avidity + 0.1
        };
        // calculate final avidity
        let new_avidity = 0.1_f64.max(new_avidity.min(1.0));

        // set new avidity
        AVIDITY.with_borrow_mut(|avidity| {
            avidity.set(new_avidity).unwrap();
        });

        // set CPM to 0
        CPM.with_borrow_mut(|cpm| {
            cpm.set(0).unwrap();
        });
        // set last_cpm to this month cpm
        LAST_CPM.with_borrow_mut(|last_cpm| {
            last_cpm.set(cpm).unwrap();
        });
        // update last month
        LAST_MONTH.with_borrow_mut(|last_month| {
            last_month.set(crate::utils::date().month() as u8).unwrap();
        });
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    const DEFAULT_REMAINING_SUPPLY: u128 = 592_006_734_000_000;

    #[tokio::test]
    async fn test_should_get_reward_if_pool_doesnt_exist() {
        assert_eq!(
            Reward::get_contract_reward(4_000, DEFAULT_REMAINING_SUPPLY, BASE_TOKEN_PRICE as u64)
                .unwrap(),
            2486428282, // 29 ekoke
        );
        assert_eq!(CPM.with_borrow(|cpm| *cpm.get()), 1);

        let remaining_supply = DEFAULT_REMAINING_SUPPLY - 29_400_000_000;

        // next reward should be less
        assert_eq!(
            Reward::get_contract_reward(4_000, remaining_supply, BASE_TOKEN_PRICE as u64).unwrap(),
            2486304802,
        );
        assert_eq!(CPM.with_borrow(|cpm| *cpm.get()), 2);
    }

    #[tokio::test]
    async fn test_should_get_less_value_if_token_price_is_lower() {
        assert_eq!(
            Reward::get_contract_reward(4_000, DEFAULT_REMAINING_SUPPLY, 1).unwrap(),
            24864283, // 0.29 ekoke
        );
        assert_eq!(CPM.with_borrow(|cpm| *cpm.get()), 1);
    }

    #[test]
    fn test_should_say_whether_it_should_halve_rmc() {
        assert_eq!(Reward::should_halve_rmc(), false);
        // set to 0
        NEXT_HALVING.with_borrow_mut(|rmc| {
            rmc.set(0).unwrap();
        });
        assert_eq!(Reward::should_halve_rmc(), true);
        // should not halve if RMC is less than 2e-12
        RMC.with_borrow_mut(|rmc| {
            rmc.set(1.8e-12).unwrap();
        });
        assert_eq!(Reward::should_halve_rmc(), false);
    }

    #[test]
    fn test_should_halve_rmc() {
        let rmc = INITIAL_RMC / 2.0;
        NEXT_HALVING.with_borrow_mut(|rmc| {
            rmc.set(0).unwrap();
        });
        Reward::halve_rmc();
        assert_eq!(RMC.with_borrow(|rmc| *rmc.get()), rmc);

        // verify that halving time was updated
        assert_ne!(NEXT_HALVING.with_borrow(|halving| *halving.get()), 0);
    }

    #[test]
    fn test_should_tell_whether_to_adjust_avidity() {
        let month = crate::utils::date().month() as u8;
        LAST_MONTH.with_borrow_mut(|last_month| {
            last_month.set(month - 1).unwrap();
        });

        assert_eq!(Reward::should_adjust_avidity(), true);
        LAST_MONTH.with_borrow_mut(|last_month| {
            last_month.set(month).unwrap();
        });
        assert_eq!(Reward::should_adjust_avidity(), false);
    }

    #[test]
    fn test_should_adjust_avidity() {
        let cpm = 10;
        let last_cpm = 5;
        let avidity = 0.5;
        let new_avidity = 0.4;
        CPM.with_borrow_mut(|cell| {
            cell.set(cpm).unwrap();
        });
        LAST_CPM.with_borrow_mut(|cell| {
            cell.set(last_cpm).unwrap();
        });
        AVIDITY.with_borrow_mut(|cell| {
            cell.set(avidity).unwrap();
        });
        Reward::adjust_avidity();
        assert_eq!(CPM.with_borrow(|cpm| *cpm.get()), 0);
        assert_eq!(LAST_CPM.with_borrow(|last_cpm| *last_cpm.get()), cpm);
        assert_eq!(AVIDITY.with_borrow(|avidity| *avidity.get()), new_avidity);

        // if cpm is less than last_cpm, avidity should increase
        CPM.with_borrow_mut(|cpm| {
            cpm.set(5).unwrap();
        });
        LAST_CPM.with_borrow_mut(|last_cpm| {
            last_cpm.set(10).unwrap();
        });
        AVIDITY.with_borrow_mut(|avidity| {
            avidity.set(new_avidity).unwrap();
        });
        Reward::adjust_avidity();
        assert_eq!(CPM.with_borrow(|cpm| *cpm.get()), 0);
        assert_eq!(LAST_CPM.with_borrow(|last_cpm| *last_cpm.get()), 5);
        assert_eq!(
            AVIDITY.with_borrow(|avidity| *avidity.get()),
            new_avidity + 0.1
        );

        // avidity should not exceed 1
        AVIDITY.with_borrow_mut(|cell| {
            cell.set(1.0).unwrap();
        });
        CPM.with_borrow_mut(|cpm| {
            cpm.set(5).unwrap();
        });
        LAST_CPM.with_borrow_mut(|last_cpm| {
            last_cpm.set(10).unwrap();
        });
        Reward::adjust_avidity();
        assert_eq!(AVIDITY.with_borrow(|avidity| *avidity.get()), 1.0);
        // avidity should not go below 0.1
        AVIDITY.with_borrow_mut(|cell| {
            cell.set(0.1).unwrap();
        });
        CPM.with_borrow_mut(|cpm| {
            cpm.set(10).unwrap();
        });
        LAST_CPM.with_borrow_mut(|last_cpm| {
            last_cpm.set(4).unwrap();
        });
        Reward::adjust_avidity();
        assert_eq!(AVIDITY.with_borrow(|avidity| *avidity.get()), 0.1);
    }
}
