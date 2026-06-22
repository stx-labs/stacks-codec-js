//! Transaction post-condition decoding (Neon entry point + types).
//!
//! All types come directly from upstream
//! (`blockstack_lib::chainstate::stacks`); parsing goes through the canonical
//! `<TransactionPostCondition as StacksMessageCodec>::consensus_deserialize`.
//! The Neon encoder for `TransactionPostCondition` (and friends) lives in
//! [`neon_encoder`] and operates on these upstream types via the
//! `Encode<'_, T>` newtype wrapper from `crate::util::neon`.

use std::{convert::TryInto, io::Cursor};

pub use blockstack_lib::chainstate::stacks::{
    AssetInfo, AssetInfoID, FungibleConditionCode, NonfungibleConditionCode,
    PostConditionPrincipal, PostConditionPrincipalID, PoxConditionCode, TransactionPostCondition,
};
use neon::prelude::*;
use stacks_codec::StacksMessageCodec;

use crate::util::hex::encode_hex;
use crate::util::neon::{arg_as_bytes_copied, Encode, NeonJsSerialize};
use crate::util::serialize::DeserializeError;

pub mod neon_encoder;

/// Deserialize a single post-condition entry from the wire format.
pub fn deserialize_post_condition(
    fd: &mut Cursor<&[u8]>,
) -> Result<TransactionPostCondition, DeserializeError> {
    <TransactionPostCondition as StacksMessageCodec>::consensus_deserialize(fd)
        .map_err(|e| DeserializeError::from(format!("Failed to decode post-condition: {}", e)))
}

pub fn decode_tx_post_conditions(mut cx: FunctionContext) -> JsResult<JsObject> {
    let input_bytes = arg_as_bytes_copied(&mut cx, 0)?;
    let resp_obj = cx.empty_object();

    // First byte is post-condition mode.
    let post_condition_mode = cx.number(input_bytes[0]);
    resp_obj.set(&mut cx, "post_condition_mode", post_condition_mode)?;

    let array_result = if input_bytes.len() > 4 {
        // Next 4 bytes are array length.
        let result_length = u32::from_be_bytes(input_bytes[1..5].try_into().or_else(|e| {
            cx.throw_error(format!(
                "Error reading post condition bytes {}, {}",
                encode_hex(&input_bytes),
                e
            ))
        })?);
        let array_result = JsArray::new(&mut cx, result_length as usize);
        let post_condition_bytes = &input_bytes[5..];
        let post_condition_bytes_len = post_condition_bytes.len() as u64;
        let mut cursor = Cursor::new(post_condition_bytes);
        let mut i: u32 = 0;
        while cursor.position() < post_condition_bytes_len {
            let post_condition = deserialize_post_condition(&mut cursor).or_else(|e| {
                cx.throw_error(format!("Error deserializing post condition: {}", e.error))
            })?;
            let value_obj = cx.empty_object();
            Encode(&post_condition).neon_js_serialize(&mut cx, &value_obj, &())?;
            array_result.set(&mut cx, i, value_obj)?;
            i += 1;
        }
        array_result
    } else {
        cx.empty_array()
    };
    resp_obj.set(&mut cx, "post_conditions", array_result)?;
    Ok(resp_obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::hex::decode_hex;
    use flate2::read::GzDecoder;
    use std::io::{prelude::*, BufReader};

    const SAMPLED_POST_CONDITIONS: &'static [u8] =
        include_bytes!("../../../tests/fixtures/sampled-post-conditions.txt.gz");

    #[test]
    fn test_decode_samples() {
        let decoder = GzDecoder::new(SAMPLED_POST_CONDITIONS);
        let reader = BufReader::new(decoder);
        for line in reader.lines().filter_map(|s| s.ok()) {
            let input_bytes = decode_hex(line).unwrap();
            let post_condition_bytes = &input_bytes[5..];
            let post_condition_bytes_len = post_condition_bytes.len() as u64;
            let mut cursor = Cursor::new(post_condition_bytes);
            while cursor.position() < post_condition_bytes_len {
                deserialize_post_condition(&mut cursor).unwrap();
            }
        }
    }
}
