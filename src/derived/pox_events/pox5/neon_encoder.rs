//! Neon (JS) serialization for decoded PoX-5 synthetic events.
//!
//! The JS-facing shape mirrors PoX-4: a top-level `{ name, data }` object,
//! where `data` carries the event-specific fields. PoX-4's per-event base
//! fields (`stacker` / `locked` / `balance` / `burnchain_unlock_height` /
//! `pox_addr` / `pox_addr_raw`) are intentionally absent because PoX-5
//! events don't carry them — only `add-to-allowlist` and
//! `update-claimable-rewards` reference a principal, and they expose it
//! inside `data` (`staker` and `signer` respectively).

use neon::prelude::*;

use super::super::neon_helpers::{set_bool, set_string, set_u128_array, set_u128_string};
use super::types::*;

/// Serialize a [`Pox5SyntheticEvent`] into a JS object of shape
/// `{ pox_version: 'pox5', name: string, data: { ... } }`.
pub fn encode_pox5_event<'a>(
    cx: &mut FunctionContext<'a>,
    event: &Pox5SyntheticEvent,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    // Discriminant for JS callers — pair with `Pox5Event['pox_version']` in `index.ts`.
    set_string(cx, &obj, "pox_version", "pox5")?;

    set_string(cx, &obj, "name", event.name.as_str())?;

    let data_obj = cx.empty_object();
    encode_event_data(cx, &data_obj, &event.data)?;
    obj.set(cx, "data", data_obj)?;

    Ok(obj)
}

fn encode_event_data<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    data: &Pox5EventData,
) -> NeonResult<()> {
    match data {
        Pox5EventData::AddToAllowlist {
            staker,
            max_sats,
            bond_index,
        } => {
            set_string(cx, obj, "staker", staker)?;
            set_u128_string(cx, obj, "max_sats", *max_sats)?;
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
        }
        Pox5EventData::CalculateRewards {
            bond_periods,
            calculation_height,
            remaining_rewards,
            accrued_rewards,
            stx_staker_rewards,
            stx_cycle,
            cycle_staked_ustx,
            next_rewards_per_ustx,
        } => {
            set_u128_array(cx, obj, "bond_periods", bond_periods)?;
            set_u128_string(cx, obj, "calculation_height", *calculation_height)?;
            set_u128_string(cx, obj, "remaining_rewards", *remaining_rewards)?;
            set_u128_string(cx, obj, "accrued_rewards", *accrued_rewards)?;
            set_u128_string(cx, obj, "stx_staker_rewards", *stx_staker_rewards)?;
            set_u128_string(cx, obj, "stx_cycle", *stx_cycle)?;
            set_u128_string(cx, obj, "cycle_staked_ustx", *cycle_staked_ustx)?;
            set_u128_string(cx, obj, "next_rewards_per_ustx", *next_rewards_per_ustx)?;
        }
        Pox5EventData::BondDistribution {
            bond_index,
            target_yield,
            earned,
        } => {
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "target_yield", *target_yield)?;
            set_u128_string(cx, obj, "earned", *earned)?;
        }
        Pox5EventData::ClaimRewards {
            stx_rewards,
            bond_rewards,
            bond_totals,
            total_rewards,
        } => {
            let stx_obj = cx.empty_object();
            set_u128_string(cx, &stx_obj, "rewards_paid", stx_rewards.rewards_paid)?;
            set_u128_string(cx, &stx_obj, "rewards_pending", stx_rewards.rewards_pending)?;
            set_u128_string(cx, &stx_obj, "shares_staked", stx_rewards.shares_staked)?;
            set_u128_string(
                cx,
                &stx_obj,
                "rewards_per_share",
                stx_rewards.rewards_per_share,
            )?;
            obj.set(cx, "stx_rewards", stx_obj)?;

            let bond_arr = JsArray::new(cx, bond_rewards.len());
            for (i, entry) in bond_rewards.iter().enumerate() {
                let entry_obj = cx.empty_object();
                set_u128_string(cx, &entry_obj, "rewards_paid", entry.rewards_paid)?;
                set_u128_string(cx, &entry_obj, "rewards_pending", entry.rewards_pending)?;
                set_u128_string(cx, &entry_obj, "shares_staked", entry.shares_staked)?;
                set_u128_string(cx, &entry_obj, "rewards_per_share", entry.rewards_per_share)?;
                set_u128_string(cx, &entry_obj, "bond_index", entry.bond_index)?;
                bond_arr.set(cx, i as u32, entry_obj)?;
            }
            obj.set(cx, "bond_rewards", bond_arr)?;

            set_u128_string(cx, obj, "bond_totals", *bond_totals)?;
            set_u128_string(cx, obj, "total_rewards", *total_rewards)?;
        }
        Pox5EventData::UpdateClaimableRewards {
            rewards_pending,
            rewards_paid,
            index,
            signer,
            is_bond,
        } => {
            set_u128_string(cx, obj, "rewards_pending", *rewards_pending)?;
            set_u128_string(cx, obj, "rewards_paid", *rewards_paid)?;
            set_u128_string(cx, obj, "index", *index)?;
            set_string(cx, obj, "signer", signer)?;
            set_bool(cx, obj, "is_bond", *is_bond)?;
        }
    }
    Ok(())
}
