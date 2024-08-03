use candid::{Nat, Principal};
use did::deferred::ContractRegistration;
use did::ID;
use dip721_rs::GenericValue;
use ic_cdk::api;
use ic_cdk::api::call::ArgDecoderConfig;
#[cfg(target_family = "wasm")]
use ic_cdk_macros::inspect_message;
use icrc::icrc1::account::Subaccount;

use crate::app::Inspect;
use crate::utils::caller;

/// NOTE: inspect is disabled for non-wasm targets because without it we are getting a weird compilation error
/// in CI:
/// > multiple definition of `canister_inspect_message'
#[cfg(target_family = "wasm")]
#[inspect_message]
fn inspect_messages() {
    inspect_message_impl()
}

#[allow(dead_code)]
fn inspect_message_impl() {
    let method = api::call::method_name();

    let check_result = match method.as_str() {
        method if method.starts_with("admin_") => Inspect::inspect_is_custodian(caller()),
        "sign_contract" => {
            let (id,) = api::call::arg_data::<(ID,)>(ArgDecoderConfig::default());
            Inspect::inspect_sign_contract(caller(), &id)
        }
        "set_logo" | "set_name" | "set_symbol" | "set_custodians" => {
            Inspect::inspect_is_custodian(caller())
        }
        "increment_contract_value" => {
            let (id, _, _) = api::call::arg_data::<(ID, u64, u64)>(ArgDecoderConfig::default());
            Inspect::inspect_increment_contract_value(caller(), id).is_ok()
        }
        "update_contract_property" => {
            let (id, key, _) =
                api::call::arg_data::<(ID, String, GenericValue)>(ArgDecoderConfig::default());
            Inspect::inspect_update_contract_property(caller(), &id, &key).is_ok()
        }
        "update_contract_buyers" => {
            let (id, _) = api::call::arg_data::<(ID, Vec<Principal>)>(ArgDecoderConfig::default());
            Inspect::inspect_is_seller(caller(), id).is_ok()
        }
        "register_contract" => {
            let data =
                api::call::arg_data::<(ContractRegistration,)>(ArgDecoderConfig::default()).0;
            Inspect::inspect_register_contract(
                caller(),
                data.value,
                &data.deposit,
                &data.sellers,
                &data.buyers,
                data.installments,
                data.expiration.as_deref(),
            )
            .is_ok()
        }
        "close_contract" => {
            let id = api::call::arg_data::<(ID,)>(ArgDecoderConfig::default()).0;
            Inspect::inspect_is_agent_for_contract(caller(), &id).is_ok()
                || Inspect::inspect_is_custodian(caller())
        }
        "withdraw_contract_deposit" => {
            let id = api::call::arg_data::<(ID, Option<Subaccount>)>(ArgDecoderConfig::default()).0;
            Inspect::inspect_is_seller(caller(), id).is_ok()
        }
        "get_unsigned_contracts" => {
            Inspect::inspect_is_agent(caller()) || Inspect::inspect_is_custodian(caller())
        }
        "burn" => {
            let token_identifier = api::call::arg_data::<(Nat,)>(ArgDecoderConfig::default()).0;
            Inspect::inspect_burn(caller(), &token_identifier).is_ok()
        }
        "transfer_from" => {
            let (_, _, token_identifier) =
                api::call::arg_data::<(Principal, Principal, Nat)>(ArgDecoderConfig::default());
            Inspect::inspect_is_owner_or_operator(caller(), &token_identifier).is_ok()
        }
        _ => true,
    };

    if check_result {
        api::call::accept_message();
    } else {
        ic_cdk::trap(&format!("Unauthorized call to {}", method));
    }
}
