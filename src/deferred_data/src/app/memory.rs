use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};
use ic_stable_structures::DefaultMemoryImpl;

pub const CONTRACTS_MEMORY_ID: MemoryId = MemoryId::new(10);

pub const MINTER_MEMORY_ID: MemoryId = MemoryId::new(20);
pub const OWNER_MEMORY_ID: MemoryId = MemoryId::new(21);

thread_local! {
    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}