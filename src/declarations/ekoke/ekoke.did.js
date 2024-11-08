export const idlFactory = ({ IDL }) => {
  const Account = IDL.Record({
    'owner' : IDL.Principal,
    'subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const EthNetwork = IDL.Variant({
    'Ethereum' : IDL.Null,
    'Goerli' : IDL.Null,
    'Sepolia' : IDL.Null,
  });
  const EkokeInitData = IDL.Record({
    'deferred_canister' : IDL.Principal,
    'icp_ledger_canister' : IDL.Principal,
    'cketh_ledger_canister' : IDL.Principal,
    'minting_account' : Account,
    'ckbtc_canister' : IDL.Principal,
    'erc20_bridge_address' : IDL.Text,
    'erc20_network' : EthNetwork,
    'initial_balances' : IDL.Vec(IDL.Tuple(Account, IDL.Nat)),
    'swap_account' : Account,
    'xrc_canister' : IDL.Principal,
    'marketplace_canister' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
    'erc20_gas_price' : IDL.Nat64,
    'cketh_minter_canister' : IDL.Principal,
    'total_supply' : IDL.Nat,
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
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : EkokeError });
  const Role = IDL.Variant({
    'DeferredCanister' : IDL.Null,
    'MarketplaceCanister' : IDL.Null,
    'Admin' : IDL.Null,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : EkokeError });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : EkokeError });
  const Result_3 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : EkokeError });
  const Transaction = IDL.Record({
    'to' : Account,
    'fee' : IDL.Nat,
    'from' : Account,
    'memo' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'created_at' : IDL.Nat64,
    'amount' : IDL.Nat,
  });
  const Result_4 = IDL.Variant({ 'Ok' : Transaction, 'Err' : EkokeError });
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
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse_1 = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const TransformArgs = IDL.Record({
    'context' : IDL.Vec(IDL.Nat8),
    'response' : HttpResponse_1,
  });
  const MetadataValue = IDL.Variant({
    'Int' : IDL.Int,
    'Nat' : IDL.Nat,
    'Blob' : IDL.Vec(IDL.Nat8),
    'Text' : IDL.Text,
  });
  const TokenExtension = IDL.Record({ 'url' : IDL.Text, 'name' : IDL.Text });
  const TransferArg = IDL.Record({
    'to' : Account,
    'fee' : IDL.Opt(IDL.Nat),
    'memo' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'created_at_time' : IDL.Opt(IDL.Nat64),
    'amount' : IDL.Nat,
  });
  const Result_5 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : TransferError });
  const AllowanceArgs = IDL.Record({
    'account' : Account,
    'spender' : Account,
  });
  const Allowance = IDL.Record({
    'allowance' : IDL.Nat,
    'expires_at' : IDL.Opt(IDL.Nat64),
  });
  const ApproveArgs = IDL.Record({
    'fee' : IDL.Opt(IDL.Nat),
    'memo' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'created_at_time' : IDL.Opt(IDL.Nat64),
    'amount' : IDL.Nat,
    'expected_allowance' : IDL.Opt(IDL.Nat),
    'expires_at' : IDL.Opt(IDL.Nat64),
    'spender' : Account,
  });
  const Result_6 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : ApproveError });
  const TransferFromArgs = IDL.Record({
    'to' : Account,
    'fee' : IDL.Opt(IDL.Nat),
    'spender_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'from' : Account,
    'memo' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'created_at_time' : IDL.Opt(IDL.Nat64),
    'amount' : IDL.Nat,
  });
  const Result_7 = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : TransferFromError });
  const LiquidityPoolAccounts = IDL.Record({
    'icp' : Account,
    'ckbtc' : Account,
  });
  const LiquidityPoolBalance = IDL.Record({
    'icp' : IDL.Nat,
    'ckbtc' : IDL.Nat,
  });
  const Result_8 = IDL.Variant({
    'Ok' : LiquidityPoolBalance,
    'Err' : EkokeError,
  });
  return IDL.Service({
    'admin_burn' : IDL.Func([IDL.Nat], [Result], []),
    'admin_cycles' : IDL.Func([], [IDL.Nat], ['query']),
    'admin_eth_wallet_address' : IDL.Func([], [IDL.Text], ['query']),
    'admin_remove_role' : IDL.Func([IDL.Principal, Role], [Result], []),
    'admin_set_ckbtc_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_cketh_ledger_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_cketh_minter_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_erc20_bridge_address' : IDL.Func([IDL.Text], [], []),
    'admin_set_erc20_gas_price' : IDL.Func([IDL.Nat64], [], []),
    'admin_set_icp_ledger_canister' : IDL.Func([IDL.Principal], [], []),
    'admin_set_role' : IDL.Func([IDL.Principal, Role], [], []),
    'admin_set_swap_account' : IDL.Func([Account], [], []),
    'admin_set_xrc_canister' : IDL.Func([IDL.Principal], [], []),
    'erc20_swap' : IDL.Func(
        [IDL.Text, IDL.Nat, IDL.Opt(IDL.Vec(IDL.Nat8))],
        [Result_1],
        [],
      ),
    'erc20_swap_fee' : IDL.Func([], [Result_2], []),
    'get_contract_reward' : IDL.Func([IDL.Nat, IDL.Nat64], [Result_3], []),
    'get_transaction' : IDL.Func([IDL.Nat64], [Result_4], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'http_transform_send_tx' : IDL.Func(
        [TransformArgs],
        [HttpResponse_1],
        ['query'],
      ),
    'icrc1_balance_of' : IDL.Func([Account], [IDL.Nat], ['query']),
    'icrc1_decimals' : IDL.Func([], [IDL.Nat8], ['query']),
    'icrc1_fee' : IDL.Func([], [IDL.Nat], ['query']),
    'icrc1_metadata' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, MetadataValue))],
        ['query'],
      ),
    'icrc1_name' : IDL.Func([], [IDL.Text], ['query']),
    'icrc1_supported_standards' : IDL.Func(
        [],
        [IDL.Vec(TokenExtension)],
        ['query'],
      ),
    'icrc1_symbol' : IDL.Func([], [IDL.Text], ['query']),
    'icrc1_total_supply' : IDL.Func([], [IDL.Nat], ['query']),
    'icrc1_transfer' : IDL.Func([TransferArg], [Result_5], []),
    'icrc2_allowance' : IDL.Func([AllowanceArgs], [Allowance], ['query']),
    'icrc2_approve' : IDL.Func([ApproveArgs], [Result_6], []),
    'icrc2_transfer_from' : IDL.Func([TransferFromArgs], [Result_7], []),
    'liquidity_pool_accounts' : IDL.Func(
        [],
        [LiquidityPoolAccounts],
        ['query'],
      ),
    'liquidity_pool_balance' : IDL.Func([], [Result_8], ['query']),
    'reserve_pool' : IDL.Func(
        [IDL.Nat, IDL.Nat, IDL.Opt(IDL.Vec(IDL.Nat8))],
        [Result_3],
        [],
      ),
    'send_reward' : IDL.Func([IDL.Nat, IDL.Nat, Account], [Result], []),
  });
};
export const init = ({ IDL }) => {
  const Account = IDL.Record({
    'owner' : IDL.Principal,
    'subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const EthNetwork = IDL.Variant({
    'Ethereum' : IDL.Null,
    'Goerli' : IDL.Null,
    'Sepolia' : IDL.Null,
  });
  const EkokeInitData = IDL.Record({
    'deferred_canister' : IDL.Principal,
    'icp_ledger_canister' : IDL.Principal,
    'cketh_ledger_canister' : IDL.Principal,
    'minting_account' : Account,
    'ckbtc_canister' : IDL.Principal,
    'erc20_bridge_address' : IDL.Text,
    'erc20_network' : EthNetwork,
    'initial_balances' : IDL.Vec(IDL.Tuple(Account, IDL.Nat)),
    'swap_account' : Account,
    'xrc_canister' : IDL.Principal,
    'marketplace_canister' : IDL.Principal,
    'admins' : IDL.Vec(IDL.Principal),
    'erc20_gas_price' : IDL.Nat64,
    'cketh_minter_canister' : IDL.Principal,
    'total_supply' : IDL.Nat,
  });
  return [EkokeInitData];
};
