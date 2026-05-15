//! Top-level entry points for Clarity value decoding exposed to JS.
//!
//! All parsing now goes directly through upstream
//! `clarity::vm::types::Value`. The `decode_clarity_value`,
//! `decode_clarity_value_to_repr`, `decode_clarity_value_type_name`, and
//! `decode_clarity_value_array` functions produce the same JS-facing shapes
//! they always did; the local `Value` / `ClarityValue` enums in
//! `clarity_value::types` are kept only for `pox_events`, which has its own
//! migration scheduled separately.
use std::io::Cursor;

use clarity::vm::types::Value as UpstreamValue;
use neon::prelude::*;

use crate::neon_util::{arg_as_bytes, arg_as_bytes_copied};

use self::neon_encoder::{
    decode_clarity_val, repr_string as upstream_repr_string,
    type_signature_string as upstream_type_signature_string,
};

pub mod deserialize;
pub mod neon_encoder;
pub mod types;

/// Read a single Clarity value from `cursor` using upstream's canonical
/// codec. Returns the value plus a borrowed slice of the bytes consumed by
/// the parse, suitable for the JS `hex` field.
fn read_upstream_value<'a>(
    cursor: &mut Cursor<&'a [u8]>,
) -> Result<(UpstreamValue, &'a [u8]), String> {
    let start = cursor.position() as usize;
    let value = UpstreamValue::deserialize_read(cursor, None, false)
        .map_err(|e| format!("Failed to decode Clarity value: {}", e))?;
    let end = cursor.position() as usize;
    let bytes = &cursor.get_ref()[start..end];
    Ok((value, bytes))
}

pub fn decode_clarity_value(mut cx: FunctionContext) -> JsResult<JsObject> {
    let val_bytes = arg_as_bytes_copied(&mut cx, 0)?;
    let mut cursor: Cursor<&[u8]> = Cursor::new(&val_bytes);
    let value = UpstreamValue::deserialize_read(&mut cursor, None, false)
        .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;

    let root_obj = cx.empty_object();
    decode_clarity_val(&mut cx, &root_obj, &value, true, &val_bytes)?;
    Ok(root_obj)
}

pub fn decode_clarity_value_type_name(mut cx: FunctionContext) -> JsResult<JsString> {
    let type_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        UpstreamValue::deserialize_read(&mut cursor, None, false)
            .map_err(|e| format!("{}", e))
            .map(|v| upstream_type_signature_string(&v))
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(type_string))
}

pub fn decode_clarity_value_to_repr(mut cx: FunctionContext) -> JsResult<JsString> {
    let repr_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        UpstreamValue::deserialize_read(&mut cursor, None, false)
            .map_err(|e| format!("{}", e))
            .map(|v| upstream_repr_string(&v))
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(repr_string))
}

pub fn decode_clarity_value_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    let input_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let result_length = if input_bytes.len() >= 4 {
        u32::from_be_bytes(input_bytes[..4].try_into().unwrap())
    } else {
        0
    };

    let array_result = JsArray::new(&mut cx, result_length as usize);

    let deep: bool = match cx.argument_opt(1) {
        Some(arg) => arg
            .downcast_or_throw::<JsBoolean, _>(&mut cx)?
            .value(&mut cx),
        None => false,
    };

    if input_bytes.len() > 4 {
        let val_slice = &input_bytes[4..];
        let mut byte_cursor = Cursor::new(val_slice);
        let val_len = val_slice.len() as u64;
        let mut i: u32 = 0;
        while byte_cursor.position() < val_len {
            let (value, decoded_bytes) = read_upstream_value(&mut byte_cursor)
                .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
            let value_obj = cx.empty_object();
            decode_clarity_val(&mut cx, &value_obj, &value, deep, decoded_bytes)?;
            array_result.set(&mut cx, i, value_obj)?;
            i += 1;
        }
    }
    Ok(array_result)
}
