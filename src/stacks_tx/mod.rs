//! Stacks transaction decoding (Neon entry point + deserializer).
//!
//! All types come directly from upstream
//! (`blockstack_lib::chainstate::stacks`); parsing goes through the canonical
//! `<StacksTransaction as StacksMessageCodec>::consensus_deserialize`. The
//! Neon encoder in [`neon_encoder`] operates on these upstream types via the
//! `Encode<'_, T>` newtype wrapper from `crate::neon_util`.

use std::io::Cursor;

pub use blockstack_lib::chainstate::stacks::{
    CoinbasePayload, MultisigHashMode, MultisigSpendingCondition, OrderIndependentMultisigHashMode,
    OrderIndependentMultisigSpendingCondition, SinglesigHashMode, SinglesigSpendingCondition,
    StacksMicroblockHeader, StacksTransaction, TenureChangeCause, TenureChangePayload,
    TransactionAnchorMode, TransactionAuth, TransactionAuthField, TransactionAuthFieldID,
    TransactionAuthFlags, TransactionContractCall, TransactionPayload, TransactionPayloadID,
    TransactionPostConditionMode, TransactionPublicKeyEncoding, TransactionSmartContract,
    TransactionSpendingCondition, TransactionVersion,
};
pub use clarity::vm::ClarityVersion;
use neon::prelude::*;
use sha2::{Digest, Sha512_256};
use stacks_codec::StacksMessageCodec;

use crate::hex::encode_hex;
use crate::neon_util::*;
use crate::serialize_util::DeserializeError;

pub mod neon_encoder;
pub use neon_encoder::neon_encoder_internal;

/// Deserialize a single Stacks transaction from the wire format.
pub fn deserialize_transaction(
    fd: &mut Cursor<&[u8]>,
) -> Result<StacksTransaction, DeserializeError> {
    <StacksTransaction as StacksMessageCodec>::consensus_deserialize(fd)
        .map_err(|e| DeserializeError::from(format!("Failed to decode transaction: {}", e)))
}

pub fn decode_transaction(mut cx: FunctionContext) -> JsResult<JsObject> {
    let (tx, tx_id_bytes) = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        let tx = deserialize_transaction(&mut cursor)
            .map_err(|e| format!("Failed to decode transaction: {:?}\n", &e))?;
        let tx_id_bytes = Sha512_256::digest(val_bytes);
        Ok((tx, tx_id_bytes))
    })
    .or_else(|e| cx.throw_error(e))?;

    let tx_json_obj = cx.empty_object();

    let tx_id = cx.string(encode_hex(&tx_id_bytes));
    tx_json_obj.set(&mut cx, "tx_id", tx_id)?;

    Encode(&tx).neon_js_serialize(&mut cx, &tx_json_obj, &())?;
    Ok(tx_json_obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hex::decode_hex,
        post_condition::{
            AssetInfo, FungibleConditionCode, NonfungibleConditionCode, PostConditionPrincipal,
            TransactionPostCondition,
        },
    };
    use clarity::vm::types::Value as UpstreamValue;
    use clarity::vm::{ClarityName, ContractName};
    use stacks_common::types::chainstate::StacksAddress as UpstreamStacksAddress;
    use stacks_common::util::hash::Hash160;

    #[test]
    fn test_decode_bug() {
        let input = b"808000000004001dc27eba0247f8cc9575e7d45e50a0bc7e72427d000000000000001d000000000000000000011dc72b6dfd9b36e414a2709e3b01eb5bbdd158f9bc77cd2ca6c3c8b0c803613e2189f6dacf709b34e8182e99d3a1af15812b75e59357d9c255c772695998665f010200000000076f2ff2c4517ab683bf2d588727f09603cc3e9328b9c500e21a939ead57c0560af8a3a132bd7d56566f2ff2c4517ab683bf2d588727f09603cc3e932828dcefb98f6b221eef731cabec7538314441c1e0ff06b44c22085d41aae447c1000000010014ff3cb19986645fd7e71282ad9fea07d540a60e";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = deserialize_transaction(&mut cursor);
        assert!(tx.is_ok());
        assert_eq!(cursor.position() as usize, bytes_len);
    }

    #[test]
    fn test_post_condition_originator_stx_sent_eq() {
        let input = b"80800000000400143e543243dfcd8c02a12ad7ea371bd07bc91df90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003030000000100010100000000000003e801047465737400000009286f6b207472756529";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = deserialize_transaction(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, bytes_len);
        assert_eq!(
            tx.post_condition_mode,
            TransactionPostConditionMode::Originator
        );
        assert_eq!(tx.post_conditions.len(), 1);
        assert_eq!(
            tx.post_conditions[0],
            TransactionPostCondition::STX(
                PostConditionPrincipal::Origin,
                FungibleConditionCode::SentEq,
                1000
            )
        );
    }

    #[test]
    fn test_post_condition_originator_ft_sent_ge() {
        let input = b"80800000000400143e543243dfcd8c02a12ad7ea371bd07bc91df900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030300000001010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0d746573742d636f6e747261637408746573742d6e667403000000000000138801047465737400000009286f6b207472756529";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = deserialize_transaction(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, bytes_len);
        assert_eq!(
            tx.post_condition_mode,
            TransactionPostConditionMode::Originator
        );
        assert_eq!(tx.post_conditions.len(), 1);
        assert_eq!(
            tx.post_conditions[0],
            TransactionPostCondition::Fungible(
                PostConditionPrincipal::Origin,
                AssetInfo {
                    contract_address: UpstreamStacksAddress::new(1, Hash160([0xaa; 20])).unwrap(),
                    contract_name: ContractName::from_literal("test-contract"),
                    asset_name: ClarityName::from_literal("test-nft"),
                },
                FungibleConditionCode::SentGe,
                5000
            )
        );
    }

    #[test]
    fn test_post_condition_originator_nft_maybe_sent() {
        let input = b"80800000000400143e543243dfcd8c02a12ad7ea371bd07bc91df900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030300000001020101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0d746573742d636f6e747261637408746573742d6e667401000000000000000000000000000000011201047465737400000009286f6b207472756529";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = deserialize_transaction(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, bytes_len);
        assert_eq!(
            tx.post_condition_mode,
            TransactionPostConditionMode::Originator
        );
        assert_eq!(tx.post_conditions.len(), 1);
        assert_eq!(
            tx.post_conditions[0],
            TransactionPostCondition::Nonfungible(
                PostConditionPrincipal::Origin,
                AssetInfo {
                    contract_address: UpstreamStacksAddress::new(1, Hash160([0xaa; 20])).unwrap(),
                    contract_name: ContractName::from_literal("test-contract"),
                    asset_name: ClarityName::from_literal("test-nft"),
                },
                UpstreamValue::UInt(1),
                NonfungibleConditionCode::MaybeSent,
            )
        );
    }

    #[test]
    fn test_post_condition_deny_nft_maybe_sent() {
        let input = b"80800000000400143e543243dfcd8c02a12ad7ea371bd07bc91df900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030200000001020101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0d746573742d636f6e747261637408746573742d6e667401000000000000000000000000000000011201047465737400000009286f6b207472756529";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = deserialize_transaction(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, bytes_len);
        assert_eq!(tx.post_condition_mode, TransactionPostConditionMode::Deny);
        assert_eq!(tx.post_conditions.len(), 1);
        assert_eq!(
            tx.post_conditions[0],
            TransactionPostCondition::Nonfungible(
                PostConditionPrincipal::Origin,
                AssetInfo {
                    contract_address: UpstreamStacksAddress::new(1, Hash160([0xaa; 20])).unwrap(),
                    contract_name: ContractName::from_literal("test-contract"),
                    asset_name: ClarityName::from_literal("test-nft"),
                },
                UpstreamValue::UInt(1),
                NonfungibleConditionCode::MaybeSent,
            )
        );
    }

    #[test]
    fn test_post_condition_originator_multiple() {
        let input = b"80800000000400143e543243dfcd8c02a12ad7ea371bd07bc91df90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003030000000200010500000000000007d0020101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0d746573742d636f6e747261637408746573742d6e6674010000000000000000000000000000002a1201047465737400000009286f6b207472756529";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = deserialize_transaction(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, bytes_len);
        assert_eq!(
            tx.post_condition_mode,
            TransactionPostConditionMode::Originator
        );
        assert_eq!(tx.post_conditions.len(), 2);
    }
}
