//! Stacks transaction deserialization.
//!
//! The wire-format parser is delegated to upstream's canonical
//! `<StacksTransaction as StacksMessageCodec>::consensus_deserialize` in
//! `stackslib`. This module keeps the local enum / struct definitions because
//! the Neon encoder operates on them directly, and converts the upstream value
//! tree into the local one at the boundary.

use std::io::Cursor;

use blockstack_lib::chainstate::stacks::{
    CoinbasePayload as UpstreamCoinbasePayload,
    MultisigSpendingCondition as UpstreamMultisigSpendingCondition,
    OrderIndependentMultisigSpendingCondition as UpstreamOrderIndependentMultisigSpendingCondition,
    SinglesigHashMode as UpstreamSinglesigHashMode,
    SinglesigSpendingCondition as UpstreamSinglesigSpendingCondition,
    StacksMicroblockHeader as UpstreamStacksMicroblockHeader,
    StacksTransaction as UpstreamStacksTransaction, TenureChangeCause as UpstreamTenureChangeCause,
    TenureChangePayload as UpstreamTenureChangePayload, TransactionAnchorMode as UpstreamAnchorMode,
    TransactionAuth as UpstreamTransactionAuth, TransactionAuthField as UpstreamTransactionAuthField,
    TransactionContractCall as UpstreamTransactionContractCall,
    TransactionPayload as UpstreamTransactionPayload,
    TransactionPostConditionMode as UpstreamPostConditionMode,
    TransactionPublicKeyEncoding as UpstreamPublicKeyEncoding,
    TransactionSmartContract as UpstreamTransactionSmartContract,
    TransactionSpendingCondition as UpstreamTransactionSpendingCondition,
    TransactionVersion as UpstreamTransactionVersion,
};
use blockstack_lib::util_lib::strings::StacksString as UpstreamStacksString;
use clarity::vm::types::PrincipalData as UpstreamPrincipalData;
use clarity::vm::ClarityVersion as UpstreamClarityVersion;
use stacks_common::codec::StacksMessageCodec;
use stacks_common::types::chainstate::StacksAddress as UpstreamStacksAddress;

use crate::address::stacks_address::StacksAddress;
use crate::clarity_value::deserialize::convert_clarity_value;
use crate::clarity_value::types::{ClarityName, ClarityValue};
use crate::post_condition::deserialize::{
    convert_post_condition, TransactionPostCondition,
};
use crate::serialize_util::DeserializeError;

// ===== Local types (kept verbatim — the Neon encoder operates on these) =====

pub struct StacksTransaction {
    pub version: TransactionVersion,
    pub chain_id: u32,
    pub auth: TransactionAuth,
    pub anchor_mode: TransactionAnchorMode,
    pub post_conditions_serialized: Vec<u8>,
    pub post_condition_mode: TransactionPostConditionMode,
    pub post_conditions: Vec<TransactionPostCondition>,
    pub payload: TransactionPayload,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TransactionVersion {
    Mainnet = 0x00,
    Testnet = 0x80,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum TransactionAnchorMode {
    OnChainOnly = 1,  // must be included in a StacksBlock
    OffChainOnly = 2, // must be included in a StacksMicroBlock
    Any = 3,          // either
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TransactionPostConditionMode {
    Allow = 0x01,      // allow any other changes not specified
    Deny = 0x02,       // deny any other changes not specified
    Originator = 0x03, // deny for the transaction origin, allow for everyone else (SIP-040)
}

#[repr(u8)]
pub enum TransactionAuthFlags {
    AuthStandard = 0x04,
    AuthSponsored = 0x05,
}

pub enum TransactionAuth {
    Standard(TransactionSpendingCondition),
    Sponsored(TransactionSpendingCondition, TransactionSpendingCondition),
}

pub enum TransactionSpendingCondition {
    Singlesig(SinglesigSpendingCondition),
    Multisig(MultisigSpendingCondition),
}

pub struct MultisigSpendingCondition {
    pub hash_mode: MultisigHashMode,
    pub signer: [u8; 20],
    pub nonce: u64,
    pub tx_fee: u64,
    pub fields: Vec<TransactionAuthField>,
    pub signatures_required: u16,
}

pub struct SinglesigSpendingCondition {
    pub hash_mode: SinglesigHashMode,
    pub signer: [u8; 20],
    pub nonce: u64,
    pub tx_fee: u64,
    pub key_encoding: TransactionPublicKeyEncoding,
    pub signature: MessageSignature,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum MultisigHashMode {
    P2SH = 0x01,
    P2SHNonSequential = 0x05,
    P2WSH = 0x03,
    P2WSHNonSequential = 0x07,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum SinglesigHashMode {
    P2PKH = 0x00,
    P2WPKH = 0x02,
}

impl SinglesigHashMode {
    pub fn from_u8(n: u8) -> Option<SinglesigHashMode> {
        match n {
            x if x == SinglesigHashMode::P2PKH as u8 => Some(SinglesigHashMode::P2PKH),
            x if x == SinglesigHashMode::P2WPKH as u8 => Some(SinglesigHashMode::P2WPKH),
            _ => None,
        }
    }
}

impl MultisigHashMode {
    pub fn from_u8(n: u8) -> Option<MultisigHashMode> {
        match n {
            x if x == MultisigHashMode::P2SH as u8 => Some(MultisigHashMode::P2SH),
            x if x == MultisigHashMode::P2SHNonSequential as u8 => {
                Some(MultisigHashMode::P2SHNonSequential)
            }
            x if x == MultisigHashMode::P2WSH as u8 => Some(MultisigHashMode::P2WSH),
            x if x == MultisigHashMode::P2WSHNonSequential as u8 => {
                Some(MultisigHashMode::P2WSHNonSequential)
            }
            _ => None,
        }
    }
}

pub struct StacksPublicKeyBuffer(pub [u8; 33]);

pub struct MessageSignature(pub [u8; 65]);

pub struct Secp256k1PublicKey {
    pub key: StacksPublicKeyBuffer,
    pub compressed: bool,
}

pub enum TransactionAuthField {
    PublicKey(Secp256k1PublicKey),
    Signature(TransactionPublicKeyEncoding, MessageSignature),
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum TransactionAuthFieldID {
    PublicKeyCompressed = 0x00,
    PublicKeyUncompressed = 0x01,
    SignatureCompressed = 0x02,
    SignatureUncompressed = 0x03,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum TransactionPublicKeyEncoding {
    Compressed = 0x00,
    Uncompressed = 0x01,
}

impl TransactionPublicKeyEncoding {
    pub fn from_u8(n: u8) -> Option<TransactionPublicKeyEncoding> {
        match n {
            x if x == TransactionPublicKeyEncoding::Compressed as u8 => {
                Some(TransactionPublicKeyEncoding::Compressed)
            }
            x if x == TransactionPublicKeyEncoding::Uncompressed as u8 => {
                Some(TransactionPublicKeyEncoding::Uncompressed)
            }
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum ClarityVersion {
    Clarity1 = 1,
    Clarity2 = 2,
    Clarity3 = 3,
    Clarity4 = 4,
    Clarity5 = 5,
    /// Locally reserved for future upstream releases. The wire-format decoder
    /// here can never produce this variant today because upstream's
    /// `clarity::vm::ClarityVersion` only goes up to `Clarity5`; it will be
    /// reachable automatically once upstream adds a `Clarity6` variant and we
    /// bump the pinned `stacks-core` SHA.
    Clarity6 = 6,
}

impl ClarityVersion {
    pub fn from_u8(n: u8) -> Option<ClarityVersion> {
        match n {
            x if x == ClarityVersion::Clarity1 as u8 => Some(ClarityVersion::Clarity1),
            x if x == ClarityVersion::Clarity2 as u8 => Some(ClarityVersion::Clarity2),
            x if x == ClarityVersion::Clarity3 as u8 => Some(ClarityVersion::Clarity3),
            x if x == ClarityVersion::Clarity4 as u8 => Some(ClarityVersion::Clarity4),
            x if x == ClarityVersion::Clarity5 as u8 => Some(ClarityVersion::Clarity5),
            x if x == ClarityVersion::Clarity6 as u8 => Some(ClarityVersion::Clarity6),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum TransactionPayloadID {
    TokenTransfer = 0,
    SmartContract = 1,
    ContractCall = 2,
    PoisonMicroblock = 3,
    Coinbase = 4,
    CoinbaseToAltRecipient = 5,
    VersionedSmartContract = 6,
    TenureChange = 7,
    NakamotoCoinbase = 8,
}

pub enum TransactionPayload {
    TokenTransfer(PrincipalData, u64, TokenTransferMemo),
    ContractCall(TransactionContractCall),
    SmartContract(TransactionSmartContract),
    PoisonMicroblock(StacksMicroblockHeader, StacksMicroblockHeader),
    Coinbase(CoinbasePayload),
    CoinbaseToAltRecipient(CoinbasePayload, PrincipalData),
    VersionedSmartContract(TransactionSmartContract, ClarityVersion),
    TenureChange(TransactionTenureChange),
    NakamotoCoinbase(CoinbasePayload, Option<PrincipalData>, VRFProof),
}

pub struct CoinbasePayload(pub [u8; 32]);

pub struct VRFProof(pub Vec<u8>);

pub struct TransactionTenureChange {
    pub tenure_consensus_hash: [u8; 20],
    pub prev_tenure_consensus_hash: [u8; 20],
    pub burn_view_consensus_hash: [u8; 20],
    pub previous_tenure_end: [u8; 32],
    pub previous_tenure_blocks: u32,
    pub cause: TenureChangeCause,
    pub pubkey_hash: [u8; 20],
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum TenureChangeCause {
    /// A valid winning block-commit
    BlockFound = 0,
    /// The next burnchain block is taking too long, so extend the runtime budget
    Extended = 1,
    /// NEW in SIP-034: extend specific dimensions
    ExtendedRuntime = 2,
    ExtendedReadCount = 3,
    ExtendedReadLength = 4,
    ExtendedWriteCount = 5,
    ExtendedWriteLength = 6,
}

impl TenureChangeCause {
    pub fn from_u8(n: u8) -> Option<TenureChangeCause> {
        match n {
            x if x == TenureChangeCause::BlockFound as u8 => Some(TenureChangeCause::BlockFound),
            x if x == TenureChangeCause::Extended as u8 => Some(TenureChangeCause::Extended),
            x if x == TenureChangeCause::ExtendedRuntime as u8 => {
                Some(TenureChangeCause::ExtendedRuntime)
            }
            x if x == TenureChangeCause::ExtendedReadCount as u8 => {
                Some(TenureChangeCause::ExtendedReadCount)
            }
            x if x == TenureChangeCause::ExtendedReadLength as u8 => {
                Some(TenureChangeCause::ExtendedReadLength)
            }
            x if x == TenureChangeCause::ExtendedWriteCount as u8 => {
                Some(TenureChangeCause::ExtendedWriteCount)
            }
            x if x == TenureChangeCause::ExtendedWriteLength as u8 => {
                Some(TenureChangeCause::ExtendedWriteLength)
            }
            _ => None,
        }
    }
}

pub struct TransactionSmartContract {
    pub name: ClarityName,
    pub code_body: StacksString,
}

pub struct StacksString(pub Vec<u8>);

pub struct BlockHeaderHash(pub [u8; 32]);

pub struct Sha512Trunc256Sum(pub [u8; 32]);

pub struct StacksMicroblockHeader {
    pub version: u8,
    pub sequence: u16,
    pub prev_block: BlockHeaderHash,
    pub tx_merkle_root: Sha512Trunc256Sum,
    pub signature: MessageSignature,
    pub serialized_bytes: Vec<u8>,
}

pub struct TokenTransferMemo(pub [u8; 34]);

pub struct StandardPrincipalData(pub u8, pub [u8; 20]);

pub struct QualifiedContractIdentifier {
    pub issuer: StandardPrincipalData,
    pub name: ClarityName,
}

pub enum PrincipalData {
    Standard(StandardPrincipalData),
    Contract(QualifiedContractIdentifier),
}

pub struct TransactionContractCall {
    pub address: StacksAddress,
    pub contract_name: ClarityName,
    pub function_name: ClarityName,
    pub function_args: Vec<ClarityValue>,
}

// ===== Façade entry point =====

impl StacksTransaction {
    /// Deserialize a Stacks transaction from the wire format.
    ///
    /// Delegates to upstream's canonical
    /// [`StacksMessageCodec::consensus_deserialize`] for `StacksTransaction`
    /// and lowers the result into the local types so the Neon encoder doesn't
    /// need to change.
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let upstream =
            <UpstreamStacksTransaction as StacksMessageCodec>::consensus_deserialize(fd)
                .map_err(|e| {
                    DeserializeError::from(format!("Failed to decode transaction: {}", e))
                })?;
        Ok(convert_transaction(&upstream))
    }
}

// ===== Conversion routines =====

fn convert_transaction(upstream: &UpstreamStacksTransaction) -> StacksTransaction {
    StacksTransaction {
        version: convert_version(upstream.version),
        chain_id: upstream.chain_id,
        auth: convert_auth(&upstream.auth),
        anchor_mode: convert_anchor_mode(upstream.anchor_mode),
        post_conditions_serialized: serialize_post_conditions_section(
            upstream.post_condition_mode,
            &upstream.post_conditions,
        ),
        post_condition_mode: convert_post_condition_mode(upstream.post_condition_mode),
        post_conditions: upstream
            .post_conditions
            .iter()
            .map(convert_post_condition)
            .collect(),
        payload: convert_payload(&upstream.payload),
    }
}

/// Re-serialize the post-conditions section as it appears on the wire:
///
/// `[1 byte mode] [4-byte BE length] [N * encoded post-condition]`
///
/// The local code historically captured this slice via cursor offsets while
/// hand-parsing each post-condition. Clarity / post-condition encoding is
/// canonical and deterministic, so re-serializing produces byte-identical
/// output. This is what the Neon encoder emits as `post_conditions_buffer`.
fn serialize_post_conditions_section(
    mode: UpstreamPostConditionMode,
    post_conditions: &[blockstack_lib::chainstate::stacks::TransactionPostCondition],
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(5 + post_conditions.len() * 32);
    buf.push(mode as u8);
    buf.extend_from_slice(&(post_conditions.len() as u32).to_be_bytes());
    for pc in post_conditions {
        pc.consensus_serialize(&mut buf)
            .expect("BUG: re-serialize of post-condition to Vec failed");
    }
    buf
}

fn convert_version(upstream: UpstreamTransactionVersion) -> TransactionVersion {
    match upstream {
        UpstreamTransactionVersion::Mainnet => TransactionVersion::Mainnet,
        UpstreamTransactionVersion::Testnet => TransactionVersion::Testnet,
    }
}

fn convert_anchor_mode(upstream: UpstreamAnchorMode) -> TransactionAnchorMode {
    match upstream {
        UpstreamAnchorMode::OnChainOnly => TransactionAnchorMode::OnChainOnly,
        UpstreamAnchorMode::OffChainOnly => TransactionAnchorMode::OffChainOnly,
        UpstreamAnchorMode::Any => TransactionAnchorMode::Any,
    }
}

fn convert_post_condition_mode(upstream: UpstreamPostConditionMode) -> TransactionPostConditionMode {
    match upstream {
        UpstreamPostConditionMode::Allow => TransactionPostConditionMode::Allow,
        UpstreamPostConditionMode::Deny => TransactionPostConditionMode::Deny,
        UpstreamPostConditionMode::Originator => TransactionPostConditionMode::Originator,
    }
}

fn convert_auth(upstream: &UpstreamTransactionAuth) -> TransactionAuth {
    match upstream {
        UpstreamTransactionAuth::Standard(origin) => {
            TransactionAuth::Standard(convert_spending_condition(origin))
        }
        UpstreamTransactionAuth::Sponsored(origin, sponsor) => TransactionAuth::Sponsored(
            convert_spending_condition(origin),
            convert_spending_condition(sponsor),
        ),
    }
}

fn convert_spending_condition(
    upstream: &UpstreamTransactionSpendingCondition,
) -> TransactionSpendingCondition {
    match upstream {
        UpstreamTransactionSpendingCondition::Singlesig(s) => {
            TransactionSpendingCondition::Singlesig(convert_singlesig(s))
        }
        UpstreamTransactionSpendingCondition::Multisig(m) => {
            TransactionSpendingCondition::Multisig(convert_multisig(m))
        }
        UpstreamTransactionSpendingCondition::OrderIndependentMultisig(m) => {
            // Upstream split the SIP-040 / non-sequential multisig flavors out
            // into their own variant; locally they live as the
            // `*NonSequential` hash modes inside the regular Multisig variant.
            TransactionSpendingCondition::Multisig(convert_order_independent_multisig(m))
        }
    }
}

fn convert_singlesig(upstream: &UpstreamSinglesigSpendingCondition) -> SinglesigSpendingCondition {
    SinglesigSpendingCondition {
        hash_mode: match upstream.hash_mode {
            UpstreamSinglesigHashMode::P2PKH => SinglesigHashMode::P2PKH,
            UpstreamSinglesigHashMode::P2WPKH => SinglesigHashMode::P2WPKH,
        },
        signer: upstream.signer.0,
        nonce: upstream.nonce,
        tx_fee: upstream.tx_fee,
        key_encoding: convert_pubkey_encoding(upstream.key_encoding),
        signature: MessageSignature(upstream.signature.0),
    }
}

fn convert_multisig(upstream: &UpstreamMultisigSpendingCondition) -> MultisigSpendingCondition {
    use blockstack_lib::chainstate::stacks::MultisigHashMode as UpHM;
    MultisigSpendingCondition {
        hash_mode: match upstream.hash_mode {
            UpHM::P2SH => MultisigHashMode::P2SH,
            UpHM::P2WSH => MultisigHashMode::P2WSH,
        },
        signer: upstream.signer.0,
        nonce: upstream.nonce,
        tx_fee: upstream.tx_fee,
        fields: upstream.fields.iter().map(convert_auth_field).collect(),
        signatures_required: upstream.signatures_required,
    }
}

fn convert_order_independent_multisig(
    upstream: &UpstreamOrderIndependentMultisigSpendingCondition,
) -> MultisigSpendingCondition {
    use blockstack_lib::chainstate::stacks::OrderIndependentMultisigHashMode as UpOIHM;
    MultisigSpendingCondition {
        hash_mode: match upstream.hash_mode {
            UpOIHM::P2SH => MultisigHashMode::P2SHNonSequential,
            UpOIHM::P2WSH => MultisigHashMode::P2WSHNonSequential,
        },
        signer: upstream.signer.0,
        nonce: upstream.nonce,
        tx_fee: upstream.tx_fee,
        fields: upstream.fields.iter().map(convert_auth_field).collect(),
        signatures_required: upstream.signatures_required,
    }
}

fn convert_pubkey_encoding(upstream: UpstreamPublicKeyEncoding) -> TransactionPublicKeyEncoding {
    match upstream {
        UpstreamPublicKeyEncoding::Compressed => TransactionPublicKeyEncoding::Compressed,
        UpstreamPublicKeyEncoding::Uncompressed => TransactionPublicKeyEncoding::Uncompressed,
    }
}

fn convert_auth_field(upstream: &UpstreamTransactionAuthField) -> TransactionAuthField {
    match upstream {
        UpstreamTransactionAuthField::PublicKey(pubk) => {
            // Wire format stores the 33-byte compressed serialization in both
            // the Compressed and Uncompressed cases; the `compressed` flag
            // tells the verifier how the original was framed.
            let compressed_bytes = pubk.to_bytes_compressed();
            let mut key = [0u8; 33];
            key.copy_from_slice(&compressed_bytes);
            TransactionAuthField::PublicKey(Secp256k1PublicKey {
                key: StacksPublicKeyBuffer(key),
                compressed: pubk.compressed(),
            })
        }
        UpstreamTransactionAuthField::Signature(encoding, sig) => TransactionAuthField::Signature(
            convert_pubkey_encoding(*encoding),
            MessageSignature(sig.0),
        ),
    }
}

fn convert_payload(upstream: &UpstreamTransactionPayload) -> TransactionPayload {
    match upstream {
        UpstreamTransactionPayload::TokenTransfer(p, amount, memo) => {
            TransactionPayload::TokenTransfer(
                convert_principal(p),
                *amount,
                TokenTransferMemo(memo.0),
            )
        }
        UpstreamTransactionPayload::ContractCall(cc) => {
            TransactionPayload::ContractCall(convert_contract_call(cc))
        }
        UpstreamTransactionPayload::SmartContract(sc, version_opt) => match version_opt {
            None => TransactionPayload::SmartContract(convert_smart_contract(sc)),
            Some(v) => TransactionPayload::VersionedSmartContract(
                convert_smart_contract(sc),
                convert_clarity_version(*v),
            ),
        },
        UpstreamTransactionPayload::PoisonMicroblock(h1, h2) => {
            TransactionPayload::PoisonMicroblock(
                convert_microblock_header(h1),
                convert_microblock_header(h2),
            )
        }
        UpstreamTransactionPayload::Coinbase(buf, recipient_opt, vrf_opt) => {
            // Upstream collapses the three on-chain coinbase shapes into a single
            // variant and discriminates by which optional fields are populated:
            //
            //   (None,    None)    -> id 4 Coinbase
            //   (Some(_), None)    -> id 5 CoinbaseToAltRecipient
            //   (_,       Some(_)) -> id 8 NakamotoCoinbase
            //
            // We fan it back out into the local enum so the JS-facing `type_id`
            // values stay stable.
            match (recipient_opt, vrf_opt) {
                (None, None) => TransactionPayload::Coinbase(convert_coinbase_payload(buf)),
                (Some(recip), None) => TransactionPayload::CoinbaseToAltRecipient(
                    convert_coinbase_payload(buf),
                    convert_principal(recip),
                ),
                (recip_opt, Some(vrf)) => TransactionPayload::NakamotoCoinbase(
                    convert_coinbase_payload(buf),
                    recip_opt.as_ref().map(convert_principal),
                    convert_vrf_proof(vrf),
                ),
            }
        }
        UpstreamTransactionPayload::TenureChange(tc) => {
            TransactionPayload::TenureChange(convert_tenure_change(tc))
        }
    }
}

fn convert_contract_call(upstream: &UpstreamTransactionContractCall) -> TransactionContractCall {
    TransactionContractCall {
        address: convert_address(&upstream.address),
        // Upstream uses ContractName here (a guarded_string with stricter regex);
        // the local Neon encoder only ever calls `as_str()` on this field, so we
        // can safely round-trip via `to_string()` into the looser local
        // ClarityName without losing information.
        contract_name: ClarityName(upstream.contract_name.to_string()),
        function_name: ClarityName(upstream.function_name.to_string()),
        function_args: upstream
            .function_args
            .iter()
            .map(|v| convert_clarity_value(v, true))
            .collect(),
    }
}

fn convert_smart_contract(upstream: &UpstreamTransactionSmartContract) -> TransactionSmartContract {
    TransactionSmartContract {
        name: ClarityName(upstream.name.to_string()),
        code_body: StacksString(convert_stacks_string(&upstream.code_body)),
    }
}

fn convert_stacks_string(upstream: &UpstreamStacksString) -> Vec<u8> {
    // Upstream's StacksString::Deref<Target=Vec<u8>> exposes the bytes; clone
    // them so the caller owns the buffer.
    upstream.as_slice().to_vec()
}

fn convert_microblock_header(upstream: &UpstreamStacksMicroblockHeader) -> StacksMicroblockHeader {
    let serialized_bytes = <UpstreamStacksMicroblockHeader as StacksMessageCodec>::serialize_to_vec(
        upstream,
    );
    StacksMicroblockHeader {
        version: upstream.version,
        sequence: upstream.sequence,
        prev_block: BlockHeaderHash(upstream.prev_block.0),
        tx_merkle_root: Sha512Trunc256Sum(upstream.tx_merkle_root.0),
        signature: MessageSignature(upstream.signature.0),
        serialized_bytes,
    }
}

fn convert_coinbase_payload(upstream: &UpstreamCoinbasePayload) -> CoinbasePayload {
    CoinbasePayload(upstream.0)
}

fn convert_vrf_proof(upstream: &stacks_common::util::vrf::VRFProof) -> VRFProof {
    VRFProof(upstream.to_bytes().to_vec())
}

fn convert_tenure_change(upstream: &UpstreamTenureChangePayload) -> TransactionTenureChange {
    TransactionTenureChange {
        tenure_consensus_hash: upstream.tenure_consensus_hash.0,
        prev_tenure_consensus_hash: upstream.prev_tenure_consensus_hash.0,
        burn_view_consensus_hash: upstream.burn_view_consensus_hash.0,
        previous_tenure_end: upstream.previous_tenure_end.0,
        previous_tenure_blocks: upstream.previous_tenure_blocks,
        cause: convert_tenure_change_cause(upstream.cause),
        pubkey_hash: upstream.pubkey_hash.0,
    }
}

fn convert_tenure_change_cause(upstream: UpstreamTenureChangeCause) -> TenureChangeCause {
    match upstream {
        UpstreamTenureChangeCause::BlockFound => TenureChangeCause::BlockFound,
        UpstreamTenureChangeCause::Extended => TenureChangeCause::Extended,
        UpstreamTenureChangeCause::ExtendedRuntime => TenureChangeCause::ExtendedRuntime,
        UpstreamTenureChangeCause::ExtendedReadCount => TenureChangeCause::ExtendedReadCount,
        UpstreamTenureChangeCause::ExtendedReadLength => TenureChangeCause::ExtendedReadLength,
        UpstreamTenureChangeCause::ExtendedWriteCount => TenureChangeCause::ExtendedWriteCount,
        UpstreamTenureChangeCause::ExtendedWriteLength => TenureChangeCause::ExtendedWriteLength,
    }
}

fn convert_clarity_version(upstream: UpstreamClarityVersion) -> ClarityVersion {
    match upstream {
        UpstreamClarityVersion::Clarity1 => ClarityVersion::Clarity1,
        UpstreamClarityVersion::Clarity2 => ClarityVersion::Clarity2,
        UpstreamClarityVersion::Clarity3 => ClarityVersion::Clarity3,
        UpstreamClarityVersion::Clarity4 => ClarityVersion::Clarity4,
        UpstreamClarityVersion::Clarity5 => ClarityVersion::Clarity5,
    }
}

fn convert_address(upstream: &UpstreamStacksAddress) -> StacksAddress {
    StacksAddress {
        version: upstream.version(),
        hash160_bytes: upstream.bytes().0,
    }
}

fn convert_principal(upstream: &UpstreamPrincipalData) -> PrincipalData {
    match upstream {
        UpstreamPrincipalData::Standard(s) => {
            PrincipalData::Standard(StandardPrincipalData(s.version(), s.1))
        }
        UpstreamPrincipalData::Contract(qci) => {
            PrincipalData::Contract(QualifiedContractIdentifier {
                issuer: StandardPrincipalData(qci.issuer.version(), qci.issuer.1),
                name: ClarityName(qci.name.to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        clarity_value::types::Value,
        hex::decode_hex,
        post_condition::deserialize::{
            AssetInfo, FungibleConditionCode, NonfungibleConditionCode, PostConditionPrincipal,
        },
    };

    #[test]
    fn test_decode_bug() {
        let input = b"808000000004001dc27eba0247f8cc9575e7d45e50a0bc7e72427d000000000000001d000000000000000000011dc72b6dfd9b36e414a2709e3b01eb5bbdd158f9bc77cd2ca6c3c8b0c803613e2189f6dacf709b34e8182e99d3a1af15812b75e59357d9c255c772695998665f010200000000076f2ff2c4517ab683bf2d588727f09603cc3e9328b9c500e21a939ead57c0560af8a3a132bd7d56566f2ff2c4517ab683bf2d588727f09603cc3e932828dcefb98f6b221eef731cabec7538314441c1e0ff06b44c22085d41aae447c1000000010014ff3cb19986645fd7e71282ad9fea07d540a60e";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = StacksTransaction::deserialize(&mut cursor);
        assert!(tx.is_ok());
        assert_eq!(cursor.position() as usize, bytes_len);
    }

    #[test]
    fn test_post_condition_originator_stx_sent_eq() {
        let input = b"80800000000400143e543243dfcd8c02a12ad7ea371bd07bc91df90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003030000000100010100000000000003e801047465737400000009286f6b207472756529";
        let bytes = decode_hex(input).unwrap();
        let bytes_len = bytes.len();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = StacksTransaction::deserialize(&mut cursor).unwrap();
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
        let tx = StacksTransaction::deserialize(&mut cursor).unwrap();
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
                    contract_address: StacksAddress::new(1, [0xaa; 20]),
                    contract_name: "test-contract".into(),
                    asset_name: "test-nft".into(),
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
        let tx = StacksTransaction::deserialize(&mut cursor).unwrap();
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
                    contract_address: StacksAddress::new(1, [0xaa; 20]),
                    contract_name: "test-contract".into(),
                    asset_name: "test-nft".into(),
                },
                ClarityValue::new_with_bytes(
                    [
                        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x01,
                    ],
                    Value::UInt(1),
                ),
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
        let tx = StacksTransaction::deserialize(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, bytes_len);
        assert_eq!(tx.post_condition_mode, TransactionPostConditionMode::Deny);
        assert_eq!(tx.post_conditions.len(), 1);
        assert_eq!(
            tx.post_conditions[0],
            TransactionPostCondition::Nonfungible(
                PostConditionPrincipal::Origin,
                AssetInfo {
                    contract_address: StacksAddress::new(1, [0xaa; 20]),
                    contract_name: "test-contract".into(),
                    asset_name: "test-nft".into(),
                },
                ClarityValue::new_with_bytes(
                    [
                        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x01,
                    ],
                    Value::UInt(1),
                ),
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
        let tx = StacksTransaction::deserialize(&mut cursor).unwrap();
        assert_eq!(cursor.position() as usize, bytes_len);
        assert_eq!(tx.post_condition_mode, TransactionPostConditionMode::Originator);
        assert_eq!(tx.post_conditions.len(), 2);
    }
}
