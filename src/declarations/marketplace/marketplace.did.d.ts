import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AllowanceError = { 'AllowanceNotFound' : null } |
  { 'BadSpender' : null } |
  { 'AllowanceChanged' : null } |
  { 'BadExpiration' : null } |
  { 'AllowanceExpired' : null } |
  { 'InsufficientFunds' : null };
export type ApproveError = {
    'GenericError' : { 'message' : string, 'error_code' : bigint }
  } |
  { 'TemporarilyUnavailable' : null } |
  { 'Duplicate' : { 'duplicate_of' : bigint } } |
  { 'BadFee' : { 'expected_fee' : bigint } } |
  { 'AllowanceChanged' : { 'current_allowance' : bigint } } |
  { 'CreatedInFuture' : { 'ledger_time' : bigint } } |
  { 'TooOld' : null } |
  { 'Expired' : { 'ledger_time' : bigint } } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export type BalanceError = { 'AccountNotFound' : null } |
  { 'InsufficientBalance' : null };
export type BuyError = { 'TokenHasNoOwner' : null } |
  { 'IcpAllowanceNotEnough' : null } |
  { 'CallerAlreadyOwnsToken' : null } |
  { 'IcpAllowanceExpired' : null };
export type CloseContractError = { 'ContractPaid' : bigint } |
  {
    'LiquidityPoolHasNotEnoughIcp' : {
      'available' : bigint,
      'required' : bigint,
    }
  } |
  { 'ContractNotFound' : bigint } |
  { 'ContractNotExpired' : bigint } |
  { 'RefundInvestors' : TransferError } |
  { 'DepositTransferFailed' : TransferError };
export type ConfigurationError = { 'AdminsCantBeEmpty' : null } |
  { 'AnonymousAdmin' : null };
export type ConfigurationError_1 = { 'CustodialsCantBeEmpty' : null } |
  { 'AnonymousCustodial' : null };
export type DeferredError = { 'Nft' : NftError } |
  { 'Ekoke' : EkokeError } |
  { 'Withdraw' : WithdrawError } |
  { 'Configuration' : ConfigurationError_1 } |
  { 'CloseContract' : CloseContractError } |
  { 'Unauthorized' : null } |
  { 'Token' : TokenError } |
  { 'StorageError' : null } |
  { 'CanisterCall' : [RejectionCode, string] };
export type EcdsaError = { 'RecoveryIdError' : null } |
  { 'InvalidSignature' : null } |
  { 'InvalidPublicKey' : null };
export type EkokeError = { 'Configuration' : ConfigurationError } |
  { 'Icrc2Approve' : ApproveError } |
  { 'Icrc1Transfer' : TransferError } |
  { 'Pool' : PoolError } |
  { 'Allowance' : AllowanceError } |
  { 'Register' : RegisterError } |
  { 'EthRpcError' : [number, string] } |
  { 'XrcError' : null } |
  { 'StorageError' : null } |
  { 'CanisterCall' : [RejectionCode, string] } |
  { 'Balance' : BalanceError } |
  { 'Icrc2Transfer' : TransferFromError } |
  { 'Ecdsa' : EcdsaError };
export type MarketplaceError = { 'Buy' : BuyError } |
  { 'Configuration' : ConfigurationError } |
  { 'Icrc1Transfer' : TransferError } |
  { 'DeferredCanister' : DeferredError } |
  { 'TokenNotFound' : null } |
  { 'EkokeCanister' : EkokeError } |
  { 'XrcError' : string } |
  { 'StorageError' : null } |
  { 'CanisterCall' : [RejectionCode, string] } |
  { 'Dip721' : NftError } |
  { 'Icrc2Transfer' : TransferFromError };
export interface MarketplaceInitData {
  'deferred_canister' : Principal,
  'icp_ledger_canister' : Principal,
  'ekoke_liquidity_pool_canister' : Principal,
  'xrc_canister' : Principal,
  'ekoke_reward_pool_canister' : Principal,
  'admins' : Array<Principal>,
}
export type NftError = { 'UnauthorizedOperator' : null } |
  { 'SelfTransfer' : null } |
  { 'TokenNotFound' : null } |
  { 'UnauthorizedOwner' : null } |
  { 'TxNotFound' : null } |
  { 'SelfApprove' : null } |
  { 'OperatorNotFound' : null } |
  { 'ExistedNFT' : null } |
  { 'OwnerNotFound' : null } |
  { 'Other' : string };
export type PoolError = { 'PoolNotFound' : bigint } |
  { 'NotEnoughTokens' : null };
export type RegisterError = { 'TransactionNotFound' : null };
export type RejectionCode = { 'NoError' : null } |
  { 'CanisterError' : null } |
  { 'SysTransient' : null } |
  { 'DestinationInvalid' : null } |
  { 'Unknown' : null } |
  { 'SysFatal' : null } |
  { 'CanisterReject' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : MarketplaceError };
export type Result_1 = { 'Ok' : bigint } |
  { 'Err' : MarketplaceError };
export type TokenError = { 'ContractAlreadySigned' : bigint } |
  { 'ContractValueIsNotMultipleOfInstallments' : null } |
  { 'TokenAlreadyExists' : bigint } |
  { 'BadBuyerDepositAccount' : null } |
  { 'TokensMismatch' : null } |
  { 'ContractAlreadyExists' : bigint } |
  { 'ContractTokensShouldBeEmpty' : null } |
  { 'TokenDoesNotBelongToContract' : bigint } |
  { 'DepositAllowanceExpired' : null } |
  { 'TokenNotFound' : bigint } |
  {
    'DepositAllowanceNotEnough' : { 'available' : bigint, 'required' : bigint }
  } |
  { 'ContractSellerQuotaIsNot100' : null } |
  { 'DepositRejected' : TransferFromError } |
  { 'ContractNotFound' : bigint } |
  { 'CannotCloseContract' : null } |
  { 'ContractValueIsLessThanDeposit' : null } |
  { 'ContractNotSigned' : bigint } |
  { 'ContractHasNoSeller' : null } |
  { 'ContractHasNoBuyer' : null } |
  { 'BadContractExpiration' : null } |
  { 'ContractHasNoTokens' : null } |
  { 'TokenIsBurned' : bigint } |
  { 'BadMintTokenOwner' : bigint } |
  { 'BadContractProperty' : null };
export type TransferError = {
    'GenericError' : { 'message' : string, 'error_code' : bigint }
  } |
  { 'TemporarilyUnavailable' : null } |
  { 'BadBurn' : { 'min_burn_amount' : bigint } } |
  { 'Duplicate' : { 'duplicate_of' : bigint } } |
  { 'BadFee' : { 'expected_fee' : bigint } } |
  { 'CreatedInFuture' : { 'ledger_time' : bigint } } |
  { 'TooOld' : null } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export type TransferFromError = {
    'GenericError' : { 'message' : string, 'error_code' : bigint }
  } |
  { 'TemporarilyUnavailable' : null } |
  { 'InsufficientAllowance' : { 'allowance' : bigint } } |
  { 'BadBurn' : { 'min_burn_amount' : bigint } } |
  { 'Duplicate' : { 'duplicate_of' : bigint } } |
  { 'BadFee' : { 'expected_fee' : bigint } } |
  { 'CreatedInFuture' : { 'ledger_time' : bigint } } |
  { 'TooOld' : null } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export type WithdrawError = { 'InvalidTransferAmount' : [bigint, number] } |
  { 'ContractNotFound' : bigint } |
  { 'DepositTransferFailed' : TransferError } |
  { 'ContractNotPaid' : bigint };
export interface _SERVICE {
  'admin_cycles' : ActorMethod<[], bigint>,
  'admin_set_admins' : ActorMethod<[Array<Principal>], Result>,
  'admin_set_deferred_canister' : ActorMethod<[Principal], undefined>,
  'admin_set_ekoke_liquidity_pool_canister' : ActorMethod<[Principal], Result>,
  'admin_set_ekoke_reward_pool_canister' : ActorMethod<[Principal], Result>,
  'admin_set_icp_ledger_canister' : ActorMethod<[Principal], undefined>,
  'admin_set_interest_rate_for_buyer' : ActorMethod<[number], undefined>,
  'admin_set_xrc_canister' : ActorMethod<[Principal], undefined>,
  'buy_token' : ActorMethod<[bigint, [] | [Uint8Array | number[]]], Result>,
  'get_token_price_icp' : ActorMethod<[bigint], Result_1>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
