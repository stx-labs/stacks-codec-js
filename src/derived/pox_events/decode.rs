use std::collections::BTreeMap;

use clarity::vm::types::{CharType, PrincipalData, SequenceData, Value as UpstreamValue};
use clarity::vm::ClarityName;

use crate::util::hex::encode_hex;

use super::btc_address::pox_address_to_btc_address;
use super::types::*;

/// Decode a Clarity value into a PoX synthetic event.
/// Returns `Ok(None)` if the value is a `ResponseErr` (non-event).
/// Returns `Err` if the structure is unexpected.
pub fn decode_pox_synthetic_event(
    clarity_value: &UpstreamValue,
    network: StacksNetwork,
) -> Result<Option<PoxSyntheticEvent>, String> {
    // 1. Root must be Response(committed=true); ResponseErr means no event.
    let inner = match clarity_value {
        UpstreamValue::Response(resp) if resp.committed => &resp.data,
        UpstreamValue::Response(resp) if !resp.committed => return Ok(None),
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event Clarity type, expected ResponseOk, got {}",
                short_type_name(other)
            ))
        }
    };

    // 2. Inner must be a Tuple
    let op_data = match inner.as_ref() {
        UpstreamValue::Tuple(t) => &t.data_map,
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event Clarity type, expected Tuple, got {}",
                short_type_name(other)
            ))
        }
    };

    // 3. Extract base fields
    let stacker = clarity_principal_to_string(get_tuple_field(op_data, "stacker")?)?;
    let locked = extract_uint(get_tuple_field(op_data, "locked")?)?;
    let balance = extract_uint(get_tuple_field(op_data, "balance")?)?;
    let burnchain_unlock_height =
        extract_uint(get_tuple_field(op_data, "burnchain-unlock-height")?)?;

    // 4. Extract event name
    let name_str = match get_tuple_field(op_data, "name")? {
        UpstreamValue::Sequence(SequenceData::String(CharType::ASCII(s))) => {
            String::from_utf8(s.data.clone()).map_err(|e| format!("Invalid event name: {}", e))?
        }
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event name type, expected StringASCII, got {}",
                short_type_name(other)
            ))
        }
    };

    let event_name = PoxEventName::parse(&name_str)
        .ok_or_else(|| format!("Unexpected PoX synthetic event data name: {}", name_str))?;

    // 5. Extract inner data tuple
    let event_data_tuple = match get_tuple_field(op_data, "data")? {
        UpstreamValue::Tuple(t) => &t.data_map,
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event data payload type, expected Tuple, got {}",
                short_type_name(other)
            ))
        }
    };

    // 6. Extract pox-addr if present
    let (pox_addr, pox_addr_raw) = if tuple_get(event_data_tuple, "pox-addr").is_some() {
        extract_pox_addr(get_tuple_field(event_data_tuple, "pox-addr")?, network)?
    } else {
        (None, None)
    };

    let mut base = PoxEventBase {
        stacker,
        locked,
        balance,
        burnchain_unlock_height,
        pox_addr,
        pox_addr_raw,
    };

    // 7. Match on event name, extract type-specific fields and apply balance patches
    let data = match event_name {
        PoxEventName::HandleUnlock => {
            let first_cycle_locked =
                extract_uint(get_tuple_field(event_data_tuple, "first-cycle-locked")?)?;
            let first_unlocked_cycle =
                extract_uint(get_tuple_field(event_data_tuple, "first-unlocked-cycle")?)?;

            base.balance = base.balance.saturating_add(base.locked);

            PoxEventData::HandleUnlock {
                first_cycle_locked,
                first_unlocked_cycle,
            }
        }
        PoxEventName::StackStx => {
            let lock_amount = extract_uint(get_tuple_field(event_data_tuple, "lock-amount")?)?;
            let lock_period = extract_uint(get_tuple_field(event_data_tuple, "lock-period")?)?;
            let start_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "start-burn-height")?)?;
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let signer_key =
                extract_optional_buffer_hex(tuple_get(event_data_tuple, "signer-key"))?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            base.burnchain_unlock_height = unlock_burn_height;
            base.balance = base.balance.saturating_sub(lock_amount);
            base.locked = lock_amount;

            PoxEventData::StackStx {
                lock_amount,
                lock_period,
                start_burn_height,
                unlock_burn_height,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackIncrease => {
            let increase_by = extract_uint(get_tuple_field(event_data_tuple, "increase-by")?)?;
            let total_locked = extract_uint(get_tuple_field(event_data_tuple, "total-locked")?)?;
            let signer_key =
                extract_optional_buffer_hex(tuple_get(event_data_tuple, "signer-key"))?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            base.balance = base.balance.saturating_sub(increase_by);
            base.locked = base.locked.saturating_add(increase_by);

            PoxEventData::StackIncrease {
                increase_by,
                total_locked,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackExtend => {
            let extend_count = extract_uint(get_tuple_field(event_data_tuple, "extend-count")?)?;
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let signer_key =
                extract_optional_buffer_hex(tuple_get(event_data_tuple, "signer-key"))?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            base.burnchain_unlock_height = unlock_burn_height;

            PoxEventData::StackExtend {
                extend_count,
                unlock_burn_height,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStx => {
            let amount_ustx = extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let delegate_to =
                clarity_principal_to_string(get_tuple_field(event_data_tuple, "delegate-to")?)?;
            let unlock_burn_height_opt = extract_optional_uint(Some(get_tuple_field(
                event_data_tuple,
                "unlock-burn-height",
            )?))?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            if let Some(ubh) = unlock_burn_height_opt {
                base.burnchain_unlock_height = ubh;
            }

            PoxEventData::DelegateStx {
                amount_ustx,
                delegate_to,
                unlock_burn_height: unlock_burn_height_opt,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStackStx => {
            let lock_amount = extract_uint(get_tuple_field(event_data_tuple, "lock-amount")?)?;
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let start_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "start-burn-height")?)?;
            let lock_period = extract_uint(get_tuple_field(event_data_tuple, "lock-period")?)?;
            let delegator =
                clarity_principal_to_string(get_tuple_field(event_data_tuple, "delegator")?)?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            base.burnchain_unlock_height = unlock_burn_height;
            base.balance = base.balance.saturating_sub(lock_amount);
            base.locked = lock_amount;

            PoxEventData::DelegateStackStx {
                lock_amount,
                unlock_burn_height,
                start_burn_height,
                lock_period,
                delegator,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStackIncrease => {
            let increase_by = extract_uint(get_tuple_field(event_data_tuple, "increase-by")?)?;
            let total_locked = extract_uint(get_tuple_field(event_data_tuple, "total-locked")?)?;
            let delegator =
                clarity_principal_to_string(get_tuple_field(event_data_tuple, "delegator")?)?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            base.balance = base.balance.saturating_sub(increase_by);
            base.locked = base.locked.saturating_add(increase_by);

            PoxEventData::DelegateStackIncrease {
                increase_by,
                total_locked,
                delegator,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStackExtend => {
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let extend_count = extract_uint(get_tuple_field(event_data_tuple, "extend-count")?)?;
            let delegator =
                clarity_principal_to_string(get_tuple_field(event_data_tuple, "delegator")?)?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            base.burnchain_unlock_height = unlock_burn_height;

            PoxEventData::DelegateStackExtend {
                unlock_burn_height,
                extend_count,
                delegator,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackAggregationCommit => {
            let reward_cycle = extract_uint(get_tuple_field(event_data_tuple, "reward-cycle")?)?;
            let amount_ustx = extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let signer_key =
                extract_optional_buffer_hex(tuple_get(event_data_tuple, "signer-key"))?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            PoxEventData::StackAggregationCommit {
                reward_cycle,
                amount_ustx,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackAggregationCommitIndexed => {
            let reward_cycle = extract_uint(get_tuple_field(event_data_tuple, "reward-cycle")?)?;
            let amount_ustx = extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let signer_key =
                extract_optional_buffer_hex(tuple_get(event_data_tuple, "signer-key"))?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            PoxEventData::StackAggregationCommitIndexed {
                reward_cycle,
                amount_ustx,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackAggregationIncrease => {
            let reward_cycle = extract_uint(get_tuple_field(event_data_tuple, "reward-cycle")?)?;
            let amount_ustx = extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            PoxEventData::StackAggregationIncrease {
                reward_cycle,
                amount_ustx,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::RevokeDelegateStx => {
            let delegate_to =
                clarity_principal_to_string(get_tuple_field(event_data_tuple, "delegate-to")?)?;
            let end_cycle_id = extract_optional_uint(tuple_get(event_data_tuple, "end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(tuple_get(event_data_tuple, "start-cycle-id"))?;

            PoxEventData::RevokeDelegateStx {
                delegate_to,
                end_cycle_id,
                start_cycle_id,
            }
        }
    };

    Ok(Some(PoxSyntheticEvent {
        base,
        name: event_name,
        data,
    }))
}

// ─── Helper functions ───────────────────────────────────────────────────────

/// `BTreeMap::get` taking a `&str` to look up a `ClarityName` key. The
/// upstream `ClarityName` is a `guarded_string`, which derefs to `&str` and
/// implements `Borrow<str>`, so an `&str` lookup works directly.
fn tuple_get<'a>(
    tuple: &'a BTreeMap<ClarityName, UpstreamValue>,
    key: &str,
) -> Option<&'a UpstreamValue> {
    tuple.get(key)
}

fn get_tuple_field<'a>(
    tuple: &'a BTreeMap<ClarityName, UpstreamValue>,
    key: &str,
) -> Result<&'a UpstreamValue, String> {
    tuple_get(tuple, key).ok_or_else(|| format!("Missing expected tuple field: {}", key))
}

fn extract_uint(val: &UpstreamValue) -> Result<u128, String> {
    match val {
        UpstreamValue::UInt(v) => Ok(*v),
        other => Err(format!("Expected UInt, got {}", short_type_name(other))),
    }
}

/// Extract an optional uint from:
/// - `None` (field absent) → `Ok(None)`
/// - `OptionalNone` → `Ok(None)`
/// - `OptionalSome(UInt(v))` → `Ok(Some(v))`
/// - `UInt(v)` → `Ok(Some(v))` (for fields that are sometimes bare uints)
fn extract_optional_uint(val: Option<&UpstreamValue>) -> Result<Option<u128>, String> {
    let Some(cv) = val else { return Ok(None) };
    match cv {
        UpstreamValue::Optional(opt) => match &opt.data {
            None => Ok(None),
            Some(inner) => match inner.as_ref() {
                UpstreamValue::UInt(v) => Ok(Some(*v)),
                other => Err(format!(
                    "Expected UInt inside OptionalSome, got {}",
                    short_type_name(other)
                )),
            },
        },
        UpstreamValue::UInt(v) => Ok(Some(*v)),
        other => Err(format!(
            "Expected OptionalSome/OptionalNone/UInt, got {}",
            short_type_name(other)
        )),
    }
}

/// Extract a buffer as a hex string from:
/// - `None` (field absent) → `Ok(None)`
/// - `OptionalNone` → `Ok(None)`
/// - `Buffer(bytes)` → `Ok(Some("0x..."))`
/// - `OptionalSome(Buffer(bytes))` → `Ok(Some("0x..."))`
fn extract_optional_buffer_hex(val: Option<&UpstreamValue>) -> Result<Option<String>, String> {
    let Some(cv) = val else { return Ok(None) };
    match cv {
        UpstreamValue::Sequence(SequenceData::Buffer(b)) => {
            Ok(Some(encode_hex(&b.data).to_string()))
        }
        UpstreamValue::Optional(opt) => match &opt.data {
            None => Ok(None),
            Some(inner) => match inner.as_ref() {
                UpstreamValue::Sequence(SequenceData::Buffer(b)) => {
                    Ok(Some(encode_hex(&b.data).to_string()))
                }
                other => Err(format!(
                    "Expected Buffer inside OptionalSome, got {}",
                    short_type_name(other)
                )),
            },
        },
        other => Err(format!(
            "Expected Buffer/OptionalSome/OptionalNone, got {}",
            short_type_name(other)
        )),
    }
}

/// Convert a Clarity principal value to a string address.
fn clarity_principal_to_string(val: &UpstreamValue) -> Result<String, String> {
    match val {
        UpstreamValue::Principal(PrincipalData::Standard(spd)) => {
            crate::upstream::address::c32_address(spd.version(), &spd.1)
        }
        UpstreamValue::Principal(PrincipalData::Contract(qci)) => {
            let addr = crate::upstream::address::c32_address(qci.issuer.version(), &qci.issuer.1)?;
            Ok(format!("{}.{}", addr, qci.name))
        }
        other => Err(format!(
            "Unexpected Clarity value type for principal: {}",
            short_type_name(other)
        )),
    }
}

/// Extract pox-addr tuple (version + hashbytes) and convert to BTC address.
/// Returns (btc_addr, raw_hex). Gracefully returns (None, None) on encoding errors.
fn extract_pox_addr(
    val: &UpstreamValue,
    network: StacksNetwork,
) -> Result<(Option<String>, Option<String>), String> {
    // Handle OptionalNone short-circuit
    if let UpstreamValue::Optional(opt) = val {
        if opt.data.is_none() {
            return Ok((None, None));
        }
    }

    let addr_tuple = match val {
        UpstreamValue::Optional(opt) => match opt.data.as_deref() {
            Some(UpstreamValue::Tuple(t)) => &t.data_map,
            Some(other) => {
                return Err(format!(
                    "Expected Tuple inside OptionalSome for pox-addr, got {}",
                    short_type_name(other)
                ));
            }
            None => return Ok((None, None)),
        },
        UpstreamValue::Tuple(t) => &t.data_map,
        other => {
            return Err(format!(
                "Expected Tuple/OptionalSome/OptionalNone for pox-addr, got {}",
                short_type_name(other)
            ))
        }
    };

    let version_bytes = match get_tuple_field(addr_tuple, "version")? {
        UpstreamValue::Sequence(SequenceData::Buffer(b)) => b.data.clone(),
        other => {
            return Err(format!(
                "Expected Buffer for pox-addr version, got {}",
                short_type_name(other)
            ))
        }
    };

    let hashbytes = match get_tuple_field(addr_tuple, "hashbytes")? {
        UpstreamValue::Sequence(SequenceData::Buffer(b)) => b.data.clone(),
        other => {
            return Err(format!(
                "Expected Buffer for pox-addr hashbytes, got {}",
                short_type_name(other)
            ))
        }
    };

    let mut raw = Vec::with_capacity(version_bytes.len() + hashbytes.len());
    raw.extend_from_slice(&version_bytes);
    raw.extend_from_slice(&hashbytes);
    let raw_hex = encode_hex(&raw).to_string();

    let version = if version_bytes.is_empty() {
        return Ok((None, Some(raw_hex)));
    } else {
        version_bytes[0]
    };

    let btc_addr = pox_address_to_btc_address(version, &hashbytes, network).ok();

    Ok((btc_addr, Some(raw_hex)))
}

/// Short human-readable name for an upstream value's outer constructor, used
/// in error messages. We don't try to reproduce the full Clarity type name
/// (that's what `crate::upstream::clarity_value::neon_encoder::type_signature_string`
/// is for); these messages just need to be diagnostic.
fn short_type_name(val: &UpstreamValue) -> &'static str {
    match val {
        UpstreamValue::Int(_) => "Int",
        UpstreamValue::UInt(_) => "UInt",
        UpstreamValue::Bool(_) => "Bool",
        UpstreamValue::Sequence(SequenceData::Buffer(_)) => "Buffer",
        UpstreamValue::Sequence(SequenceData::List(_)) => "List",
        UpstreamValue::Sequence(SequenceData::String(CharType::ASCII(_))) => "StringASCII",
        UpstreamValue::Sequence(SequenceData::String(CharType::UTF8(_))) => "StringUTF8",
        UpstreamValue::Principal(PrincipalData::Standard(_)) => "PrincipalStandard",
        UpstreamValue::Principal(PrincipalData::Contract(_)) => "PrincipalContract",
        UpstreamValue::Tuple(_) => "Tuple",
        UpstreamValue::Optional(opt) => {
            if opt.data.is_none() {
                "OptionalNone"
            } else {
                "OptionalSome"
            }
        }
        UpstreamValue::Response(r) => {
            if r.committed {
                "ResponseOk"
            } else {
                "ResponseErr"
            }
        }
        UpstreamValue::CallableContract(_) => "CallableContract",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clarity::vm::types::{BuffData, OptionalData, ResponseData};

    fn make_response_err(inner: UpstreamValue) -> UpstreamValue {
        UpstreamValue::Response(ResponseData {
            committed: false,
            data: Box::new(inner),
        })
    }

    #[test]
    fn test_response_err_returns_none() {
        let cv = make_response_err(UpstreamValue::UInt(1));
        let result = decode_pox_synthetic_event(&cv, StacksNetwork::Mainnet).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_non_response_errors() {
        let cv = UpstreamValue::UInt(42);
        let result = decode_pox_synthetic_event(&cv, StacksNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_uint_works() {
        let cv = UpstreamValue::UInt(12345);
        assert_eq!(extract_uint(&cv).unwrap(), 12345);
    }

    #[test]
    fn test_extract_optional_uint_none() {
        assert_eq!(extract_optional_uint(None).unwrap(), None);
        let cv = UpstreamValue::Optional(OptionalData { data: None });
        assert_eq!(extract_optional_uint(Some(&cv)).unwrap(), None);
    }

    #[test]
    fn test_extract_optional_uint_some() {
        let cv = UpstreamValue::Optional(OptionalData {
            data: Some(Box::new(UpstreamValue::UInt(999))),
        });
        assert_eq!(extract_optional_uint(Some(&cv)).unwrap(), Some(999));
    }

    #[test]
    fn test_extract_optional_buffer_hex() {
        let cv = UpstreamValue::Sequence(SequenceData::Buffer(BuffData {
            data: vec![0xab, 0xcd],
        }));
        assert_eq!(
            extract_optional_buffer_hex(Some(&cv)).unwrap(),
            Some("0xabcd".to_string())
        );

        let cv_none = UpstreamValue::Optional(OptionalData { data: None });
        assert_eq!(extract_optional_buffer_hex(Some(&cv_none)).unwrap(), None);

        assert_eq!(extract_optional_buffer_hex(None).unwrap(), None);
    }
}
