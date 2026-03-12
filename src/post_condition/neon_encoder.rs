use neon::prelude::*;

use crate::address::stacks_address::StacksAddress;
use crate::clarity_value::neon_encoder::decode_clarity_val;
use crate::{address::c32::c32_address, hex::encode_hex};

use super::deserialize::{
    AssetInfo, AssetInfoID, FungibleConditionCode, NonfungibleConditionCode,
    PostConditionPrincipal, PostConditionPrincipalID, TransactionPostCondition,
};

impl TransactionPostCondition {
    pub fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
    ) -> NeonResult<()> {
        match *self {
            TransactionPostCondition::STX(ref principal, ref fungible_condition, ref amount) => {
                let asset_info_id = cx.number(AssetInfoID::STX as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let pricipal_obj = cx.empty_object();
                principal.neon_js_serialize(cx, &pricipal_obj)?;
                obj.set(cx, "principal", pricipal_obj)?;

                fungible_condition.neon_js_serialize(cx, obj)?;

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
                principal.neon_js_serialize(cx, &pricipal_obj)?;
                obj.set(cx, "principal", pricipal_obj)?;

                let asset_info_obj = cx.empty_object();
                asset_info.neon_js_serialize(cx, &asset_info_obj)?;
                obj.set(cx, "asset", asset_info_obj)?;

                fungible_condition.neon_js_serialize(cx, obj)?;

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
                principal.neon_js_serialize(cx, &pricipal_obj)?;
                obj.set(cx, "principal", pricipal_obj)?;

                let asset_info_obj = cx.empty_object();
                asset_info.neon_js_serialize(cx, &asset_info_obj)?;
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

                nonfungible_condition.neon_js_serialize(cx, obj)?;
            }
        };
        Ok(())
    }
}

impl PostConditionPrincipal {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
    ) -> NeonResult<()> {
        match *self {
            PostConditionPrincipal::Origin => {
                let type_id = cx.number(PostConditionPrincipalID::Origin as u8);
                obj.set(cx, "type_id", type_id)?;
            }
            PostConditionPrincipal::Standard(ref address) => {
                let type_id = cx.number(PostConditionPrincipalID::Standard as u8);
                obj.set(cx, "type_id", type_id)?;

                address.neon_js_serialize(cx, obj)?;
            }
            PostConditionPrincipal::Contract(ref address, ref contract_name) => {
                let type_id = cx.number(PostConditionPrincipalID::Contract as u8);
                obj.set(cx, "type_id", type_id)?;

                address.neon_js_serialize(cx, obj)?;

                let contract_str = cx.string(contract_name.as_str());
                obj.set(cx, "contract_name", contract_str)?;
            }
        }
        Ok(())
    }
}

impl StacksAddress {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
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

impl FungibleConditionCode {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
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

impl AssetInfo {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
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

impl NonfungibleConditionCode {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
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
