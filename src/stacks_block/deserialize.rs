//! Stacks block deserialization.
//!
//! The Nakamoto block header is parsed via upstream's canonical
//! `<NakamotoBlockHeader as StacksMessageCodec>::consensus_deserialize` in
//! `stackslib` (every field is a fixed-width integer or opaque byte buffer,
//! plus a length-prefixed signature vector and a `BitVec`, none of which
//! upstream rejects on content).
//!
//! The Stacks 2.x block header is read field-by-field here, because upstream's
//! parser validates the VRF proof bytes form a valid curve point and the JS
//! test suite (and presumably user fixtures) rely on the historical permissive
//! behavior of accepting any 80-byte buffer at that position.
//!
//! The block bodies (header + transactions vector) are read directly here so
//! we don't pull in upstream's extra "block must contain at least one
//! transaction" / merkle-root / unique-txid validations — those checks differ
//! from what this crate has historically shipped (the JS test suite, for
//! instance, exercises a zero-transaction Stacks 2.x block) and changing them
//! would be a user-visible behavior change.

use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use blockstack_lib::chainstate::nakamoto::NakamotoBlockHeader as UpstreamNakamotoBlockHeader;
use blockstack_lib::chainstate::stacks::StacksTransaction as UpstreamStacksTransaction;
use stacks_common::bitvec::BitVec as UpstreamBitVec;
use stacks_common::codec::StacksMessageCodec;

use crate::serialize_util::DeserializeError;
use crate::stacks_tx::deserialize::{
    convert_transaction, BlockHeaderHash, MessageSignature, Sha512Trunc256Sum, StacksTransaction,
};

// ===== Local types (kept verbatim — the Neon encoder operates on these) =====

/// Consensus hash - 20 bytes
pub struct ConsensusHash(pub [u8; 20]);

/// Stacks block ID - 32 bytes (hash of consensus hash + block header hash)
pub struct StacksBlockId(pub [u8; 32]);

/// Trie hash for MARF - 32 bytes
pub struct TrieHash(pub [u8; 32]);

/// A bitvector with a maximum size.
///
/// Note: `get(i)` here uses MSB-first bit ordering within each byte (bit 0 is
/// the most-significant bit of `data[0]`), which is *different* from upstream
/// `stacks_common::bitvec::BitVec`'s LSB-first ordering. The JS-facing `bits`
/// array has been shipping this MSB-first convention since the first release,
/// so we deliberately preserve it here. The raw `data` bytes themselves are
/// the canonical wire bytes, so the hex `data` field and the block-hash
/// computation are unaffected by this choice.
pub struct BitVec {
    pub data: Vec<u8>,
    pub len: u16,
}

impl BitVec {
    /// Get the value at the given index (MSB-first within each byte).
    pub fn get(&self, index: u16) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        let byte_index = (index / 8) as usize;
        let bit_index = index % 8;
        Some((self.data[byte_index] & (1 << (7 - bit_index))) != 0)
    }
}

/// Header for a Nakamoto block (Stacks 3.x+)
pub struct NakamotoBlockHeader {
    pub version: u8,
    pub chain_length: u64,
    pub burn_spent: u64,
    pub consensus_hash: ConsensusHash,
    pub parent_block_id: StacksBlockId,
    pub tx_merkle_root: Sha512Trunc256Sum,
    pub state_index_root: TrieHash,
    pub timestamp: u64,
    pub miner_signature: MessageSignature,
    pub signer_signature: Vec<MessageSignature>,
    pub pox_treatment: BitVec,
}

impl NakamotoBlockHeader {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let upstream =
            <UpstreamNakamotoBlockHeader as StacksMessageCodec>::consensus_deserialize(fd)
                .map_err(|e| {
                    DeserializeError::from(format!("Failed to decode Nakamoto block header: {}", e))
                })?;
        Ok(convert_nakamoto_header(&upstream))
    }

    /// Compute the block hash (sha512/256 of header fields excluding signer_signature).
    /// This is the same as the "signer signature hash" in the reference implementation.
    pub fn block_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha512_256};

        let mut hasher = Sha512_256::new();

        hasher.update([self.version]);
        hasher.update(self.chain_length.to_be_bytes());
        hasher.update(self.burn_spent.to_be_bytes());
        hasher.update(&self.consensus_hash.0);
        hasher.update(&self.parent_block_id.0);
        hasher.update(&self.tx_merkle_root.0);
        hasher.update(&self.state_index_root.0);
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(&self.miner_signature.0);
        hasher.update(self.pox_treatment.len.to_be_bytes());
        hasher.update((self.pox_treatment.data.len() as u32).to_be_bytes());
        hasher.update(&self.pox_treatment.data);

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Compute the block ID (sha512/256 of block_hash + consensus_hash)
    pub fn block_id(&self) -> [u8; 32] {
        use sha2::{Digest, Sha512_256};

        let block_hash = self.block_hash();
        let mut hasher = Sha512_256::new();
        hasher.update(&block_hash);
        hasher.update(&self.consensus_hash.0);

        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        id
    }
}

/// A Nakamoto block (Stacks 3.x+)
pub struct NakamotoBlock {
    pub header: NakamotoBlockHeader,
    pub txs: Vec<StacksTransaction>,
}

impl NakamotoBlock {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let header = NakamotoBlockHeader::deserialize(fd)?;
        let txs = read_transactions(fd)?;
        Ok(NakamotoBlock { header, txs })
    }
}

/// Header for Stacks 2.x blocks
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

/// Work score for Stacks 2.x consensus
pub struct StacksWorkScore {
    pub burn: u64,
    pub work: u64,
}

/// VRF proof - 80 bytes
pub struct VRFProof(pub [u8; 80]);

impl StacksBlockHeader {
    /// Deserialize a Stacks 2.x block header from the wire.
    ///
    /// We read each field directly here rather than delegating to upstream's
    /// `StacksBlockHeader::consensus_deserialize`, because upstream validates
    /// the VRF proof bytes form a valid curve point — and the JS test suite
    /// (and presumably some user fixtures) rely on the historical permissive
    /// behavior of accepting any 80-byte buffer at the VRF position. Every
    /// field is otherwise either a fixed-size byte buffer or a fixed-width
    /// integer, so there's no canonical-decoder dependency to lose by reading
    /// them one at a time.
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

/// A Stacks 2.x block
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

// ===== Conversion routines =====

fn convert_nakamoto_header(upstream: &UpstreamNakamotoBlockHeader) -> NakamotoBlockHeader {
    NakamotoBlockHeader {
        version: upstream.version,
        chain_length: upstream.chain_length,
        burn_spent: upstream.burn_spent,
        consensus_hash: ConsensusHash(upstream.consensus_hash.0),
        parent_block_id: StacksBlockId(upstream.parent_block_id.0),
        tx_merkle_root: Sha512Trunc256Sum(upstream.tx_merkle_root.0),
        state_index_root: TrieHash(upstream.state_index_root.0),
        timestamp: upstream.timestamp,
        miner_signature: MessageSignature(upstream.miner_signature.0),
        signer_signature: upstream
            .signer_signature
            .iter()
            .map(|s| MessageSignature(s.0))
            .collect(),
        pox_treatment: convert_bitvec(&upstream.pox_treatment),
    }
}

/// Lower an upstream `BitVec<MAX_SIZE>` into our local [`BitVec`] without
/// touching upstream's private `data` field.
///
/// We round-trip through the canonical wire encoding (`u16 len`, `u32
/// data_len`, then the raw bytes) and slice off the 6-byte header — the rest
/// is the byte buffer we want to keep, byte-identical to what was on the
/// wire.
fn convert_bitvec<const MAX_SIZE: u16>(upstream: &UpstreamBitVec<MAX_SIZE>) -> BitVec {
    let serialized = <UpstreamBitVec<MAX_SIZE> as StacksMessageCodec>::serialize_to_vec(upstream);
    // 6 bytes = u16 len (2 bytes) + u32 data length prefix (4 bytes).
    debug_assert!(serialized.len() >= 6);
    let data = serialized[6..].to_vec();
    BitVec {
        data,
        len: upstream.len(),
    }
}

/// Read the length-prefixed transaction vector that follows a block header on
/// the wire, dispatching to the upstream `StacksTransaction` codec for each
/// entry and lowering the result into our local types.
///
/// We deliberately do *not* call upstream's `StacksBlock::consensus_deserialize`
/// or `NakamotoBlock::consensus_deserialize` because they reject blocks that
/// don't satisfy higher-level invariants (no zero-tx blocks, tx-merkle-root
/// must match the header, no duplicate txids). Those checks differ from what
/// this crate has historically shipped, and the JS test suite leans on the
/// more permissive behavior.
fn read_transactions(
    fd: &mut Cursor<&[u8]>,
) -> Result<Vec<StacksTransaction>, DeserializeError> {
    let tx_count = fd.read_u32::<BigEndian>()?;
    let mut txs = Vec::with_capacity(tx_count as usize);
    for _ in 0..tx_count {
        let upstream = <UpstreamStacksTransaction as StacksMessageCodec>::consensus_deserialize(fd)
            .map_err(|e| DeserializeError::from(format!("Failed to decode block tx: {}", e)))?;
        txs.push(convert_transaction(&upstream));
    }
    Ok(txs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::encode_hex;

    #[test]
    fn test_bitvec_local_get_msb_first() {
        // Sanity check: confirm the local `get()` keeps the MSB-first bit
        // ordering that the JS-facing `bits` array has shipped with.
        let bitvec = BitVec {
            data: vec![0b10101010],
            len: 8,
        };
        assert_eq!(bitvec.get(0), Some(true));
        assert_eq!(bitvec.get(1), Some(false));
        assert_eq!(bitvec.get(7), Some(false));
        assert_eq!(bitvec.get(8), None);
    }

    #[test]
    fn test_nakamoto_block_deserialize() {
        let data = include_bytes!("../../tests/fixtures/nakamoto-block.bin");
        let mut cursor = Cursor::new(data.as_ref());
        let block = NakamotoBlock::deserialize(&mut cursor);
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
        assert_eq!(block.header.pox_treatment.len, 3891);
        assert_eq!(block.header.pox_treatment.data.len(), 487);
        assert_eq!(encode_hex(&block.header.block_hash()).as_ref(), "0x536b854fa6ada87643e00c4a4880967b4f52404b95dca75780babb048f6a69fc");
        assert_eq!(encode_hex(&block.header.block_id()).as_ref(), "0x05b7fbc03e541271a29baf21ad43e68e48070df018ebe5baa13892f3828be9bd");
        assert_eq!(block.txs.len(), 1);
        assert_eq!(cursor.position() as usize, data.len());
    }
}
