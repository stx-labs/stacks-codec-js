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
    extract_list, extract_tuple, extract_uint, get_tuple_field, tuple_get,
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
        Pox5EventName::SetupBond => Pox5EventData::SetupBond {
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            target_rate: extract_uint(get_tuple_field(tuple, "target-rate")?)?,
            stx_value_ratio: extract_uint(get_tuple_field(tuple, "stx-value-ratio")?)?,
            min_ustx_ratio: extract_uint(get_tuple_field(tuple, "min-ustx-ratio")?)?,
            early_unlock_bytes: extract_buffer_hex(get_tuple_field(tuple, "early-unlock-bytes")?)?,
            early_unlock_admin: clarity_principal_to_string(get_tuple_field(
                tuple,
                "early-unlock-admin",
            )?)?,
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
            remaining_rewards: extract_uint(get_tuple_field(tuple, "remaining-rewards")?)?,
            accrued_rewards: extract_uint(get_tuple_field(tuple, "accrued-rewards")?)?,
            // The contract emits `calculate-rewards` twice per call (see the
            // doc on the variant). The two prints share the same topic but
            // differ in which of these two fields they carry, so use
            // tuple_get to map an absent field to None rather than failing.
            new_reserve: tuple_get(tuple, "new-reserve")
                .map(extract_uint)
                .transpose()?,
            stranded_staker_cut: tuple_get(tuple, "stranded-staker-cut")
                .map(extract_uint)
                .transpose()?,
            stx_staker_rewards: extract_uint(get_tuple_field(tuple, "stx-staker-rewards")?)?,
            stx_cycle: extract_uint(get_tuple_field(tuple, "stx-cycle")?)?,
            cycle_staked_ustx: extract_uint(get_tuple_field(tuple, "cycle-staked-ustx")?)?,
            next_rewards_per_ustx: extract_uint(get_tuple_field(tuple, "next-rewards-per-ustx")?)?,
        },

        Pox5EventName::BondDistribution => Pox5EventData::BondDistribution {
            bond_index: extract_uint(get_tuple_field(tuple, "bond-index")?)?,
            target_yield: extract_uint(get_tuple_field(tuple, "target-yield")?)?,
            earned: extract_uint(get_tuple_field(tuple, "earned")?)?,
        },

        Pox5EventName::ClaimRewards => Pox5EventData::ClaimRewards {
            stx_rewards: extract_claim_rewards_info(get_tuple_field(tuple, "stx-rewards")?)?,
            bond_rewards: extract_list(
                get_tuple_field(tuple, "bond-rewards")?,
                extract_bond_rewards_info,
            )?,
            bond_totals: extract_uint(get_tuple_field(tuple, "bond-totals")?)?,
            total_rewards: extract_uint(get_tuple_field(tuple, "total-rewards")?)?,
        },
    };

    Ok(Some(Pox5SyntheticEvent { name, data }))
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
    fn setup_bond_decodes() {
        let admin_bytes = [0x33u8; 20];
        let cv = make_tuple(vec![
            ("topic", ascii("setup-bond")),
            ("bond-index", UpstreamValue::UInt(0)),
            ("target-rate", UpstreamValue::UInt(1500)),
            ("stx-value-ratio", UpstreamValue::UInt(800)),
            ("min-ustx-ratio", UpstreamValue::UInt(10)),
            ("early-unlock-bytes", buff(vec![0xab, 0xcd, 0xef])),
            ("early-unlock-admin", principal(admin_bytes)),
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
                early_unlock_admin,
                first_reward_cycle,
                bond_start_height,
                unlock_cycle,
                unlock_burn_height,
                ..
            } => {
                assert_eq!(bond_index, 0);
                assert_eq!(target_rate, 1500);
                assert_eq!(early_unlock_bytes, "0xabcdef");
                assert!(
                    early_unlock_admin.starts_with("SP") || early_unlock_admin.starts_with("ST")
                );
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
    fn register_for_bond_decodes() {
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
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::RegisterForBond {
                amount_ustx,
                sats_total,
                is_l1_lock,
                ..
            } => {
                assert_eq!(amount_ustx, 100);
                assert_eq!(sats_total, 50);
                assert!(is_l1_lock);
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

    /// Phase-2 (post-distribution) `calculate-rewards`: carries `new-reserve`,
    /// omits `stranded-staker-cut`.
    #[test]
    fn calculate_rewards_phase2_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("calculate-rewards")),
            ("bond-periods", make_uint_list(&[10, 11, 12])),
            ("calculation-height", UpstreamValue::UInt(500_000)),
            ("remaining-rewards", UpstreamValue::UInt(10_000)),
            ("accrued-rewards", UpstreamValue::UInt(20_000)),
            ("new-reserve", UpstreamValue::UInt(2_000)),
            ("stx-staker-rewards", UpstreamValue::UInt(5_000)),
            ("stx-cycle", UpstreamValue::UInt(42)),
            ("cycle-staked-ustx", UpstreamValue::UInt(1_000_000_000)),
            ("next-rewards-per-ustx", UpstreamValue::UInt(7)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::CalculateRewards {
                bond_periods,
                new_reserve,
                stranded_staker_cut,
                accrued_rewards,
                ..
            } => {
                assert_eq!(bond_periods, vec![10, 11, 12]);
                assert_eq!(new_reserve, Some(2_000));
                assert_eq!(stranded_staker_cut, None);
                assert_eq!(accrued_rewards, 20_000);
            }
            _ => panic!("wrong variant"),
        }
    }

    /// Phase-1 (pre-distribution) `calculate-rewards`: carries
    /// `stranded-staker-cut`, omits `new-reserve`.
    #[test]
    fn calculate_rewards_phase1_decodes() {
        let cv = make_tuple(vec![
            ("topic", ascii("calculate-rewards")),
            ("bond-periods", make_uint_list(&[10, 11, 12])),
            ("calculation-height", UpstreamValue::UInt(500_000)),
            ("remaining-rewards", UpstreamValue::UInt(10_000)),
            ("accrued-rewards", UpstreamValue::UInt(20_000)),
            ("stx-staker-rewards", UpstreamValue::UInt(5_000)),
            ("stx-cycle", UpstreamValue::UInt(42)),
            ("cycle-staked-ustx", UpstreamValue::UInt(0)),
            ("next-rewards-per-ustx", UpstreamValue::UInt(0)),
            ("stranded-staker-cut", UpstreamValue::UInt(5_000)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::CalculateRewards {
                new_reserve,
                stranded_staker_cut,
                cycle_staked_ustx,
                ..
            } => {
                assert_eq!(new_reserve, None);
                assert_eq!(stranded_staker_cut, Some(5_000));
                assert_eq!(cycle_staked_ustx, 0);
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
            ("earned", UpstreamValue::UInt(95)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::BondDistribution {
                bond_index,
                target_yield,
                earned,
            } => {
                assert_eq!(bond_index, 7);
                assert_eq!(target_yield, 100);
                assert_eq!(earned, 95);
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
            ("stx-rewards", stx_rewards),
            ("bond-rewards", make_tuple_list(vec![bond_reward])),
            ("bond-totals", UpstreamValue::UInt(50)),
            ("total-rewards", UpstreamValue::UInt(61)),
        ]);
        let event = decode_pox5_synthetic_event(&cv).unwrap().unwrap();
        match event.data {
            Pox5EventData::ClaimRewards {
                stx_rewards,
                bond_rewards,
                bond_totals,
                total_rewards,
            } => {
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
}
