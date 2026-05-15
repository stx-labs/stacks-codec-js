//! Stacks block deserialization (Phase 2).
//!
//! The Nakamoto path returns upstream's [`NakamotoBlock`] /
//! [`NakamotoBlockHeader`] types directly, via the canonical
//! `consensus_deserialize` codec. We bypass upstream's `NakamotoBlock`
//! consensus deserializer (which performs higher-level invariants like
//! merkle-root consistency and minimum-tx checks) by parsing the header
//! through the canonical codec and then reading the transaction vector
//! ourselves, identically to how upstream lays it out on the wire.
//!
//! The Stacks 2.x path retains a small local shadow type for the header:
//! upstream's `StacksBlockHeader::consensus_deserialize` validates that the
//! VRF proof bytes form a valid curve point, but JS callers and test
//! fixtures rely on the historical permissive behavior of accepting any
//! 80-byte buffer at the VRF position. Every other field in the header is
//! either a fixed-size byte buffer or a fixed-width integer.
//!
//! The block-level read for 2.x avoids upstream's `StacksBlock`
//! consensus deserializer for the same reason it does for Nakamoto: that
//! parser rejects zero-transaction blocks and enforces merkle/tx-id
//! uniqueness invariants, which the JS test suite (which exercises a
//! zero-transaction Stacks 2.x block) and historical user fixtures have
//! never required.

use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

pub use blockstack_lib::chainstate::nakamoto::{
    NakamotoBlock, NakamotoBlockHeader,
};
pub use blockstack_lib::chainstate::stacks::StacksTransaction;
pub use stacks_common::bitvec::BitVec;
pub use stacks_common::types::chainstate::{
    BlockHeaderHash, ConsensusHash, StacksBlockId, TrieHash,
};
pub use stacks_common::util::hash::{Hash160, Sha512Trunc256Sum};
pub use stacks_common::util::secp256k1::MessageSignature;

use stacks_common::codec::StacksMessageCodec;

use crate::serialize_util::DeserializeError;

/// Deserialize a Nakamoto block (header + tx vector) from the wire.
///
/// We deliberately do *not* call upstream's
/// `NakamotoBlock::consensus_deserialize` because it rejects blocks that
/// don't satisfy higher-level invariants (no zero-tx blocks, tx-merkle-root
/// must match the header, no duplicate txids). Those checks differ from
/// what this crate has historically shipped, and the JS test suite leans
/// on the more permissive behavior.
pub fn deserialize_nakamoto_block(fd: &mut Cursor<&[u8]>) -> Result<NakamotoBlock, DeserializeError> {
    let header =
        <NakamotoBlockHeader as StacksMessageCodec>::consensus_deserialize(fd).map_err(|e| {
            DeserializeError::from(format!("Failed to decode Nakamoto block header: {}", e))
        })?;
    let txs = read_transactions(fd)?;
    Ok(NakamotoBlock { header, txs })
}

/// Header for Stacks 2.x blocks.
///
/// We keep a local shadow struct here (instead of upstream's
/// `StacksBlockHeader`) so we can store the VRF proof as raw 80 bytes
/// without going through upstream's curve-point-validating constructor.
pub struct StacksBlockHeader {
    pub version: u8,
    pub total_work: StacksWorkScore,
    pub proof: VRFProof,
    pub parent_block: BlockHeaderHash,
    pub parent_microblock: BlockHeaderHash,
    pub parent_microblock_sequence: u16,
    pub tx_merkle_root: Sha512Trunc256Sum,
    pub state_index_root: TrieHash,
    pub microblock_pubkey_hash: [u8; 20],
}

/// Work score for Stacks 2.x consensus.
pub struct StacksWorkScore {
    pub burn: u64,
    pub work: u64,
}

/// VRF proof - 80 bytes. Stored unvalidated.
pub struct VRFProof(pub [u8; 80]);

impl StacksBlockHeader {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let version = fd.read_u8()?;

        let burn = fd.read_u64::<BigEndian>()?;
        let work = fd.read_u64::<BigEndian>()?;
        let total_work = StacksWorkScore { burn, work };

        let mut proof_bytes = [0u8; 80];
        fd.read_exact(&mut proof_bytes)?;
        let proof = VRFProof(proof_bytes);

        let mut parent_block_bytes = [0u8; 32];
        fd.read_exact(&mut parent_block_bytes)?;
        let parent_block = BlockHeaderHash(parent_block_bytes);

        let mut parent_microblock_bytes = [0u8; 32];
        fd.read_exact(&mut parent_microblock_bytes)?;
        let parent_microblock = BlockHeaderHash(parent_microblock_bytes);

        let parent_microblock_sequence = fd.read_u16::<BigEndian>()?;

        let mut tx_merkle_root_bytes = [0u8; 32];
        fd.read_exact(&mut tx_merkle_root_bytes)?;
        let tx_merkle_root = Sha512Trunc256Sum(tx_merkle_root_bytes);

        let mut state_index_root_bytes = [0u8; 32];
        fd.read_exact(&mut state_index_root_bytes)?;
        let state_index_root = TrieHash(state_index_root_bytes);

        let mut microblock_pubkey_hash = [0u8; 20];
        fd.read_exact(&mut microblock_pubkey_hash)?;

        Ok(StacksBlockHeader {
            version,
            total_work,
            proof,
            parent_block,
            parent_microblock,
            parent_microblock_sequence,
            tx_merkle_root,
            state_index_root,
            microblock_pubkey_hash,
        })
    }

    /// Compute the block hash.
    ///
    /// This intentionally does *not* short-circuit on `total_work.work == 0`
    /// the way upstream's `StacksBlockHeader::block_hash` does (it returns
    /// `FIRST_STACKS_BLOCK_HASH` for the boot block). The local code has
    /// shipped this unconditional hashing form since v1.0; preserving it
    /// keeps the JS-facing output byte-identical for every block our users
    /// have ever fed in.
    pub fn block_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha512_256};

        let mut hasher = Sha512_256::new();

        hasher.update([self.version]);
        hasher.update(self.total_work.burn.to_be_bytes());
        hasher.update(self.total_work.work.to_be_bytes());
        hasher.update(&self.proof.0);
        hasher.update(&self.parent_block.0);
        hasher.update(&self.parent_microblock.0);
        hasher.update(self.parent_microblock_sequence.to_be_bytes());
        hasher.update(&self.tx_merkle_root.0);
        hasher.update(&self.state_index_root.0);
        hasher.update(&self.microblock_pubkey_hash);

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// A Stacks 2.x block.
pub struct StacksBlock {
    pub header: StacksBlockHeader,
    pub txs: Vec<StacksTransaction>,
}

impl StacksBlock {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let header = StacksBlockHeader::deserialize(fd)?;
        let txs = read_transactions(fd)?;
        Ok(StacksBlock { header, txs })
    }
}

/// Read the length-prefixed transaction vector that follows a block header on
/// the wire, dispatching to the upstream `StacksTransaction` codec for each
/// entry.
fn read_transactions(
    fd: &mut Cursor<&[u8]>,
) -> Result<Vec<StacksTransaction>, DeserializeError> {
    let tx_count = fd.read_u32::<BigEndian>()?;
    let mut txs = Vec::with_capacity(tx_count as usize);
    for _ in 0..tx_count {
        let tx = <StacksTransaction as StacksMessageCodec>::consensus_deserialize(fd)
            .map_err(|e| DeserializeError::from(format!("Failed to decode block tx: {}", e)))?;
        txs.push(tx);
    }
    Ok(txs)
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
        assert!(block.is_ok());
        let block = block.unwrap();
        assert_eq!(block.header.version, 0);
        assert_eq!(block.header.chain_length, 557923);
        assert_eq!(block.header.burn_spent, 403018706956);
        assert_eq!(encode_hex(&block.header.consensus_hash.0).as_ref(), "0xe86587f4ed4ca465b87649ace9341d9fdfd113ba");
        assert_eq!(encode_hex(&block.header.parent_block_id.0).as_ref(), "0x8de0fa074023b893f73c8491ab5c93bb3f5af4bd5f0449578b99b508cca61595");
        assert_eq!(encode_hex(&block.header.tx_merkle_root.0).as_ref(), "0x080d35f6c5c02929a00fca1cc6f00a1c3828d905eb61e002ffd4e48f1ecef29d");
        assert_eq!(encode_hex(&block.header.state_index_root.0).as_ref(), "0xbf5ed8f745df2629d0d971fe9667f75a352a5dea4c8a0e451dcaa72b375d28fc");
        assert_eq!(block.header.timestamp, 1738687125);
        assert_eq!(encode_hex(&block.header.miner_signature.0).as_ref(), "0x01b7ef0ca6fb1e109afb5d3a9f08bfee71b8fef82ad9a7e06a5fa9b732394513be7cc962950ce2fc940d4ae7c1cb731d33cd65ec032a3a097ac2669439fe31031d");
        assert_eq!(block.header.signer_signature.len(), 24);
        assert_eq!(block.header.pox_treatment.len(), 3891);
        assert_eq!(encode_hex(&block.header.block_hash().0).as_ref(), "0x536b854fa6ada87643e00c4a4880967b4f52404b95dca75780babb048f6a69fc");
        assert_eq!(encode_hex(&block.header.block_id().0).as_ref(), "0x05b7fbc03e541271a29baf21ad43e68e48070df018ebe5baa13892f3828be9bd");
        assert_eq!(block.txs.len(), 1);
        assert_eq!(cursor.position() as usize, data.len());
    }
}
