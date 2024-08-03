use candid::{CandidType, Deserialize, Nat};
use dip721_rs::{NftError, TokenIdentifier};
use ic_cdk::api::call::RejectionCode;
use icrc::icrc1::transfer::TransferError;
use icrc::icrc2::transfer_from::TransferFromError;
use thiserror::Error;

use crate::ekoke::EkokeError;
use crate::ID;

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum DeferredError {
    #[error("unauthorized caller")]
    Unauthorized,
    #[error("ekoke error: {0}")]
    Ekoke(#[from] EkokeError),
    #[error("token error: {0}")]
    Token(TokenError),
    #[error("deposit error: {0}")]
    Deposit(DepositError),
    #[error("withdraw error: {0}")]
    Withdraw(#[from] WithdrawError),
    #[error("close contract error: {0}")]
    CloseContract(#[from] CloseContractError),
    #[error("configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
    #[error("storage error")]
    StorageError,
    #[error("nft error: {0}")]
    Nft(#[from] NftError),
    #[error("inter-canister call error: ({0:?}): {1}")]
    CanisterCall(RejectionCode, String),
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum TokenError {
    #[error("contract properties should start with 'contract:'")]
    BadContractProperty,
    #[error("the provided contract expiration is invalid")]
    BadContractExpiration,
    #[error("the provided contract ID ({0}) already exists in the canister storage")]
    ContractAlreadyExists(ID),
    #[error("the provided contract ID ({0}) is already signed")]
    ContractAlreadySigned(ID),
    #[error("the provided contract ID ({0}) is not signed")]
    ContractNotSigned(ID),
    #[error("the provided contract ID should be empty on register")]
    ContractTokensShouldBeEmpty,
    #[error("the provided contract ID ({0}) doesn't exist in the canister storage")]
    ContractNotFound(ID),
    #[error("the provided token ID ({0}) already exists in the canister storage")]
    TokenAlreadyExists(TokenIdentifier),
    #[error("the provided token ({0}) doesn't belong to the provided contract")]
    TokenDoesNotBelongToContract(TokenIdentifier),
    #[error("the token {0} owner should be the seller on mint")]
    BadMintTokenOwner(TokenIdentifier),
    #[error("the token defined in the contract differ from the provided tokens")]
    TokensMismatch,
    #[error("the contract provided has no tokens")]
    ContractHasNoTokens,
    #[error("the provided token ID ({0}) doesn't exist in the canister storage")]
    TokenNotFound(TokenIdentifier),
    #[error("the provided token ID ({0}) is burned, so it cannot be touched by any operation")]
    TokenIsBurned(TokenIdentifier),
    #[error("the provided contract value is not a multiple of the number of installments")]
    ContractValueIsNotMultipleOfInstallments,
    #[error("the provided contract value is less than the deposit")]
    ContractValueIsLessThanDeposit,
    #[error("the provided contract has no seller")]
    ContractHasNoSeller,
    #[error("the provided contract has no buyer")]
    ContractHasNoBuyer,
    #[error("the provided deposit account for the buyers is invalid")]
    BadBuyerDepositAccount,
    #[error("in order to close the contract, all the tokens must be owned by the seller")]
    CannotCloseContract,
    #[error("the provided contract seller quota sum is not 100")]
    ContractSellerQuotaIsNot100,
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum ConfigurationError {
    #[error("there must be at least one custodial")]
    CustodialsCantBeEmpty,
    #[error("the canister custodial cannot be anonymous")]
    AnonymousCustodial,
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum DepositError {
    #[error("the deposit allowance for the buyer has expired")]
    AllowanceExpired,
    #[error("the deposit allowance for the buyer is not enough. Required: {required}, available: {available}")]
    AllowanceNotEnough { required: Nat, available: Nat },
    #[error("buyer deposit rejected")]
    Rejected(TransferFromError),
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum WithdrawError {
    #[error("the provided contract ID ({0}) doesn't exist in the canister storage")]
    ContractNotFound(ID),
    #[error("the contract {0} has not been completely paid yet")]
    ContractNotPaid(ID),
    #[error("deposit transfer failed: {0}")]
    DepositTransferFailed(TransferError),
    #[error("invalid transfer amount: {0} for quota {1}")]
    InvalidTransferAmount(u64, u8),
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum CloseContractError {
    #[error("the provided contract ID ({0}) doesn't exist in the canister storage")]
    ContractNotFound(ID),
    #[error("the contract {0} hasn't expired yet")]
    ContractNotExpired(ID),
    #[error("the contract {0} is actually completely paid")]
    ContractPaid(ID),
    #[error("transfer to liquidity pool failed: {0}")]
    DepositTransferFailed(TransferError),
    #[error("refund to investors call failed: {0}")]
    RefundInvestors(TransferError),
    #[error("liquidity pool has not enough ICP to refund investors. required: {required}, available: {available}")]
    LiquidityPoolHasNotEnoughIcp { required: Nat, available: Nat },
}
