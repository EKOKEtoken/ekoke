/// The number of nanoseconds in a day
pub const NANOSECONDS_IN_A_DAY: u64 = 86_400_000_000_000;

/// The interest multiplier to apply to a deferred nft price in case the caller is a contract buyer (10%)
pub const DEFAULT_INTEREST_MULTIPLIER_FOR_BUYER: f64 = 1.1;

/// The ledger canister id of the ICP token
#[cfg(target_arch = "wasm32")]
pub const ICP_LEDGER_CANISTER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
