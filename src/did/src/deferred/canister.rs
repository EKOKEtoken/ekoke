use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use dip721_rs::TxEvent;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;

/// These are the arguments which are taken by the sell contract canister on init
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct DeferredInitData {
    pub custodians: Vec<Principal>,
    pub ekoke_reward_pool_canister: Principal,
    pub icp_ledger_canister: Principal,
    pub liquidity_pool_canister: Principal,
    pub marketplace_canister: Principal,
}

/// Storable TxEvent DIP721 transaction
pub struct StorableTxEvent(pub TxEvent);

impl StorableTxEvent {
    pub fn as_tx_event(&self) -> &TxEvent {
        &self.0
    }
}

impl From<TxEvent> for StorableTxEvent {
    fn from(tx_event: TxEvent) -> Self {
        Self(tx_event)
    }
}

impl Storable for StorableTxEvent {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Encode!(&self.0).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, TxEvent).unwrap().into()
    }
}

/// Deferred user roles. Defines permissions
#[derive(Clone, Copy, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum Role {
    /// Administrator, follows DIP721 standard
    Custodian,
    /// A user who can create contracts, but cannot sign them
    Agent,
}

impl Storable for Role {
    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Encode!(&self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, Role).unwrap()
    }
}

/// List of roles
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct Roles(pub Vec<Role>);

impl From<Vec<Role>> for Roles {
    fn from(roles: Vec<Role>) -> Self {
        Self(roles)
    }
}

impl Storable for Roles {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Encode!(&self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, Vec<Role>).unwrap().into()
    }
}
