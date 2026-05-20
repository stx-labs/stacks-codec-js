//! Event-shape types emitted by the PoX-5 Clarity contract.
//!
//! Unlike PoX-4 (where synthetic events are node-emitted `Response(Ok(...))`
//! tuples), PoX-5 events come from explicit `(print ...)` calls in the
//! contract and look like flat tuples with a `topic` ASCII string field plus
//! event-specific data.
//!
//! See the upstream contract:
//! <https://github.com/stacks-network/stacks-core/blob/pox-wf-integration/stackslib/src/chainstate/stacks/boot/pox-5.clar>

/// All synthetic event names emitted by the PoX-5 Clarity contract.
///
/// Each variant maps 1:1 to a `(print { topic: "...", ... })` site.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pox5EventName {
    AddToAllowlist,
    CalculateRewards,
    BondDistribution,
    ClaimRewards,
    UpdateClaimableRewards,
}

impl Pox5EventName {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "add-to-allowlist" => Some(Pox5EventName::AddToAllowlist),
            "calculate-rewards" => Some(Pox5EventName::CalculateRewards),
            "bond-distribution" => Some(Pox5EventName::BondDistribution),
            "claim-rewards" => Some(Pox5EventName::ClaimRewards),
            "update-claimable-rewards" => Some(Pox5EventName::UpdateClaimableRewards),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Pox5EventName::AddToAllowlist => "add-to-allowlist",
            Pox5EventName::CalculateRewards => "calculate-rewards",
            Pox5EventName::BondDistribution => "bond-distribution",
            Pox5EventName::ClaimRewards => "claim-rewards",
            Pox5EventName::UpdateClaimableRewards => "update-claimable-rewards",
        }
    }
}

/// A fully decoded PoX-5 synthetic event.
#[derive(Debug, Clone)]
pub struct Pox5SyntheticEvent {
    pub name: Pox5EventName,
    pub data: Pox5EventData,
}

/// Event-specific data payload for PoX-5. Field names mirror the Clarity
/// contract's `kebab-case` keys (rewritten to `snake_case` on the JS side
/// by the Neon encoder).
#[derive(Debug, Clone)]
pub enum Pox5EventData {
    /// Logged by `add-staker-to-bond` when a staker is added to a bond's
    /// allowlist.
    AddToAllowlist {
        staker: String,
        max_sats: u128,
        bond_index: u128,
    },

    /// Logged by `calculate-rewards` after a per-cycle reward calculation.
    CalculateRewards {
        bond_periods: Vec<u128>,
        calculation_height: u128,
        remaining_rewards: u128,
        accrued_rewards: u128,
        stx_staker_rewards: u128,
        stx_cycle: u128,
        cycle_staked_ustx: u128,
        next_rewards_per_ustx: u128,
    },

    /// Logged by `calculate-bond-rewards` for each bond period processed
    /// inside a `calculate-rewards` fold.
    BondDistribution {
        bond_index: u128,
        target_yield: u128,
        earned: u128,
    },

    /// Logged by `claim-rewards` when a signer claims accrued rewards.
    ClaimRewards {
        stx_rewards: StxRewardsInfo,
        bond_rewards: Vec<BondRewardsInfo>,
        bond_totals: u128,
        total_rewards: u128,
    },

    /// Logged by `update-claimable-rewards` (called from `claim-rewards` for
    /// each cycle / bond index being settled).
    UpdateClaimableRewards {
        rewards_pending: u128,
        rewards_paid: u128,
        index: u128,
        signer: String,
        is_bond: bool,
    },
}

/// The `stx-rewards` sub-tuple from `claim-rewards` events.
#[derive(Debug, Clone)]
pub struct StxRewardsInfo {
    pub rewards_paid: u128,
    pub rewards_pending: u128,
    pub shares_staked: u128,
    pub rewards_per_share: u128,
}

/// One entry in the `bond-rewards` list from `claim-rewards` events.
#[derive(Debug, Clone)]
pub struct BondRewardsInfo {
    pub rewards_paid: u128,
    pub rewards_pending: u128,
    pub shares_staked: u128,
    pub rewards_per_share: u128,
    pub bond_index: u128,
}
