//! PoX synthetic-event decoding.
//!
//! Per-contract-version decoders live under their own sub-module
//! (`pox4`, `pox5`, ...). Shared bits stay at this level:
//!
//! - [`types::StacksNetwork`] — network type used by the BTC-address encoder.
//! - [`btc_address`] — BTC-address encoding helper (version-agnostic).
//! - [`clarity_helpers`] — generic Clarity-tree walkers shared by every
//!   per-version decoder.
//! - [`neon_helpers`] — JS object setters shared by every per-version
//!   encoder.
//!
//! The Neon entry point [`decode_pox_event`] dispatches by sniffing the
//! shape of the decoded Clarity value:
//!
//! - A bare tuple containing a `topic` ASCII string → routes to [`pox5`]
//!   (matches the contract's `(print { topic: "...", ... })` form).
//! - Anything else → routes to [`pox4`], which expects the historical
//!   node-synthesized `Response(Ok({ name, data, ... }))` shape.
//!
//! Returns JS `null` if neither decoder claims the value (e.g. PoX-4
//! `Response(Err _)` events, or any pre-PoX-4 `Response(Err _)` we just want
//! to ignore).

use std::io::Cursor;

use clarity::vm::types::Value as UpstreamValue;
use neon::prelude::*;
use stacks_common::codec::StacksMessageCodec;

use crate::util::neon::arg_as_bytes_copied;

use self::types::StacksNetwork;

pub mod btc_address;
pub mod clarity_helpers;
pub mod neon_helpers;
pub mod pox4;
pub mod pox5;
pub mod types;

/// Neon-exported function: decodePoxSyntheticEvent(arg: string | Buffer, network: string)
///
/// Returns one of:
/// - a PoX-5 event object `{ name, data }`
/// - a PoX-4 event object `{ stacker, locked, ..., name, data }`
/// - `null` if the value doesn't match either decoder's shape
///
/// Throws if the Clarity value can't be deserialized or if a recognized
/// event shape is missing required fields.
pub fn decode_pox_event(mut cx: FunctionContext) -> JsResult<JsValue> {
    let val_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let network_str = cx.argument::<JsString>(1)?.value(&mut cx);
    let network = StacksNetwork::parse(&network_str).or_else(|e| cx.throw_error(e))?;

    let mut cursor: Cursor<&[u8]> = Cursor::new(&val_bytes);
    let clarity_value =
        <UpstreamValue as StacksMessageCodec>::consensus_deserialize(&mut cursor)
            .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;

    // PoX-5 events are bare printed tuples with a `topic` field. Try that
    // path first; if it doesn't match, fall back to PoX-4 (which expects a
    // node-synthesized `Response(Ok({ name, data, ... }))`).
    let pox5_event = pox5::decode::decode_pox5_synthetic_event(&clarity_value)
        .or_else(|e| cx.throw_error(format!("Error decoding PoX-5 synthetic event: {}", e)))?;

    if let Some(evt) = pox5_event {
        let obj = pox5::neon_encoder::encode_pox5_event(&mut cx, &evt)?;
        return Ok(obj.upcast());
    }

    let pox4_event = pox4::decode::decode_pox_synthetic_event(&clarity_value, network)
        .or_else(|e| cx.throw_error(format!("Error decoding PoX-4 synthetic event: {}", e)))?;

    match pox4_event {
        Some(evt) => {
            let obj = pox4::neon_encoder::encode_pox_event(&mut cx, &evt)?;
            Ok(obj.upcast())
        }
        None => Ok(cx.null().upcast()),
    }
}
