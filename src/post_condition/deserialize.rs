//! Transaction post-condition deserialization.
//!
//! Phase 2: this module is now a thin re-export shim. All types come
//! directly from upstream (`blockstack_lib::chainstate::stacks`), and
//! parsing is done via the canonical `StacksMessageCodec::consensus_deserialize`
//! impl on `TransactionPostCondition`.
//!
//! The Neon encoder lives in `post_condition::neon_encoder` and operates on
//! these upstream types via the `Encode<'_, T>` newtype wrapper from
//! `neon_util`.

use std::io::Cursor;

pub use blockstack_lib::chainstate::stacks::{
    AssetInfo, AssetInfoID, FungibleConditionCode, NonfungibleConditionCode,
    PostConditionPrincipal, PostConditionPrincipalID, TransactionPostCondition,
};
use stacks_codec::StacksMessageCodec;

use crate::serialize_util::DeserializeError;

/// Deserialize a single post-condition entry from the wire format.
///
/// Convenience wrapper around upstream's canonical
/// `<TransactionPostCondition as StacksMessageCodec>::consensus_deserialize`,
/// adapting the error type to the local [`DeserializeError`] surface that
/// the JS-facing callers throw with.
pub fn deserialize_post_condition(
    fd: &mut Cursor<&[u8]>,
) -> Result<TransactionPostCondition, DeserializeError> {
    <TransactionPostCondition as StacksMessageCodec>::consensus_deserialize(fd)
        .map_err(|e| DeserializeError::from(format!("Failed to decode post-condition: {}", e)))
}
