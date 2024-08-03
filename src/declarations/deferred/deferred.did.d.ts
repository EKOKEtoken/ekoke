import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
}
export interface Agency {
  'vat' : string,
  'region' : string,
  'zip_code' : string,
  'country' : string,
  'agent' : string,
  'city' : string,
  'logo' : [] | [string],
  'name' : string,
  'continent' : Continent,
  'email' : string,
  'website' : string,
  'address' : string,
  'mobile' : string,
}
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
export interface Buyers {
  'deposit_account' : Account,
  'principals' : Array<Principal>,
}
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
export type Continent = { 'Africa' : null } |
  { 'Antarctica' : null } |
  { 'Asia' : null } |
  { 'Europe' : null } |
  { 'SouthAmerica' : null } |
  { 'Oceania' : null } |
  { 'NorthAmerica' : null };
export interface Contract {
  'id' : bigint,
  'value' : bigint,
  'type' : ContractType,
  'is_signed' : boolean,
  'agency' : [] | [Agency],
  'restricted_properties' : Array<[string, RestrictedProperty]>,
  'properties' : Array<[string, GenericValue]>,
  'deposit' : Deposit,
  'sellers' : Array<Seller>,
  'expiration' : [] | [string],
  'tokens' : Array<bigint>,
  'currency' : string,
  'installments' : bigint,
  'initial_value' : bigint,
  'buyers' : Array<Principal>,
}
export interface ContractRegistration {
  'value' : bigint,
  'type' : ContractType,
  'restricted_properties' : Array<[string, RestrictedProperty]>,
  'properties' : Array<[string, GenericValue]>,
  'deposit' : Deposit,
  'sellers' : Array<Seller>,
  'expiration' : [] | [string],
  'currency' : string,
  'installments' : bigint,
  'buyers' : Buyers,
}
export type ContractType = { 'Sell' : null } |
  { 'Financing' : null };
export type DeferredError = { 'Nft' : NftError } |
  { 'Ekoke' : EkokeError } |
  { 'Withdraw' : WithdrawError } |
  { 'Configuration' : ConfigurationError_1 } |
  { 'CloseContract' : CloseContractError } |
  { 'Unauthorized' : null } |
  { 'Token' : TokenError } |
  { 'StorageError' : null } |
  { 'CanisterCall' : [RejectionCode, string] };
export interface DeferredInitData {
  'icp_ledger_canister' : Principal,
  'custodians' : Array<Principal>,
  'ekoke_reward_pool_canister' : Principal,
  'liquidity_pool_canister' : Principal,
  'marketplace_canister' : Principal,
}
export interface Deposit { 'value_fiat' : bigint, 'value_icp' : bigint }
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
export type GenericValue = { 'Nat64Content' : bigint } |
  { 'Nat32Content' : number } |
  { 'BoolContent' : boolean } |
  { 'Nat8Content' : number } |
  { 'Int64Content' : bigint } |
  { 'IntContent' : bigint } |
  { 'NatContent' : bigint } |
  { 'Nat16Content' : number } |
  { 'Int32Content' : number } |
  { 'Int8Content' : number } |
  { 'FloatContent' : number } |
  { 'Int16Content' : number } |
  { 'BlobContent' : Uint8Array | number[] } |
  { 'NestedContent' : Array<[string, GenericValue]> } |
  { 'Principal' : Principal } |
  { 'TextContent' : string };
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
  'upgrade' : [] | [boolean],
  'status_code' : number,
}
export interface Metadata {
  'logo' : [] | [string],
  'name' : [] | [string],
  'created_at' : bigint,
  'upgraded_at' : bigint,
  'custodians' : Array<Principal>,
  'symbol' : [] | [string],
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
export interface RestrictedProperty {
  'value' : GenericValue,
  'access_list' : Array<RestrictionLevel>,
}
export type RestrictionLevel = { 'Buyer' : null } |
  { 'Seller' : null } |
  { 'Agent' : null };
export type Result = { 'Ok' : null } |
  { 'Err' : DeferredError };
export type Result_1 = { 'Ok' : bigint } |
  { 'Err' : NftError };
export type Result_2 = { 'Ok' : boolean } |
  { 'Err' : NftError };
export type Result_3 = { 'Ok' : [] | [Principal] } |
  { 'Err' : NftError };
export type Result_4 = { 'Ok' : Array<bigint> } |
  { 'Err' : NftError };
export type Result_5 = { 'Ok' : Array<TokenMetadata> } |
  { 'Err' : NftError };
export type Result_6 = { 'Ok' : TokenMetadata } |
  { 'Err' : NftError };
export type Result_7 = { 'Ok' : TxEvent } |
  { 'Err' : NftError };
export type Result_8 = { 'Ok' : bigint } |
  { 'Err' : DeferredError };
export type Role = { 'Custodian' : null } |
  { 'Agent' : null };
export interface Seller { 'principal' : Principal, 'quota' : number }
export interface Stats {
  'cycles' : bigint,
  'total_transactions' : bigint,
  'total_unique_holders' : bigint,
  'total_supply' : bigint,
}
export type SupportedInterface = { 'Burn' : null } |
  { 'Mint' : null } |
  { 'Approval' : null } |
  { 'TransactionHistory' : null };
export interface Token {
  'id' : bigint,
  'transferred_at' : [] | [bigint],
  'transferred_by' : [] | [Principal],
  'value' : bigint,
  'owner' : [] | [Principal],
  'operator' : [] | [Principal],
  'approved_at' : [] | [bigint],
  'approved_by' : [] | [Principal],
  'contract_id' : bigint,
  'ekoke_reward' : bigint,
  'is_burned' : boolean,
  'burned_at' : [] | [bigint],
  'burned_by' : [] | [Principal],
  'minted_at' : bigint,
  'minted_by' : Principal,
}
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
export interface TokenInfo { 'token' : Token, 'contract' : Contract }
export interface TokenMetadata {
  'transferred_at' : [] | [bigint],
  'transferred_by' : [] | [Principal],
  'owner' : [] | [Principal],
  'operator' : [] | [Principal],
  'approved_at' : [] | [bigint],
  'approved_by' : [] | [Principal],
  'properties' : Array<[string, GenericValue]>,
  'is_burned' : boolean,
  'token_identifier' : bigint,
  'burned_at' : [] | [bigint],
  'burned_by' : [] | [Principal],
  'minted_at' : bigint,
  'minted_by' : Principal,
}
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
export interface TxEvent {
  'time' : bigint,
  'operation' : string,
  'details' : Array<[string, GenericValue]>,
  'caller' : Principal,
}
export type WithdrawError = { 'InvalidTransferAmount' : [bigint, number] } |
  { 'ContractNotFound' : bigint } |
  { 'DepositTransferFailed' : TransferError } |
  { 'ContractNotPaid' : bigint };
export interface _SERVICE {
  'admin_register_agency' : ActorMethod<[Principal, Agency], undefined>,
  'admin_remove_role' : ActorMethod<[Principal, Role], Result>,
  'admin_set_ekoke_liquidity_pool_canister' : ActorMethod<
    [Principal],
    undefined
  >,
  'admin_set_ekoke_reward_pool_canister' : ActorMethod<[Principal], undefined>,
  'admin_set_marketplace_canister' : ActorMethod<[Principal], undefined>,
  'admin_set_role' : ActorMethod<[Principal, Role], undefined>,
  'close_contract' : ActorMethod<[bigint], Result>,
  'dip721_approve' : ActorMethod<[Principal, bigint], Result_1>,
  'dip721_balance_of' : ActorMethod<[Principal], Result_1>,
  'dip721_burn' : ActorMethod<[bigint], Result_1>,
  'dip721_custodians' : ActorMethod<[], Array<Principal>>,
  'dip721_cycles' : ActorMethod<[], bigint>,
  'dip721_is_approved_for_all' : ActorMethod<[Principal, Principal], Result_2>,
  'dip721_logo' : ActorMethod<[], [] | [string]>,
  'dip721_metadata' : ActorMethod<[], Metadata>,
  'dip721_mint' : ActorMethod<
    [Principal, bigint, Array<[string, GenericValue]>],
    Result_1
  >,
  'dip721_name' : ActorMethod<[], [] | [string]>,
  'dip721_operator_of' : ActorMethod<[bigint], Result_3>,
  'dip721_operator_token_identifiers' : ActorMethod<[Principal], Result_4>,
  'dip721_operator_token_metadata' : ActorMethod<[Principal], Result_5>,
  'dip721_owner_of' : ActorMethod<[bigint], Result_3>,
  'dip721_owner_token_identifiers' : ActorMethod<[Principal], Result_4>,
  'dip721_owner_token_metadata' : ActorMethod<[Principal], Result_5>,
  'dip721_set_approval_for_all' : ActorMethod<[Principal, boolean], Result_1>,
  'dip721_set_custodians' : ActorMethod<[Array<Principal>], undefined>,
  'dip721_set_logo' : ActorMethod<[string], undefined>,
  'dip721_set_name' : ActorMethod<[string], undefined>,
  'dip721_set_symbol' : ActorMethod<[string], undefined>,
  'dip721_stats' : ActorMethod<[], Stats>,
  'dip721_supported_interfaces' : ActorMethod<[], Array<SupportedInterface>>,
  'dip721_symbol' : ActorMethod<[], [] | [string]>,
  'dip721_token_metadata' : ActorMethod<[bigint], Result_6>,
  'dip721_total_supply' : ActorMethod<[], bigint>,
  'dip721_total_transactions' : ActorMethod<[], bigint>,
  'dip721_total_unique_holders' : ActorMethod<[], bigint>,
  'dip721_transaction' : ActorMethod<[bigint], Result_7>,
  'dip721_transfer' : ActorMethod<[Principal, bigint], Result_1>,
  'dip721_transfer_from' : ActorMethod<
    [Principal, Principal, bigint],
    Result_1
  >,
  'get_agencies' : ActorMethod<[], Array<Agency>>,
  'get_contract' : ActorMethod<[bigint], [] | [Contract]>,
  'get_restricted_contract_properties' : ActorMethod<
    [bigint],
    [] | [Array<[string, RestrictedProperty]>]
  >,
  'get_signed_contracts' : ActorMethod<[], Array<bigint>>,
  'get_token' : ActorMethod<[bigint], [] | [TokenInfo]>,
  'get_unsigned_contracts' : ActorMethod<[], Array<bigint>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'increment_contract_value' : ActorMethod<[bigint, bigint, bigint], Result>,
  'register_contract' : ActorMethod<[ContractRegistration], Result_8>,
  'remove_agency' : ActorMethod<[Principal], Result>,
  'sign_contract' : ActorMethod<[bigint], Result>,
  'update_contract_buyers' : ActorMethod<[bigint, Array<Principal>], Result>,
  'update_contract_property' : ActorMethod<
    [bigint, string, GenericValue],
    Result
  >,
  'update_restricted_contract_property' : ActorMethod<
    [bigint, string, RestrictedProperty],
    Result
  >,
  'withdraw_contract_deposit' : ActorMethod<
    [bigint, [] | [Uint8Array | number[]]],
    Result
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
