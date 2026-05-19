//! JS-facing encoding of Clarity values.
//!
//! All emission walks `clarity::vm::types::Value` directly. Two formatters
//! are reimplemented here as free functions because their output is part of
//! the public JS contract and intentionally differs from upstream
//! `fmt::Display`:
//!
//! * `repr_string` keeps the historical wire format used by Stacks API
//!   responses (e.g. `(list ...)` keyword for sequences, single-quote prefix
//!   on principals).
//! * `type_signature_string` keeps the historical, value-derived type tag
//!   format (e.g. `(string-utf8 N)` where `N` is `chars * 4`).
//!
//! Nested values do not carry their own serialized-bytes cache; we recompute
//! them on the fly via the canonical `StacksMessageCodec` impl. The
//! top-level call still takes the original input bytes so the `hex` field
//! matches the user's input verbatim.
use std::io::Write;

use clarity::vm::types::serialization::TypePrefix as UpstreamTypePrefix;
use clarity::vm::types::{CharType, PrincipalData, SequenceData, Value as UpstreamValue};
use neon::prelude::*;
use stacks_codec::StacksMessageCodec;

use crate::address::c32_address;
use crate::hex::{encode_hex, encode_hex_no_prefix};

/// Emit a single Clarity value as a JS object on `cur_obj`.
///
/// `bytes` is the raw wire encoding of `val` (the input slice that produced
/// `val`). For nested sub-values inside lists / tuples / optionals /
/// responses we recompute the wire bytes via the canonical codec.
pub fn decode_clarity_val<T: AsRef<[u8]>>(
    cx: &mut FunctionContext,
    cur_obj: &Handle<JsObject>,
    val: &UpstreamValue,
    deep: bool,
    bytes: T,
) -> NeonResult<()> {
    let repr = cx.string(repr_string(val));
    cur_obj.set(cx, "repr", repr)?;

    let hex = cx.string(encode_hex(bytes.as_ref()));
    cur_obj.set(cx, "hex", hex)?;

    let type_id = cx.number(UpstreamTypePrefix::from(val).to_u8());
    cur_obj.set(cx, "type_id", type_id)?;

    if !deep {
        return Ok(());
    }

    match val {
        UpstreamValue::Int(v) => {
            let s = cx.string(v.to_string());
            cur_obj.set(cx, "value", s)?;
        }
        UpstreamValue::UInt(v) => {
            let s = cx.string(v.to_string());
            cur_obj.set(cx, "value", s)?;
        }
        UpstreamValue::Bool(b) => {
            let v = cx.boolean(*b);
            cur_obj.set(cx, "value", v)?;
        }
        UpstreamValue::Sequence(SequenceData::Buffer(buff)) => {
            let s = cx.string(encode_hex(&buff.data));
            cur_obj.set(cx, "buffer", s)?;
        }
        UpstreamValue::Sequence(SequenceData::List(list)) => {
            let list_obj = JsArray::new(cx, list.data.len());
            for (i, child) in list.data.iter().enumerate() {
                let item_obj = cx.empty_object();
                let child_bytes = serialize_value(child);
                decode_clarity_val(cx, &item_obj, child, deep, &child_bytes)?;
                list_obj.set(cx, i as u32, item_obj)?;
            }
            cur_obj.set(cx, "list", list_obj)?;
        }
        UpstreamValue::Sequence(SequenceData::String(CharType::ASCII(ascii))) => {
            let s = cx.string(String::from_utf8_lossy(&ascii.data));
            cur_obj.set(cx, "data", s)?;
        }
        UpstreamValue::Sequence(SequenceData::String(CharType::UTF8(utf8))) => {
            let utf8_bytes: Vec<u8> = utf8.data.iter().flatten().copied().collect();
            let utf8_str = String::from_utf8_lossy(&utf8_bytes);
            let s = cx.string(utf8_str);
            cur_obj.set(cx, "data", s)?;
        }
        UpstreamValue::Principal(PrincipalData::Standard(spd)) => {
            let version = spd.version();
            let hash = &spd.1;
            emit_principal_standard(cx, cur_obj, version, hash)?;
        }
        UpstreamValue::Principal(PrincipalData::Contract(qci)) => {
            let version = qci.issuer.version();
            let hash = &qci.issuer.1;
            emit_principal_standard(cx, cur_obj, version, hash)?;
            let name = cx.string(qci.name.as_str());
            cur_obj.set(cx, "contract_name", name)?;
        }
        UpstreamValue::Tuple(tuple) => {
            let tuple_obj = cx.empty_object();
            for (key, child) in tuple.data_map.iter() {
                let val_obj = cx.empty_object();
                let child_bytes = serialize_value(child);
                decode_clarity_val(cx, &val_obj, child, deep, &child_bytes)?;
                tuple_obj.set(cx, key.as_str(), val_obj)?;
            }
            cur_obj.set(cx, "data", tuple_obj)?;
        }
        UpstreamValue::Optional(opt) => match &opt.data {
            Some(inner) => {
                let inner_obj = cx.empty_object();
                let inner_bytes = serialize_value(inner);
                decode_clarity_val(cx, &inner_obj, inner, deep, &inner_bytes)?;
                cur_obj.set(cx, "value", inner_obj)?;
            }
            None => {
                let null = cx.null();
                cur_obj.set(cx, "value", null)?;
            }
        },
        UpstreamValue::Response(resp) => {
            let inner_obj = cx.empty_object();
            let inner_bytes = serialize_value(&resp.data);
            decode_clarity_val(cx, &inner_obj, &resp.data, deep, &inner_bytes)?;
            cur_obj.set(cx, "value", inner_obj)?;
        }
        UpstreamValue::CallableContract(_) => {
            // Callable values are runtime-only and never appear on the wire,
            // so we should not see them here in practice. Emit an empty
            // payload to match `deep`'s contract instead of panicking.
        }
    }
    Ok(())
}

fn emit_principal_standard(
    cx: &mut FunctionContext,
    cur_obj: &Handle<JsObject>,
    version: u8,
    hash: &[u8; 20],
) -> NeonResult<()> {
    let address_version = cx.number(version);
    cur_obj.set(cx, "address_version", address_version)?;

    let address_hash_bytes = cx.string(encode_hex(hash));
    cur_obj.set(cx, "address_hash_bytes", address_hash_bytes)?;

    let address_string = c32_address(version, hash)
        .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;
    let address = cx.string(address_string);
    cur_obj.set(cx, "address", address)?;
    Ok(())
}

/// Canonical wire encoding of a Clarity value.
///
/// Upstream `Value` implements `StacksMessageCodec`, so `serialize_to_vec`
/// is guaranteed to produce the bytes that would deserialize back to the
/// same value.
fn serialize_value(val: &UpstreamValue) -> Vec<u8> {
    <UpstreamValue as StacksMessageCodec>::serialize_to_vec(val)
}

/// JS-facing `repr` string. Walks the value tree to produce the historical
/// Stacks-API representation (e.g. `(list ...)`, leading `'` on principals).
pub fn repr_string(val: &UpstreamValue) -> String {
    let mut buf: Vec<u8> = Vec::new();
    repr_to_buffer(val, &mut buf).expect("writing to Vec<u8> cannot fail");
    // SAFETY: all writers below emit valid UTF-8 (ASCII escapes, c32 strings,
    // and hex encodings). We accept escaped UTF-8 sequences inside string
    // values verbatim, which `escape_default` keeps within ASCII range.
    unsafe { String::from_utf8_unchecked(buf) }
}

fn repr_to_buffer(val: &UpstreamValue, w: &mut Vec<u8>) -> std::io::Result<()> {
    match val {
        UpstreamValue::Int(n) => write!(w, "{}", n),
        UpstreamValue::UInt(n) => write!(w, "u{}", n),
        UpstreamValue::Bool(b) => write!(w, "{}", b),
        UpstreamValue::Optional(opt) => match &opt.data {
            Some(inner) => {
                write!(w, "(some ")?;
                repr_to_buffer(inner, w)?;
                write!(w, ")")
            }
            None => write!(w, "none"),
        },
        UpstreamValue::Response(resp) => {
            let tag = if resp.committed { "ok" } else { "err" };
            write!(w, "({} ", tag)?;
            repr_to_buffer(&resp.data, w)?;
            write!(w, ")")
        }
        UpstreamValue::Tuple(tuple) => {
            write!(w, "(tuple")?;
            for (name, inner) in tuple.data_map.iter() {
                write!(w, " ({} ", name.as_str())?;
                repr_to_buffer(inner, w)?;
                write!(w, ")")?;
            }
            write!(w, ")")
        }
        UpstreamValue::Principal(PrincipalData::Standard(spd)) => {
            write!(w, "'{}", c32_address(spd.version(), &spd.1).unwrap())
        }
        UpstreamValue::Principal(PrincipalData::Contract(qci)) => {
            write!(
                w,
                "'{}.{}",
                c32_address(qci.issuer.version(), &qci.issuer.1).unwrap(),
                qci.name.as_str()
            )
        }
        UpstreamValue::Sequence(SequenceData::Buffer(buff)) => {
            write!(w, "{}", encode_hex(&buff.data))
        }
        UpstreamValue::Sequence(SequenceData::List(list)) => {
            write!(w, "(list")?;
            for child in &list.data {
                write!(w, " ")?;
                repr_to_buffer(child, w)?;
            }
            write!(w, ")")
        }
        UpstreamValue::Sequence(SequenceData::String(CharType::ASCII(ascii))) => {
            write!(w, "\"")?;
            for c in ascii.data.iter() {
                write!(w, "{}", std::ascii::escape_default(*c))?;
            }
            write!(w, "\"")
        }
        UpstreamValue::Sequence(SequenceData::String(CharType::UTF8(utf8))) => {
            write!(w, "u\"")?;
            for c in utf8.data.iter() {
                if c.len() > 1 {
                    write!(w, "\\u{{{}}}", encode_hex_no_prefix(c))?;
                } else {
                    write!(w, "{}", std::ascii::escape_default(c[0]))?;
                }
            }
            write!(w, "\"")
        }
        UpstreamValue::CallableContract(callable) => {
            // Same shape as a contract principal; runtime-only so unreachable
            // in practice, but we keep this defensive branch to avoid
            // panics if it ever leaks into a deserialized value.
            let qci = &callable.contract_identifier;
            write!(
                w,
                "'{}.{}",
                c32_address(qci.issuer.version(), &qci.issuer.1).unwrap(),
                qci.name.as_str()
            )
        }
    }
}

/// JS-facing `type_signature` string. Derived from the value tree (not from
/// upstream `TypeSignature::to_string`) because the historical contract
/// uses value-derived sizes (e.g. `(buff N)` where N is the actual buffer
/// length, and `(string-utf8 N)` where N is `chars * 4`).
pub fn type_signature_string(val: &UpstreamValue) -> String {
    let mut buf: Vec<u8> = Vec::new();
    type_signature_to_buffer(val, &mut buf).expect("writing to Vec<u8> cannot fail");
    unsafe { String::from_utf8_unchecked(buf) }
}

fn type_signature_to_buffer(val: &UpstreamValue, w: &mut Vec<u8>) -> std::io::Result<()> {
    match val {
        UpstreamValue::Int(_) => write!(w, "int"),
        UpstreamValue::UInt(_) => write!(w, "uint"),
        UpstreamValue::Bool(_) => write!(w, "bool"),
        UpstreamValue::Optional(opt) => match &opt.data {
            Some(inner) => {
                write!(w, "(optional ")?;
                type_signature_to_buffer(inner, w)?;
                write!(w, ")")
            }
            None => write!(w, "(optional UnknownType)"),
        },
        UpstreamValue::Response(resp) => {
            if resp.committed {
                write!(w, "(response ")?;
                type_signature_to_buffer(&resp.data, w)?;
                write!(w, " UnknownType)")
            } else {
                write!(w, "(response UnknownType ")?;
                type_signature_to_buffer(&resp.data, w)?;
                write!(w, ")")
            }
        }
        UpstreamValue::Tuple(tuple) => {
            write!(w, "(tuple")?;
            for (name, inner) in tuple.data_map.iter() {
                write!(w, " ({} ", name.as_str())?;
                type_signature_to_buffer(inner, w)?;
                write!(w, ")")?;
            }
            write!(w, ")")
        }
        UpstreamValue::Principal(_) | UpstreamValue::CallableContract(_) => write!(w, "principal"),
        UpstreamValue::Sequence(SequenceData::Buffer(buff)) => {
            write!(w, "(buff {})", buff.data.len())
        }
        UpstreamValue::Sequence(SequenceData::List(list)) => {
            write!(w, "(list {} ", list.data.len())?;
            if let Some(first) = list.data.first() {
                // TODO: this should use the least common supertype.
                type_signature_to_buffer(first, w)?;
            } else {
                write!(w, "UnknownType")?;
            }
            write!(w, ")")
        }
        UpstreamValue::Sequence(SequenceData::String(CharType::ASCII(ascii))) => {
            write!(w, "(string-ascii {})", ascii.data.len())
        }
        UpstreamValue::Sequence(SequenceData::String(CharType::UTF8(utf8))) => {
            // Historical contract: report char-count × 4 (the worst-case
            // byte length of a UTF-8 codepoint). Upstream `TypeSignature`
            // tracks the same quantity but we recompute from `chars * 4` to
            // avoid pulling in the typechecker just for a string.
            write!(w, "(string-utf8 {})", utf8.data.len() * 4)
        }
    }
}
