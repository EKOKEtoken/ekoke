export const idlFactory = ({ IDL }) => {
  const EthNetwork = IDL.Variant({
    'Ethereum' : IDL.Null,
    'Goerli' : IDL.Null,
    'Sepolia' : IDL.Null,
  });
  const EkokeErc20SwapInitData = IDL.Record({
    'cketh_ledger_canister' : IDL.Principal,
    'erc20_bridge_address' : IDL.Text,
    'erc20_network' : EthNetwork,
    'ledger_id' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
    'erc20_gas_price' : IDL.Nat64,
    'cketh_minter_canister' : IDL.Principal,
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const TransformArgs = IDL.Record({
    'context' : IDL.Vec(IDL.Nat8),
    'response' : HttpResponse,
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
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : EkokeError });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : EkokeError });
  return IDL.Service({
    'admin_cycles' : IDL.Func([], [IDL.Nat], ['query']),
    'admin_eth_wallet_address' : IDL.Func([], [IDL.Text], ['query']),
    'admin_set_admins' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'admin_set_cketh_ledger_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_cketh_minter_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_erc20_bridge_address' : IDL.Func([IDL.Text], [], []),
    'admin_set_erc20_gas_price' : IDL.Func([IDL.Nat64], [], []),
    'admin_set_ledger_canister' : IDL.Func([IDL.Principal], [], []),
    'http_transform_send_tx' : IDL.Func(
        [TransformArgs],
        [HttpResponse],
        ['query'],
      ),
    'swap' : IDL.Func(
        [IDL.Text, IDL.Nat, IDL.Opt(IDL.Vec(IDL.Nat8))],
        [Result],
        [],
      ),
    'swap_fee' : IDL.Func([], [Result_1], []),
  });
};
export const init = ({ IDL }) => {
  const EthNetwork = IDL.Variant({
    'Ethereum' : IDL.Null,
    'Goerli' : IDL.Null,
    'Sepolia' : IDL.Null,
  });
  const EkokeErc20SwapInitData = IDL.Record({
    'cketh_ledger_canister' : IDL.Principal,
    'erc20_bridge_address' : IDL.Text,
    'erc20_network' : EthNetwork,
    'ledger_id' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
    'erc20_gas_price' : IDL.Nat64,
    'cketh_minter_canister' : IDL.Principal,
  });
  return [EkokeErc20SwapInitData];
};
