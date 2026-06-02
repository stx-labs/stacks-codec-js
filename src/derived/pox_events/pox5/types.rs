//! Event-shape types emitted by the PoX-5 Clarity contract.
//!
//! PoX-5 events come from explicit `(print { topic: "...", ... })` calls
//! in the contract source, so each one arrives as a flat Clarity tuple with
//! a `topic` ASCII string field plus event-specific data.
//!
//! Source contract:
//! <https://github.com/stacks-network/stacks-core/blob/main/stackslib/src/chainstate/stacks/boot/pox-5.clar>

/// All synthetic event names emitted by the PoX-5 Clarity contract. Each
/// variant maps 1:1 to a `(print { topic: "...", ... })` site in the source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pox5EventName {
    SetupBond,
    AddToAllowlist,
    RegisterForBond,
    UpdateBondRegistration,
    RegisterSigner,
    Stake,
    StakeUpdate,
    AnnounceL1EarlyExit,
    UnstakeSbtc,
    Unstake,
    CalculateRewards,
    BondDistribution,
    ClaimRewards,
}

impl Pox5EventName {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "setup-bond" => Some(Pox5EventName::SetupBond),
            "add-to-allowlist" => Some(Pox5EventName::AddToAllowlist),
            "register-for-bond" => Some(Pox5EventName::RegisterForBond),
            "update-bond-registration" => Some(Pox5EventName::UpdateBondRegistration),
            "register-signer" => Some(Pox5EventName::RegisterSigner),
            "stake" => Some(Pox5EventName::Stake),
            "stake-update" => Some(Pox5EventName::StakeUpdate),
            "announce-l1-early-exit" => Some(Pox5EventName::AnnounceL1EarlyExit),
            "unstake-sbtc" => Some(Pox5EventName::UnstakeSbtc),
            "unstake" => Some(Pox5EventName::Unstake),
            "calculate-rewards" => Some(Pox5EventName::CalculateRewards),
            "bond-distribution" => Some(Pox5EventName::BondDistribution),
            "claim-rewards" => Some(Pox5EventName::ClaimRewards),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Pox5EventName::SetupBond => "setup-bond",
            Pox5EventName::AddToAllowlist => "add-to-allowlist",
            Pox5EventName::RegisterForBond => "register-for-bond",
            Pox5EventName::UpdateBondRegistration => "update-bond-registration",
            Pox5EventName::RegisterSigner => "register-signer",
            Pox5EventName::Stake => "stake",
            Pox5EventName::StakeUpdate => "stake-update",
            Pox5EventName::AnnounceL1EarlyExit => "announce-l1-early-exit",
            Pox5EventName::UnstakeSbtc => "unstake-sbtc",
            Pox5EventName::Unstake => "unstake",
            Pox5EventName::CalculateRewards => "calculate-rewards",
            Pox5EventName::BondDistribution => "bond-distribution",
            Pox5EventName::ClaimRewards => "claim-rewards",
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
/// contract's `kebab-case` keys (rewritten to `snake_case` for the JS layer
/// by the Neon encoder).
#[derive(Debug, Clone)]
pub enum Pox5EventData {
    /// Logged by `setup-bond` when a new protocol bond is created.
    SetupBond {
        bond_index: u128,
        target_rate: u128,
        stx_value_ratio: u128,
        min_ustx_ratio: u128,
        /// `(buff 683)` — opaque early-unlock authorization script (e.g. a
        /// pubkey + `OP_CHECKSIGVERIFY`, or an M-of-N multisig template).
        /// Surfaced as a hex string; callers decode per their protocol.
        early_unlock_bytes: String,
        /// Principal allowed to call `announce-l1-early-exit` for stakers in
        /// this bond.
        early_unlock_admin: String,
        first_reward_cycle: u128,
        bond_start_height: u128,
        unlock_cycle: u128,
        unlock_burn_height: u128,
    },

    /// Logged by `add-staker-to-bond` when a staker is added to a bond's
    /// allowlist.
    AddToAllowlist {
        staker: String,
        max_sats: u128,
        bond_index: u128,
    },

    /// Logged by `register-for-bond` when a participant locks (sBTC or L1
    /// BTC proof) into a bond period.
    RegisterForBond {
        signer: String,
        staker: String,
        amount_ustx: u128,
        sats_total: u128,
        bond_index: u128,
        first_reward_cycle: u128,
        unlock_burn_height: u128,
        unlock_cycle: u128,
        is_l1_lock: bool,
    },

    /// Logged by `update-bond-registration` when a participant switches
    /// signers mid-bond.
    UpdateBondRegistration {
        staker: String,
        signer: String,
        old_signer: String,
        bond_index: u128,
        amount_ustx: u128,
        amount_sats: u128,
        first_reward_cycle: u128,
        num_cycles: u128,
        is_l1_lock: bool,
    },

    /// Logged by `register-signer` when a signer registers its key with the
    /// PoX-5 contract.
    RegisterSigner {
        signer: String,
        /// `(buff 33)` — compressed secp256k1 public key.
        signer_key: String,
    },

    /// Logged by `stake` when a participant locks STX for a number of cycles.
    Stake {
        signer: String,
        staker: String,
        amount_ustx: u128,
        num_cycles: u128,
        first_reward_cycle: u128,
        unlock_burn_height: u128,
        unlock_cycle: u128,
    },

    /// Logged by `stake-update` when a participant changes signers, extends
    /// their lock, or increases their locked STX.
    StakeUpdate {
        unlock_burn_height: u128,
        staker: String,
        signer: String,
        old_signer: String,
        prev_unlock_height: u128,
        unlock_cycle: u128,
        num_cycles: u128,
        amount_ustx: u128,
        amount_increase: u128,
        cycles_to_extend: u128,
    },

    /// Logged by `announce-l1-early-exit` when an early-unlock admin releases
    /// an L1-locked staker's shares.
    AnnounceL1EarlyExit {
        staker: String,
        signer: String,
        bond_index: u128,
        amount_sats_released: u128,
    },

    /// Logged by `unstake-sbtc` when a bond participant withdraws part or
    /// all of their locked sBTC.
    UnstakeSbtc {
        staker: String,
        signer: String,
        bond_index: u128,
        amount_withdrawn_sats: u128,
        new_amount_sats: u128,
    },

    /// Logged by `unstake` when a staker requests STX unlock at the end of
    /// the current cycle.
    Unstake {
        staker: String,
        signer: String,
        amount_ustx: u128,
        first_reward_cycle: u128,
        unlock_cycle: u128,
        unlock_burn_height: u128,
    },

    /// Logged by `calculate-rewards`. The contract emits this topic **twice**
    /// per call:
    ///
    /// - Phase 1 (pre-distribution): carries `stranded_staker_cut` but no
    ///   `new_reserve`. Fired before the reserve and accounting state are
    ///   updated; useful for inspecting whether the staker cut got folded
    ///   into the reserve because no STX was staked.
    /// - Phase 2 (post-distribution): carries `new_reserve` but no
    ///   `stranded_staker_cut`. Fired after state is committed and matches
    ///   the value returned to the caller.
    ///
    /// To keep a single Rust/JS variant for both phases, the two
    /// phase-specific fields are surfaced as optionals — exactly one will be
    /// `Some` per event.
    CalculateRewards {
        bond_periods: Vec<u128>,
        calculation_height: u128,
        remaining_rewards: u128,
        accrued_rewards: u128,
        /// Phase-2 only.
        new_reserve: Option<u128>,
        /// Phase-1 only.
        stranded_staker_cut: Option<u128>,
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
        stx_rewards: ClaimRewardsInfo,
        bond_rewards: Vec<BondRewardsInfo>,
        bond_totals: u128,
        total_rewards: u128,
    },
}

/// The `stx-rewards` sub-tuple from `claim-rewards` events. Same shape as
/// the entries inside `bond-rewards`, minus the `bond_index`.
#[derive(Debug, Clone)]
pub struct ClaimRewardsInfo {
    pub earned: u128,
    pub rewards_per_token: u128,
}

/// One entry in the `bond-rewards` list from `claim-rewards` events.
#[derive(Debug, Clone)]
pub struct BondRewardsInfo {
    pub earned: u128,
    pub rewards_per_token: u128,
    pub bond_index: u128,
}
