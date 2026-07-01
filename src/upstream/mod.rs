//! Modules that forward types from upstream `stacks-network/stacks-core`
//! crates (`stackslib`, `clarity`, `stacks-common`, `stacks-codec`).
//!
//! Each child module is a thin layer over upstream:
//!
//! - [`address`] — c32/b58 wrappers, the `decodeStacksAddress` / address
//!   helpers, and shared `NeonJsSerialize` impls for `StacksAddress` /
//!   `PrincipalData`.
//! - [`clarity_value`] — Neon entry points calling
//!   `clarity::vm::types::Value::deserialize_read` directly; the JS-facing
//!   `repr_string` / `type_signature_string` formatters live in
//!   `clarity_value::neon_encoder`.
//! - [`post_condition`] — re-exports upstream post-condition types and wraps
//!   `<TransactionPostCondition as StacksMessageCodec>::consensus_deserialize`.
//! - [`stacks_block`] — re-exports upstream block types and wraps the 2.x /
//!   Nakamoto `consensus_deserialize` codecs.
//! - [`stacks_tx`] — re-exports upstream transaction types and wraps
//!   `<StacksTransaction as StacksMessageCodec>::consensus_deserialize`.

pub mod address;
pub mod clarity_value;
pub mod post_condition;
pub mod stacks_block;
pub mod stacks_tx;
