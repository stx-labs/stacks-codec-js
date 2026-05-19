//! Shared `NeonJsSerialize` impls for address-shaped upstream types.
//!
//! `StacksAddress`, `StandardPrincipalData`, and `PrincipalData` all surface
//! the same JS shape (`address_version`, `address_hash_bytes`, `address`,
//! plus a `contract_name` and `type_id` where applicable) and several
//! encoders need to emit them. We define the trait impls once here, against
//! the `Encode<'_, UpstreamX>` wrapper, so `post_condition`, `stacks_tx`,
//! and `stacks_block` can all reuse them.
use clarity::vm::types::serialization::TypePrefix as UpstreamTypePrefix;
use clarity::vm::types::{
    PrincipalData as UpstreamPrincipalData, StandardPrincipalData as UpstreamStandardPrincipalData,
};
use neon::prelude::*;
use stacks_common::types::chainstate::StacksAddress as UpstreamStacksAddress;

use crate::address::c32_address;
use crate::hex::encode_hex;
use crate::neon_util::{Encode, NeonJsSerialize};

impl NeonJsSerialize for Encode<'_, UpstreamStacksAddress> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let version = self.0.version();
        let hash = self.0.bytes().as_bytes();
        let address_version = cx.number(version);
        obj.set(cx, "address_version", address_version)?;

        let address_hash_bytes = cx.string(encode_hex(hash));
        obj.set(cx, "address_hash_bytes", address_hash_bytes)?;

        let address_str = c32_address(version, hash)
            .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;
        let address = cx.string(address_str);
        obj.set(cx, "address", address)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, UpstreamStandardPrincipalData> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let version = self.0.version();
        let hash = &self.0 .1;
        let address_version = cx.number(version);
        obj.set(cx, "address_version", address_version)?;

        let address_hash_bytes = cx.string(encode_hex(hash));
        obj.set(cx, "address_hash_bytes", address_hash_bytes)?;

        let address_str = c32_address(version, hash)
            .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;
        let address = cx.string(address_str);
        obj.set(cx, "address", address)?;
        Ok(())
    }
}

/// Emit a Clarity principal (Standard or Contract) using the historical
/// Stacks-API shape: a `type_id` byte that matches the Clarity wire
/// `TypePrefix`, plus the underlying address fields. Contract principals
/// also include a `contract_name`.
impl NeonJsSerialize for Encode<'_, UpstreamPrincipalData> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match self.0 {
            UpstreamPrincipalData::Standard(spd) => {
                let type_id = cx.number(UpstreamTypePrefix::PrincipalStandard.to_u8());
                obj.set(cx, "type_id", type_id)?;
                Encode(spd).neon_js_serialize(cx, obj, &())?;
            }
            UpstreamPrincipalData::Contract(qci) => {
                let type_id = cx.number(UpstreamTypePrefix::PrincipalContract.to_u8());
                obj.set(cx, "type_id", type_id)?;

                let contract_name = cx.string(qci.name.as_str());
                obj.set(cx, "contract_name", contract_name)?;

                Encode(&qci.issuer).neon_js_serialize(cx, obj, &())?;
            }
        }
        Ok(())
    }
}
