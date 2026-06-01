//! Generic helpers for walking `clarity::vm::types::Value` trees while
//! decoding PoX synthetic events.
//!
//! These functions are version-agnostic and shared between every per-contract
//! decoder under `pox_events::pox{4,5,...}`.

use std::collections::BTreeMap;

use clarity::vm::types::{
    CharType, PrincipalData, SequenceData, TupleData, Value as UpstreamValue,
};
use clarity::vm::ClarityName;

use crate::util::hex::encode_hex;

/// `BTreeMap::get` taking a `&str` to look up a `ClarityName` key. The
/// upstream `ClarityName` is a `guarded_string` that derefs to `&str` and
/// implements `Borrow<str>`, so an `&str` lookup works directly.
pub fn tuple_get<'a>(
    tuple: &'a BTreeMap<ClarityName, UpstreamValue>,
    key: &str,
) -> Option<&'a UpstreamValue> {
    tuple.get(key)
}

/// Required-field variant: errors if the key is missing.
pub fn get_tuple_field<'a>(
    tuple: &'a BTreeMap<ClarityName, UpstreamValue>,
    key: &str,
) -> Result<&'a UpstreamValue, String> {
    tuple_get(tuple, key).ok_or_else(|| format!("Missing expected tuple field: {}", key))
}

/// Borrow a `&Tuple`'s data map, or error with a type-tag description.
pub fn extract_tuple(val: &UpstreamValue) -> Result<&BTreeMap<ClarityName, UpstreamValue>, String> {
    match val {
        UpstreamValue::Tuple(TupleData { data_map, .. }) => Ok(data_map),
        other => Err(format!("Expected Tuple, got {}", short_type_name(other))),
    }
}

pub fn extract_uint(val: &UpstreamValue) -> Result<u128, String> {
    match val {
        UpstreamValue::UInt(v) => Ok(*v),
        other => Err(format!("Expected UInt, got {}", short_type_name(other))),
    }
}

pub fn extract_bool(val: &UpstreamValue) -> Result<bool, String> {
    match val {
        UpstreamValue::Bool(b) => Ok(*b),
        other => Err(format!("Expected Bool, got {}", short_type_name(other))),
    }
}

/// Extract an ASCII string from a `string-ascii` Clarity value.
pub fn extract_ascii_string(val: &UpstreamValue) -> Result<String, String> {
    match val {
        UpstreamValue::Sequence(SequenceData::String(CharType::ASCII(s))) => {
            String::from_utf8(s.data.clone())
                .map_err(|e| format!("Invalid ASCII string bytes: {}", e))
        }
        other => Err(format!(
            "Expected StringASCII, got {}",
            short_type_name(other)
        )),
    }
}

/// Extract a list of items, mapping each element through `f`.
pub fn extract_list<T, F>(val: &UpstreamValue, f: F) -> Result<Vec<T>, String>
where
    F: Fn(&UpstreamValue) -> Result<T, String>,
{
    match val {
        UpstreamValue::Sequence(SequenceData::List(list)) => {
            list.data.iter().map(f).collect::<Result<Vec<_>, _>>()
        }
        other => Err(format!("Expected List, got {}", short_type_name(other))),
    }
}

/// Extract an optional uint from:
/// - `None` (field absent) → `Ok(None)`
/// - `OptionalNone` → `Ok(None)`
/// - `OptionalSome(UInt(v))` → `Ok(Some(v))`
/// - `UInt(v)` → `Ok(Some(v))` (for fields that are sometimes bare uints)
pub fn extract_optional_uint(val: Option<&UpstreamValue>) -> Result<Option<u128>, String> {
    let Some(cv) = val else { return Ok(None) };
    match cv {
        UpstreamValue::Optional(opt) => match &opt.data {
            None => Ok(None),
            Some(inner) => match inner.as_ref() {
                UpstreamValue::UInt(v) => Ok(Some(*v)),
                other => Err(format!(
                    "Expected UInt inside OptionalSome, got {}",
                    short_type_name(other)
                )),
            },
        },
        UpstreamValue::UInt(v) => Ok(Some(*v)),
        other => Err(format!(
            "Expected OptionalSome/OptionalNone/UInt, got {}",
            short_type_name(other)
        )),
    }
}

/// Extract a buffer field as a `0x`-prefixed hex string. Errors if the value
/// isn't a `Buffer`.
pub fn extract_buffer_hex(val: &UpstreamValue) -> Result<String, String> {
    match val {
        UpstreamValue::Sequence(SequenceData::Buffer(b)) => Ok(encode_hex(&b.data).to_string()),
        other => Err(format!("Expected Buffer, got {}", short_type_name(other))),
    }
}

/// Extract a buffer as a hex string from:
/// - `None` (field absent) → `Ok(None)`
/// - `OptionalNone` → `Ok(None)`
/// - `Buffer(bytes)` → `Ok(Some("0x..."))`
/// - `OptionalSome(Buffer(bytes))` → `Ok(Some("0x..."))`
pub fn extract_optional_buffer_hex(val: Option<&UpstreamValue>) -> Result<Option<String>, String> {
    let Some(cv) = val else { return Ok(None) };
    match cv {
        UpstreamValue::Sequence(SequenceData::Buffer(b)) => {
            Ok(Some(encode_hex(&b.data).to_string()))
        }
        UpstreamValue::Optional(opt) => match &opt.data {
            None => Ok(None),
            Some(inner) => match inner.as_ref() {
                UpstreamValue::Sequence(SequenceData::Buffer(b)) => {
                    Ok(Some(encode_hex(&b.data).to_string()))
                }
                other => Err(format!(
                    "Expected Buffer inside OptionalSome, got {}",
                    short_type_name(other)
                )),
            },
        },
        other => Err(format!(
            "Expected Buffer/OptionalSome/OptionalNone, got {}",
            short_type_name(other)
        )),
    }
}

/// Convert a Clarity principal value to a c32-encoded string address.
pub fn clarity_principal_to_string(val: &UpstreamValue) -> Result<String, String> {
    match val {
        UpstreamValue::Principal(PrincipalData::Standard(spd)) => {
            crate::upstream::address::c32_address(spd.version(), &spd.1)
        }
        UpstreamValue::Principal(PrincipalData::Contract(qci)) => {
            let addr = crate::upstream::address::c32_address(qci.issuer.version(), &qci.issuer.1)?;
            Ok(format!("{}.{}", addr, qci.name))
        }
        other => Err(format!(
            "Unexpected Clarity value type for principal: {}",
            short_type_name(other)
        )),
    }
}

/// Short human-readable name for an upstream value's outer constructor, used
/// in diagnostic error messages. We don't try to reproduce the full Clarity
/// type name (that's what `crate::upstream::clarity_value::neon_encoder::type_signature_string`
/// is for).
pub fn short_type_name(val: &UpstreamValue) -> &'static str {
    match val {
        UpstreamValue::Int(_) => "Int",
        UpstreamValue::UInt(_) => "UInt",
        UpstreamValue::Bool(_) => "Bool",
        UpstreamValue::Sequence(SequenceData::Buffer(_)) => "Buffer",
        UpstreamValue::Sequence(SequenceData::List(_)) => "List",
        UpstreamValue::Sequence(SequenceData::String(CharType::ASCII(_))) => "StringASCII",
        UpstreamValue::Sequence(SequenceData::String(CharType::UTF8(_))) => "StringUTF8",
        UpstreamValue::Principal(PrincipalData::Standard(_)) => "PrincipalStandard",
        UpstreamValue::Principal(PrincipalData::Contract(_)) => "PrincipalContract",
        UpstreamValue::Tuple(_) => "Tuple",
        UpstreamValue::Optional(opt) => {
            if opt.data.is_none() {
                "OptionalNone"
            } else {
                "OptionalSome"
            }
        }
        UpstreamValue::Response(r) => {
            if r.committed {
                "ResponseOk"
            } else {
                "ResponseErr"
            }
        }
        UpstreamValue::CallableContract(_) => "CallableContract",
    }
}
