export const idlFactory = ({ IDL }) => {
  const EkokeLiquidityPoolInitData = IDL.Record({
    'icp_ledger_canister' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
    'deferred_canister_id' : IDL.Principal,
  });
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
  const LiquidityPoolAccounts = IDL.Record({ 'icp' : Account });
  const LiquidityPoolBalance = IDL.Record({ 'icp' : IDL.Nat });
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
  const Result = IDL.Variant({
    'Ok' : LiquidityPoolBalance,
    'Err' : EkokeError,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : TransferError });
  return IDL.Service({
    'admin_cycles' : IDL.Func([], [IDL.Nat], ['query']),
    'admin_set_admins' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'admin_set_deferred_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_icp_ledger_canister' : IDL.Func([IDL.Principal], [], []),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'liquidity_pool_accounts' : IDL.Func(
        [],
        [LiquidityPoolAccounts],
        ['query'],
      ),
    'liquidity_pool_balance' : IDL.Func([], [Result], ['query']),
    'refund_investors' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat))],
        [Result_1],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const EkokeLiquidityPoolInitData = IDL.Record({
    'icp_ledger_canister' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
    'deferred_canister_id' : IDL.Principal,
  });
  return [EkokeLiquidityPoolInitData];
};
