//! Neon (JS) serialization for upstream `SignerMessage` types.
//!
//! Impls are written against `Encode<'_, UpstreamX>` wrappers so the orphan
//! rule is satisfied for the upstream `libsigner` / `stacks-common` types.
//!
//! The JS shape is a discriminated union keyed by `type_id` (the wire
//! `SignerMessageTypePrefix` byte) plus a human-readable `type_name`. Each
//! decoded variant nests its data under a variant-named key; the recognized-
//! but-unsupported variants (`Mock*`, `StateMachineUpdate`) include only the
//! discriminant fields plus `unsupported: true`.

use blockstack_lib::net::api::postblock_proposal::ValidateRejectCode;
use libsigner::v0::messages::{
    BlockAccepted, BlockRejection, BlockResponse, BlockResponseData, RejectCode, RejectReason,
    SignerMessage, SignerMessageMetadata,
};
use libsigner::{BlockProposal, BlockProposalData};
use neon::prelude::*;
use stacks_common::types::{MinerDiagnosticData, MiningReason};

use crate::util::hex::encode_hex;
use crate::util::neon::{Encode, NeonJsSerialize};

/// The `SignerMessageTypePrefix` wire byte for each variant, paired with a
/// stable snake_case name for JS consumers.
fn signer_message_type(message: &SignerMessage) -> (u8, &'static str) {
    match message {
        SignerMessage::BlockProposal(_) => (0, "block_proposal"),
        SignerMessage::BlockResponse(_) => (1, "block_response"),
        SignerMessage::BlockPushed(_) => (2, "block_pushed"),
        SignerMessage::MockProposal(_) => (3, "mock_proposal"),
        SignerMessage::MockSignature(_) => (4, "mock_signature"),
        SignerMessage::MockBlock(_) => (5, "mock_block"),
        SignerMessage::StateMachineUpdate(_) => (6, "state_machine_update"),
        SignerMessage::BlockPreCommit(_) => (7, "block_pre_commit"),
    }
}

impl NeonJsSerialize for Encode<'_, SignerMessage> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let (type_id, type_name) = signer_message_type(self.0);
        let type_id_num = cx.number(type_id);
        obj.set(cx, "type_id", type_id_num)?;
        let type_name_str = cx.string(type_name);
        obj.set(cx, "type_name", type_name_str)?;

        match self.0 {
            SignerMessage::BlockProposal(block_proposal) => {
                let proposal_obj = cx.empty_object();
                Encode(block_proposal).neon_js_serialize(cx, &proposal_obj, &())?;
                obj.set(cx, "block_proposal", proposal_obj)?;
            }
            SignerMessage::BlockResponse(block_response) => {
                let response_obj = cx.empty_object();
                Encode(block_response).neon_js_serialize(cx, &response_obj, &())?;
                obj.set(cx, "block_response", response_obj)?;
            }
            SignerMessage::BlockPushed(block) => {
                let block_obj = cx.empty_object();
                let block_id = cx.string(encode_hex(&block.header.block_id().0));
                block_obj.set(cx, "block_id", block_id)?;
                Encode(block).neon_js_serialize(cx, &block_obj, &())?;
                obj.set(cx, "block_pushed", block_obj)?;
            }
            SignerMessage::BlockPreCommit(signer_signature_hash) => {
                let precommit_obj = cx.empty_object();
                let hash = cx.string(encode_hex(&signer_signature_hash.0));
                precommit_obj.set(cx, "signer_signature_hash", hash)?;
                obj.set(cx, "block_pre_commit", precommit_obj)?;
            }
            // Recognized but out of scope: surface the discriminant + `unsupported: true`.
            SignerMessage::MockProposal(_)
            | SignerMessage::MockSignature(_)
            | SignerMessage::MockBlock(_)
            | SignerMessage::StateMachineUpdate(_) => {
                let unsupported = cx.boolean(true);
                obj.set(cx, "unsupported", unsupported)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, BlockProposal> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let proposal = self.0;

        let block_obj = cx.empty_object();
        let block_id = cx.string(encode_hex(&proposal.block.header.block_id().0));
        block_obj.set(cx, "block_id", block_id)?;
        Encode(&proposal.block).neon_js_serialize(cx, &block_obj, &())?;
        obj.set(cx, "block", block_obj)?;

        let burn_height = cx.string(proposal.burn_height.to_string());
        obj.set(cx, "burn_height", burn_height)?;

        let reward_cycle = cx.string(proposal.reward_cycle.to_string());
        obj.set(cx, "reward_cycle", reward_cycle)?;

        let data_obj = cx.empty_object();
        Encode(&proposal.block_proposal_data).neon_js_serialize(cx, &data_obj, &())?;
        obj.set(cx, "block_proposal_data", data_obj)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, BlockProposalData> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let data = self.0;

        let version = cx.number(data.version);
        obj.set(cx, "version", version)?;

        let server_version = cx.string(&data.server_version);
        obj.set(cx, "server_version", server_version)?;

        match &data.miner_diagnostic_data {
            Some(diag) => {
                let diag_obj = cx.empty_object();
                Encode(diag).neon_js_serialize(cx, &diag_obj, &())?;
                obj.set(cx, "miner_diagnostic_data", diag_obj)?;
            }
            None => {
                let null_val = cx.null();
                obj.set(cx, "miner_diagnostic_data", null_val)?;
            }
        }

        let unknown_bytes = cx.string(encode_hex(&data.unknown_bytes));
        obj.set(cx, "unknown_bytes", unknown_bytes)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, MinerDiagnosticData> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let diag = self.0;

        let tip_height = cx.string(diag.burnchain_tip_height.to_string());
        obj.set(cx, "burnchain_tip_height", tip_height)?;

        let tip_ch = cx.string(encode_hex(&diag.burnchain_tip_consensus_hash.0));
        obj.set(cx, "burnchain_tip_consensus_hash", tip_ch)?;

        let tip_hh = cx.string(encode_hex(&diag.burnchain_tip_header_hash.0));
        obj.set(cx, "burnchain_tip_header_hash", tip_hh)?;

        let extend_ts = cx.string(diag.tenure_extend_time_stamp.to_string());
        obj.set(cx, "tenure_extend_time_stamp", extend_ts)?;

        let read_count_ts = cx.string(diag.read_count_extend_timestamp.to_string());
        obj.set(cx, "read_count_extend_timestamp", read_count_ts)?;

        let (reason_id, reason_name) = mining_reason(&diag.mining_reason);
        let reason_id_num = cx.number(reason_id);
        obj.set(cx, "mining_reason_id", reason_id_num)?;
        let reason_name_str = cx.string(reason_name);
        obj.set(cx, "mining_reason_name", reason_name_str)?;
        Ok(())
    }
}

fn mining_reason(reason: &MiningReason) -> (u8, &'static str) {
    match reason {
        MiningReason::BlockFound => (0, "block_found"),
        MiningReason::Extended => (1, "extended"),
        MiningReason::ReadCountExtend => (2, "read_count_extend"),
    }
}

impl NeonJsSerialize for Encode<'_, BlockResponse> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match self.0 {
            BlockResponse::Accepted(accepted) => {
                let response_type = cx.string("accepted");
                obj.set(cx, "response_type", response_type)?;
                Encode(accepted).neon_js_serialize(cx, obj, &())?;
            }
            BlockResponse::Rejected(rejection) => {
                let response_type = cx.string("rejected");
                obj.set(cx, "response_type", response_type)?;
                Encode(rejection).neon_js_serialize(cx, obj, &())?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, BlockAccepted> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let accepted = self.0;

        let hash = cx.string(encode_hex(&accepted.signer_signature_hash.0));
        obj.set(cx, "signer_signature_hash", hash)?;

        let signature = cx.string(encode_hex(&accepted.signature.0));
        obj.set(cx, "signature", signature)?;

        let metadata_obj = cx.empty_object();
        Encode(&accepted.metadata).neon_js_serialize(cx, &metadata_obj, &())?;
        obj.set(cx, "metadata", metadata_obj)?;

        let response_data_obj = cx.empty_object();
        Encode(&accepted.response_data).neon_js_serialize(cx, &response_data_obj, &())?;
        obj.set(cx, "response_data", response_data_obj)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, BlockRejection> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let rejection = self.0;

        let reason = cx.string(&rejection.reason);
        obj.set(cx, "reason", reason)?;

        let reason_code_obj = cx.empty_object();
        Encode(&rejection.reason_code).neon_js_serialize(cx, &reason_code_obj, &())?;
        obj.set(cx, "reason_code", reason_code_obj)?;

        let hash = cx.string(encode_hex(&rejection.signer_signature_hash.0));
        obj.set(cx, "signer_signature_hash", hash)?;

        let signature = cx.string(encode_hex(&rejection.signature.0));
        obj.set(cx, "signature", signature)?;

        let chain_id = cx.number(rejection.chain_id);
        obj.set(cx, "chain_id", chain_id)?;

        let metadata_obj = cx.empty_object();
        Encode(&rejection.metadata).neon_js_serialize(cx, &metadata_obj, &())?;
        obj.set(cx, "metadata", metadata_obj)?;

        let response_data_obj = cx.empty_object();
        Encode(&rejection.response_data).neon_js_serialize(cx, &response_data_obj, &())?;
        obj.set(cx, "response_data", response_data_obj)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, SignerMessageMetadata> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let server_version = cx.string(&self.0.server_version);
        obj.set(cx, "server_version", server_version)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, BlockResponseData> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let data = self.0;

        let version = cx.number(data.version);
        obj.set(cx, "version", version)?;

        let extend_ts = cx.string(data.tenure_extend_timestamp.to_string());
        obj.set(cx, "tenure_extend_timestamp", extend_ts)?;

        let reject_reason_obj = cx.empty_object();
        Encode(&data.reject_reason).neon_js_serialize(cx, &reject_reason_obj, &())?;
        obj.set(cx, "reject_reason", reject_reason_obj)?;

        let read_count_ts = cx.string(data.tenure_extend_read_count_timestamp.to_string());
        obj.set(cx, "tenure_extend_read_count_timestamp", read_count_ts)?;

        match &data.failed_txid {
            Some(txid) => {
                let txid_str = cx.string(encode_hex(&txid.0));
                obj.set(cx, "failed_txid", txid_str)?;
            }
            None => {
                let null_val = cx.null();
                obj.set(cx, "failed_txid", null_val)?;
            }
        }

        let unknown_bytes = cx.string(encode_hex(&data.unknown_bytes));
        obj.set(cx, "unknown_bytes", unknown_bytes)?;
        Ok(())
    }
}

/// The `ValidateRejectCode` wire byte + a stable snake_case name.
fn validate_reject_code(code: &ValidateRejectCode) -> (u8, &'static str) {
    match code {
        ValidateRejectCode::BadBlockHash => (0, "bad_block_hash"),
        ValidateRejectCode::BadTransaction => (1, "bad_transaction"),
        ValidateRejectCode::InvalidBlock => (2, "invalid_block"),
        ValidateRejectCode::ChainstateError => (3, "chainstate_error"),
        ValidateRejectCode::UnknownParent => (4, "unknown_parent"),
        ValidateRejectCode::NonCanonicalTenure => (5, "non_canonical_tenure"),
        ValidateRejectCode::NoSuchTenure => (6, "no_such_tenure"),
        ValidateRejectCode::InvalidTransactionReplay => (7, "invalid_transaction_replay"),
        ValidateRejectCode::InvalidParentBlock => (8, "invalid_parent_block"),
        ValidateRejectCode::InvalidTimestamp => (9, "invalid_timestamp"),
        ValidateRejectCode::NetworkChainMismatch => (10, "network_chain_mismatch"),
        ValidateRejectCode::NotFoundError => (11, "not_found_error"),
        ValidateRejectCode::ProblematicTransaction => (12, "problematic_transaction"),
    }
}

fn set_validate_reject_code(
    cx: &mut FunctionContext,
    obj: &Handle<JsObject>,
    code: &ValidateRejectCode,
) -> NeonResult<()> {
    let (id, name) = validate_reject_code(code);
    let id_num = cx.number(id);
    obj.set(cx, "validate_reject_code", id_num)?;
    let name_str = cx.string(name);
    obj.set(cx, "validate_reject_code_name", name_str)?;
    Ok(())
}

impl NeonJsSerialize for Encode<'_, RejectCode> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let name = match self.0 {
            RejectCode::ValidationFailed(code) => {
                set_validate_reject_code(cx, obj, code)?;
                "validation_failed"
            }
            RejectCode::NoSortitionView => "no_sortition_view",
            RejectCode::ConnectivityIssues(msg) => {
                let msg_str = cx.string(msg);
                obj.set(cx, "message", msg_str)?;
                "connectivity_issues"
            }
            RejectCode::RejectedInPriorRound => "rejected_in_prior_round",
            RejectCode::SortitionViewMismatch => "sortition_view_mismatch",
            RejectCode::TestingDirective => "testing_directive",
        };
        let name_str = cx.string(name);
        obj.set(cx, "reject_code_name", name_str)?;
        Ok(())
    }
}

impl NeonJsSerialize for Encode<'_, RejectReason> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let name = match self.0 {
            RejectReason::ValidationFailed(code) => {
                set_validate_reject_code(cx, obj, code)?;
                "validation_failed"
            }
            RejectReason::NoSortitionView => "no_sortition_view",
            RejectReason::ConnectivityIssues(msg) => {
                let msg_str = cx.string(msg);
                obj.set(cx, "message", msg_str)?;
                "connectivity_issues"
            }
            RejectReason::RejectedInPriorRound => "rejected_in_prior_round",
            RejectReason::SortitionViewMismatch => "sortition_view_mismatch",
            RejectReason::TestingDirective => "testing_directive",
            RejectReason::ReorgNotAllowed => "reorg_not_allowed",
            RejectReason::InvalidBitvec => "invalid_bitvec",
            RejectReason::PubkeyHashMismatch => "pubkey_hash_mismatch",
            RejectReason::InvalidMiner => "invalid_miner",
            RejectReason::NotLatestSortitionWinner => "not_latest_sortition_winner",
            RejectReason::InvalidParentBlock => "invalid_parent_block",
            RejectReason::DuplicateBlockFound => "duplicate_block_found",
            RejectReason::InvalidTenureExtend => "invalid_tenure_extend",
            RejectReason::IrrecoverablePubkeyHash => "irrecoverable_pubkey_hash",
            RejectReason::NoSignerConsensus => "no_signer_consensus",
            RejectReason::ProblematicTransactions => "problematic_transactions",
            RejectReason::ConsensusHashMismatch { expected, actual } => {
                let expected_str = cx.string(encode_hex(&expected.0));
                obj.set(cx, "expected_consensus_hash", expected_str)?;
                let actual_str = cx.string(encode_hex(&actual.0));
                obj.set(cx, "actual_consensus_hash", actual_str)?;
                "consensus_hash_mismatch"
            }
            RejectReason::NotRejected => "not_rejected",
            RejectReason::Unknown(code) => {
                let code_num = cx.number(*code);
                obj.set(cx, "unknown_code", code_num)?;
                "unknown"
            }
        };
        let name_str = cx.string(name);
        obj.set(cx, "reject_reason_name", name_str)?;
        Ok(())
    }
}
