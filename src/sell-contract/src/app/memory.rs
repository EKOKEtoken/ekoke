use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};
use ic_stable_structures::DefaultMemoryImpl;

pub const TOKENS_MEMORY_ID: MemoryId = MemoryId::new(10);
pub const CONTRACTS_MEMORY_ID: MemoryId = MemoryId::new(11);

/// Memory for the fly canister principal
pub const FLY_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(20);
/// Canister Administrators
pub const CANISTER_CUSTODIANS_MEMORY_ID: MemoryId = MemoryId::new(21);

thread_local! {

    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());


}
