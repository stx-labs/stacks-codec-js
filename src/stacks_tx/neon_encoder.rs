//! Neon (JS) serialization for upstream Stacks transaction types.
//!
//! All impls are written against `Encode<'_, UpstreamX>` wrappers so the
//! Rust orphan rule is satisfied for the upstream types in
//! `blockstack_lib::chainstate::stacks` and `clarity::vm`.
//!
//! Two payload-level fan-outs / fan-ins are worth calling out:
//!
//! * Upstream collapses three on-chain coinbase shapes into a single
//!   `TransactionPayload::Coinbase(buf, recipient?, vrf?)` variant. We
//!   re-discriminate by which optionals are populated when picking the
//!   JS-facing `type_id` (4 = Coinbase, 5 = CoinbaseToAltRecipient,
//!   8 = NakamotoCoinbase).
//!
//! * Upstream `SmartContract(_, Option<ClarityVersion>)` becomes either
//!   `type_id` 1 (legacy `SmartContract`) or 6 (`VersionedSmartContract`)
//!   depending on whether the version is populated.
//!
//! * Upstream `TransactionSpendingCondition` has three variants
//!   (`Singlesig`, `Multisig`, `OrderIndependentMultisig`). The local
//!   shadow types historically flattened the last two into a single
//!   `Multisig` shape with a `hash_mode` byte that encoded the order
//!   independence; we preserve the same JS-facing emission by dispatching
//!   on the upstream variant directly.
use clarity::vm::types::Value as UpstreamValue;
use clarity::vm::ClarityVersion;
use neon::prelude::*;
use stacks_codec::StacksMessageCodec;
use stacks_common::address::AddressHashMode;
use stacks_common::types::chainstate::StacksAddress as UpstreamStacksAddress;

use crate::address::c32::c32_address;
use crate::clarity_value::neon_encoder::decode_clarity_val;
use crate::hex::encode_hex;
use crate::neon_util::{Encode, NeonJsSerialize};

use super::deserialize::{
    CoinbasePayload, MultisigSpendingCondition, OrderIndependentMultisigSpendingCondition,
    SinglesigSpendingCondition, StacksMicroblockHeader, StacksTransaction, TenureChangePayload,
    TransactionAuth, TransactionAuthField, TransactionAuthFieldID, TransactionAuthFlags,
    TransactionContractCall, TransactionPayload, TransactionPayloadID, TransactionSmartContract,
    TransactionSpendingCondition, TransactionVersion,
};

/// Context threaded into the spending-condition encoder so it can lift the
/// 1-byte `hash_mode` into a full Stacks address version (mainnet vs testnet
/// single- vs multi-sig).
pub struct TxSerializationContext {
    pub transaction_version: TransactionVersion,
}

impl TxSerializationContext {
    pub fn new(version: TransactionVersion) -> Self {
        TxSerializationContext {
            transaction_version: version,
        }
    }
}

/// Re-export to mirror the previous module path used by the `stacks_block`
/// encoder.
pub mod neon_encoder_internal {
    pub use super::TxSerializationContext;
}

impl NeonJsSerialize for Encode<'_, StacksTransaction> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let tx = self.0;
        let version_number = cx.number(tx.version as u8);
        obj.set(cx, "version", version_number)?;

        let chain_id = cx.number(tx.chain_id);
        obj.set(cx, "chain_id", chain_id)?;

        let auth_obj = cx.empty_object();
        Encode(&tx.auth).neon_js_serialize(
            cx,
            &auth_obj,
            &TxSerializationContext::new(tx.version),
        )?;
        obj.set(cx, "auth", auth_obj)?;

        let anchor_mode = cx.number(tx.anchor_mode as u8);
        obj.set(cx, "anchor_mode", anchor_mode)?;

        let post_condition_mode_byte = tx.post_condition_mode as u8;
        let post_condition_mode = cx.number(post_condition_mode_byte);
        obj.set(cx, "post_condition_mode", post_condition_mode)?;

        // Wire format: `[1 byte mode] [4-byte BE length] [N * encoded post-condition]`.
        // Clarity / post-condition encoding is canonical so re-serializing
        // produces the same bytes that were on the wire.
        let mut post_conditions_buf = Vec::<u8>::with_capacity(5 + tx.post_conditions.len() * 32);
        post_conditions_buf.push(post_condition_mode_byte);
        post_conditions_buf.extend_from_slice(&(tx.post_conditions.len() as u32).to_be_bytes());

        let post_conditions = JsArray::new(cx, tx.post_conditions.len());
        for (i, x) in tx.post_conditions.iter().enumerate() {
            x.consensus_serialize(&mut post_conditions_buf)
                .expect("BUG: re-serialize of post-condition to Vec cannot fail");
            let post_condition_obj = cx.empty_object();
            Encode(x).neon_js_serialize(cx, &post_condition_obj, &())?;
            post_conditions.set(cx, i as u32, post_condition_obj)?;
        }
        obj.set(cx, "post_conditions", post_conditions)?;

        let post_conditions_buff = cx.string(encode_hex(&post_conditions_buf));
        obj.set(cx, "post_conditions_buffer", post_conditions_buff)?;

        let payload_obj = cx.empty_object();
        Encode(&tx.payload).neon_js_serialize(cx, &payload_obj, &())?;
        obj.set(cx, "payload", payload_obj)?;

        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for Encode<'_, TransactionAuth> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        match self.0 {
            TransactionAuth::Standard(origin) => {
                let type_id = cx.number(TransactionAuthFlags::AuthStandard as u8);
                obj.set(cx, "type_id", type_id)?;

                let origin_obj = cx.empty_object();
                Encode(origin).neon_js_serialize(cx, &origin_obj, extra_ctx)?;
                obj.set(cx, "origin_condition", origin_obj)?;
            }
            TransactionAuth::Sponsored(origin, sponsor) => {
                let type_id = cx.number(TransactionAuthFlags::AuthSponsored as u8);
                obj.set(cx, "type_id", type_id)?;

                let origin_obj = cx.empty_object();
                Encode(origin).neon_js_serialize(cx, &origin_obj, extra_ctx)?;
                obj.set(cx, "origin_condition", origin_obj)?;

                let sponsor_obj = cx.empty_object();
                Encode(sponsor).neon_js_serialize(cx, &sponsor_obj, extra_ctx)?;
                obj.set(cx, "sponsor_condition", sponsor_obj)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for Encode<'_, TransactionSpendingCondition> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        match self.0 {
            TransactionSpendingCondition::Singlesig(s) => {
                Encode(s).neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionSpendingCondition::Multisig(m) => {
                Encode(m).neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionSpendingCondition::OrderIndependentMultisig(m) => {
                Encode(m).neon_js_serialize(cx, obj, extra_ctx)?;
            }
        }
        Ok(())
    }
}

/// Lift an upstream [`clarity::vm::ClarityVersion`] into the wire byte the
/// JS layer has historically reported in `clarity_version`. Upstream's enum
/// is a bare `Clarity1..Clarity6` without explicit discriminants, so `as u8`
/// would produce a 0-based index; the JS-facing value has always been the
/// 1-based wire byte (Clarity1 = 1, Clarity6 = 6).
fn clarity_version_to_wire_byte(version: ClarityVersion) -> u8 {
    match version {
        ClarityVersion::Clarity1 => 1,
        ClarityVersion::Clarity2 => 2,
        ClarityVersion::Clarity3 => 3,
        ClarityVersion::Clarity4 => 4,
        ClarityVersion::Clarity5 => 5,
        ClarityVersion::Clarity6 => 6,
    }
}

/// Lift a single-byte spending-condition `hash_mode` byte into the Stacks
/// address version expected on the JS side. Mainnet vs testnet is decided
/// by the parent transaction's version; P2PKH is the only "singlesig" mode,
/// everything else collapses to the multisig version byte.
fn address_version_for_hash_mode_byte(hash_mode: u8, tx_version: TransactionVersion) -> u8 {
    let address_hash_mode = match hash_mode {
        0x00 => AddressHashMode::SerializeP2PKH,
        0x02 => AddressHashMode::SerializeP2WPKH,
        0x01 | 0x05 => AddressHashMode::SerializeP2SH,
        0x03 | 0x07 => AddressHashMode::SerializeP2WSH,
        // Fallback for any future variant: treat as multisig.
        _ => AddressHashMode::SerializeP2SH,
    };
    match tx_version {
        TransactionVersion::Mainnet => address_hash_mode.to_version_mainnet(),
        TransactionVersion::Testnet => address_hash_mode.to_version_testnet(),
    }
}

impl NeonJsSerialize<TxSerializationContext> for Encode<'_, SinglesigSpendingCondition> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        let cond = self.0;
        let hash_mode_byte = cond.hash_mode.clone() as u8;

        let hash_mode = cx.number(hash_mode_byte);
        obj.set(cx, "hash_mode", hash_mode)?;

        let address_version =
            address_version_for_hash_mode_byte(hash_mode_byte, extra_ctx.transaction_version);
        let stacks_address = UpstreamStacksAddress::new(address_version, cond.signer.clone())
            .or_else(|e| cx.throw_error(format!("Invalid stacks address: {}", e)))?;
        let signer_obj = cx.empty_object();
        Encode(&stacks_address).neon_js_serialize(cx, &signer_obj, &())?;
        obj.set(cx, "signer", signer_obj)?;

        let nonce = cx.string(cond.nonce.to_string());
        obj.set(cx, "nonce", nonce)?;

        let tx_fee = cx.string(cond.tx_fee.to_string());
        obj.set(cx, "tx_fee", tx_fee)?;

        let key_encoding = cx.number(cond.key_encoding as u8);
        obj.set(cx, "key_encoding", key_encoding)?;

        let signature = cx.string(encode_hex(&cond.signature.0));
        obj.set(cx, "signature", signature)?;

        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for Encode<'_, MultisigSpendingCondition> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        let cond = self.0;
        let hash_mode_byte = cond.hash_mode.clone() as u8;
        encode_multisig_body(
            cx,
            obj,
            hash_mode_byte,
            &cond.signer,
            cond.nonce,
            cond.tx_fee,
            &cond.fields,
            cond.signatures_required,
            extra_ctx,
        )
    }
}

impl NeonJsSerialize<TxSerializationContext>
    for Encode<'_, OrderIndependentMultisigSpendingCondition>
{
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        let cond = self.0;
        let hash_mode_byte = cond.hash_mode.clone() as u8;
        encode_multisig_body(
            cx,
            obj,
            hash_mode_byte,
            &cond.signer,
            cond.nonce,
            cond.tx_fee,
            &cond.fields,
            cond.signatures_required,
            extra_ctx,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_multisig_body(
    cx: &mut FunctionContext,
    obj: &Handle<JsObject>,
    hash_mode_byte: u8,
    signer: &stacks_common::util::hash::Hash160,
    nonce: u64,
    tx_fee: u64,
    fields: &[TransactionAuthField],
    signatures_required: u16,
    extra_ctx: &TxSerializationContext,
) -> NeonResult<()> {
    let hash_mode = cx.number(hash_mode_byte);
    obj.set(cx, "hash_mode", hash_mode)?;

    let address_version =
        address_version_for_hash_mode_byte(hash_mode_byte, extra_ctx.transaction_version);
    let stacks_address = UpstreamStacksAddress::new(address_version, signer.clone())
        .or_else(|e| cx.throw_error(format!("Invalid stacks address: {}", e)))?;
    let signer_obj = cx.empty_object();
    Encode(&stacks_address).neon_js_serialize(cx, &signer_obj, &())?;
    obj.set(cx, "signer", signer_obj)?;

    let nonce_str = cx.string(nonce.to_string());
    obj.set(cx, "nonce", nonce_str)?;

    let tx_fee_str = cx.string(tx_fee.to_string());
    obj.set(cx, "tx_fee", tx_fee_str)?;

    let fields_js = JsArray::new(cx, fields.len());
    for (i, field) in fields.iter().enumerate() {
        let field_obj = cx.empty_object();
        Encode(field).neon_js_serialize(cx, &field_obj, &())?;
        fields_js.set(cx, i as u32, field_obj)?;
    }
    obj.set(cx, "fields", fields_js)?;

    let signatures_required_n = cx.number(signatures_required);
    obj.set(cx, "signatures_required", signatures_required_n)?;
    Ok(())
}

impl NeonJsSerialize for Encode<'_, TransactionAuthField> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        use blockstack_lib::chainstate::stacks::TransactionPublicKeyEncoding;
        match self.0 {
            TransactionAuthField::PublicKey(pubkey) => {
                let field_id = if pubkey.compressed() {
                    TransactionAuthFieldID::PublicKeyCompressed
                } else {
                    TransactionAuthFieldID::PublicKeyUncompressed
                };
                let type_id = cx.number(field_id as u8);
                obj.set(cx, "type_id", type_id)?;

                let pubkey_bytes = pubkey.to_bytes_compressed();
                let pubkey_hex = cx.string(encode_hex(&pubkey_bytes));
                obj.set(cx, "public_key", pubkey_hex)?;
            }
            TransactionAuthField::Signature(key_encoding, sig) => {
                let field_id = if *key_encoding == TransactionPublicKeyEncoding::Compressed {
                    TransactionAuthFieldID::SignatureCompressed
                } else {
                    TransactionAuthFieldID::SignatureUncompressed
                };
                let type_id = cx.number(field_id as u8);
                obj.set(cx, "type_id", type_id)?;

                let sig_hex = cx.string(encode_hex(&sig.0));
                obj.set(cx, "signature", sig_hex)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, TransactionPayload> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match self.0 {
            TransactionPayload::TokenTransfer(principal, amount, memo) => {
                let type_id = cx.number(TransactionPayloadID::TokenTransfer as u8);
                obj.set(cx, "type_id", type_id)?;

                let recipient_obj = cx.empty_object();
                Encode(principal).neon_js_serialize(cx, &recipient_obj, &())?;
                obj.set(cx, "recipient", recipient_obj)?;

                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;

                let memo_hex = cx.string(encode_hex(&memo.0));
                obj.set(cx, "memo_hex", memo_hex)?;
            }
            TransactionPayload::ContractCall(cc) => {
                let type_id = cx.number(TransactionPayloadID::ContractCall as u8);
                obj.set(cx, "type_id", type_id)?;

                Encode(cc).neon_js_serialize(cx, obj, &())?;
            }
            TransactionPayload::SmartContract(sc, version_opt) => match version_opt {
                None => {
                    let type_id = cx.number(TransactionPayloadID::SmartContract as u8);
                    obj.set(cx, "type_id", type_id)?;
                    Encode(sc).neon_js_serialize(cx, obj, &())?;
                }
                Some(version) => {
                    let type_id = cx.number(TransactionPayloadID::VersionedSmartContract as u8);
                    obj.set(cx, "type_id", type_id)?;

                    let version_n = cx.number(clarity_version_to_wire_byte(*version));
                    obj.set(cx, "clarity_version", version_n)?;

                    Encode(sc).neon_js_serialize(cx, obj, &())?;
                }
            },
            TransactionPayload::PoisonMicroblock(h1, h2) => {
                let type_id = cx.number(TransactionPayloadID::PoisonMicroblock as u8);
                obj.set(cx, "type_id", type_id)?;

                let h1_obj = cx.empty_object();
                Encode(h1).neon_js_serialize(cx, &h1_obj, &())?;
                obj.set(cx, "microblock_header_1", h1_obj)?;

                let h2_obj = cx.empty_object();
                Encode(h2).neon_js_serialize(cx, &h2_obj, &())?;
                obj.set(cx, "microblock_header_2", h2_obj)?;
            }
            TransactionPayload::Coinbase(buf, recipient_opt, vrf_opt) => {
                encode_coinbase(cx, obj, buf, recipient_opt.as_ref(), vrf_opt.as_ref())?;
            }
            TransactionPayload::TenureChange(tc) => {
                let type_id = cx.number(TransactionPayloadID::TenureChange as u8);
                obj.set(cx, "type_id", type_id)?;

                Encode(tc).neon_js_serialize(cx, obj, &())?;
            }
        }
        Ok(())
    }
}

fn encode_coinbase(
    cx: &mut FunctionContext,
    obj: &Handle<JsObject>,
    buf: &CoinbasePayload,
    recipient_opt: Option<&clarity::vm::types::PrincipalData>,
    vrf_opt: Option<&stacks_common::util::vrf::VRFProof>,
) -> NeonResult<()> {
    match (recipient_opt, vrf_opt) {
        (None, None) => {
            let type_id = cx.number(TransactionPayloadID::Coinbase as u8);
            obj.set(cx, "type_id", type_id)?;

            let payload_buffer = cx.string(encode_hex(&buf.0));
            obj.set(cx, "payload_buffer", payload_buffer)?;
        }
        (Some(recipient), None) => {
            let type_id = cx.number(TransactionPayloadID::CoinbaseToAltRecipient as u8);
            obj.set(cx, "type_id", type_id)?;

            let payload_buffer = cx.string(encode_hex(&buf.0));
            obj.set(cx, "payload_buffer", payload_buffer)?;

            let recipient_obj = cx.empty_object();
            Encode(recipient).neon_js_serialize(cx, &recipient_obj, &())?;
            obj.set(cx, "recipient", recipient_obj)?;
        }
        (recip_opt, Some(vrf)) => {
            let type_id = cx.number(TransactionPayloadID::NakamotoCoinbase as u8);
            obj.set(cx, "type_id", type_id)?;

            let payload_buffer = cx.string(encode_hex(&buf.0));
            obj.set(cx, "payload_buffer", payload_buffer)?;

            if let Some(recipient) = recip_opt {
                let recipient_obj = cx.empty_object();
                Encode(recipient).neon_js_serialize(cx, &recipient_obj, &())?;
                obj.set(cx, "recipient", recipient_obj)?;
            } else {
                let recipient_obj = cx.null();
                obj.set(cx, "recipient", recipient_obj)?;
            }

            let vrf_proof_bytes = vrf.to_bytes();
            let vrf_proof_buffer = cx.string(encode_hex(&vrf_proof_bytes));
            obj.set(cx, "vrf_proof", vrf_proof_buffer)?;
        }
    }
    Ok(())
}

impl NeonJsSerialize for Encode<'_, TransactionContractCall> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let cc = self.0;
        Encode(&cc.address).neon_js_serialize(cx, obj, &())?;

        let contract_name = cx.string(cc.contract_name.as_str());
        obj.set(cx, "contract_name", contract_name)?;

        let function_name = cx.string(cc.function_name.as_str());
        obj.set(cx, "function_name", function_name)?;

        // Wire format prefixes the function args section with a u32-BE length.
        let mut function_args_raw = u32::to_be_bytes(cc.function_args.len() as u32).to_vec();
        let function_args = JsArray::new(cx, cc.function_args.len());
        for (i, clarity_val) in cc.function_args.iter().enumerate() {
            let val_obj = cx.empty_object();
            let val_bytes = <UpstreamValue as StacksMessageCodec>::serialize_to_vec(clarity_val);
            function_args_raw.extend_from_slice(&val_bytes);
            decode_clarity_val(cx, &val_obj, clarity_val, false, &val_bytes)?;
            function_args.set(cx, i as u32, val_obj)?;
        }
        obj.set(cx, "function_args", function_args)?;

        let function_args_buff = cx.string(encode_hex(&function_args_raw));
        obj.set(cx, "function_args_buffer", function_args_buff)?;

        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, TransactionSmartContract> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let sc = self.0;
        let contract_name = cx.string(sc.name.as_str());
        obj.set(cx, "contract_name", contract_name)?;

        let code_body = cx.string(String::from_utf8_lossy(&sc.code_body));
        obj.set(cx, "code_body", code_body)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, TenureChangePayload> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let tc = self.0;
        let tenure_consensus_hash = cx.string(encode_hex(&tc.tenure_consensus_hash.0));
        obj.set(cx, "tenure_consensus_hash", tenure_consensus_hash)?;

        let prev_tenure_consensus_hash = cx.string(encode_hex(&tc.prev_tenure_consensus_hash.0));
        obj.set(cx, "prev_tenure_consensus_hash", prev_tenure_consensus_hash)?;

        let burn_view_consensus_hash = cx.string(encode_hex(&tc.burn_view_consensus_hash.0));
        obj.set(cx, "burn_view_consensus_hash", burn_view_consensus_hash)?;

        let previous_tenure_end = cx.string(encode_hex(&tc.previous_tenure_end.0));
        obj.set(cx, "previous_tenure_end", previous_tenure_end)?;

        let previous_tenure_blocks = cx.number(tc.previous_tenure_blocks);
        obj.set(cx, "previous_tenure_blocks", previous_tenure_blocks)?;

        let cause = cx.number(tc.cause as u8);
        obj.set(cx, "cause", cause)?;

        let pubkey_hash = cx.string(encode_hex(tc.pubkey_hash.as_bytes()));
        obj.set(cx, "pubkey_hash", pubkey_hash)?;

        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, StacksMicroblockHeader> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let h = self.0;
        // Microblock headers carry no length-prefixed children, so a fresh
        // `serialize_to_vec` reproduces the exact bytes that were on the wire.
        let serialized = <StacksMicroblockHeader as StacksMessageCodec>::serialize_to_vec(h);
        let buffer = cx.string(encode_hex(&serialized));
        obj.set(cx, "buffer", buffer)?;

        let version = cx.number(h.version);
        obj.set(cx, "version", version)?;

        let sequence = cx.number(h.sequence);
        obj.set(cx, "sequence", sequence)?;

        let prev_block = cx.string(encode_hex(&h.prev_block.0));
        obj.set(cx, "prev_block", prev_block)?;

        let tx_merkle_root = cx.string(encode_hex(&h.tx_merkle_root.0));
        obj.set(cx, "tx_merkle_root", tx_merkle_root)?;

        let signature = cx.string(encode_hex(&h.signature.0));
        obj.set(cx, "signature", signature)?;

        Ok(())
    }
}

// Helper used by older call sites in the address module. Kept as a free
// function rather than an inherent method on upstream's `StacksAddress`.
#[allow(dead_code)]
fn stacks_address_c32(addr: &UpstreamStacksAddress) -> Result<String, String> {
    c32_address(addr.version(), addr.bytes().as_bytes())
}
