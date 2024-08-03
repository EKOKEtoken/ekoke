use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};
use ic_stable_structures::DefaultMemoryImpl;

pub const TOKENS_MEMORY_ID: MemoryId = MemoryId::new(10);
pub const CONTRACTS_MEMORY_ID: MemoryId = MemoryId::new(11);
pub const TRANSACTIONS_MEMORY_ID: MemoryId = MemoryId::new(12);
pub const AGENCIES_MEMORY_ID: MemoryId = MemoryId::new(13);

pub const LOGO_MEMORY_ID: MemoryId = MemoryId::new(20);
pub const NAME_MEMORY_ID: MemoryId = MemoryId::new(21);
pub const SYMBOL_MEMORY_ID: MemoryId = MemoryId::new(22);
pub const CREATED_AT_MEMORY_ID: MemoryId = MemoryId::new(23);
pub const UPGRADED_AT_MEMORY_ID: MemoryId = MemoryId::new(24);
pub const EKOKE_REWARD_POOL_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(25);
pub const MARKETPLACE_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(26);
pub const ICP_LEDGER_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(27);
pub const LIQUIDITY_POOL_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(28);

pub const ROLES_MEMORY_ID: MemoryId = MemoryId::new(30);

thread_local! {

    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());


}
