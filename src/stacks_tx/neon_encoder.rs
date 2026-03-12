use std::convert::{TryFrom, TryInto};

use neon::prelude::*;

use crate::address::c32::c32_address;
use crate::address::stacks_address::{AddressHashMode, StacksAddress};
use crate::clarity_value::deserialize::TypePrefix;
use crate::clarity_value::neon_encoder::decode_clarity_val;
use crate::hex::encode_hex;
use crate::neon_util::NeonJsSerialize;

use crate::post_condition::deserialize::{
    AssetInfo, AssetInfoID, FungibleConditionCode, NonfungibleConditionCode,
    PostConditionPrincipal, PostConditionPrincipalID, TransactionPostCondition,
};

use super::deserialize::{
    MultisigSpendingCondition, PrincipalData, SinglesigSpendingCondition, StacksMicroblockHeader,
    StacksTransaction, StandardPrincipalData, TransactionAuth, TransactionAuthField,
    TransactionAuthFieldID, TransactionAuthFlags, TransactionContractCall, TransactionPayload,
    TransactionPayloadID, TransactionPublicKeyEncoding, TransactionSmartContract,
    TransactionSpendingCondition, TransactionTenureChange, TransactionVersion,
};

/// Re-export for use in stacks_block module
pub mod neon_encoder_internal {
    use super::*;
    
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
}

use neon_encoder_internal::TxSerializationContext;

impl NeonJsSerialize for StacksTransaction {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let version_number = cx.number(self.version as u8);
        obj.set(cx, "version", version_number)?;

        let chain_id = cx.number(self.chain_id);
        obj.set(cx, "chain_id", chain_id)?;

        let auth_obj = cx.empty_object();
        self.auth.neon_js_serialize(
            cx,
            &auth_obj,
            &TxSerializationContext {
                transaction_version: self.version,
            },
        )?;
        obj.set(cx, "auth", auth_obj)?;

        let anchor_mode = cx.number(self.anchor_mode as u8);
        obj.set(cx, "anchor_mode", anchor_mode)?;

        let post_condition_mode = cx.number(self.post_condition_mode as u8);
        obj.set(cx, "post_condition_mode", post_condition_mode)?;

        let post_conditions = JsArray::new(cx, self.post_conditions.len());
        for (i, x) in self.post_conditions.iter().enumerate() {
            let post_condition_obj = cx.empty_object();
            x.neon_js_serialize(cx, &post_condition_obj)?;
            post_conditions.set(cx, i as u32, post_condition_obj)?;
        }
        obj.set(cx, "post_conditions", post_conditions)?;

        let post_conditions_buff = cx.string(encode_hex(&self.post_conditions_serialized));
        obj.set(cx, "post_conditions_buffer", post_conditions_buff)?;

        let payload_obj = cx.empty_object();
        self.payload.neon_js_serialize(cx, &payload_obj, &())?;
        obj.set(cx, "payload", payload_obj)?;

        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for TransactionAuth {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        match *self {
            TransactionAuth::Standard(ref origin_condition) => {
                let type_id = cx.number(TransactionAuthFlags::AuthStandard as u8);
                obj.set(cx, "type_id", type_id)?;

                let origin_condition_obj = cx.empty_object();
                origin_condition.neon_js_serialize(cx, &origin_condition_obj, extra_ctx)?;
                obj.set(cx, "origin_condition", origin_condition_obj)?;
            }
            TransactionAuth::Sponsored(ref origin_condition, ref sponsor_condition) => {
                let type_id = cx.number(TransactionAuthFlags::AuthSponsored as u8);
                obj.set(cx, "type_id", type_id)?;

                let origin_condition_obj = cx.empty_object();
                origin_condition.neon_js_serialize(cx, &origin_condition_obj, extra_ctx)?;
                obj.set(cx, "origin_condition", origin_condition_obj)?;

                let sponsor_condition_obj = cx.empty_object();
                sponsor_condition.neon_js_serialize(cx, &sponsor_condition_obj, extra_ctx)?;
                obj.set(cx, "sponsor_condition", sponsor_condition_obj)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for TransactionSpendingCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        match *self {
            TransactionSpendingCondition::Singlesig(ref data) => {
                data.neon_js_serialize(cx, obj, &extra_ctx)?;
            }
            TransactionSpendingCondition::Multisig(ref data) => {
                data.neon_js_serialize(cx, obj, &extra_ctx)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for SinglesigSpendingCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        let hash_mode_int = self.hash_mode as u8;

        let hash_mode = cx.number(hash_mode_int);
        obj.set(cx, "hash_mode", hash_mode)?;

        let stacks_address_hash_mode = AddressHashMode::try_from(hash_mode_int).unwrap();
        let stacks_address_version = match extra_ctx.transaction_version {
            TransactionVersion::Mainnet => stacks_address_hash_mode.to_version_mainnet(),
            TransactionVersion::Testnet => stacks_address_hash_mode.to_version_testnet(),
        };
        let stacks_address = StacksAddress::new(stacks_address_version, self.signer);
        let stacks_address_obj = cx.empty_object();
        stacks_address.neon_js_serialize(cx, &stacks_address_obj, &())?;
        obj.set(cx, "signer", stacks_address_obj)?;

        // TODO: bigint
        let nonce = cx.string(self.nonce.to_string());
        obj.set(cx, "nonce", nonce)?;

        // TODO: bigint
        let tx_fee = cx.string(self.tx_fee.to_string());
        obj.set(cx, "tx_fee", tx_fee)?;

        let key_encoding = cx.number(self.key_encoding as u8);
        obj.set(cx, "key_encoding", key_encoding)?;

        let signature = cx.string(encode_hex(&self.signature.0));
        obj.set(cx, "signature", signature)?;

        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for MultisigSpendingCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        let hash_mode_int = self.hash_mode as u8;
        let hash_mode = cx.number(hash_mode_int);
        obj.set(cx, "hash_mode", hash_mode)?;

        let stacks_address_hash_mode = AddressHashMode::try_from(hash_mode_int).unwrap();
        let stacks_address_version = match extra_ctx.transaction_version {
            TransactionVersion::Mainnet => stacks_address_hash_mode.to_version_mainnet(),
            TransactionVersion::Testnet => stacks_address_hash_mode.to_version_testnet(),
        };
        let stacks_address = StacksAddress::new(stacks_address_version, self.signer);
        let stacks_address_obj = cx.empty_object();
        stacks_address.neon_js_serialize(cx, &stacks_address_obj, &())?;
        obj.set(cx, "signer", stacks_address_obj)?;

        // TODO: bigint
        let nonce = cx.string(self.nonce.to_string());
        obj.set(cx, "nonce", nonce)?;

        // TODO: bigint
        let tx_fee = cx.string(self.tx_fee.to_string());
        obj.set(cx, "tx_fee", tx_fee)?;

        let fields = JsArray::new(cx, self.fields.len().try_into().unwrap());
        for (i, x) in self.fields.iter().enumerate() {
            let field_obj = cx.empty_object();
            x.neon_js_serialize(cx, &field_obj, &())?;
            fields.set(cx, i as u32, field_obj)?;
        }
        obj.set(cx, "fields", fields)?;

        let signatures_required = cx.number(self.signatures_required);
        obj.set(cx, "signatures_required", signatures_required)?;

        Ok(())
    }
}

impl NeonJsSerialize for StacksAddress {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let address_version = cx.number(self.version);
        obj.set(cx, "address_version", address_version)?;

        let address_hash_bytes = cx.string(encode_hex(&self.hash160_bytes));
        obj.set(cx, "address_hash_bytes", address_hash_bytes)?;

        let address_str = c32_address(self.version, &self.hash160_bytes)
            .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;
        let address = cx.string(address_str);
        obj.set(cx, "address", address)?;

        Ok(())
    }
}

impl NeonJsSerialize for TransactionAuthField {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match *self {
            TransactionAuthField::PublicKey(ref pubkey) => {
                let field_id = if pubkey.compressed {
                    TransactionAuthFieldID::PublicKeyCompressed
                } else {
                    TransactionAuthFieldID::PublicKeyUncompressed
                };
                let type_id = cx.number(field_id as u8);
                obj.set(cx, "type_id", type_id)?;

                let pubkey_hex = cx.string(encode_hex(&pubkey.key.0));
                obj.set(cx, "public_key", pubkey_hex)?;
            }
            TransactionAuthField::Signature(ref key_encoding, ref sig) => {
                let field_id = if *key_encoding == TransactionPublicKeyEncoding::Compressed {
                    TransactionAuthFieldID::SignatureCompressed
                } else {
                    TransactionAuthFieldID::SignatureUncompressed
                };
                let type_id = cx.number(field_id as u8);
                obj.set(cx, "type_id", type_id)?;

                let pubkey_hex = cx.string(encode_hex(&sig.0));
                obj.set(cx, "signature", pubkey_hex)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for TransactionPostCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<()> {
        match *self {
            TransactionPostCondition::STX(ref principal, ref fungible_condition, ref amount) => {
                let asset_info_id = cx.number(AssetInfoID::STX as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let pricipal_obj = cx.empty_object();
                principal.neon_js_serialize(cx, &pricipal_obj, extra_ctx)?;
                obj.set(cx, "principal", pricipal_obj)?;

                fungible_condition.neon_js_serialize(cx, obj, extra_ctx)?;

                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;
            }
            TransactionPostCondition::Fungible(
                ref principal,
                ref asset_info,
                ref fungible_condition,
                ref amount,
            ) => {
                let asset_info_id = cx.number(AssetInfoID::FungibleAsset as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let pricipal_obj = cx.empty_object();
                principal.neon_js_serialize(cx, &pricipal_obj, extra_ctx)?;
                obj.set(cx, "principal", pricipal_obj)?;

                let asset_info_obj = cx.empty_object();
                asset_info.neon_js_serialize(cx, &asset_info_obj, extra_ctx)?;
                obj.set(cx, "asset", asset_info_obj)?;

                fungible_condition.neon_js_serialize(cx, obj, extra_ctx)?;

                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;
            }
            TransactionPostCondition::Nonfungible(
                ref principal,
                ref asset_info,
                ref asset_value,
                ref nonfungible_condition,
            ) => {
                let asset_info_id = cx.number(AssetInfoID::NonfungibleAsset as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let pricipal_obj = cx.empty_object();
                principal.neon_js_serialize(cx, &pricipal_obj, extra_ctx)?;
                obj.set(cx, "principal", pricipal_obj)?;

                let asset_info_obj = cx.empty_object();
                asset_info.neon_js_serialize(cx, &asset_info_obj, extra_ctx)?;
                obj.set(cx, "asset", asset_info_obj)?;

                let asset_value_obj = cx.empty_object();
                decode_clarity_val(
                    cx,
                    &asset_value_obj,
                    &asset_value,
                    false,
                    asset_value.serialized_bytes.as_ref().unwrap(),
                )?;
                obj.set(cx, "asset_value", asset_value_obj)?;

                nonfungible_condition.neon_js_serialize(cx, obj, extra_ctx)?;
            }
        };
        Ok(())
    }
}

impl NeonJsSerialize for FungibleConditionCode {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let condition_name = match *self {
            FungibleConditionCode::SentEq => "sent_equal_to",
            FungibleConditionCode::SentGt => "sent_greater_than",
            FungibleConditionCode::SentGe => "sent_greater_than_or_equal_to",
            FungibleConditionCode::SentLt => "sent_less_than",
            FungibleConditionCode::SentLe => "sent_less_than_or_equal_to",
        };
        let condition_code = cx.number(*self as u8);
        obj.set(cx, "condition_code", condition_code)?;
        let condition_name_str = cx.string(condition_name);
        obj.set(cx, "condition_name", condition_name_str)?;
        Ok(())
    }
}

impl NeonJsSerialize for NonfungibleConditionCode {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let condition_name = match *self {
            NonfungibleConditionCode::Sent => "sent",
            NonfungibleConditionCode::NotSent => "not_sent",
            NonfungibleConditionCode::MaybeSent => "maybe_sent",
        };
        let condition_code = cx.number(*self as u8);
        obj.set(cx, "condition_code", condition_code)?;
        let condition_name_str = cx.string(condition_name);
        obj.set(cx, "condition_name", condition_name_str)?;
        Ok(())
    }
}

impl NeonJsSerialize for PostConditionPrincipal {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<()> {
        match *self {
            PostConditionPrincipal::Origin => {
                let type_id = cx.number(PostConditionPrincipalID::Origin as u8);
                obj.set(cx, "type_id", type_id)?;
            }
            PostConditionPrincipal::Standard(ref address) => {
                let type_id = cx.number(PostConditionPrincipalID::Standard as u8);
                obj.set(cx, "type_id", type_id)?;

                address.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            PostConditionPrincipal::Contract(ref address, ref contract_name) => {
                let type_id = cx.number(PostConditionPrincipalID::Contract as u8);
                obj.set(cx, "type_id", type_id)?;

                address.neon_js_serialize(cx, obj, extra_ctx)?;

                let contract_str = cx.string(contract_name.as_str());
                obj.set(cx, "contract_name", contract_str)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for AssetInfo {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let contract_address_str = c32_address(
            self.contract_address.version,
            &self.contract_address.hash160_bytes,
        )
        .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;
        let contract_address = cx.string(contract_address_str);
        obj.set(cx, "contract_address", contract_address)?;

        let contract_name = cx.string(self.contract_name.as_str());
        obj.set(cx, "contract_name", contract_name)?;

        let asset_name = cx.string(self.asset_name.as_str());
        obj.set(cx, "asset_name", asset_name)?;
        Ok(())
    }
}

impl NeonJsSerialize for TransactionPayload {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<()> {
        match *self {
            TransactionPayload::TokenTransfer(ref address, ref amount, ref memo) => {
                let type_id = cx.number(TransactionPayloadID::TokenTransfer as u8);
                obj.set(cx, "type_id", type_id)?;

                let recipient_obj = cx.empty_object();
                address.neon_js_serialize(cx, &recipient_obj, extra_ctx)?;
                obj.set(cx, "recipient", recipient_obj)?;

                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;

                let memo_hex = cx.string(encode_hex(&memo.0));
                obj.set(cx, "memo_hex", memo_hex)?;
            }
            TransactionPayload::ContractCall(ref contract_call) => {
                let type_id = cx.number(TransactionPayloadID::ContractCall as u8);
                obj.set(cx, "type_id", type_id)?;

                contract_call.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionPayload::SmartContract(ref smart_contract) => {
                let type_id = cx.number(TransactionPayloadID::SmartContract as u8);
                obj.set(cx, "type_id", type_id)?;

                smart_contract.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionPayload::PoisonMicroblock(ref h1, ref h2) => {
                let type_id = cx.number(TransactionPayloadID::PoisonMicroblock as u8);
                obj.set(cx, "type_id", type_id)?;

                let microblock_header_1_obj = cx.empty_object();
                h1.neon_js_serialize(cx, &microblock_header_1_obj, extra_ctx)?;
                obj.set(cx, "microblock_header_1", microblock_header_1_obj)?;

                let microblock_header_2_obj = cx.empty_object();
                h2.neon_js_serialize(cx, &microblock_header_2_obj, extra_ctx)?;
                obj.set(cx, "microblock_header_2", microblock_header_2_obj)?;
            }
            TransactionPayload::Coinbase(ref buf) => {
                let type_id = cx.number(TransactionPayloadID::Coinbase as u8);
                obj.set(cx, "type_id", type_id)?;

                let payload_buffer = cx.string(encode_hex(&buf.0));
                obj.set(cx, "payload_buffer", payload_buffer)?;
            }
            TransactionPayload::CoinbaseToAltRecipient(ref buf, ref address) => {
                let type_id = cx.number(TransactionPayloadID::CoinbaseToAltRecipient as u8);
                obj.set(cx, "type_id", type_id)?;

                let payload_buffer = cx.string(encode_hex(&buf.0));
                obj.set(cx, "payload_buffer", payload_buffer)?;

                let recipient_obj = cx.empty_object();
                address.neon_js_serialize(cx, &recipient_obj, extra_ctx)?;
                obj.set(cx, "recipient", recipient_obj)?;
            }
            TransactionPayload::VersionedSmartContract(ref smart_contract, ref version) => {
                let type_id = cx.number(TransactionPayloadID::VersionedSmartContract as u8);
                obj.set(cx, "type_id", type_id)?;

                let type_id = cx.number(*version as u8);
                obj.set(cx, "clarity_version", type_id)?;

                smart_contract.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionPayload::TenureChange(ref tenure_change) => {
                let type_id = cx.number(TransactionPayloadID::TenureChange as u8);
                obj.set(cx, "type_id", type_id)?;

                tenure_change.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionPayload::NakamotoCoinbase(ref buf, ref principal, ref vrf_proof) => {
                let type_id = cx.number(TransactionPayloadID::NakamotoCoinbase as u8);
                obj.set(cx, "type_id", type_id)?;

                let payload_buffer = cx.string(encode_hex(&buf.0));
                obj.set(cx, "payload_buffer", payload_buffer)?;

                if let Some(principal) = principal {
                    let recipient_obj = cx.empty_object();
                    principal.neon_js_serialize(cx, &recipient_obj, extra_ctx)?;
                    obj.set(cx, "recipient", recipient_obj)?;
                } else {
                    let recipient_obj = cx.null();
                    obj.set(cx, "recipient", recipient_obj)?;
                }

                let vrf_proof_buffer = cx.string(encode_hex(&vrf_proof.0));
                obj.set(cx, "vrf_proof", vrf_proof_buffer)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for PrincipalData {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<()> {
        match self {
            PrincipalData::Standard(standard_principal) => {
                let type_prefix = TypePrefix::PrincipalStandard.to_u8();
                let type_id = cx.number(type_prefix);
                obj.set(cx, "type_id", type_id)?;
                standard_principal.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            PrincipalData::Contract(contract_identifier) => {
                let type_prefix = TypePrefix::PrincipalContract.to_u8();
                let type_id = cx.number(type_prefix);
                obj.set(cx, "type_id", type_id)?;

                let contract_name = cx.string(contract_identifier.name.as_str());
                obj.set(cx, "contract_name", contract_name)?;

                contract_identifier
                    .issuer
                    .neon_js_serialize(cx, obj, extra_ctx)?;
            }
        };
        Ok(())
    }
}

impl NeonJsSerialize for StandardPrincipalData {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let address_version = cx.number(self.0);
        obj.set(cx, "address_version", address_version)?;

        let address_hash_bytes = cx.string(encode_hex(&self.1));
        obj.set(cx, "address_hash_bytes", address_hash_bytes)?;

        let address_string = c32_address(self.0, &self.1)
            .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;

        let address = cx.string(address_string);
        obj.set(cx, "address", address)?;

        Ok(())
    }
}

impl NeonJsSerialize for TransactionContractCall {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<()> {
        self.address.neon_js_serialize(cx, obj, extra_ctx)?;

        let contract_name = cx.string(self.contract_name.as_str());
        obj.set(cx, "contract_name", contract_name)?;

        let function_name = cx.string(self.function_name.as_str());
        obj.set(cx, "function_name", function_name)?;

        let mut function_args_raw = u32::to_be_bytes(self.function_args.len() as u32).to_vec();
        let function_args = JsArray::new(cx, self.function_args.len());
        for (i, clarity_val) in self.function_args.iter().enumerate() {
            let val_obj = cx.empty_object();
            function_args_raw.extend_from_slice(&clarity_val.serialized_bytes.as_ref().unwrap());
            decode_clarity_val(
                cx,
                &val_obj,
                &clarity_val,
                false,
                &clarity_val.serialized_bytes.as_ref().unwrap(),
            )?;
            function_args.set(cx, i as u32, val_obj)?;
        }
        obj.set(cx, "function_args", function_args)?;

        let function_args_buff = cx.string(encode_hex(&function_args_raw));
        obj.set(cx, "function_args_buffer", function_args_buff)?;

        Ok(())
    }
}

impl NeonJsSerialize for TransactionSmartContract {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let contract_name = cx.string(self.name.as_str());
        obj.set(cx, "contract_name", contract_name)?;

        let code_body = cx.string(String::from_utf8_lossy(&self.code_body.0));
        obj.set(cx, "code_body", code_body)?;
        Ok(())
    }
}

impl NeonJsSerialize for TransactionTenureChange {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let tenure_consensus_hash = cx.string(encode_hex(&self.tenure_consensus_hash));
        obj.set(cx, "tenure_consensus_hash", tenure_consensus_hash)?;

        let prev_tenure_consensus_hash = cx.string(encode_hex(&self.prev_tenure_consensus_hash));
        obj.set(cx, "prev_tenure_consensus_hash", prev_tenure_consensus_hash)?;

        let burn_view_consensus_hash = cx.string(encode_hex(&self.burn_view_consensus_hash));
        obj.set(cx, "burn_view_consensus_hash", burn_view_consensus_hash)?;

        let previous_tenure_end = cx.string(encode_hex(&self.previous_tenure_end));
        obj.set(cx, "previous_tenure_end", previous_tenure_end)?;

        let previous_tenure_blocks = cx.number(self.previous_tenure_blocks);
        obj.set(cx, "previous_tenure_blocks", previous_tenure_blocks)?;

        let cause = cx.number(self.cause as u8);
        obj.set(cx, "cause", cause)?;

        let pubkey_hash = cx.string(encode_hex(&self.pubkey_hash));
        obj.set(cx, "pubkey_hash", pubkey_hash)?;

        Ok(())
    }
}

impl NeonJsSerialize for StacksMicroblockHeader {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let buffer = cx.string(encode_hex(&self.serialized_bytes));
        obj.set(cx, "buffer", buffer)?;

        let version = cx.number(self.version);
        obj.set(cx, "version", version)?;

        let sequence = cx.number(self.sequence);
        obj.set(cx, "sequence", sequence)?;

        let prev_block = cx.string(encode_hex(&self.prev_block.0));
        obj.set(cx, "prev_block", prev_block)?;

        let tx_merkle_root = cx.string(encode_hex(&self.tx_merkle_root.0));
        obj.set(cx, "tx_merkle_root", tx_merkle_root)?;

        let signature = cx.string(encode_hex(&self.signature.0));
        obj.set(cx, "signature", signature)?;

        Ok(())
    }
}
