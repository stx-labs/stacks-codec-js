//! Stacks block deserialization.
//!
//! Both the Stacks 2.x and Nakamoto paths now go through upstream's canonical
//! `consensus_deserialize` codecs. That means:
//!
//! - 2.x blocks: the VRF proof bytes must be a valid curve point; zero-tx
//!   blocks are rejected; the tx Merkle root must match the header; tx-ids
//!   must be unique; and only one coinbase is permitted per block. Anchor-mode
//!   constraints (`OnChainOnly` / `Any`) are also enforced.
//! - Nakamoto blocks: the same family of structural checks (no duplicate
//!   tx-ids, Merkle root consistency, tenure-change rules) is enforced by
//!   `<NakamotoBlock as StacksMessageCodec>::consensus_deserialize`.
//!
//! `block_hash` for 2.x headers short-circuits to `FIRST_STACKS_BLOCK_HASH`
//! when `total_work.work == 0` (boot block) — this matches upstream.

use std::io::Cursor;

pub use blockstack_lib::chainstate::nakamoto::{NakamotoBlock, NakamotoBlockHeader};
pub use blockstack_lib::chainstate::stacks::{StacksBlock, StacksBlockHeader, StacksTransaction};
pub use stacks_common::bitvec::BitVec;
pub use stacks_common::types::chainstate::{
    BlockHeaderHash, ConsensusHash, StacksBlockId, StacksWorkScore, TrieHash,
};
pub use stacks_common::util::hash::{Hash160, Sha512Trunc256Sum};
pub use stacks_common::util::secp256k1::MessageSignature;

use stacks_common::codec::StacksMessageCodec;

use crate::serialize_util::DeserializeError;

pub fn deserialize_nakamoto_block(
    fd: &mut Cursor<&[u8]>,
) -> Result<NakamotoBlock, DeserializeError> {
    <NakamotoBlock as StacksMessageCodec>::consensus_deserialize(fd)
        .map_err(|e| DeserializeError::from(format!("Failed to decode Nakamoto block: {}", e)))
}

pub fn deserialize_stacks_block(fd: &mut Cursor<&[u8]>) -> Result<StacksBlock, DeserializeError> {
    <StacksBlock as StacksMessageCodec>::consensus_deserialize(fd)
        .map_err(|e| DeserializeError::from(format!("Failed to decode Stacks block: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::encode_hex;

    #[test]
    fn test_nakamoto_block_deserialize() {
        let data = include_bytes!("../../tests/fixtures/nakamoto-block.bin");
        let mut cursor = Cursor::new(data.as_ref());
        let block = deserialize_nakamoto_block(&mut cursor);
        assert!(block.is_ok(), "deserialize failed: {:?}", block.err());
        let block = block.unwrap();
        assert_eq!(block.header.version, 0);
        assert_eq!(block.header.chain_length, 557923);
        assert_eq!(block.header.burn_spent, 403018706956);
        assert_eq!(
            encode_hex(&block.header.consensus_hash.0).as_ref(),
            "0xe86587f4ed4ca465b87649ace9341d9fdfd113ba"
        );
        assert_eq!(
            encode_hex(&block.header.parent_block_id.0).as_ref(),
            "0x8de0fa074023b893f73c8491ab5c93bb3f5af4bd5f0449578b99b508cca61595"
        );
        assert_eq!(
            encode_hex(&block.header.tx_merkle_root.0).as_ref(),
            "0x080d35f6c5c02929a00fca1cc6f00a1c3828d905eb61e002ffd4e48f1ecef29d"
        );
        assert_eq!(
            encode_hex(&block.header.state_index_root.0).as_ref(),
            "0xbf5ed8f745df2629d0d971fe9667f75a352a5dea4c8a0e451dcaa72b375d28fc"
        );
        assert_eq!(block.header.timestamp, 1738687125);
        assert_eq!(
            encode_hex(&block.header.miner_signature.0).as_ref(),
            "0x01b7ef0ca6fb1e109afb5d3a9f08bfee71b8fef82ad9a7e06a5fa9b732394513be7cc962950ce2fc940d4ae7c1cb731d33cd65ec032a3a097ac2669439fe31031d"
        );
        assert_eq!(block.header.signer_signature.len(), 24);
        assert_eq!(block.header.pox_treatment.len(), 3891);
        assert_eq!(
            encode_hex(&block.header.block_hash().0).as_ref(),
            "0x536b854fa6ada87643e00c4a4880967b4f52404b95dca75780babb048f6a69fc"
        );
        assert_eq!(
            encode_hex(&block.header.block_id().0).as_ref(),
            "0x05b7fbc03e541271a29baf21ad43e68e48070df018ebe5baa13892f3828be9bd"
        );
        assert_eq!(block.txs.len(), 1);
        assert_eq!(cursor.position() as usize, data.len());
    }
}
