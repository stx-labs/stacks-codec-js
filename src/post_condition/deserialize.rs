//! Transaction post-condition deserialization.
//!
//! The wire-format parser is now delegated to upstream's canonical
//! `<TransactionPostCondition as StacksMessageCodec>::consensus_deserialize`
//! implementation in `stackslib`. This module keeps the local enum / struct
//! definitions because the Neon encoder operates on them directly, and converts
//! the upstream value tree to the local one at the boundary.
//!
//! The `StacksAddress::deserialize` impl is retained here because the
//! not-yet-migrated `stacks_tx` module still calls it. It will move once that
//! module is migrated.

use byteorder::ReadBytesExt;
use std::convert::TryFrom;
use std::io::{Cursor, Read};

use blockstack_lib::chainstate::stacks::{
    AssetInfo as UpstreamAssetInfo, FungibleConditionCode as UpstreamFungibleConditionCode,
    NonfungibleConditionCode as UpstreamNonfungibleConditionCode,
    PostConditionPrincipal as UpstreamPostConditionPrincipal,
    TransactionPostCondition as UpstreamTransactionPostCondition,
};
use stacks_common::codec::StacksMessageCodec;
use stacks_common::types::chainstate::StacksAddress as UpstreamStacksAddress;

use crate::clarity_value::deserialize::convert_clarity_value;
use crate::clarity_value::types::{ClarityName, ClarityValue};
use crate::{address::stacks_address::StacksAddress, serialize_util::DeserializeError};

#[derive(Debug, PartialEq)]
pub enum TransactionPostCondition {
    STX(PostConditionPrincipal, FungibleConditionCode, u64),
    Fungible(
        PostConditionPrincipal,
        AssetInfo,
        FungibleConditionCode,
        u64,
    ),
    Nonfungible(
        PostConditionPrincipal,
        AssetInfo,
        ClarityValue,
        NonfungibleConditionCode,
    ),
}

#[derive(Debug, PartialEq)]
pub enum PostConditionPrincipal {
    Origin,
    Standard(StacksAddress),
    Contract(StacksAddress, ClarityName),
}

#[repr(u8)]
pub enum PostConditionPrincipalID {
    Origin = 0x01,
    Standard = 0x02,
    Contract = 0x03,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FungibleConditionCode {
    SentEq = 0x01,
    SentGt = 0x02,
    SentGe = 0x03,
    SentLt = 0x04,
    SentLe = 0x05,
}

impl TryFrom<u8> for FungibleConditionCode {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x01 => Ok(FungibleConditionCode::SentEq),
            0x02 => Ok(FungibleConditionCode::SentGt),
            0x03 => Ok(FungibleConditionCode::SentGe),
            0x04 => Ok(FungibleConditionCode::SentLt),
            0x05 => Ok(FungibleConditionCode::SentLe),
            _ => Err(()),
        }
    }
}

impl From<UpstreamFungibleConditionCode> for FungibleConditionCode {
    fn from(v: UpstreamFungibleConditionCode) -> Self {
        match v {
            UpstreamFungibleConditionCode::SentEq => FungibleConditionCode::SentEq,
            UpstreamFungibleConditionCode::SentGt => FungibleConditionCode::SentGt,
            UpstreamFungibleConditionCode::SentGe => FungibleConditionCode::SentGe,
            UpstreamFungibleConditionCode::SentLt => FungibleConditionCode::SentLt,
            UpstreamFungibleConditionCode::SentLe => FungibleConditionCode::SentLe,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NonfungibleConditionCode {
    Sent = 0x10,
    NotSent = 0x11,
    /** `MaybeSent` — The NFT may or may not be sent; always passes (SIP-040) */
    MaybeSent = 0x12,
}

impl TryFrom<u8> for NonfungibleConditionCode {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x10 => Ok(NonfungibleConditionCode::Sent),
            0x11 => Ok(NonfungibleConditionCode::NotSent),
            0x12 => Ok(NonfungibleConditionCode::MaybeSent),
            _ => Err(()),
        }
    }
}

impl From<UpstreamNonfungibleConditionCode> for NonfungibleConditionCode {
    fn from(v: UpstreamNonfungibleConditionCode) -> Self {
        match v {
            UpstreamNonfungibleConditionCode::Sent => NonfungibleConditionCode::Sent,
            UpstreamNonfungibleConditionCode::NotSent => NonfungibleConditionCode::NotSent,
            UpstreamNonfungibleConditionCode::MaybeSent => NonfungibleConditionCode::MaybeSent,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AssetInfo {
    pub contract_address: StacksAddress,
    pub contract_name: ClarityName,
    pub asset_name: ClarityName,
}

#[repr(u8)]
pub enum AssetInfoID {
    STX = 0,
    FungibleAsset = 1,
    NonfungibleAsset = 2,
}

impl TransactionPostCondition {
    /// Deserialize a single post-condition entry from the wire format.
    ///
    /// Delegates to upstream's canonical
    /// [`StacksMessageCodec::consensus_deserialize`] for
    /// `TransactionPostCondition` and adapts the resulting value tree into the
    /// local enums so the existing Neon encoder doesn't need to change.
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let upstream =
            <UpstreamTransactionPostCondition as StacksMessageCodec>::consensus_deserialize(fd)
                .map_err(|e| {
                    DeserializeError::from(format!("Failed to decode post-condition: {}", e))
                })?;
        Ok(convert_post_condition(&upstream))
    }
}

impl StacksAddress {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let version: u8 = fd.read_u8()?;
        let mut hash160 = [0u8; 20];
        fd.read_exact(&mut hash160)?;
        Ok(StacksAddress {
            version: version,
            hash160_bytes: hash160,
        })
    }
}

fn convert_post_condition(upstream: &UpstreamTransactionPostCondition) -> TransactionPostCondition {
    match upstream {
        UpstreamTransactionPostCondition::STX(principal, code, amount) => {
            TransactionPostCondition::STX(
                convert_principal(principal),
                FungibleConditionCode::from(*code),
                *amount,
            )
        }
        UpstreamTransactionPostCondition::Fungible(principal, asset, code, amount) => {
            TransactionPostCondition::Fungible(
                convert_principal(principal),
                convert_asset_info(asset),
                FungibleConditionCode::from(*code),
                *amount,
            )
        }
        UpstreamTransactionPostCondition::Nonfungible(principal, asset, value, code) => {
            // The neon encoder unwraps `serialized_bytes.as_ref().unwrap()` for
            // the asset value, so we must capture the canonical hex form here.
            TransactionPostCondition::Nonfungible(
                convert_principal(principal),
                convert_asset_info(asset),
                convert_clarity_value(value, true),
                NonfungibleConditionCode::from(*code),
            )
        }
    }
}

fn convert_principal(upstream: &UpstreamPostConditionPrincipal) -> PostConditionPrincipal {
    match upstream {
        UpstreamPostConditionPrincipal::Origin => PostConditionPrincipal::Origin,
        UpstreamPostConditionPrincipal::Standard(addr) => {
            PostConditionPrincipal::Standard(convert_address(addr))
        }
        UpstreamPostConditionPrincipal::Contract(addr, contract_name) => {
            PostConditionPrincipal::Contract(
                convert_address(addr),
                ClarityName(contract_name.to_string()),
            )
        }
    }
}

fn convert_asset_info(upstream: &UpstreamAssetInfo) -> AssetInfo {
    AssetInfo {
        contract_address: convert_address(&upstream.contract_address),
        contract_name: ClarityName(upstream.contract_name.to_string()),
        asset_name: ClarityName(upstream.asset_name.to_string()),
    }
}

fn convert_address(upstream: &UpstreamStacksAddress) -> StacksAddress {
    StacksAddress {
        version: upstream.version(),
        hash160_bytes: upstream.bytes().0,
    }
}

