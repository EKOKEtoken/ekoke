use candid::{Nat, Principal};
use time::{Date, OffsetDateTime};

/// Returns current time in nanoseconds
pub fn time() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        time.as_nanos() as u64
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::time()
    }
}

pub fn cycles() -> Nat {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Nat::from(30_000_000_000_u64)
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::canister_balance().into()
    }
}

/// Returns canister id
pub fn id() -> Principal {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Principal::from_text("lj532-6iaaa-aaaah-qcc7a-cai").unwrap()
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::id()
    }
}

pub fn caller() -> Principal {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
            .unwrap()
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::caller()
    }
}

/// Returns current date
pub fn date() -> Date {
    let time = time();

    let date = OffsetDateTime::from_unix_timestamp_nanos(time as i128).unwrap();
    date.date()
}
