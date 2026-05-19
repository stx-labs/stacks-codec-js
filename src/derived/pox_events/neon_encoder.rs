use neon::prelude::*;

use super::types::*;

/// Serialize a `PoxSyntheticEvent` into a Neon JS object.
/// All u128 values are string-quoted. Optional fields become `null`.
pub fn encode_pox_event<'a>(
    cx: &mut FunctionContext<'a>,
    event: &PoxSyntheticEvent,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    // Base fields
    set_string(cx, &obj, "stacker", &event.base.stacker)?;
    set_u128_string(cx, &obj, "locked", event.base.locked)?;
    set_u128_string(cx, &obj, "balance", event.base.balance)?;
    set_u128_string(
        cx,
        &obj,
        "burnchain_unlock_height",
        event.base.burnchain_unlock_height,
    )?;
    set_optional_string(cx, &obj, "pox_addr", event.base.pox_addr.as_deref())?;
    set_optional_string(cx, &obj, "pox_addr_raw", event.base.pox_addr_raw.as_deref())?;

    // Event name
    set_string(cx, &obj, "name", event.name.as_str())?;

    // Event-specific data
    let data_obj = cx.empty_object();
    encode_event_data(cx, &data_obj, &event.data)?;
    obj.set(cx, "data", data_obj)?;

    Ok(obj)
}

fn encode_event_data<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    data: &PoxEventData,
) -> NeonResult<()> {
    match data {
        PoxEventData::HandleUnlock {
            first_cycle_locked,
            first_unlocked_cycle,
        } => {
            set_u128_string(cx, obj, "first_cycle_locked", *first_cycle_locked)?;
            set_u128_string(cx, obj, "first_unlocked_cycle", *first_unlocked_cycle)?;
        }
        PoxEventData::StackStx {
            lock_amount,
            lock_period,
            start_burn_height,
            unlock_burn_height,
            signer_key,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "lock_amount", *lock_amount)?;
            set_u128_string(cx, obj, "lock_period", *lock_period)?;
            set_u128_string(cx, obj, "start_burn_height", *start_burn_height)?;
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_optional_string(cx, obj, "signer_key", signer_key.as_deref())?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::StackIncrease {
            increase_by,
            total_locked,
            signer_key,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "increase_by", *increase_by)?;
            set_u128_string(cx, obj, "total_locked", *total_locked)?;
            set_optional_string(cx, obj, "signer_key", signer_key.as_deref())?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::StackExtend {
            extend_count,
            unlock_burn_height,
            signer_key,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "extend_count", *extend_count)?;
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_optional_string(cx, obj, "signer_key", signer_key.as_deref())?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::DelegateStx {
            amount_ustx,
            delegate_to,
            unlock_burn_height,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_string(cx, obj, "delegate_to", delegate_to)?;
            set_optional_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::DelegateStackStx {
            lock_amount,
            unlock_burn_height,
            start_burn_height,
            lock_period,
            delegator,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "lock_amount", *lock_amount)?;
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_u128_string(cx, obj, "start_burn_height", *start_burn_height)?;
            set_u128_string(cx, obj, "lock_period", *lock_period)?;
            set_string(cx, obj, "delegator", delegator)?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::DelegateStackIncrease {
            increase_by,
            total_locked,
            delegator,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "increase_by", *increase_by)?;
            set_u128_string(cx, obj, "total_locked", *total_locked)?;
            set_string(cx, obj, "delegator", delegator)?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::DelegateStackExtend {
            unlock_burn_height,
            extend_count,
            delegator,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_u128_string(cx, obj, "extend_count", *extend_count)?;
            set_string(cx, obj, "delegator", delegator)?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::StackAggregationCommit {
            reward_cycle,
            amount_ustx,
            signer_key,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "reward_cycle", *reward_cycle)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_optional_string(cx, obj, "signer_key", signer_key.as_deref())?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::StackAggregationCommitIndexed {
            reward_cycle,
            amount_ustx,
            signer_key,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "reward_cycle", *reward_cycle)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_optional_string(cx, obj, "signer_key", signer_key.as_deref())?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::StackAggregationIncrease {
            reward_cycle,
            amount_ustx,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_u128_string(cx, obj, "reward_cycle", *reward_cycle)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
        PoxEventData::RevokeDelegateStx {
            delegate_to,
            end_cycle_id,
            start_cycle_id,
        } => {
            set_string(cx, obj, "delegate_to", delegate_to)?;
            set_optional_u128_string(cx, obj, "end_cycle_id", *end_cycle_id)?;
            set_optional_u128_string(cx, obj, "start_cycle_id", *start_cycle_id)?;
        }
    }
    Ok(())
}

// ─── Neon helper functions ──────────────────────────────────────────────────

fn set_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: &str,
) -> NeonResult<()> {
    let val = cx.string(value);
    obj.set(cx, key, val)?;
    Ok(())
}

fn set_u128_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: u128,
) -> NeonResult<()> {
    let val = cx.string(value.to_string());
    obj.set(cx, key, val)?;
    Ok(())
}

fn set_optional_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: Option<&str>,
) -> NeonResult<()> {
    match value {
        Some(s) => {
            let val = cx.string(s);
            obj.set(cx, key, val)?;
        }
        None => {
            let val = cx.null();
            obj.set(cx, key, val)?;
        }
    }
    Ok(())
}

fn set_optional_u128_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: Option<u128>,
) -> NeonResult<()> {
    match value {
        Some(v) => {
            let val = cx.string(v.to_string());
            obj.set(cx, key, val)?;
        }
        None => {
            let val = cx.null();
            obj.set(cx, key, val)?;
        }
    }
    Ok(())
}
