//! Signer-message decoding (Neon entry point + deserializer).
//!
//! `SignerMessage` and its sub-types live in the upstream `libsigner` crate
//! (`libsigner::v0::messages`), which — unlike the other wire-format crates —
//! is not re-exported through `stackslib`, so it is pinned as its own git
//! dependency (kept in sync by `scripts/update-stacks-core.sh`).
//!
//! Decoding goes through the canonical
//! `<SignerMessage as StacksMessageCodec>::consensus_deserialize`; the Neon
//! encoders for `SignerMessage` and friends live in [`neon_encoder`].
//!
//! Scope: the block-related messages that indexers consume are fully decoded
//! (`BlockProposal`, `BlockResponse`, `BlockPushed`, `BlockPreCommit`). The
//! epoch-2.5 `Mock*` messages and `StateMachineUpdate` are recognized but
//! surfaced as an `unsupported` shape (their `type_id` / `type_name` only).

use std::io::Cursor;

pub use libsigner::v0::messages::{
    BlockAccepted, BlockRejection, BlockResponse, BlockResponseData, RejectCode, RejectReason,
    SignerMessage, SignerMessageMetadata,
};
pub use libsigner::{BlockProposal, BlockProposalData};
use neon::prelude::*;
use stacks_common::codec::StacksMessageCodec;

use crate::util::neon::{arg_as_bytes, Encode, NeonJsSerialize};
use crate::util::serialize::DeserializeError;

mod neon_encoder;

pub fn deserialize_signer_message(
    fd: &mut Cursor<&[u8]>,
) -> Result<SignerMessage, DeserializeError> {
    <SignerMessage as StacksMessageCodec>::consensus_deserialize(fd)
        .map_err(|e| DeserializeError::from(format!("Failed to decode signer message: {}", e)))
}

/// Decode a StackerDB `SignerMessage` from its consensus wire format.
pub fn decode_signer_message(mut cx: FunctionContext) -> JsResult<JsObject> {
    let message = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        deserialize_signer_message(&mut cursor)
            .map_err(|e| format!("Failed to decode signer message: {:?}\n", &e))
    })
    .or_else(|e| cx.throw_error(e))?;

    let message_obj = cx.empty_object();
    Encode(&message).neon_js_serialize(&mut cx, &message_obj, &())?;
    Ok(message_obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use stacks_common::util::hash::Sha512Trunc256Sum;
    use stacks_common::util::secp256k1::MessageSignature;

    /// Round-trip a message: build → serialize → `deserialize_signer_message`
    /// and assert equality. Exercises the deserialize path this crate exposes
    /// (the Neon encoding is covered by the JS tests).
    fn round_trip(message: SignerMessage) {
        let bytes = message.serialize_to_vec();
        let mut cursor = Cursor::new(bytes.as_slice());
        let decoded = deserialize_signer_message(&mut cursor).unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn block_pre_commit_round_trips() {
        let message = SignerMessage::BlockPreCommit(Sha512Trunc256Sum([0x01; 32]));
        round_trip(message.clone());
        // Wire format is the type-prefix byte (7) followed by the 32-byte hash.
        let bytes = message.serialize_to_vec();
        assert_eq!(bytes[0], 7);
        assert_eq!(&bytes[1..], &[0x01u8; 32]);
    }

    #[test]
    fn block_response_accepted_round_trips() {
        let message = SignerMessage::BlockResponse(BlockResponse::accepted(
            Sha512Trunc256Sum([0x02; 32]),
            MessageSignature::empty(),
            1_700_000_000,
            1_700_000_001,
        ));
        round_trip(message);
    }

    #[test]
    fn unknown_type_prefix_errors() {
        // 0xff is not a valid SignerMessageTypePrefix.
        let bytes = [0xffu8, 0x00];
        let mut cursor = Cursor::new(bytes.as_slice());
        assert!(deserialize_signer_message(&mut cursor).is_err());
    }
}
