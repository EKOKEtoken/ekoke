use candid::{CandidType, Decode, Deserialize, Encode, Principal};
pub use dip721_rs::{GenericValue, TokenIdentifier};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use icrc::icrc1::account::Account;
use serde::Serialize;
use time::Date;

pub use crate::ID;

mod agency;
mod info;
mod token;

pub use agency::{Agency, Continent};
pub use info::TokenInfo;
pub use token::Token;

use super::{DeferredError, DeferredResult, TokenError};

/// A sell contract for a building
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Contract {
    /// Contract ID
    pub id: ID,
    /// Contract type
    pub r#type: ContractType,
    /// The contractors selling the building with their quota
    pub sellers: Vec<Seller>,
    /// Contract buyers. Those who must pay
    pub buyers: Vec<Principal>,
    /// Tokens associated to the contract, by id
    pub tokens: Vec<TokenIdentifier>,
    /// Number of installments
    pub installments: u64,
    /// Whether the contract is signed. Tokens are minted only if the contract is signed
    pub is_signed: bool,
    /// Initial Fiat value of the contract
    pub initial_value: u64,
    /// Current Fiat value of the contract (to pay)
    pub value: u64,
    /// Deposit value
    pub deposit: Deposit,
    /// Currency symbol
    pub currency: String,
    /// Data associated to the contract
    pub properties: ContractProperties,
    /// Restricted data associated to the contract
    pub restricted_properties: RestrictedContractProperties,
    /// Agency data
    pub agency: Option<Agency>,
    /// Contract expiration date YYYY-MM-DD
    pub expiration: Option<String>,
}

impl Contract {
    pub fn is_seller(&self, principal: &Principal) -> bool {
        self.sellers.iter().any(|s| &s.principal == principal)
    }

    pub fn expiration(&self) -> Option<DeferredResult<Date>> {
        let format = time::macros::format_description!("[year]-[month]-[day]");
        self.expiration
            .as_deref()
            .map(|expiration| match time::Date::parse(expiration, format) {
                Ok(expiration) => Ok(expiration),
                Err(_) => Err(DeferredError::Token(TokenError::BadContractExpiration)),
            })
    }

    /// Returns the total value of the installments
    pub fn installments_value(&self) -> u64 {
        self.value - self.deposit.value_fiat
    }
}

impl Storable for Contract {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Encode!(&self).unwrap().into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

/// A list of properties associated to a contract
pub type ContractProperties = Vec<(String, GenericValue)>;

/// A list of restricted properties associated to a contract
pub type RestrictedContractProperties = Vec<(String, RestrictedProperty)>;

/// A restricted property, which defines the access level to the property and its value
#[derive(Clone, Debug, CandidType, PartialEq, Serialize, Deserialize)]
pub struct RestrictedProperty {
    pub access_list: Vec<RestrictionLevel>,
    pub value: GenericValue,
}

/// A variant which defines the restriction level for a contract property
#[derive(Clone, Debug, CandidType, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestrictionLevel {
    /// Seller can access the property
    Seller,
    /// Buyer can access the property
    Buyer,
    /// Agent can access the property
    Agent,
}

/// A variant which defines the contract type
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum ContractType {
    Financing,
    Sell,
}

/// A variant which defines a contract seller.
/// A contract may have more than one seller and the quota defines the percentage of the contract ownership.
/// The sum of all quotas must be 100.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub struct Seller {
    pub principal: Principal,
    pub quota: u8,
}

/// Contract buyers principals and deposit account
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub struct Buyers {
    /// List of buyers principals
    pub principals: Vec<Principal>,
    /// Account used to pay the down payment
    pub deposit_account: Account,
}

/// Buyer deposit value
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize, Serialize)]
pub struct Deposit {
    /// The value for the deposit in FIAT currency.
    /// Must be subtracted to `ContractRegistration::value` to get the remaining value to pay
    pub value_fiat: u64,
    pub value_icp: u64,
}

/// Data to be provided to register a contract
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ContractRegistration {
    pub r#type: ContractType,
    pub sellers: Vec<Seller>,
    pub buyers: Buyers,
    /// Total value of the contract.
    pub value: u64,
    pub currency: String,
    /// Buyer deposit value in $ICP
    pub deposit: Deposit,
    /// Must be a divisor of `value - deposit_value_fiat`
    pub installments: u64,
    pub expiration: Option<String>,
    pub properties: ContractProperties,
    pub restricted_properties: RestrictedContractProperties,
}
