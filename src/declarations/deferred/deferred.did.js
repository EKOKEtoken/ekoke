export const idlFactory = ({ IDL }) => {
  const GenericValue = IDL.Rec();
  const DeferredInitData = IDL.Record({
    'icp_ledger_canister' : IDL.Principal,
    'custodians' : IDL.Vec(IDL.Principal),
    'ekoke_reward_pool_canister' : IDL.Principal,
    'liquidity_pool_canister' : IDL.Principal,
    'marketplace_canister' : IDL.Principal,
  });
  const Continent = IDL.Variant({
    'Africa' : IDL.Null,
    'Antarctica' : IDL.Null,
    'Asia' : IDL.Null,
    'Europe' : IDL.Null,
    'SouthAmerica' : IDL.Null,
    'Oceania' : IDL.Null,
    'NorthAmerica' : IDL.Null,
  });
  const Agency = IDL.Record({
    'vat' : IDL.Text,
    'region' : IDL.Text,
    'zip_code' : IDL.Text,
    'country' : IDL.Text,
    'agent' : IDL.Text,
    'city' : IDL.Text,
    'logo' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'continent' : Continent,
    'email' : IDL.Text,
    'website' : IDL.Text,
    'address' : IDL.Text,
    'mobile' : IDL.Text,
  });
  const Role = IDL.Variant({ 'Custodian' : IDL.Null, 'Agent' : IDL.Null });
  const NftError = IDL.Variant({
    'UnauthorizedOperator' : IDL.Null,
    'SelfTransfer' : IDL.Null,
    'TokenNotFound' : IDL.Null,
    'UnauthorizedOwner' : IDL.Null,
    'TxNotFound' : IDL.Null,
    'SelfApprove' : IDL.Null,
    'OperatorNotFound' : IDL.Null,
    'ExistedNFT' : IDL.Null,
    'OwnerNotFound' : IDL.Null,
    'Other' : IDL.Text,
  });
  const ConfigurationError = IDL.Variant({
    'AdminsCantBeEmpty' : IDL.Null,
    'AnonymousAdmin' : IDL.Null,
  });
  const ApproveError = IDL.Variant({
    'GenericError' : IDL.Record({
      'message' : IDL.Text,
      'error_code' : IDL.Nat,
    }),
    'TemporarilyUnavailable' : IDL.Null,
    'Duplicate' : IDL.Record({ 'duplicate_of' : IDL.Nat }),
    'BadFee' : IDL.Record({ 'expected_fee' : IDL.Nat }),
    'AllowanceChanged' : IDL.Record({ 'current_allowance' : IDL.Nat }),
    'CreatedInFuture' : IDL.Record({ 'ledger_time' : IDL.Nat64 }),
    'TooOld' : IDL.Null,
    'Expired' : IDL.Record({ 'ledger_time' : IDL.Nat64 }),
    'InsufficientFunds' : IDL.Record({ 'balance' : IDL.Nat }),
  });
  const TransferError = IDL.Variant({
    'GenericError' : IDL.Record({
      'message' : IDL.Text,
      'error_code' : IDL.Nat,
    }),
    'TemporarilyUnavailable' : IDL.Null,
    'BadBurn' : IDL.Record({ 'min_burn_amount' : IDL.Nat }),
    'Duplicate' : IDL.Record({ 'duplicate_of' : IDL.Nat }),
    'BadFee' : IDL.Record({ 'expected_fee' : IDL.Nat }),
    'CreatedInFuture' : IDL.Record({ 'ledger_time' : IDL.Nat64 }),
    'TooOld' : IDL.Null,
    'InsufficientFunds' : IDL.Record({ 'balance' : IDL.Nat }),
  });
  const PoolError = IDL.Variant({
    'PoolNotFound' : IDL.Nat,
    'NotEnoughTokens' : IDL.Null,
  });
  const AllowanceError = IDL.Variant({
    'AllowanceNotFound' : IDL.Null,
    'BadSpender' : IDL.Null,
    'AllowanceChanged' : IDL.Null,
    'BadExpiration' : IDL.Null,
    'AllowanceExpired' : IDL.Null,
    'InsufficientFunds' : IDL.Null,
  });
  const RegisterError = IDL.Variant({ 'TransactionNotFound' : IDL.Null });
  const RejectionCode = IDL.Variant({
    'NoError' : IDL.Null,
    'CanisterError' : IDL.Null,
    'SysTransient' : IDL.Null,
    'DestinationInvalid' : IDL.Null,
    'Unknown' : IDL.Null,
    'SysFatal' : IDL.Null,
    'CanisterReject' : IDL.Null,
  });
  const BalanceError = IDL.Variant({
    'AccountNotFound' : IDL.Null,
    'InsufficientBalance' : IDL.Null,
  });
  const TransferFromError = IDL.Variant({
    'GenericError' : IDL.Record({
      'message' : IDL.Text,
      'error_code' : IDL.Nat,
    }),
    'TemporarilyUnavailable' : IDL.Null,
    'InsufficientAllowance' : IDL.Record({ 'allowance' : IDL.Nat }),
    'BadBurn' : IDL.Record({ 'min_burn_amount' : IDL.Nat }),
    'Duplicate' : IDL.Record({ 'duplicate_of' : IDL.Nat }),
    'BadFee' : IDL.Record({ 'expected_fee' : IDL.Nat }),
    'CreatedInFuture' : IDL.Record({ 'ledger_time' : IDL.Nat64 }),
    'TooOld' : IDL.Null,
    'InsufficientFunds' : IDL.Record({ 'balance' : IDL.Nat }),
  });
  const EcdsaError = IDL.Variant({
    'RecoveryIdError' : IDL.Null,
    'InvalidSignature' : IDL.Null,
    'InvalidPublicKey' : IDL.Null,
  });
  const EkokeError = IDL.Variant({
    'Configuration' : ConfigurationError,
    'Icrc2Approve' : ApproveError,
    'Icrc1Transfer' : TransferError,
    'Pool' : PoolError,
    'Allowance' : AllowanceError,
    'Register' : RegisterError,
    'EthRpcError' : IDL.Tuple(IDL.Int32, IDL.Text),
    'XrcError' : IDL.Null,
    'StorageError' : IDL.Null,
    'CanisterCall' : IDL.Tuple(RejectionCode, IDL.Text),
    'Balance' : BalanceError,
    'Icrc2Transfer' : TransferFromError,
    'Ecdsa' : EcdsaError,
  });
  const WithdrawError = IDL.Variant({
    'InvalidTransferAmount' : IDL.Tuple(IDL.Nat64, IDL.Nat8),
    'ContractNotFound' : IDL.Nat,
    'DepositTransferFailed' : TransferError,
    'ContractNotPaid' : IDL.Nat,
  });
  const ConfigurationError_1 = IDL.Variant({
    'CustodialsCantBeEmpty' : IDL.Null,
    'AnonymousCustodial' : IDL.Null,
  });
  const CloseContractError = IDL.Variant({
    'ContractPaid' : IDL.Nat,
    'LiquidityPoolHasNotEnoughIcp' : IDL.Record({
      'available' : IDL.Nat,
      'required' : IDL.Nat,
    }),
    'ContractNotFound' : IDL.Nat,
    'ContractNotExpired' : IDL.Nat,
    'RefundInvestors' : TransferError,
    'DepositTransferFailed' : TransferError,
  });
  const TokenError = IDL.Variant({
    'ContractAlreadySigned' : IDL.Nat,
    'ContractValueIsNotMultipleOfInstallments' : IDL.Null,
    'TokenAlreadyExists' : IDL.Nat,
    'BadBuyerDepositAccount' : IDL.Null,
    'TokensMismatch' : IDL.Null,
    'ContractAlreadyExists' : IDL.Nat,
    'ContractTokensShouldBeEmpty' : IDL.Null,
    'TokenDoesNotBelongToContract' : IDL.Nat,
    'DepositAllowanceExpired' : IDL.Null,
    'TokenNotFound' : IDL.Nat,
    'DepositAllowanceNotEnough' : IDL.Record({
      'available' : IDL.Nat,
      'required' : IDL.Nat,
    }),
    'ContractSellerQuotaIsNot100' : IDL.Null,
    'DepositRejected' : TransferFromError,
    'ContractNotFound' : IDL.Nat,
    'CannotCloseContract' : IDL.Null,
    'ContractValueIsLessThanDeposit' : IDL.Null,
    'ContractNotSigned' : IDL.Nat,
    'ContractHasNoSeller' : IDL.Null,
    'ContractHasNoBuyer' : IDL.Null,
    'BadContractExpiration' : IDL.Null,
    'ContractHasNoTokens' : IDL.Null,
    'TokenIsBurned' : IDL.Nat,
    'BadMintTokenOwner' : IDL.Nat,
    'BadContractProperty' : IDL.Null,
  });
  const DeferredError = IDL.Variant({
    'Nft' : NftError,
    'Ekoke' : EkokeError,
    'Withdraw' : WithdrawError,
    'Configuration' : ConfigurationError_1,
    'CloseContract' : CloseContractError,
    'Unauthorized' : IDL.Null,
    'Token' : TokenError,
    'StorageError' : IDL.Null,
    'CanisterCall' : IDL.Tuple(RejectionCode, IDL.Text),
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : DeferredError });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : NftError });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : NftError });
  const Metadata = IDL.Record({
    'logo' : IDL.Opt(IDL.Text),
    'name' : IDL.Opt(IDL.Text),
    'created_at' : IDL.Nat64,
    'upgraded_at' : IDL.Nat64,
    'custodians' : IDL.Vec(IDL.Principal),
    'symbol' : IDL.Opt(IDL.Text),
  });
  GenericValue.fill(
    IDL.Variant({
      'Nat64Content' : IDL.Nat64,
      'Nat32Content' : IDL.Nat32,
      'BoolContent' : IDL.Bool,
      'Nat8Content' : IDL.Nat8,
      'Int64Content' : IDL.Int64,
      'IntContent' : IDL.Int,
      'NatContent' : IDL.Nat,
      'Nat16Content' : IDL.Nat16,
      'Int32Content' : IDL.Int32,
      'Int8Content' : IDL.Int8,
      'FloatContent' : IDL.Float64,
      'Int16Content' : IDL.Int16,
      'BlobContent' : IDL.Vec(IDL.Nat8),
      'NestedContent' : IDL.Vec(IDL.Tuple(IDL.Text, GenericValue)),
      'Principal' : IDL.Principal,
      'TextContent' : IDL.Text,
    })
  );
  const Result_3 = IDL.Variant({
    'Ok' : IDL.Opt(IDL.Principal),
    'Err' : NftError,
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Vec(IDL.Nat), 'Err' : NftError });
  const TokenMetadata = IDL.Record({
    'transferred_at' : IDL.Opt(IDL.Nat64),
    'transferred_by' : IDL.Opt(IDL.Principal),
    'owner' : IDL.Opt(IDL.Principal),
    'operator' : IDL.Opt(IDL.Principal),
    'approved_at' : IDL.Opt(IDL.Nat64),
    'approved_by' : IDL.Opt(IDL.Principal),
    'properties' : IDL.Vec(IDL.Tuple(IDL.Text, GenericValue)),
    'is_burned' : IDL.Bool,
    'token_identifier' : IDL.Nat,
    'burned_at' : IDL.Opt(IDL.Nat64),
    'burned_by' : IDL.Opt(IDL.Principal),
    'minted_at' : IDL.Nat64,
    'minted_by' : IDL.Principal,
  });
  const Result_5 = IDL.Variant({
    'Ok' : IDL.Vec(TokenMetadata),
    'Err' : NftError,
  });
  const Stats = IDL.Record({
    'cycles' : IDL.Nat,
    'total_transactions' : IDL.Nat,
    'total_unique_holders' : IDL.Nat,
    'total_supply' : IDL.Nat,
  });
  const SupportedInterface = IDL.Variant({
    'Burn' : IDL.Null,
    'Mint' : IDL.Null,
    'Approval' : IDL.Null,
    'TransactionHistory' : IDL.Null,
  });
  const Result_6 = IDL.Variant({ 'Ok' : TokenMetadata, 'Err' : NftError });
  const TxEvent = IDL.Record({
    'time' : IDL.Nat64,
    'operation' : IDL.Text,
    'details' : IDL.Vec(IDL.Tuple(IDL.Text, GenericValue)),
    'caller' : IDL.Principal,
  });
  const Result_7 = IDL.Variant({ 'Ok' : TxEvent, 'Err' : NftError });
  const ContractType = IDL.Variant({
    'Sell' : IDL.Null,
    'Financing' : IDL.Null,
  });
  const RestrictionLevel = IDL.Variant({
    'Buyer' : IDL.Null,
    'Seller' : IDL.Null,
    'Agent' : IDL.Null,
  });
  const RestrictedProperty = IDL.Record({
    'value' : GenericValue,
    'access_list' : IDL.Vec(RestrictionLevel),
  });
  const Deposit = IDL.Record({
    'value_fiat' : IDL.Nat64,
    'value_icp' : IDL.Nat64,
  });
  const Seller = IDL.Record({
    'principal' : IDL.Principal,
    'quota' : IDL.Nat8,
  });
  const Contract = IDL.Record({
    'id' : IDL.Nat,
    'value' : IDL.Nat64,
    'type' : ContractType,
    'is_signed' : IDL.Bool,
    'agency' : IDL.Opt(Agency),
    'restricted_properties' : IDL.Vec(IDL.Tuple(IDL.Text, RestrictedProperty)),
    'properties' : IDL.Vec(IDL.Tuple(IDL.Text, GenericValue)),
    'deposit' : Deposit,
    'sellers' : IDL.Vec(Seller),
    'expiration' : IDL.Opt(IDL.Text),
    'tokens' : IDL.Vec(IDL.Nat),
    'currency' : IDL.Text,
    'installments' : IDL.Nat64,
    'initial_value' : IDL.Nat64,
    'buyers' : IDL.Vec(IDL.Principal),
  });
  const Token = IDL.Record({
    'id' : IDL.Nat,
    'transferred_at' : IDL.Opt(IDL.Nat64),
    'transferred_by' : IDL.Opt(IDL.Principal),
    'value' : IDL.Nat64,
    'owner' : IDL.Opt(IDL.Principal),
    'operator' : IDL.Opt(IDL.Principal),
    'approved_at' : IDL.Opt(IDL.Nat64),
    'approved_by' : IDL.Opt(IDL.Principal),
    'contract_id' : IDL.Nat,
    'ekoke_reward' : IDL.Nat,
    'is_burned' : IDL.Bool,
    'burned_at' : IDL.Opt(IDL.Nat64),
    'burned_by' : IDL.Opt(IDL.Principal),
    'minted_at' : IDL.Nat64,
    'minted_by' : IDL.Principal,
  });
  const TokenInfo = IDL.Record({ 'token' : Token, 'contract' : Contract });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'upgrade' : IDL.Opt(IDL.Bool),
    'status_code' : IDL.Nat16,
  });
  const Account = IDL.Record({
    'owner' : IDL.Principal,
    'subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const Buyers = IDL.Record({
    'deposit_account' : Account,
    'principals' : IDL.Vec(IDL.Principal),
  });
  const ContractRegistration = IDL.Record({
    'value' : IDL.Nat64,
    'type' : ContractType,
    'restricted_properties' : IDL.Vec(IDL.Tuple(IDL.Text, RestrictedProperty)),
    'properties' : IDL.Vec(IDL.Tuple(IDL.Text, GenericValue)),
    'deposit' : Deposit,
    'sellers' : IDL.Vec(Seller),
    'expiration' : IDL.Opt(IDL.Text),
    'currency' : IDL.Text,
    'installments' : IDL.Nat64,
    'buyers' : Buyers,
  });
  const Result_8 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : DeferredError });
  return IDL.Service({
    'admin_register_agency' : IDL.Func([IDL.Principal, Agency], [], []),
    'admin_remove_role' : IDL.Func([IDL.Principal, Role], [Result], []),
    'admin_set_ekoke_liquidity_pool_canister' : IDL.Func(
        [IDL.Principal],
        [],
        [],
      ),
    'admin_set_ekoke_reward_pool_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_marketplace_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_role' : IDL.Func([IDL.Principal, Role], [], []),
    'close_contract' : IDL.Func([IDL.Nat], [Result], []),
    'dip721_approve' : IDL.Func([IDL.Principal, IDL.Nat], [Result_1], []),
    'dip721_balance_of' : IDL.Func([IDL.Principal], [Result_1], ['query']),
    'dip721_burn' : IDL.Func([IDL.Nat], [Result_1], []),
    'dip721_custodians' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'dip721_cycles' : IDL.Func([], [IDL.Nat], ['query']),
    'dip721_is_approved_for_all' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [Result_2],
        [],
      ),
    'dip721_logo' : IDL.Func([], [IDL.Opt(IDL.Text)], ['query']),
    'dip721_metadata' : IDL.Func([], [Metadata], ['query']),
    'dip721_mint' : IDL.Func(
        [IDL.Principal, IDL.Nat, IDL.Vec(IDL.Tuple(IDL.Text, GenericValue))],
        [Result_1],
        [],
      ),
    'dip721_name' : IDL.Func([], [IDL.Opt(IDL.Text)], ['query']),
    'dip721_operator_of' : IDL.Func([IDL.Nat], [Result_3], ['query']),
    'dip721_operator_token_identifiers' : IDL.Func(
        [IDL.Principal],
        [Result_4],
        ['query'],
      ),
    'dip721_operator_token_metadata' : IDL.Func(
        [IDL.Principal],
        [Result_5],
        ['query'],
      ),
    'dip721_owner_of' : IDL.Func([IDL.Nat], [Result_3], ['query']),
    'dip721_owner_token_identifiers' : IDL.Func(
        [IDL.Principal],
        [Result_4],
        ['query'],
      ),
    'dip721_owner_token_metadata' : IDL.Func(
        [IDL.Principal],
        [Result_5],
        ['query'],
      ),
    'dip721_set_approval_for_all' : IDL.Func(
        [IDL.Principal, IDL.Bool],
        [Result_1],
        [],
      ),
    'dip721_set_custodians' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'dip721_set_logo' : IDL.Func([IDL.Text], [], []),
    'dip721_set_name' : IDL.Func([IDL.Text], [], []),
    'dip721_set_symbol' : IDL.Func([IDL.Text], [], []),
    'dip721_stats' : IDL.Func([], [Stats], ['query']),
    'dip721_supported_interfaces' : IDL.Func(
        [],
        [IDL.Vec(SupportedInterface)],
        ['query'],
      ),
    'dip721_symbol' : IDL.Func([], [IDL.Opt(IDL.Text)], ['query']),
    'dip721_token_metadata' : IDL.Func([IDL.Nat], [Result_6], ['query']),
    'dip721_total_supply' : IDL.Func([], [IDL.Nat], ['query']),
    'dip721_total_transactions' : IDL.Func([], [IDL.Nat], ['query']),
    'dip721_total_unique_holders' : IDL.Func([], [IDL.Nat], ['query']),
    'dip721_transaction' : IDL.Func([IDL.Nat], [Result_7], ['query']),
    'dip721_transfer' : IDL.Func([IDL.Principal, IDL.Nat], [Result_1], []),
    'dip721_transfer_from' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Nat],
        [Result_1],
        [],
      ),
    'get_agencies' : IDL.Func([], [IDL.Vec(Agency)], ['query']),
    'get_contract' : IDL.Func([IDL.Nat], [IDL.Opt(Contract)], ['query']),
    'get_restricted_contract_properties' : IDL.Func(
        [IDL.Nat],
        [IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, RestrictedProperty)))],
        ['query'],
      ),
    'get_signed_contracts' : IDL.Func([], [IDL.Vec(IDL.Nat)], ['query']),
    'get_token' : IDL.Func([IDL.Nat], [IDL.Opt(TokenInfo)], ['query']),
    'get_unsigned_contracts' : IDL.Func([], [IDL.Vec(IDL.Nat)], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'increment_contract_value' : IDL.Func(
        [IDL.Nat, IDL.Nat64, IDL.Nat64],
        [Result],
        [],
      ),
    'register_contract' : IDL.Func([ContractRegistration], [Result_8], []),
    'remove_agency' : IDL.Func([IDL.Principal], [Result], []),
    'sign_contract' : IDL.Func([IDL.Nat], [Result], []),
    'update_contract_buyers' : IDL.Func(
        [IDL.Nat, IDL.Vec(IDL.Principal)],
        [Result],
        [],
      ),
    'update_contract_property' : IDL.Func(
        [IDL.Nat, IDL.Text, GenericValue],
        [Result],
        [],
      ),
    'update_restricted_contract_property' : IDL.Func(
        [IDL.Nat, IDL.Text, RestrictedProperty],
        [Result],
        [],
      ),
    'withdraw_contract_deposit' : IDL.Func(
        [IDL.Nat, IDL.Opt(IDL.Vec(IDL.Nat8))],
        [Result],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const DeferredInitData = IDL.Record({
    'icp_ledger_canister' : IDL.Principal,
    'custodians' : IDL.Vec(IDL.Principal),
    'ekoke_reward_pool_canister' : IDL.Principal,
    'liquidity_pool_canister' : IDL.Principal,
    'marketplace_canister' : IDL.Principal,
  });
  return [DeferredInitData];
};
