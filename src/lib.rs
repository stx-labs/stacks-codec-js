use git_version::git_version;
use neon::prelude::*;

use crate::derived::memo::memo_to_string;
use crate::derived::pox_events::decode_pox_event;
use crate::upstream::address::{
    bitcoin_to_stacks_address, decode_clarity_value_to_principal, decode_stacks_address,
    is_valid_stacks_address, stacks_address_from_parts, stacks_to_bitcoin_address,
};
use crate::upstream::clarity_value::{
    decode_clarity_value, decode_clarity_value_array, decode_clarity_value_to_repr,
    decode_clarity_value_type_name,
};
use crate::upstream::post_condition::decode_tx_post_conditions;
use crate::upstream::stacks_block::{decode_nakamoto_block, decode_stacks_block};
use crate::upstream::stacks_tx::decode_transaction;

pub mod derived;
pub mod upstream;
pub mod util;

const GIT_VERSION: &str = git_version!(
    args = ["--all", "--long", "--always"],
    fallback = "unavailable"
);

fn get_version(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.string(GIT_VERSION);
    Ok(version)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("getVersion", get_version)?;
    cx.export_function("decodeClarityValueToRepr", decode_clarity_value_to_repr)?;
    cx.export_function(
        "decodeClarityValueToTypeName",
        decode_clarity_value_type_name,
    )?;
    cx.export_function("decodeClarityValue", decode_clarity_value)?;
    cx.export_function("decodeClarityValueList", decode_clarity_value_array)?;
    cx.export_function("decodePostConditions", decode_tx_post_conditions)?;
    cx.export_function("decodeTransaction", decode_transaction)?;
    cx.export_function("decodeNakamotoBlock", decode_nakamoto_block)?;
    cx.export_function("decodeStacksBlock", decode_stacks_block)?;
    cx.export_function("stacksToBitcoinAddress", stacks_to_bitcoin_address)?;
    cx.export_function("bitcoinToStacksAddress", bitcoin_to_stacks_address)?;
    cx.export_function("isValidStacksAddress", is_valid_stacks_address)?;
    cx.export_function("decodeStacksAddress", decode_stacks_address)?;
    cx.export_function(
        "decodeClarityValueToPrincipal",
        decode_clarity_value_to_principal,
    )?;
    cx.export_function("stacksAddressFromParts", stacks_address_from_parts)?;
    cx.export_function("memoToString", memo_to_string)?;
    cx.export_function("decodePoxSyntheticEvent", decode_pox_event)?;

    Ok(())
}
