//! Decode a printed Clarity tuple from the PoX-5 contract into a typed
//! [`Pox5SyntheticEvent`].
//!
//! PoX-5 events are produced by `(print ...)` calls in the contract; each
//! one is a flat tuple with a `topic: (string-ascii N)` field. The
//! [`decode_pox5_synthetic_event`] entry point sniffs the `topic` value and
//! dispatches to a per-event extractor.

use clarity::vm::types::Value as UpstreamValue;

use super::super::clarity_helpers::{
    clarity_principal_to_string, extract_ascii_string, extract_bool, extract_buffer_hex,
    extract_list, extract_optional_uint, extract_tuple, extract_uint, get_tuple_field, tuple_get,
};
use super::types::*;

/// Decode a PoX-5 print payload into a [`Pox5SyntheticEvent`].
///
/// Returns:
/// - `Ok(None)` if the value is not a PoX-5 event (no `topic` field on the
///   top-level tuple). This lets a parent dispatcher fall back to other
///   decoders (e.g. PoX-4) when the input is something else.
/// - `Ok(Some(_))` if the value is a recognized PoX-5 event.
/// - `Err(_)` if the value claims to be a PoX-5 event (has a `topic`) but
///   the topic is unknown or required fields are missing / malformed.
pub fn decode_pox5_synthetic_event(
    clarity_value: &UpstreamValue,
) -> Result<Option<Pox5SyntheticEvent>, String> {
    let UpstreamValue::Tuple(_) = clarity_value else {
        return Ok(None);
    };
    let tuple = extract_tuple(clarity_value)?;

    // `topic` is what identifies a PoX-5 event. Without it, the input belongs
    // to some other decoder (or none at all).
    let Some(topic_val) = tuple.get("topic") else {
        return Ok(None);
    };
    let topic_str = extract_ascii_string(topic_val)?;

    let name = Pox5EventName::parse(&topic_str).ok_or_else(|| {
        format!(
            "Unrecognized PoX-5 event topic: {} (printed tuple matched the PoX-5 shape but the topic name is unknown)",
            topic_str
        )
    })?;

    let data = match name {
        Pox5EventName::SetBondAdmin => Pox5EventData::SetBondAdmin {
            old_admin: clarity_principal_to_string(get_tuple_field(tuple, "old-admin")?)?,
            new_admin: clarity_principal_to_string(get_tuple_field(tuple, "new-admin")?)?,
        },

        Pox5EventName::SetupBond => Pox5EventData::SetupBond {
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            target_rate: extract_uint(get_tuple_field(tuple, "target-rate")?)?,
            stx_value_ratio: extract_uint(get_tuple_field(tuple, "stx-value-ratio")?)?,
            min_ustx_ratio: extract_uint(get_tuple_field(tuple, "min-ustx-ratio")?)?,
            early_unlock_bytes: extract_buffer_hex(get_tuple_field(tuple, "early-unlock-bytes")?)?,
            first_reward_cycle: extract_uint(get_tuple_field(tuple, "first-reward-cycle")?)?,
            bond_start_height: extract_uint(get_tuple_field(tuple, "bond-start-height")?)?,
            unlock_cycle: extract_uint(get_tuple_field(tuple, "unlock-cycle")?)?,
            unlock_burn_height: extract_uint(get_tuple_field(tuple, "unlock-burn-height")?)?,
        },

        Pox5EventName::AddToAllowlist => Pox5EventData::AddToAllowlist {
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            max_sats: extract_uint(get_tuple_field(tuple, "max-sats")?)?,
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
        },

        Pox5EventName::RegisterForBond => Pox5EventData::RegisterForBond {
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            amount_ustx: extract_uint(get_tuple_field(tuple, "amount-ustx")?)?,
            sats_total: extract_uint(get_tuple_field(tuple, "sats-total")?)?,
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            first_reward_cycle: extract_uint(get_tuple_field(tuple, "first-reward-cycle")?)?,
            unlock_burn_height: extract_uint(get_tuple_field(tuple, "unlock-burn-height")?)?,
            unlock_cycle: extract_uint(get_tuple_field(tuple, "unlock-cycle")?)?,
            is_l1_lock: extract_bool(get_tuple_field(tuple, "is-l1-lock")?)?,
            btc_lockup: extract_btc_lockup(get_tuple_field(tuple, "btc-lockup")?)?,
        },

        Pox5EventName::UpdateBondRegistration => Pox5EventData::UpdateBondRegistration {
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            old_signer: clarity_principal_to_string(get_tuple_field(tuple, "old-signer")?)?,
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            amount_ustx: extract_uint(get_tuple_field(tuple, "amount-ustx")?)?,
            amount_sats: extract_uint(get_tuple_field(tuple, "amount-sats")?)?,
            first_reward_cycle: extract_uint(get_tuple_field(tuple, "first-reward-cycle")?)?,
            num_cycles: extract_uint(get_tuple_field(tuple, "num-cycles")?)?,
            is_l1_lock: extract_bool(get_tuple_field(tuple, "is-l1-lock")?)?,
        },

        Pox5EventName::RegisterSigner => Pox5EventData::RegisterSigner {
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            signer_key: extract_buffer_hex(get_tuple_field(tuple, "signer-key")?)?,
        },

        Pox5EventName::Stake => Pox5EventData::Stake {
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            amount_ustx: extract_uint(get_tuple_field(tuple, "amount-ustx")?)?,
            num_cycles: extract_uint(get_tuple_field(tuple, "num-cycles")?)?,
            first_reward_cycle: extract_uint(get_tuple_field(tuple, "first-reward-cycle")?)?,
            unlock_burn_height: extract_uint(get_tuple_field(tuple, "unlock-burn-height")?)?,
            unlock_cycle: extract_uint(get_tuple_field(tuple, "unlock-cycle")?)?,
        },

        Pox5EventName::StakeUpdate => Pox5EventData::StakeUpdate {
            unlock_burn_height: extract_uint(get_tuple_field(tuple, "unlock-burn-height")?)?,
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            old_signer: clarity_principal_to_string(get_tuple_field(tuple, "old-signer")?)?,
            prev_unlock_height: extract_uint(get_tuple_field(tuple, "prev-unlock-height")?)?,
            unlock_cycle: extract_uint(get_tuple_field(tuple, "unlock-cycle")?)?,
            num_cycles: extract_uint(get_tuple_field(tuple, "num-cycles")?)?,
            amount_ustx: extract_uint(get_tuple_field(tuple, "amount-ustx")?)?,
            amount_increase: extract_uint(get_tuple_field(tuple, "amount-increase")?)?,
            cycles_to_extend: extract_uint(get_tuple_field(tuple, "cycles-to-extend")?)?,
        },

        Pox5EventName::AnnounceL1EarlyExit => Pox5EventData::AnnounceL1EarlyExit {
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            amount_sats_released: extract_uint(get_tuple_field(tuple, "amount-sats-released")?)?,
        },

        Pox5EventName::UnstakeSbtc => Pox5EventData::UnstakeSbtc {
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            amount_withdrawn_sats: extract_uint(get_tuple_field(tuple, "amount-withdrawn-sats")?)?,
            new_amount_sats: extract_uint(get_tuple_field(tuple, "new-amount-sats")?)?,
        },

        Pox5EventName::Unstake => Pox5EventData::Unstake {
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            signer: clarity_principal_to_string(get_tuple_field(tuple, "signer")?)?,
            amount_ustx: extract_uint(get_tuple_field(tuple, "amount-ustx")?)?,
            first_reward_cycle: extract_uint(get_tuple_field(tuple, "first-reward-cycle")?)?,
            unlock_cycle: extract_uint(get_tuple_field(tuple, "unlock-cycle")?)?,
            unlock_burn_height: extract_uint(get_tuple_field(tuple, "unlock-burn-height")?)?,
        },

        Pox5EventName::CalculateRewards => Pox5EventData::CalculateRewards {
            bond_periods: extract_list(get_tuple_field(tuple, "bond-periods")?, extract_uint)?,
            calculation_height: extract_uint(get_tuple_field(tuple, "calculation-height")?)?,
            gross_accrued_rewards: extract_uint(get_tuple_field(tuple, "gross-accrued-rewards")?)?,
            total_bond_rewards: extract_uint(get_tuple_field(tuple, "total-bond-rewards")?)?,
            reserve_deposit: extract_uint(get_tuple_field(tuple, "reserve-deposit")?)?,
            reserve_balance: extract_uint(get_tuple_field(tuple, "reserve-balance")?)?,
            stx_cycle: extract_uint(get_tuple_field(tuple, "stx-cycle")?)?,
            total_stx_staker_rewards: extract_uint(get_tuple_field(
                tuple,
                "total-stx-staker-rewards",
            )?)?,
            cycle_staked_ustx: extract_uint(get_tuple_field(tuple, "cycle-staked-ustx")?)?,
            accrued_rewards_per_ustx: extract_uint(get_tuple_field(
                tuple,
                "accrued-rewards-per-ustx",
            )?)?,
            cumulative_rewards_per_ustx: extract_uint(get_tuple_field(
                tuple,
                "cumulative-rewards-per-ustx",
            )?)?,
        },

        Pox5EventName::BondDistribution => Pox5EventData::BondDistribution {
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            target_yield: extract_uint(get_tuple_field(tuple, "target-yield")?)?,
            bond_rewards: extract_uint(get_tuple_field(tuple, "bond-rewards")?)?,
            bond_staked_sats: extract_uint(get_tuple_field(tuple, "bond-staked-sats")?)?,
            accrued_rewards_per_sat: extract_uint(get_tuple_field(
                tuple,
                "accrued-rewards-per-sat",
            )?)?,
            cumulative_rewards_per_sat: extract_uint(get_tuple_field(
                tuple,
                "cumulative-rewards-per-sat",
            )?)?,
        },

        Pox5EventName::ClaimRewards => Pox5EventData::ClaimRewards {
            signer_manager: clarity_principal_to_string(get_tuple_field(tuple, "signer-manager")?)?,
            reward_cycle: extract_uint(get_tuple_field(tuple, "reward-cycle")?)?,
            stx_rewards: extract_claim_rewards_info(get_tuple_field(tuple, "stx-rewards")?)?,
            bond_rewards: extract_list(
                get_tuple_field(tuple, "bond-rewards")?,
                extract_bond_rewards_info,
            )?,
            bond_totals: extract_uint(get_tuple_field(tuple, "bond-totals")?)?,
            total_rewards: extract_uint(get_tuple_field(tuple, "total-rewards")?)?,
        },

        Pox5EventName::ClaimStakerRewardsForSigner => Pox5EventData::ClaimStakerRewardsForSigner {
            signer_manager: clarity_principal_to_string(get_tuple_field(tuple, "signer-manager")?)?,
            staker: clarity_principal_to_string(get_tuple_field(tuple, "staker")?)?,
            reward_cycle: extract_uint(get_tuple_field(tuple, "reward-cycle")?)?,
            bond_index: extract_optional_uint(tuple_get(tuple, "bond-index"))?,
            rewards_claimed: extract_uint(get_tuple_field(tuple, "rewards-claimed")?)?,
        },

        Pox5EventName::GrantSignerKey => Pox5EventData::GrantSignerKey {
            signer_key: extract_buffer_hex(get_tuple_field(tuple, "signer-key")?)?,
            signer_manager: clarity_principal_to_string(get_tuple_field(tuple, "signer-manager")?)?,
            auth_id: extract_uint(get_tuple_field(tuple, "auth-id")?)?,
        },

        Pox5EventName::RevokeSignerGrant => Pox5EventData::RevokeSignerGrant {
            signer_key: extract_buffer_hex(get_tuple_field(tuple, "signer-key")?)?,
            signer_manager: clarity_principal_to_string(get_tuple_field(tuple, "signer-manager")?)?,
        },

        Pox5EventName::DisallowContractCaller => Pox5EventData::DisallowContractCaller {
            sender: clarity_principal_to_string(get_tuple_field(tuple, "sender")?)?,
            contract_caller: clarity_principal_to_string(get_tuple_field(
                tuple,
                "contract-caller",
            )?)?,
        },

        Pox5EventName::AllowContractCaller => Pox5EventData::AllowContractCaller {
            sender: clarity_principal_to_string(get_tuple_field(tuple, "sender")?)?,
            contract_caller: clarity_principal_to_string(get_tuple_field(
                tuple,
                "contract-caller",
            )?)?,
            until_burn_ht: extract_optional_uint(tuple_get(tuple, "until-burn-ht"))?,
        },
    };

    Ok(Some(Pox5SyntheticEvent { name, data }))
}

/// Extract the `btc-lockup` sub-tuple `{ type, txs }` from
/// `register-for-bond`. `txs` is `(optional (list { txid, output-index }))` —
/// `Some` for an L1 lockup, `None` for an sBTC (L2) lockup.
fn extract_btc_lockup(val: &UpstreamValue) -> Result<BtcLockup, String> {
    let tuple = extract_tuple(val)?;
    let txs_val = get_tuple_field(tuple, "txs")?;
    let txs = match txs_val {
        UpstreamValue::Optional(opt) => match &opt.data {
            None => None,
            Some(inner) => Some(extract_list(inner.as_ref(), extract_btc_lockup_tx)?),
        },
        other => {
            return Err(format!(
                "Expected OptionalSome/OptionalNone for btc-lockup txs, got {:?}",
                other
            ))
        }
    };
    Ok(BtcLockup {
        lockup_type: extract_ascii_string(get_tuple_field(tuple, "type")?)?,
        txs,
    })
}

/// Extract one entry from a `btc-lockup` `txs` list:
/// `{ txid, output-index }`.
fn extract_btc_lockup_tx(val: &UpstreamValue) -> Result<BtcLockupTx, String> {
    let tuple = extract_tuple(val)?;
    Ok(BtcLockupTx {
        txid: extract_buffer_hex(get_tuple_field(tuple, "txid")?)?,
        output_index: extract_uint(get_tuple_field(tuple, "output-index")?)?,
    })
}

/// Extract the `stx-rewards` sub-tuple `{ earned, rewards-per-token }`.
fn extract_claim_rewards_info(val: &UpstreamValue) -> Result<ClaimRewardsInfo, String> {
    let tuple = extract_tuple(val)?;
    Ok(ClaimRewardsInfo {
        earned: extract_uint(get_tuple_field(tuple, "earned")?)?,
        rewards_per_token: extract_uint(get_tuple_field(tuple, "rewards-per-token")?)?,
    })
}

/// Extract one entry from the `bond-rewards` list:
/// `{ earned, rewards-per-token, bond-index }`.
fn extract_bond_rewards_info(val: &UpstreamValue) -> Result<BondRewardsInfo, String> {
    let tuple = extract_tuple(val)?;
    Ok(BondRewardsInfo {
        earned: extract_uint(get_tuple_field(tuple, "earned")?)?,
        rewards_per_token: extract_uint(get_tuple_field(tuple, "rewards-per-token")?)?,
        bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clarity::vm::types::{
        BuffData, ListData, ListTypeData, PrincipalData, SequenceData, StandardPrincipalData,
        TupleData, TypeSignature,
    };
    use clarity::vm::ClarityName;
    use std::convert::TryFrom;

    fn name(s: &str) -> ClarityName {
        ClarityName::try_from(s.to_string()).unwrap()
    }

    fn ascii(s: &str) -> UpstreamValue {
        UpstreamValue::string_ascii_from_bytes(s.as_bytes().to_vec()).unwrap()
    }

    fn principal(addr_bytes: [u8; 20]) -> UpstreamValue {
        UpstreamValue::Principal(PrincipalData::Standard(
            StandardPrincipalData::new(22, addr_bytes).unwrap(),
        ))
    }

    fn buff(bytes: Vec<u8>) -> UpstreamValue {
        UpstreamValue::Sequence(SequenceData::Buffer(BuffData { data: bytes }))
    }

    fn some_uint(n: u128) -> UpstreamValue {
        UpstreamValue::some(UpstreamValue::UInt(n)).unwrap()
    }

    fn none_value() -> UpstreamValue {
        UpstreamValue::none()
    }

    fn make_tuple(fields: Vec<(&str, UpstreamValue)>) -> UpstreamValue {
        let mut data = Vec::with_capacity(fields.len());
        for (key, value) in fields {
            data.push((name(key), value));
        }
        UpstreamValue::Tuple(TupleData::from_data(data).unwrap())
    }

    fn make_uint_list(items: &[u128]) -> UpstreamValue {
        let values: Vec<UpstreamValue> = items.iter().map(|n| UpstreamValue::UInt(*n)).collect();
        UpstreamValue::Sequence(SequenceData::List(ListData {
            data: values,
            type_signature: ListTypeData::new_list(TypeSignature::UIntType, items.len() as u32)
                .unwrap(),
        }))
    }

    fn make_tuple_list(items: Vec<UpstreamValue>) -> UpstreamValue {
        let inner_type = TypeSignature::type_of(items.first().unwrap()).unwrap();
        let list_type = ListTypeData::new_list(inner_type, items.len() as u32).unwrap();
        UpstreamValue::Sequence(SequenceData::List(ListData {
            data: items,
            type_signature: list_type,
        }))
    }

    // ─── Guards ──────────────────────────────────────────────────────────────

    #[test]
    fn non_tuple_returns_none() {
        let cv = UpstreamValue::UInt(42);
        let result = decode_pox5_synthetic_event(&cv).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn tuple_without_topic_returns_none() {
        let cv = make_tuple(vec![("foo", UpstreamValue::UInt(1))]);
        let result = decode_pox5_synthetic_event(&cv).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn unknown_topic_errors() {
        let cv = make_tuple(vec![("topic", ascii("not-a-real-event"))]);
        let result = decode_pox5_synthetic_event(&cv);
        assert!(result.is_err());
    }

    #[test]
    fn missing_required_field_errors() {
        let cv = make_tuple(vec![
            ("topic", ascii("bond-distribution")),
            ("bond-index", UpstreamValue::UInt(7)),
            // target-yield missing
            ("earned", UpstreamValue::UInt(95)),
        ]);
        let result = decode_pox5_synthetic_event(&cv);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("target-yield"), "got: {}", err);
    }

    // ─── Per-event happy paths ───────────────────────────────────────────────

    #[test]
    fn set_bond_admin_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("set-bond-admin")),
            ("old-admin", principal([0x11u8; 20])),
            ("new-admin", principal([0x22u8; 20])),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        assert_eq!(event.name, Pox5EventName::SetBondAdmin);
        match event.data {
            Pox5EventData::SetBondAdmin {
                old_admin,
                new_admin,
            } => {
                assert!(old_admin.starts_with("SP") || old_admin.starts_with("ST"));
                assert!(new_admin.starts_with("SP") || new_admin.starts_with("ST"));
                assert_ne!(old_admin, new_admin);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn setup_bond_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("setup-bond")),
            ("bond-index", UpstreamValue::UInt(0)),
            ("target-rate", UpstreamValue::UInt(1500)),
            ("stx-value-ratio", UpstreamValue::UInt(800)),
            ("min-ustx-ratio", UpstreamValue::UInt(10)),
            ("early-unlock-bytes", buff(vec![0xab, 0xcd, 0xef])),
            ("first-reward-cycle", UpstreamValue::UInt(50)),
            ("bond-start-height", UpstreamValue::UInt(1_050_000)),
            ("unlock-cycle", UpstreamValue::UInt(56)),
            ("unlock-burn-height", UpstreamValue::UInt(1_063_080)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        assert_eq!(event.name, Pox5EventName::SetupBond);
        match event.data {
            Pox5EventData::SetupBond {
                bond_index,
                target_rate,
                early_unlock_bytes,
                first_reward_cycle,
                bond_start_height,
                unlock_cycle,
                unlock_burn_height,
                ..
            } => {
                assert_eq!(bond_index, 0);
                assert_eq!(target_rate, 1500);
                assert_eq!(early_unlock_bytes, "0xabcdef");
                assert_eq!(first_reward_cycle, 50);
                assert_eq!(bond_start_height, 1_050_000);
                assert_eq!(unlock_cycle, 56);
                assert_eq!(unlock_burn_height, 1_063_080);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn add_to_allowlist_decodes() {
        let staker_bytes = [0x11u8; 20];
        let cv = make_tuple(vec![
            ("topic", ascii("add-to-allowlist")),
            ("staker", principal(staker_bytes)),
            ("max-sats", UpstreamValue::UInt(1_000_000)),
            ("bond-index", UpstreamValue::UInt(3)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        assert_eq!(event.name, Pox5EventName::AddToAllowlist);
        match event.data {
            Pox5EventData::AddToAllowlist {
                max_sats,
                bond_index,
                ..
            } => {
                assert_eq!(max_sats, 1_000_000);
                assert_eq!(bond_index, 3);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn register_for_bond_l1_decodes() {
        // L1 lockup: btc-lockup.type == "l1", txs == (some (list ...)).
        let tx0 = make_tuple(vec![
            ("txid", buff(vec![0xde, 0xad, 0xbe, 0xef])),
            ("output-index", UpstreamValue::UInt(0)),
        ]);
        let tx1 = make_tuple(vec![
            ("txid", buff(vec![0x01, 0x02])),
            ("output-index", UpstreamValue::UInt(3)),
        ]);
        let btc_lockup = make_tuple(vec![
            ("type", ascii("l1")),
            (
                "txs",
                UpstreamValue::some(make_tuple_list(vec![tx0, tx1])).unwrap(),
            ),
        ]);
        let cv = make_tuple(vec![
            ("topic", ascii("register-for-bond")),
            ("signer", principal([0x11; 20])),
            ("staker", principal([0x22; 20])),
            ("amount-ustx", UpstreamValue::UInt(100)),
            ("sats-total", UpstreamValue::UInt(50)),
            ("bond-index", UpstreamValue::UInt(2)),
            ("first-reward-cycle", UpstreamValue::UInt(40)),
            ("unlock-burn-height", UpstreamValue::UInt(900_000)),
            ("unlock-cycle", UpstreamValue::UInt(46)),
            ("is-l1-lock", UpstreamValue::Bool(true)),
            ("btc-lockup", btc_lockup),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::RegisterForBond {
                amount_ustx,
                sats_total,
                is_l1_lock,
                btc_lockup,
                ..
            } => {
                assert_eq!(amount_ustx, 100);
                assert_eq!(sats_total, 50);
                assert!(is_l1_lock);
                assert_eq!(btc_lockup.lockup_type, "l1");
                let txs = btc_lockup.txs.expect("l1 should have txs");
                assert_eq!(txs.len(), 2);
                assert_eq!(txs[0].txid, "0xdeadbeef");
                assert_eq!(txs[0].output_index, 0);
                assert_eq!(txs[1].txid, "0x0102");
                assert_eq!(txs[1].output_index, 3);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn register_for_bond_l2_decodes() {
        // sBTC lockup: btc-lockup.type == "l2", txs == none.
        let btc_lockup = make_tuple(vec![("type", ascii("l2")), ("txs", none_value())]);
        let cv = make_tuple(vec![
            ("topic", ascii("register-for-bond")),
            ("signer", principal([0x11; 20])),
            ("staker", principal([0x22; 20])),
            ("amount-ustx", UpstreamValue::UInt(100)),
            ("sats-total", UpstreamValue::UInt(50)),
            ("bond-index", UpstreamValue::UInt(2)),
            ("first-reward-cycle", UpstreamValue::UInt(40)),
            ("unlock-burn-height", UpstreamValue::UInt(900_000)),
            ("unlock-cycle", UpstreamValue::UInt(46)),
            ("is-l1-lock", UpstreamValue::Bool(false)),
            ("btc-lockup", btc_lockup),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::RegisterForBond {
                is_l1_lock,
                btc_lockup,
                ..
            } => {
                assert!(!is_l1_lock);
                assert_eq!(btc_lockup.lockup_type, "l2");
                assert!(btc_lockup.txs.is_none());
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn update_bond_registration_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("update-bond-registration")),
            ("staker", principal([0x11; 20])),
            ("signer", principal([0x22; 20])),
            ("old-signer", principal([0x33; 20])),
            ("bond-index", UpstreamValue::UInt(1)),
            ("amount-ustx", UpstreamValue::UInt(200)),
            ("amount-sats", UpstreamValue::UInt(100)),
            ("first-reward-cycle", UpstreamValue::UInt(41)),
            ("num-cycles", UpstreamValue::UInt(5)),
            ("is-l1-lock", UpstreamValue::Bool(false)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::UpdateBondRegistration {
                bond_index,
                amount_sats,
                num_cycles,
                is_l1_lock,
                ..
            } => {
                assert_eq!(bond_index, 1);
                assert_eq!(amount_sats, 100);
                assert_eq!(num_cycles, 5);
                assert!(!is_l1_lock);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn register_signer_decodes() {
        let key = vec![0x02u8; 33];
        let cv = make_tuple(vec![
            ("topic", ascii("register-signer")),
            ("signer", principal([0x22; 20])),
            ("signer-key", buff(key.clone())),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::RegisterSigner { signer_key, .. } => {
                assert_eq!(signer_key.len(), "0x".len() + key.len() * 2);
                assert!(signer_key.starts_with("0x02"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn stake_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("stake")),
            ("signer", principal([0x22; 20])),
            ("staker", principal([0x11; 20])),
            ("amount-ustx", UpstreamValue::UInt(5_000)),
            ("num-cycles", UpstreamValue::UInt(6)),
            ("first-reward-cycle", UpstreamValue::UInt(42)),
            ("unlock-burn-height", UpstreamValue::UInt(910_000)),
            ("unlock-cycle", UpstreamValue::UInt(48)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::Stake {
                amount_ustx,
                num_cycles,
                ..
            } => {
                assert_eq!(amount_ustx, 5_000);
                assert_eq!(num_cycles, 6);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn stake_update_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("stake-update")),
            ("unlock-burn-height", UpstreamValue::UInt(920_000)),
            ("staker", principal([0x11; 20])),
            ("signer", principal([0x22; 20])),
            ("old-signer", principal([0x33; 20])),
            ("prev-unlock-height", UpstreamValue::UInt(48)),
            ("unlock-cycle", UpstreamValue::UInt(52)),
            ("num-cycles", UpstreamValue::UInt(10)),
            ("amount-ustx", UpstreamValue::UInt(7_000)),
            ("amount-increase", UpstreamValue::UInt(2_000)),
            ("cycles-to-extend", UpstreamValue::UInt(4)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::StakeUpdate {
                amount_increase,
                cycles_to_extend,
                ..
            } => {
                assert_eq!(amount_increase, 2_000);
                assert_eq!(cycles_to_extend, 4);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn announce_l1_early_exit_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("announce-l1-early-exit")),
            ("staker", principal([0x11; 20])),
            ("signer", principal([0x22; 20])),
            ("bond-index", UpstreamValue::UInt(1)),
            ("amount-sats-released", UpstreamValue::UInt(123_456)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::AnnounceL1EarlyExit {
                amount_sats_released,
                ..
            } => {
                assert_eq!(amount_sats_released, 123_456);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn unstake_sbtc_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("unstake-sbtc")),
            ("staker", principal([0x11; 20])),
            ("signer", principal([0x22; 20])),
            ("bond-index", UpstreamValue::UInt(2)),
            ("amount-withdrawn-sats", UpstreamValue::UInt(500)),
            ("new-amount-sats", UpstreamValue::UInt(1_500)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::UnstakeSbtc {
                amount_withdrawn_sats,
                new_amount_sats,
                ..
            } => {
                assert_eq!(amount_withdrawn_sats, 500);
                assert_eq!(new_amount_sats, 1_500);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn unstake_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("unstake")),
            ("staker", principal([0x11; 20])),
            ("signer", principal([0x22; 20])),
            ("amount-ustx", UpstreamValue::UInt(3_000)),
            ("first-reward-cycle", UpstreamValue::UInt(40)),
            ("unlock-cycle", UpstreamValue::UInt(43)),
            ("unlock-burn-height", UpstreamValue::UInt(930_000)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::Unstake {
                amount_ustx,
                unlock_cycle,
                ..
            } => {
                assert_eq!(amount_ustx, 3_000);
                assert_eq!(unlock_cycle, 43);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn calculate_rewards_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("calculate-rewards")),
            ("bond-periods", make_uint_list(&[10, 11, 12])),
            ("calculation-height", UpstreamValue::UInt(500_000)),
            ("gross-accrued-rewards", UpstreamValue::UInt(20_000)),
            ("total-bond-rewards", UpstreamValue::UInt(8_000)),
            ("reserve-deposit", UpstreamValue::UInt(2_000)),
            ("reserve-balance", UpstreamValue::UInt(50_000)),
            ("stx-cycle", UpstreamValue::UInt(42)),
            ("total-stx-staker-rewards", UpstreamValue::UInt(5_000)),
            ("cycle-staked-ustx", UpstreamValue::UInt(1_000_000_000)),
            ("accrued-rewards-per-ustx", UpstreamValue::UInt(7)),
            ("cumulative-rewards-per-ustx", UpstreamValue::UInt(107)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::CalculateRewards {
                bond_periods,
                gross_accrued_rewards,
                total_bond_rewards,
                reserve_deposit,
                reserve_balance,
                total_stx_staker_rewards,
                accrued_rewards_per_ustx,
                cumulative_rewards_per_ustx,
                ..
            } => {
                assert_eq!(bond_periods, vec![10, 11, 12]);
                assert_eq!(gross_accrued_rewards, 20_000);
                assert_eq!(total_bond_rewards, 8_000);
                assert_eq!(reserve_deposit, 2_000);
                assert_eq!(reserve_balance, 50_000);
                assert_eq!(total_stx_staker_rewards, 5_000);
                assert_eq!(accrued_rewards_per_ustx, 7);
                assert_eq!(cumulative_rewards_per_ustx, 107);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn bond_distribution_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("bond-distribution")),
            ("bond-index", UpstreamValue::UInt(7)),
            ("target-yield", UpstreamValue::UInt(100)),
            ("bond-rewards", UpstreamValue::UInt(95)),
            ("bond-staked-sats", UpstreamValue::UInt(1_000_000)),
            ("accrued-rewards-per-sat", UpstreamValue::UInt(3)),
            ("cumulative-rewards-per-sat", UpstreamValue::UInt(303)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::BondDistribution {
                bond_index,
                target_yield,
                bond_rewards,
                bond_staked_sats,
                accrued_rewards_per_sat,
                cumulative_rewards_per_sat,
            } => {
                assert_eq!(bond_index, 7);
                assert_eq!(target_yield, 100);
                assert_eq!(bond_rewards, 95);
                assert_eq!(bond_staked_sats, 1_000_000);
                assert_eq!(accrued_rewards_per_sat, 3);
                assert_eq!(cumulative_rewards_per_sat, 303);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn claim_rewards_decodes() {
        let stx_rewards = make_tuple(vec![
            ("earned", UpstreamValue::UInt(11)),
            ("rewards-per-token", UpstreamValue::UInt(22)),
        ]);
        let bond_reward = make_tuple(vec![
            ("earned", UpstreamValue::UInt(33)),
            ("rewards-per-token", UpstreamValue::UInt(44)),
            ("bond-index", UpstreamValue::UInt(5)),
        ]);
        let cv = make_tuple(vec![
            ("topic", ascii("claim-rewards")),
            ("signer-manager", principal([0x44u8; 20])),
            ("reward-cycle", UpstreamValue::UInt(42)),
            ("stx-rewards", stx_rewards),
            ("bond-rewards", make_tuple_list(vec![bond_reward])),
            ("bond-totals", UpstreamValue::UInt(50)),
            ("total-rewards", UpstreamValue::UInt(61)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::ClaimRewards {
                signer_manager,
                reward_cycle,
                stx_rewards,
                bond_rewards,
                bond_totals,
                total_rewards,
            } => {
                assert!(signer_manager.starts_with("SP") || signer_manager.starts_with("ST"));
                assert_eq!(reward_cycle, 42);
                assert_eq!(stx_rewards.earned, 11);
                assert_eq!(stx_rewards.rewards_per_token, 22);
                assert_eq!(bond_rewards.len(), 1);
                assert_eq!(bond_rewards[0].earned, 33);
                assert_eq!(bond_rewards[0].rewards_per_token, 44);
                assert_eq!(bond_rewards[0].bond_index, 5);
                assert_eq!(bond_totals, 50);
                assert_eq!(total_rewards, 61);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn claim_staker_rewards_for_signer_decodes() {
        // bond rewards: bond-index is OptionalSome.
        let cv = make_tuple(vec![
            ("topic", ascii("claim-staker-rewards-for-signer")),
            ("signer-manager", principal([0x44u8; 20])),
            ("staker", principal([0x55u8; 20])),
            ("reward-cycle", UpstreamValue::UInt(42)),
            ("bond-index", some_uint(7)),
            ("rewards-claimed", UpstreamValue::UInt(1234)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::ClaimStakerRewardsForSigner {
                reward_cycle,
                bond_index,
                rewards_claimed,
                ..
            } => {
                assert_eq!(reward_cycle, 42);
                assert_eq!(bond_index, Some(7));
                assert_eq!(rewards_claimed, 1234);
            }
            _ => panic!("wrong variant"),
        }

        // STX-only rewards: bond-index is OptionalNone.
        let cv = make_tuple(vec![
            ("topic", ascii("claim-staker-rewards-for-signer")),
            ("signer-manager", principal([0x44u8; 20])),
            ("staker", principal([0x55u8; 20])),
            ("reward-cycle", UpstreamValue::UInt(42)),
            ("bond-index", none_value()),
            ("rewards-claimed", UpstreamValue::UInt(0)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::ClaimStakerRewardsForSigner { bond_index, .. } => {
                assert_eq!(bond_index, None);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn grant_signer_key_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("grant-signer-key")),
            ("signer-key", buff(vec![0x02, 0xaa, 0xbb])),
            ("signer-manager", principal([0x44u8; 20])),
            ("auth-id", UpstreamValue::UInt(99)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::GrantSignerKey {
                signer_key,
                auth_id,
                ..
            } => {
                assert_eq!(signer_key, "0x02aabb");
                assert_eq!(auth_id, 99);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn revoke_signer_grant_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("revoke-signer-grant")),
            ("signer-key", buff(vec![0x03, 0xcc])),
            ("signer-manager", principal([0x44u8; 20])),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::RevokeSignerGrant {
                signer_key,
                signer_manager,
            } => {
                assert_eq!(signer_key, "0x03cc");
                assert!(signer_manager.starts_with("SP") || signer_manager.starts_with("ST"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn disallow_contract_caller_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("disallow-contract-caller")),
            ("sender", principal([0x66u8; 20])),
            ("contract-caller", principal([0x77u8; 20])),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::DisallowContractCaller {
                sender,
                contract_caller,
            } => {
                assert!(sender.starts_with("SP") || sender.starts_with("ST"));
                assert!(contract_caller.starts_with("SP") || contract_caller.starts_with("ST"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn allow_contract_caller_decodes() {
        // With an expiry height.
        let cv = make_tuple(vec![
            ("topic", ascii("allow-contract-caller")),
            ("sender", principal([0x66u8; 20])),
            ("contract-caller", principal([0x77u8; 20])),
            ("until-burn-ht", some_uint(1_000_000)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::AllowContractCaller { until_burn_ht, .. } => {
                assert_eq!(until_burn_ht, Some(1_000_000));
            }
            _ => panic!("wrong variant"),
        }

        // No expiry (never expires).
        let cv = make_tuple(vec![
            ("topic", ascii("allow-contract-caller")),
            ("sender", principal([0x66u8; 20])),
            ("contract-caller", principal([0x77u8; 20])),
            ("until-burn-ht", none_value()),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::AllowContractCaller { until_burn_ht, .. } => {
                assert_eq!(until_burn_ht, None);
            }
            _ => panic!("wrong variant"),
        }
    }
}
