//! Stacks / Bitcoin address conversions and Neon entry points.
//!
//! All wire-format work is delegated to upstream
//! (`stacks_common::address::{c32,b58}`, `clarity::vm::types::PrincipalData`,
//! `blockstack_lib::burnchains::bitcoin::address::LegacyBitcoinAddress`,
//! `stacks_common::types::chainstate::StacksAddress`). This module hosts:
//!
//! - Thin wrappers over `c32_address` / `c32_address_decode` that normalize
//!   the error type to `String` and the byte vec to `[u8; 20]` for callers.
//! - The four C32 version-byte constants for the Stacks address space.
//! - The b58 re-exports the rest of the crate uses.
//! - The local btc↔stx version-byte mapping and the Neon-facing JS bindings.
//!
//! The Neon `NeonJsSerialize` impls for `StacksAddress` / `PrincipalData` /
//! `StandardPrincipalData` live in `neon_encoder.rs` — that file is the only
//! sub-module remaining under this directory.

use std::io::Cursor;

use blockstack_lib::burnchains::bitcoin::address::{
    legacy_address_type_to_version_byte, LegacyBitcoinAddress, ADDRESS_VERSION_MAINNET_MULTISIG,
    ADDRESS_VERSION_MAINNET_SINGLESIG, ADDRESS_VERSION_TESTNET_MULTISIG,
    ADDRESS_VERSION_TESTNET_SINGLESIG,
};
use clarity::vm::types::PrincipalData;
use neon::prelude::*;
#[cfg(feature = "profiling")]
use neon::types::buffer::TypedArray;
use stacks_common::codec::StacksMessageCodec;
use stacks_common::types::chainstate::StacksAddress;
use stacks_common::util::hash::Hash160;

use crate::hex::encode_hex;
use crate::neon_util::{arg_as_bytes, arg_as_bytes_copied};

pub mod neon_encoder;

// Base58check helpers. The full implementation lives in
// `stacks_common::address::b58`; re-exported here so callers can use
// `crate::address::{check_encode_slice, from_check}` directly.
pub use stacks_common::address::b58::{check_encode_slice, from_check};

// C32 version bytes for the Stacks address space.
pub const C32_ADDRESS_VERSION_MAINNET_SINGLESIG: u8 = 22; // P
pub const C32_ADDRESS_VERSION_MAINNET_MULTISIG: u8 = 20; // M
pub const C32_ADDRESS_VERSION_TESTNET_SINGLESIG: u8 = 26; // T
pub const C32_ADDRESS_VERSION_TESTNET_MULTISIG: u8 = 21; // N

/// Wrapper over `stacks_common::address::c32::c32_address` that normalizes
/// the error type to `String`.
pub fn c32_address(version: u8, data: &[u8]) -> Result<String, String> {
    stacks_common::address::c32::c32_address(version, data).map_err(|e| format!("{}", e))
}

/// Wrapper over `stacks_common::address::c32::c32_address_decode` that
/// reshapes the returned `Vec<u8>` into a fixed `[u8; 20]` (matching how
/// every caller in this crate consumes it) and normalizes the error type
/// to `String`.
pub fn c32_address_decode(c32_address_str: &str) -> Result<(u8, [u8; 20]), String> {
    let (version, bytes) = stacks_common::address::c32::c32_address_decode(c32_address_str)
        .map_err(|e| format!("{}", e))?;
    let bytes: [u8; 20] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| format!("c32 address decoded to {} bytes, expected 20", bytes.len()))?;
    Ok((version, bytes))
}

fn btc_to_stx_addr_version_byte(version: u8) -> Option<u8> {
    match version {
        ADDRESS_VERSION_MAINNET_SINGLESIG => Some(C32_ADDRESS_VERSION_MAINNET_SINGLESIG),
        ADDRESS_VERSION_MAINNET_MULTISIG => Some(C32_ADDRESS_VERSION_MAINNET_MULTISIG),
        ADDRESS_VERSION_TESTNET_SINGLESIG => Some(C32_ADDRESS_VERSION_TESTNET_SINGLESIG),
        ADDRESS_VERSION_TESTNET_MULTISIG => Some(C32_ADDRESS_VERSION_TESTNET_MULTISIG),
        _ => None,
    }
}

fn stx_to_btc_version_byte(version: u8) -> Option<u8> {
    match version {
        C32_ADDRESS_VERSION_MAINNET_SINGLESIG => Some(ADDRESS_VERSION_MAINNET_SINGLESIG),
        C32_ADDRESS_VERSION_MAINNET_MULTISIG => Some(ADDRESS_VERSION_MAINNET_MULTISIG),
        C32_ADDRESS_VERSION_TESTNET_SINGLESIG => Some(ADDRESS_VERSION_TESTNET_SINGLESIG),
        C32_ADDRESS_VERSION_TESTNET_MULTISIG => Some(ADDRESS_VERSION_TESTNET_MULTISIG),
        _ => None,
    }
}

fn btc_addr_to_stx_addr_version(addr: &LegacyBitcoinAddress) -> Result<u8, String> {
    let btc_version = legacy_address_type_to_version_byte(addr.addrtype, addr.network_id);
    btc_to_stx_addr_version_byte(btc_version).ok_or_else(|| {
        format!(
            "Failed to decode Bitcoin version byte to Stacks version byte: {}",
            btc_version
        )
    })
}

fn btc_addr_to_stx_addr(addr: &LegacyBitcoinAddress) -> Result<StacksAddress, String> {
    let version = btc_addr_to_stx_addr_version(addr)?;
    StacksAddress::new(version, addr.bytes.clone())
        .map_err(|e| format!("Invalid Stacks address version {}: {}", version, e))
}

fn stx_addr_to_btc_addr(addr: &StacksAddress) -> String {
    let btc_version = stx_to_btc_version_byte(addr.version())
        // fallback to version
        .unwrap_or(addr.version());
    let mut all_bytes = vec![btc_version];
    all_bytes.extend_from_slice(addr.bytes().as_bytes());
    check_encode_slice(&all_bytes)
}

pub fn is_valid_stacks_address(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let address_string = cx.argument::<JsString>(0)?.value(&mut cx);
    let address = c32_address_decode(&address_string);
    match address {
        Ok(_) => Ok(cx.boolean(true)),
        Err(_) => Ok(cx.boolean(false)),
    }
}

pub fn decode_stacks_address(mut cx: FunctionContext) -> JsResult<JsArray> {
    let address_string = cx.argument::<JsString>(0)?.value(&mut cx);
    let address = c32_address_decode(&address_string)
        .or_else(|e| cx.throw_error(format!("Error parsing Stacks address {}", e)))?;
    let version = cx.number(address.0);

    let hash160 = cx.string(encode_hex(&address.1));

    let array_resp = JsArray::new(&mut cx, 2);
    array_resp.set(&mut cx, 0, version)?;
    array_resp.set(&mut cx, 1, hash160)?;
    Ok(array_resp)
}

fn decode_clarity_value_to_principal_inner(arg_bytes: &[u8]) -> Result<String, String> {
    let mut cursor: Cursor<&[u8]> = Cursor::new(arg_bytes);
    let principal = PrincipalData::consensus_deserialize(&mut cursor)
        .map_err(|e| format!("Failed to deserialize principal: {}", e))?;
    let addr = match principal {
        PrincipalData::Standard(p) => {
            let (version, bytes) = p.destruct();
            c32_address(version, &bytes)
                .map_err(|e| format!("Failed to encode principal to c32 address: {}", e))?
        }
        PrincipalData::Contract(qci) => {
            let (version, bytes) = qci.issuer.destruct();
            let c32_addr = c32_address(version, &bytes)
                .map_err(|e| format!("Failed to encode principal to c32 address: {}", e))?;
            format!("{}.{}", c32_addr, qci.name.as_str())
        }
    };
    Ok(addr)
}

pub fn decode_clarity_value_to_principal(mut cx: FunctionContext) -> JsResult<JsString> {
    let arg_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let addr = decode_clarity_value_to_principal_inner(&arg_bytes).or_else(|e| {
        cx.throw_error(format!(
            "Error decoding clarity value to principal string: {}",
            e
        ))
    })?;

    Ok(cx.string(addr))
}

pub fn stacks_address_from_parts(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let stacks_address = arg_as_bytes(&mut cx, 1, |bytes| {
        let addr = c32_address(version as u8, bytes)
            .map_err(|e| format!("Error converting to C32 address: {}", e))?;
        Ok(addr)
    })
    .or_else(|e| cx.throw_error(e)?)?;
    let resp = cx.string(stacks_address);
    Ok(resp)
}

fn stacks_to_bitcoin_address_internal(input: String) -> Result<String, String> {
    let (version, bytes) =
        c32_address_decode(&input).map_err(|e| format!("Error decoding c32 address: {}", e))?;
    let stacks_address = StacksAddress::new(version, Hash160(bytes))
        .map_err(|e| format!("Invalid Stacks address version {}: {}", version, e))?;
    Ok(stx_addr_to_btc_addr(&stacks_address))
}

pub fn stacks_to_bitcoin_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let stacks_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    let btc_address =
        stacks_to_bitcoin_address_internal(stacks_address_arg).or_else(|e| cx.throw_error(e))?;
    let btc_address = cx.string(btc_address);
    Ok(btc_address)
}

pub fn bitcoin_to_stacks_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let bitcoin_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    let bitcoin_address = LegacyBitcoinAddress::from_b58(&bitcoin_address_arg)
        .or_else(|e| cx.throw_error(format!("Error parsing Bitcoin address: {:?}", e)))?;

    let stacks_addr = btc_addr_to_stx_addr(&bitcoin_address).or_else(|e| {
        cx.throw_error(format!(
            "Error getting Stacks address version from Bitcoin address: {}",
            e
        ))
    })?;

    let stacks_addr = c32_address(stacks_addr.version(), stacks_addr.bytes().as_bytes())
        .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;

    Ok(cx.string(stacks_addr))
}

#[cfg(feature = "profiling")]
pub fn perf_test_c32_encode(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    use rand::Rng;
    let mut inputs: Vec<(u8, [u8; 20])> = vec![];
    for _ in 0..2000 {
        let random_version: u8 = rand::thread_rng().gen_range(0..31);
        let random_bytes = rand::thread_rng().gen::<[u8; 20]>();
        inputs.push((random_version, random_bytes));
    }

    let profiler = pprof::ProfilerGuard::new(100)
        .or_else(|e| cx.throw_error(format!("Failed to create profiler guard: {}", e))?)?;

    for (version, bytes) in inputs {
        for _ in 0..50_000 {
            c32_address(version, &bytes).unwrap();
        }
    }

    let report = profiler.report().build().unwrap();
    let mut buf = Vec::new();
    report
        .flamegraph(&mut buf)
        .or_else(|e| cx.throw_error(format!("Error creating flamegraph: {}", e)))?;

    let mut result = cx.buffer(buf.len())?;
    result.as_mut_slice(&mut cx).copy_from_slice(&buf);
    Ok(result)
}

#[cfg(feature = "profiling")]
pub fn perf_test_c32_decode(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    use rand::Rng;
    let mut inputs: Vec<String> = vec![];
    for _ in 0..2000 {
        let random_version: u8 = rand::thread_rng().gen_range(0..31);
        let random_bytes = rand::thread_rng().gen::<[u8; 20]>();
        let addr = c32_address(random_version, &random_bytes).unwrap();
        inputs.push(addr);
    }

    let profiler = pprof::ProfilerGuard::new(100)
        .or_else(|e| cx.throw_error(format!("Failed to create profiler guard: {}", e))?)?;

    for _ in 0..50_000 {
        for addr in &inputs {
            c32_address_decode(&addr).unwrap();
        }
    }

    let report = profiler.report().build().unwrap();
    let mut buf = Vec::new();
    report
        .flamegraph(&mut buf)
        .or_else(|e| cx.throw_error(format!("Error creating flamegraph: {}", e)))?;

    let mut result = cx.buffer(buf.len())?;
    result.as_mut_slice(&mut cx).copy_from_slice(&buf);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::hex::decode_hex;

    use super::*;

    #[test]
    fn test_stacks_to_bitcoin_address_mainnet() {
        let input = "SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM";
        let output = stacks_to_bitcoin_address_internal(input.to_string()).unwrap();
        let expected = "1FhZqHcrXaWcNCJPEGn2BRZ9angJvYfTBT";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_stacks_to_bitcoin_address_testnet() {
        let input = "ST2M9C0SHDV4FMXF3R0P98H8GQPW5824DVEJ9MVQZ";
        let output = stacks_to_bitcoin_address_internal(input.to_string()).unwrap();
        let expected = "mvtMXL9MYH8HaNz7u9AgapGqoFYpNDfKBx";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_clarity_value_to_principal() {
        let input = decode_hex("0x0516a13dce8114be0f707f94470a2e5e86eb402f2923").unwrap();
        let output = decode_clarity_value_to_principal_inner(&input).unwrap();
        assert_eq!(output, "SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM");
    }
}
