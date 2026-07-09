use neon::prelude::*;
use stacks_common::codec::StacksMessageCodec;

use crate::util::hex::encode_hex;
use crate::util::neon::{Encode, NeonJsSerialize};

use super::{
    BitVec, NakamotoBlock, NakamotoBlockHeader, StacksBlock, StacksBlockHeader, StacksWorkScore,
};
use stacks_common::util::vrf::VRFProof;

impl NeonJsSerialize for Encode<'_, NakamotoBlock> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let block = self.0;

        let header_obj = cx.empty_object();
        Encode(&block.header).neon_js_serialize(cx, &header_obj, &())?;
        obj.set(cx, "header", header_obj)?;

        // Upstream made `NakamotoBlock::txs` private; `txs()` now yields
        // `TxToProcess` wrappers that carry each tx's problematic-marker state.
        // We only need the raw transactions here, so unwrap each via the
        // `tx_ignoring_problematic_state()` escape hatch (order is preserved,
        // problematic txs included — matching the previous `block.txs` field).
        let txs: Vec<_> = block.txs().collect();
        let txs_array = JsArray::new(cx, txs.len());
        for (i, tx_to_process) in txs.iter().enumerate() {
            let tx_obj = cx.empty_object();
            Encode(tx_to_process.tx_ignoring_problematic_state()).neon_js_serialize(
                cx,
                &tx_obj,
                &(),
            )?;
            txs_array.set(cx, i as u32, tx_obj)?;
        }
        obj.set(cx, "txs", txs_array)?;

        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, NakamotoBlockHeader> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let header = self.0;

        let version = cx.number(header.version);
        obj.set(cx, "version", version)?;

        let chain_length = cx.string(header.chain_length.to_string());
        obj.set(cx, "chain_length", chain_length)?;

        let burn_spent = cx.string(header.burn_spent.to_string());
        obj.set(cx, "burn_spent", burn_spent)?;

        let consensus_hash = cx.string(encode_hex(&header.consensus_hash.0));
        obj.set(cx, "consensus_hash", consensus_hash)?;

        let parent_block_id = cx.string(encode_hex(&header.parent_block_id.0));
        obj.set(cx, "parent_block_id", parent_block_id)?;

        let tx_merkle_root = cx.string(encode_hex(&header.tx_merkle_root.0));
        obj.set(cx, "tx_merkle_root", tx_merkle_root)?;

        let state_index_root = cx.string(encode_hex(&header.state_index_root.0));
        obj.set(cx, "state_index_root", state_index_root)?;

        let timestamp = cx.string(header.timestamp.to_string());
        obj.set(cx, "timestamp", timestamp)?;

        let miner_signature = cx.string(encode_hex(&header.miner_signature.0));
        obj.set(cx, "miner_signature", miner_signature)?;

        let signer_sigs_array = JsArray::new(cx, header.signer_signature.len());
        for (i, sig) in header.signer_signature.iter().enumerate() {
            let sig_hex = cx.string(encode_hex(&sig.0));
            signer_sigs_array.set(cx, i as u32, sig_hex)?;
        }
        obj.set(cx, "signer_signature", signer_sigs_array)?;

        let pox_treatment_obj = cx.empty_object();
        Encode(&header.pox_treatment).neon_js_serialize(cx, &pox_treatment_obj, &())?;
        obj.set(cx, "pox_treatment", pox_treatment_obj)?;

        let block_hash = cx.string(encode_hex(&header.block_hash().0));
        obj.set(cx, "block_hash", block_hash)?;

        let block_id = cx.string(encode_hex(&header.block_id().0));
        obj.set(cx, "index_block_hash", block_id)?;

        Ok(())
    }
}

impl<const MAX_SIZE: u16> NeonJsSerialize for Encode<'_, BitVec<MAX_SIZE>> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let bitvec = self.0;
        let len = bitvec.len();

        let len_n = cx.number(len);
        obj.set(cx, "len", len_n)?;

        // Recover the canonical data bytes by serializing the bitvec and
        // skipping the 6-byte header (u16 len + u32 data_len). Upstream's
        // wire format keeps the raw byte buffer intact, so the trailing
        // slice is byte-identical to what was on the wire.
        let wire = <BitVec<MAX_SIZE> as StacksMessageCodec>::serialize_to_vec(bitvec);
        debug_assert!(wire.len() >= 6);
        let data_bytes = &wire[6..];

        let data = cx.string(encode_hex(data_bytes));
        obj.set(cx, "data", data)?;

        // JS-facing `bits` array uses MSB-first ordering within each byte,
        // matching the convention this crate has shipped with since v1.0.
        // Upstream's `BitVec::get` is LSB-first, so we compute the array
        // directly from the data bytes here instead of using `get()`.
        let bits_array = JsArray::new(cx, len as usize);
        for i in 0..len {
            let byte_index = (i / 8) as usize;
            let bit_index = i % 8;
            let bit = data_bytes
                .get(byte_index)
                .map(|b| (b & (1 << (7 - bit_index))) != 0)
                .unwrap_or(false);
            let bit_val = cx.boolean(bit);
            bits_array.set(cx, i as u32, bit_val)?;
        }
        obj.set(cx, "bits", bits_array)?;

        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, StacksBlock> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let block = self.0;

        let header_obj = cx.empty_object();
        Encode(&block.header).neon_js_serialize(cx, &header_obj, &())?;
        obj.set(cx, "header", header_obj)?;

        let txs_array = JsArray::new(cx, block.txs.len());
        for (i, tx) in block.txs.iter().enumerate() {
            let tx_obj = cx.empty_object();
            Encode(tx).neon_js_serialize(cx, &tx_obj, &())?;
            txs_array.set(cx, i as u32, tx_obj)?;
        }
        obj.set(cx, "txs", txs_array)?;

        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, StacksBlockHeader> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let header = self.0;

        let version = cx.number(header.version);
        obj.set(cx, "version", version)?;

        let total_work_obj = cx.empty_object();
        Encode(&header.total_work).neon_js_serialize(cx, &total_work_obj, &())?;
        obj.set(cx, "total_work", total_work_obj)?;

        let proof = cx.string(encode_hex(&Encode(&header.proof).vrf_to_bytes()));
        obj.set(cx, "proof", proof)?;

        let parent_block = cx.string(encode_hex(&header.parent_block.0));
        obj.set(cx, "parent_block", parent_block)?;

        let parent_microblock = cx.string(encode_hex(&header.parent_microblock.0));
        obj.set(cx, "parent_microblock", parent_microblock)?;

        let parent_microblock_sequence = cx.number(header.parent_microblock_sequence);
        obj.set(cx, "parent_microblock_sequence", parent_microblock_sequence)?;

        let tx_merkle_root = cx.string(encode_hex(&header.tx_merkle_root.0));
        obj.set(cx, "tx_merkle_root", tx_merkle_root)?;

        let state_index_root = cx.string(encode_hex(&header.state_index_root.0));
        obj.set(cx, "state_index_root", state_index_root)?;

        let microblock_pubkey_hash = cx.string(encode_hex(&header.microblock_pubkey_hash.0));
        obj.set(cx, "microblock_pubkey_hash", microblock_pubkey_hash)?;

        let block_hash = cx.string(encode_hex(&header.block_hash().0));
        obj.set(cx, "block_hash", block_hash)?;

        Ok(())
    }
}

impl Encode<'_, VRFProof> {
    fn vrf_to_bytes(&self) -> [u8; 80] {
        self.0.to_bytes()
    }
}

impl NeonJsSerialize for Encode<'_, StacksWorkScore> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let burn = cx.string(self.0.burn.to_string());
        obj.set(cx, "burn", burn)?;

        let work = cx.string(self.0.work.to_string());
        obj.set(cx, "work", work)?;

        Ok(())
    }
}
