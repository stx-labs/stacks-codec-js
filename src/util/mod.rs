//! Crate-internal helpers shared by both `upstream` and `derived` modules.
//!
//! - [`hex`] — `0x`-prefixed hex encode/decode.
//! - [`neon`] — Neon binding helpers: the `Encode<'_, T>` newtype wrapper,
//!   the `NeonJsSerialize` trait, and `arg_as_bytes` / `arg_as_bytes_copied`
//!   for parsing JS string/buffer arguments.
//! - [`serialize`] — the `DeserializeError` type used by every wrapped
//!   `consensus_deserialize` call.

pub mod hex;
pub mod neon;
pub mod serialize;
