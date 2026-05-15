//! Clarity value deserialization.
//!
//! The wire-format parser for Clarity values now lives upstream in
//! `clarity::vm::types::Value::deserialize_read`. This module is a thin façade
//! that runs the upstream decoder and adapts its value tree into the local
//! [`ClarityValue`] type so the rest of this crate (and the Neon encoder in
//! particular) can keep their existing shapes and import paths.
//!
//! The smaller helpers (`ClarityName::deserialize`, `ContractName::deserialize`,
//! `StandardPrincipalData::deserialize`) are kept here because the not-yet-
//! migrated modules (`stacks_tx`, `post_condition`, `address`) still call them
//! directly to parse standalone wire-format components. They will go away once
//! those modules are migrated to upstream as well.

use byteorder::ReadBytesExt;
use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use clarity::vm::types::{
    BuffData, CharType, ListData, OptionalData, PrincipalData, ResponseData, SequenceData,
    TupleData, Value as UpstreamValue,
};
use stacks_common::codec::StacksMessageCodec;

use super::types::*;
use crate::serialize_util::DeserializeError;

/// Wire-format type prefix bytes for Clarity values.
///
/// Numerically identical to upstream's `clarity_types::types::serialization::TypePrefix`,
/// but redefined here because: (a) other modules in this crate import it via
/// `crate::clarity_value::deserialize::TypePrefix`, and (b) we don't want to leak
/// the upstream import path into call sites that don't otherwise touch upstream.
#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum TypePrefix {
    Int = 0,
    UInt = 1,
    Buffer = 2,
    BoolTrue = 3,
    BoolFalse = 4,
    PrincipalStandard = 5,
    PrincipalContract = 6,
    ResponseOk = 7,
    ResponseErr = 8,
    OptionalNone = 9,
    OptionalSome = 10,
    List = 11,
    Tuple = 12,
    StringASCII = 13,
    StringUTF8 = 14,
}

impl TypePrefix {
    pub fn to_u8(&self) -> u8 {
        self.clone() as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Int),
            1 => Some(Self::UInt),
            2 => Some(Self::Buffer),
            3 => Some(Self::BoolTrue),
            4 => Some(Self::BoolFalse),
            5 => Some(Self::PrincipalStandard),
            6 => Some(Self::PrincipalContract),
            7 => Some(Self::ResponseOk),
            8 => Some(Self::ResponseErr),
            9 => Some(Self::OptionalNone),
            10 => Some(Self::OptionalSome),
            11 => Some(Self::List),
            12 => Some(Self::Tuple),
            13 => Some(Self::StringASCII),
            14 => Some(Self::StringUTF8),
            _ => None,
        }
    }
}

impl ContractName {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let len_byte: u8 = fd.read_u8()?;
        if (len_byte as usize) < CONTRACT_MIN_NAME_LENGTH
            || (len_byte as usize) > CONTRACT_MAX_NAME_LENGTH
        {
            return Err(format!(
                "Failed to deserialize contract name: too short or too long: {}",
                len_byte
            ))?;
        }
        let mut bytes = vec![0u8; len_byte as usize];
        fd.read_exact(&mut bytes)?;

        let s = String::from_utf8(bytes).map_err(|e| {
            format!(
                "Failed to parse Contract name: could not construct from utf8: {}",
                e
            )
        })?;

        Ok(ContractName(s))
    }
}

impl ClarityName {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let len_byte = fd.read_u8()?;
        if len_byte > MAX_STRING_LEN {
            return Err(format!(
                "Failed to deserialize clarity name: too long: {}",
                len_byte,
            ))?;
        }
        let mut bytes = vec![0u8; len_byte as usize];
        fd.read_exact(&mut bytes)?;

        let s = String::from_utf8(bytes).map_err(|e| {
            format!(
                "Failed to parse Clarity name: could not contruct from utf8: {}",
                e
            )
        })?;

        Ok(ClarityName(s))
    }
}

impl StandardPrincipalData {
    pub fn deserialize(r: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let version = r.read_u8()?;
        let mut data = [0; 20];
        r.read_exact(&mut data)?;
        Ok(StandardPrincipalData(version, data))
    }
}

impl ClarityValue {
    /// Deserialize a Clarity value from the wire format.
    ///
    /// Internally delegates to upstream's canonical implementation
    /// ([`clarity::vm::types::Value::deserialize_read`]) and then converts the
    /// resulting tree into the local [`ClarityValue`] type so the existing Neon
    /// encoder doesn't need to change.
    ///
    /// When `with_bytes` is true, every node in the resulting tree carries its
    /// own serialized byte slice. The top-level slice is taken directly from the
    /// input cursor; nested-value slices are produced by re-serializing each
    /// node via upstream's canonical encoder. Clarity wire encoding is
    /// deterministic, so the round-tripped bytes match the original input.
    pub fn deserialize(
        r: &mut Cursor<&[u8]>,
        with_bytes: bool,
    ) -> Result<ClarityValue, DeserializeError> {
        let cursor_start = r.position() as usize;
        let upstream_value = UpstreamValue::deserialize_read(r, None, false)
            .map_err(|e| DeserializeError::from(format!("Failed to decode Clarity value: {}", e)))?;
        let cursor_end = r.position() as usize;

        let value = convert_value(&upstream_value, with_bytes);

        if with_bytes {
            let bytes = &r.get_ref()[cursor_start..cursor_end];
            Ok(ClarityValue::new_with_bytes(bytes, value))
        } else {
            Ok(ClarityValue::new(value))
        }
    }
}

fn convert_value(upstream: &UpstreamValue, with_bytes: bool) -> Value {
    match upstream {
        UpstreamValue::Int(v) => Value::Int(*v),
        UpstreamValue::UInt(v) => Value::UInt(*v),
        UpstreamValue::Bool(b) => Value::Bool(*b),
        UpstreamValue::Sequence(seq) => match seq {
            SequenceData::Buffer(BuffData { data }) => Value::Buffer(data.clone()),
            SequenceData::List(ListData { data, .. }) => {
                let items = data
                    .iter()
                    .map(|v| convert_clarity_value(v, with_bytes))
                    .collect();
                Value::List(items)
            }
            SequenceData::String(CharType::ASCII(ascii)) => Value::StringASCII(ascii.data.clone()),
            SequenceData::String(CharType::UTF8(utf8)) => Value::StringUTF8(utf8.data.clone()),
        },
        UpstreamValue::Principal(PrincipalData::Standard(principal)) => {
            Value::PrincipalStandard(StandardPrincipalData(principal.version(), principal.1))
        }
        UpstreamValue::Principal(PrincipalData::Contract(qci)) => {
            Value::PrincipalContract(QualifiedContractIdentifier {
                issuer: StandardPrincipalData(qci.issuer.version(), qci.issuer.1),
                name: ClarityName(qci.name.to_string()),
            })
        }
        UpstreamValue::Tuple(TupleData { data_map, .. }) => {
            let mut data = BTreeMap::new();
            for (key, value) in data_map.iter() {
                data.insert(
                    ClarityName(key.to_string()),
                    convert_clarity_value(value, with_bytes),
                );
            }
            Value::Tuple(data)
        }
        UpstreamValue::Optional(OptionalData {
            data: Some(boxed), ..
        }) => Value::OptionalSome(Box::new(convert_clarity_value(boxed, with_bytes))),
        UpstreamValue::Optional(OptionalData { data: None, .. }) => Value::OptionalNone,
        UpstreamValue::Response(ResponseData {
            committed: true,
            data,
            ..
        }) => Value::ResponseOk(Box::new(convert_clarity_value(data, with_bytes))),
        UpstreamValue::Response(ResponseData {
            committed: false,
            data,
            ..
        }) => Value::ResponseErr(Box::new(convert_clarity_value(data, with_bytes))),
        UpstreamValue::CallableContract(_) => {
            // Runtime-only variant constructed when invoking traits; it has no
            // wire-format representation, so deserialize_read above can never
            // produce one from the byte stream.
            unreachable!("CallableContract is not part of the consensus serialization")
        }
    }
}

fn convert_clarity_value(upstream: &UpstreamValue, with_bytes: bool) -> ClarityValue {
    let value = convert_value(upstream, with_bytes);
    let serialized_bytes = if with_bytes {
        // Use the StacksMessageCodec trait method (returns Vec<u8>) rather than
        // Value's identically-named inherent method (returns Result<Vec<u8>, ..>).
        // Serialization to an in-memory buffer cannot actually fail.
        Some(<UpstreamValue as StacksMessageCodec>::serialize_to_vec(upstream))
    } else {
        None
    };
    ClarityValue {
        serialized_bytes,
        value,
    }
}
