//! Neon (JS) serialization for upstream transaction post-condition types.
//!
//! All impls are written against `Encode<'_, UpstreamX>` wrappers so the
//! orphan rule is satisfied for the upstream `TransactionPostCondition`
//! enum and its sub-types from `blockstack_lib::chainstate::stacks`.
use clarity::vm::types::Value as UpstreamValue;
use neon::prelude::*;
use stacks_codec::StacksMessageCodec;

use crate::clarity_value::neon_encoder::decode_clarity_val;
use crate::neon_util::{Encode, NeonJsSerialize};

use super::deserialize::{
    AssetInfo, AssetInfoID, FungibleConditionCode, NonfungibleConditionCode,
    PostConditionPrincipal, PostConditionPrincipalID, TransactionPostCondition,
};

impl NeonJsSerialize for Encode<'_, TransactionPostCondition> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match self.0 {
            TransactionPostCondition::STX(principal, fungible_condition, amount) => {
                let asset_info_id = cx.number(AssetInfoID::STX as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let principal_obj = cx.empty_object();
                Encode(principal).neon_js_serialize(cx, &principal_obj, &())?;
                obj.set(cx, "principal", principal_obj)?;

                Encode(fungible_condition).neon_js_serialize(cx, obj, &())?;

                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;
            }
            TransactionPostCondition::Fungible(principal, asset_info, fungible_condition, amount) => {
                let asset_info_id = cx.number(AssetInfoID::FungibleAsset as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let principal_obj = cx.empty_object();
                Encode(principal).neon_js_serialize(cx, &principal_obj, &())?;
                obj.set(cx, "principal", principal_obj)?;

                let asset_info_obj = cx.empty_object();
                Encode(asset_info).neon_js_serialize(cx, &asset_info_obj, &())?;
                obj.set(cx, "asset", asset_info_obj)?;

                Encode(fungible_condition).neon_js_serialize(cx, obj, &())?;

                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;
            }
            TransactionPostCondition::Nonfungible(principal, asset_info, asset_value, nonfungible_condition) => {
                let asset_info_id = cx.number(AssetInfoID::NonfungibleAsset as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let principal_obj = cx.empty_object();
                Encode(principal).neon_js_serialize(cx, &principal_obj, &())?;
                obj.set(cx, "principal", principal_obj)?;

                let asset_info_obj = cx.empty_object();
                Encode(asset_info).neon_js_serialize(cx, &asset_info_obj, &())?;
                obj.set(cx, "asset", asset_info_obj)?;

                let asset_value_obj = cx.empty_object();
                let asset_value_bytes =
                    <UpstreamValue as StacksMessageCodec>::serialize_to_vec(asset_value);
                decode_clarity_val(cx, &asset_value_obj, asset_value, false, &asset_value_bytes)?;
                obj.set(cx, "asset_value", asset_value_obj)?;

                Encode(nonfungible_condition).neon_js_serialize(cx, obj, &())?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, PostConditionPrincipal> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match self.0 {
            PostConditionPrincipal::Origin => {
                let type_id = cx.number(PostConditionPrincipalID::Origin as u8);
                obj.set(cx, "type_id", type_id)?;
            }
            PostConditionPrincipal::Standard(address) => {
                let type_id = cx.number(PostConditionPrincipalID::Standard as u8);
                obj.set(cx, "type_id", type_id)?;

                Encode(address).neon_js_serialize(cx, obj, &())?;
            }
            PostConditionPrincipal::Contract(address, contract_name) => {
                let type_id = cx.number(PostConditionPrincipalID::Contract as u8);
                obj.set(cx, "type_id", type_id)?;

                Encode(address).neon_js_serialize(cx, obj, &())?;

                let contract_str = cx.string(contract_name.as_str());
                obj.set(cx, "contract_name", contract_str)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, FungibleConditionCode> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let condition_name = match self.0 {
            FungibleConditionCode::SentEq => "sent_equal_to",
            FungibleConditionCode::SentGt => "sent_greater_than",
            FungibleConditionCode::SentGe => "sent_greater_than_or_equal_to",
            FungibleConditionCode::SentLt => "sent_less_than",
            FungibleConditionCode::SentLe => "sent_less_than_or_equal_to",
        };
        let condition_code = cx.number(*self.0 as u8);
        obj.set(cx, "condition_code", condition_code)?;
        let condition_name_str = cx.string(condition_name);
        obj.set(cx, "condition_name", condition_name_str)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, NonfungibleConditionCode> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let condition_name = match self.0 {
            NonfungibleConditionCode::Sent => "sent",
            NonfungibleConditionCode::NotSent => "not_sent",
            NonfungibleConditionCode::MaybeSent => "maybe_sent",
        };
        let condition_code = cx.number(*self.0 as u8);
        obj.set(cx, "condition_code", condition_code)?;
        let condition_name_str = cx.string(condition_name);
        obj.set(cx, "condition_name", condition_name_str)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, AssetInfo> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let addr = &self.0.contract_address;
        let contract_address_str =
            crate::address::c32::c32_address(addr.version(), addr.bytes().as_bytes())
                .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;
        let contract_address = cx.string(contract_address_str);
        obj.set(cx, "contract_address", contract_address)?;

        let contract_name = cx.string(self.0.contract_name.as_str());
        obj.set(cx, "contract_name", contract_name)?;

        let asset_name = cx.string(self.0.asset_name.as_str());
        obj.set(cx, "asset_name", asset_name)?;
        Ok(())
    }
}
