//! Neon (JS) serialization for decoded PoX-5 synthetic events.
//!
//! The JS-facing shape:
//! `{ pox_version: 'pox5', name: <string>, data: { ... } }`.
//!
//! Field naming converts the contract's `kebab-case` keys to `snake_case`.
//! All `u128` values are emitted as JS strings (numeric precision would be
//! lost past 2^53). Principal fields are c32-encoded strings. Buffer fields
//! are `0x`-prefixed hex strings.

use neon::prelude::*;

use super::super::neon_helpers::{
    set_bool, set_optional_u128_string, set_string, set_u128_array, set_u128_string,
};
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
        Pox5EventData::SetBondAdmin {
            old_admin,
            new_admin,
        } => {
            set_string(cx, obj, "old_admin", old_admin)?;
            set_string(cx, obj, "new_admin", new_admin)?;
        }

        Pox5EventData::SetupBond {
            bond_index,
            target_rate,
            stx_value_ratio,
            min_ustx_ratio,
            early_unlock_bytes,
            first_reward_cycle,
            bond_start_height,
            unlock_cycle,
            unlock_burn_height,
        } => {
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "target_rate", *target_rate)?;
            set_u128_string(cx, obj, "stx_value_ratio", *stx_value_ratio)?;
            set_u128_string(cx, obj, "min_ustx_ratio", *min_ustx_ratio)?;
            set_string(cx, obj, "early_unlock_bytes", early_unlock_bytes)?;
            set_u128_string(cx, obj, "first_reward_cycle", *first_reward_cycle)?;
            set_u128_string(cx, obj, "bond_start_height", *bond_start_height)?;
            set_u128_string(cx, obj, "unlock_cycle", *unlock_cycle)?;
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
        }

        Pox5EventData::AddToAllowlist {
            staker,
            max_sats,
            bond_index,
        } => {
            set_string(cx, obj, "staker", staker)?;
            set_u128_string(cx, obj, "max_sats", *max_sats)?;
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
        }

        Pox5EventData::RegisterForBond {
            signer,
            staker,
            amount_ustx,
            sats_total,
            bond_index,
            first_reward_cycle,
            unlock_burn_height,
            unlock_cycle,
            is_l1_lock,
            btc_lockup,
        } => {
            set_string(cx, obj, "signer", signer)?;
            set_string(cx, obj, "staker", staker)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_u128_string(cx, obj, "sats_total", *sats_total)?;
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "first_reward_cycle", *first_reward_cycle)?;
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_u128_string(cx, obj, "unlock_cycle", *unlock_cycle)?;
            set_bool(cx, obj, "is_l1_lock", *is_l1_lock)?;

            let lockup_obj = cx.empty_object();
            set_string(cx, &lockup_obj, "type", &btc_lockup.lockup_type)?;
            match &btc_lockup.txs {
                Some(txs) => {
                    let txs_arr = JsArray::new(cx, txs.len());
                    for (i, tx) in txs.iter().enumerate() {
                        let tx_obj = cx.empty_object();
                        set_string(cx, &tx_obj, "txid", &tx.txid)?;
                        set_u128_string(cx, &tx_obj, "output_index", tx.output_index)?;
                        txs_arr.set(cx, i as u32, tx_obj)?;
                    }
                    lockup_obj.set(cx, "txs", txs_arr)?;
                }
                None => {
                    let null_val = cx.null();
                    lockup_obj.set(cx, "txs", null_val)?;
                }
            }
            obj.set(cx, "btc_lockup", lockup_obj)?;
        }

        Pox5EventData::UpdateBondRegistration {
            staker,
            signer,
            old_signer,
            bond_index,
            amount_ustx,
            amount_sats,
            first_reward_cycle,
            num_cycles,
            is_l1_lock,
        } => {
            set_string(cx, obj, "staker", staker)?;
            set_string(cx, obj, "signer", signer)?;
            set_string(cx, obj, "old_signer", old_signer)?;
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_u128_string(cx, obj, "amount_sats", *amount_sats)?;
            set_u128_string(cx, obj, "first_reward_cycle", *first_reward_cycle)?;
            set_u128_string(cx, obj, "num_cycles", *num_cycles)?;
            set_bool(cx, obj, "is_l1_lock", *is_l1_lock)?;
        }

        Pox5EventData::RegisterSigner { signer, signer_key } => {
            set_string(cx, obj, "signer", signer)?;
            set_string(cx, obj, "signer_key", signer_key)?;
        }

        Pox5EventData::Stake {
            signer,
            staker,
            amount_ustx,
            num_cycles,
            first_reward_cycle,
            unlock_burn_height,
            unlock_cycle,
        } => {
            set_string(cx, obj, "signer", signer)?;
            set_string(cx, obj, "staker", staker)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_u128_string(cx, obj, "num_cycles", *num_cycles)?;
            set_u128_string(cx, obj, "first_reward_cycle", *first_reward_cycle)?;
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_u128_string(cx, obj, "unlock_cycle", *unlock_cycle)?;
        }

        Pox5EventData::StakeUpdate {
            unlock_burn_height,
            staker,
            signer,
            old_signer,
            prev_unlock_height,
            unlock_cycle,
            num_cycles,
            amount_ustx,
            amount_increase,
            cycles_to_extend,
        } => {
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
            set_string(cx, obj, "staker", staker)?;
            set_string(cx, obj, "signer", signer)?;
            set_string(cx, obj, "old_signer", old_signer)?;
            set_u128_string(cx, obj, "prev_unlock_height", *prev_unlock_height)?;
            set_u128_string(cx, obj, "unlock_cycle", *unlock_cycle)?;
            set_u128_string(cx, obj, "num_cycles", *num_cycles)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_u128_string(cx, obj, "amount_increase", *amount_increase)?;
            set_u128_string(cx, obj, "cycles_to_extend", *cycles_to_extend)?;
        }

        Pox5EventData::AnnounceL1EarlyExit {
            staker,
            signer,
            bond_index,
            amount_sats_released,
        } => {
            set_string(cx, obj, "staker", staker)?;
            set_string(cx, obj, "signer", signer)?;
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "amount_sats_released", *amount_sats_released)?;
        }

        Pox5EventData::UnstakeSbtc {
            staker,
            signer,
            bond_index,
            amount_withdrawn_sats,
            new_amount_sats,
        } => {
            set_string(cx, obj, "staker", staker)?;
            set_string(cx, obj, "signer", signer)?;
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "amount_withdrawn_sats", *amount_withdrawn_sats)?;
            set_u128_string(cx, obj, "new_amount_sats", *new_amount_sats)?;
        }

        Pox5EventData::Unstake {
            staker,
            signer,
            amount_ustx,
            first_reward_cycle,
            unlock_cycle,
            unlock_burn_height,
        } => {
            set_string(cx, obj, "staker", staker)?;
            set_string(cx, obj, "signer", signer)?;
            set_u128_string(cx, obj, "amount_ustx", *amount_ustx)?;
            set_u128_string(cx, obj, "first_reward_cycle", *first_reward_cycle)?;
            set_u128_string(cx, obj, "unlock_cycle", *unlock_cycle)?;
            set_u128_string(cx, obj, "unlock_burn_height", *unlock_burn_height)?;
        }

        Pox5EventData::CalculateRewards {
            bond_periods,
            calculation_height,
            gross_accrued_rewards,
            total_bond_rewards,
            reserve_deposit,
            reserve_balance,
            stx_cycle,
            total_stx_staker_rewards,
            cycle_staked_ustx,
            accrued_rewards_per_ustx,
            cumulative_rewards_per_ustx,
        } => {
            set_u128_array(cx, obj, "bond_periods", bond_periods)?;
            set_u128_string(cx, obj, "calculation_height", *calculation_height)?;
            set_u128_string(cx, obj, "gross_accrued_rewards", *gross_accrued_rewards)?;
            set_u128_string(cx, obj, "total_bond_rewards", *total_bond_rewards)?;
            set_u128_string(cx, obj, "reserve_deposit", *reserve_deposit)?;
            set_u128_string(cx, obj, "reserve_balance", *reserve_balance)?;
            set_u128_string(cx, obj, "stx_cycle", *stx_cycle)?;
            set_u128_string(
                cx,
                obj,
                "total_stx_staker_rewards",
                *total_stx_staker_rewards,
            )?;
            set_u128_string(cx, obj, "cycle_staked_ustx", *cycle_staked_ustx)?;
            set_u128_string(
                cx,
                obj,
                "accrued_rewards_per_ustx",
                *accrued_rewards_per_ustx,
            )?;
            set_u128_string(
                cx,
                obj,
                "cumulative_rewards_per_ustx",
                *cumulative_rewards_per_ustx,
            )?;
        }

        Pox5EventData::BondDistribution {
            bond_index,
            target_yield,
            bond_rewards,
            bond_staked_sats,
            accrued_rewards_per_sat,
            cumulative_rewards_per_sat,
        } => {
            set_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "target_yield", *target_yield)?;
            set_u128_string(cx, obj, "bond_rewards", *bond_rewards)?;
            set_u128_string(cx, obj, "bond_staked_sats", *bond_staked_sats)?;
            set_u128_string(cx, obj, "accrued_rewards_per_sat", *accrued_rewards_per_sat)?;
            set_u128_string(
                cx,
                obj,
                "cumulative_rewards_per_sat",
                *cumulative_rewards_per_sat,
            )?;
        }

        Pox5EventData::ClaimRewards {
            signer_manager,
            reward_cycle,
            stx_rewards,
            bond_rewards,
            bond_totals,
            total_rewards,
        } => {
            set_string(cx, obj, "signer_manager", signer_manager)?;
            set_u128_string(cx, obj, "reward_cycle", *reward_cycle)?;
            let stx_obj = cx.empty_object();
            set_u128_string(cx, &stx_obj, "earned", stx_rewards.earned)?;
            set_u128_string(
                cx,
                &stx_obj,
                "rewards_per_token",
                stx_rewards.rewards_per_token,
            )?;
            obj.set(cx, "stx_rewards", stx_obj)?;

            let bond_arr = JsArray::new(cx, bond_rewards.len());
            for (i, entry) in bond_rewards.iter().enumerate() {
                let entry_obj = cx.empty_object();
                set_u128_string(cx, &entry_obj, "earned", entry.earned)?;
                set_u128_string(cx, &entry_obj, "rewards_per_token", entry.rewards_per_token)?;
                set_u128_string(cx, &entry_obj, "bond_index", entry.bond_index)?;
                bond_arr.set(cx, i as u32, entry_obj)?;
            }
            obj.set(cx, "bond_rewards", bond_arr)?;

            set_u128_string(cx, obj, "bond_totals", *bond_totals)?;
            set_u128_string(cx, obj, "total_rewards", *total_rewards)?;
        }

        Pox5EventData::ClaimStakerRewardsForSigner {
            signer_manager,
            staker,
            reward_cycle,
            bond_index,
            rewards_claimed,
        } => {
            set_string(cx, obj, "signer_manager", signer_manager)?;
            set_string(cx, obj, "staker", staker)?;
            set_u128_string(cx, obj, "reward_cycle", *reward_cycle)?;
            set_optional_u128_string(cx, obj, "bond_index", *bond_index)?;
            set_u128_string(cx, obj, "rewards_claimed", *rewards_claimed)?;
        }

        Pox5EventData::GrantSignerKey {
            signer_key,
            signer_manager,
            auth_id,
        } => {
            set_string(cx, obj, "signer_key", signer_key)?;
            set_string(cx, obj, "signer_manager", signer_manager)?;
            set_u128_string(cx, obj, "auth_id", *auth_id)?;
        }

        Pox5EventData::RevokeSignerGrant {
            signer_key,
            signer_manager,
        } => {
            set_string(cx, obj, "signer_key", signer_key)?;
            set_string(cx, obj, "signer_manager", signer_manager)?;
        }

        Pox5EventData::DisallowContractCaller {
            sender,
            contract_caller,
        } => {
            set_string(cx, obj, "sender", sender)?;
            set_string(cx, obj, "contract_caller", contract_caller)?;
        }

        Pox5EventData::AllowContractCaller {
            sender,
            contract_caller,
            until_burn_ht,
        } => {
            set_string(cx, obj, "sender", sender)?;
            set_string(cx, obj, "contract_caller", contract_caller)?;
            set_optional_u128_string(cx, obj, "until_burn_ht", *until_burn_ht)?;
        }
    }
    Ok(())
}
