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
    SetBondAdmin,
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
    ClaimStakerRewardsForSigner,
    GrantSignerKey,
    RevokeSignerGrant,
    DisallowContractCaller,
    AllowContractCaller,
}

impl Pox5EventName {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "set-bond-admin" => Some(Pox5EventName::SetBondAdmin),
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
            "claim-staker-rewards-for-signer" => Some(Pox5EventName::ClaimStakerRewardsForSigner),
            "grant-signer-key" => Some(Pox5EventName::GrantSignerKey),
            "revoke-signer-grant" => Some(Pox5EventName::RevokeSignerGrant),
            "disallow-contract-caller" => Some(Pox5EventName::DisallowContractCaller),
            "allow-contract-caller" => Some(Pox5EventName::AllowContractCaller),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Pox5EventName::SetBondAdmin => "set-bond-admin",
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
            Pox5EventName::ClaimStakerRewardsForSigner => "claim-staker-rewards-for-signer",
            Pox5EventName::GrantSignerKey => "grant-signer-key",
            Pox5EventName::RevokeSignerGrant => "revoke-signer-grant",
            Pox5EventName::DisallowContractCaller => "disallow-contract-caller",
            Pox5EventName::AllowContractCaller => "allow-contract-caller",
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
    /// Logged by `set-bond-admin` when the bond admin principal is rotated.
    SetBondAdmin {
        old_admin: String,
        new_admin: String,
    },

    /// Logged by `setup-bond` when a new protocol bond is created.
    SetupBond {
        bond_index: u128,
        target_rate: u128,
        stx_value_ratio: u128,
        min_ustx_ratio: u128,
        /// `(buff 683)` — Bitcoin script subscript guarding the early-exit
        /// (`OP_ELSE`) branch of the L1 lockup (e.g. `<pubkey> OP_CHECKSIG`
        /// or an M-of-N `CHECKMULTISIG` template). Surfaced as a hex string.
        early_unlock_bytes: String,
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
        /// Describes how the BTC was locked: an `"l1"` lockup carries the
        /// list of proven L1 outputs in `txs`; an `"l2"` (sBTC) lockup has
        /// `txs == None`.
        btc_lockup: BtcLockup,
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

    /// Logged once per `calculate-rewards` call, after all per-bond
    /// distributions have been folded and the STX reward cycle accounting
    /// has been committed.
    CalculateRewards {
        bond_periods: Vec<u128>,
        calculation_height: u128,
        /// Total new rewards accrued since the last calculation.
        gross_accrued_rewards: u128,
        /// Portion of `gross_accrued_rewards` paid out to bonds
        /// (`gross_accrued_rewards - remaining`).
        total_bond_rewards: u128,
        /// Amount added to the reserve this calculation (reserve cut, plus
        /// the STX staker cut when no STX is staked for the cycle).
        reserve_deposit: u128,
        /// Reserve balance after `reserve_deposit` was applied.
        reserve_balance: u128,
        stx_cycle: u128,
        /// Rewards allocated to STX stakers for the cycle (before the
        /// no-stakers fold into the reserve).
        total_stx_staker_rewards: u128,
        cycle_staked_ustx: u128,
        /// Per-uSTX rewards accrued this calculation (zero when no STX is
        /// staked).
        accrued_rewards_per_ustx: u128,
        /// Running per-uSTX reward total for the cycle after this calculation.
        cumulative_rewards_per_ustx: u128,
    },

    /// Logged by `calculate-bond-rewards` for each bond period processed
    /// inside a `calculate-rewards` fold.
    BondDistribution {
        bond_index: u128,
        target_yield: u128,
        /// Rewards earned by this bond this calculation.
        bond_rewards: u128,
        bond_staked_sats: u128,
        /// Per-sat rewards accrued this calculation.
        accrued_rewards_per_sat: u128,
        /// Running per-sat reward total for the bond after this calculation.
        cumulative_rewards_per_sat: u128,
    },

    /// Logged by `claim-rewards` when a signer claims accrued rewards.
    ClaimRewards {
        signer_manager: String,
        reward_cycle: u128,
        stx_rewards: ClaimRewardsInfo,
        bond_rewards: Vec<BondRewardsInfo>,
        bond_totals: u128,
        total_rewards: u128,
    },

    /// Logged by `claim-staker-rewards-for-signer` when a signer manager
    /// marks a staker as having claimed rewards for a cycle.
    ClaimStakerRewardsForSigner {
        signer_manager: String,
        staker: String,
        reward_cycle: u128,
        /// `(optional uint)` — present for bond rewards, `None` for STX-only
        /// staking rewards.
        bond_index: Option<u128>,
        rewards_claimed: u128,
    },

    /// Logged by `grant-signer-key` when a signer key grant is recorded.
    GrantSignerKey {
        /// `(buff 33)` — compressed secp256k1 public key, hex-encoded.
        signer_key: String,
        signer_manager: String,
        auth_id: u128,
    },

    /// Logged by `revoke-signer-grant` when a signer key grant is revoked.
    RevokeSignerGrant {
        /// `(buff 33)` — compressed secp256k1 public key, hex-encoded.
        signer_key: String,
        signer_manager: String,
    },

    /// Logged by `disallow-contract-caller` when a caller allowance is
    /// removed.
    DisallowContractCaller {
        sender: String,
        contract_caller: String,
    },

    /// Logged by `allow-contract-caller` when a caller allowance is granted.
    AllowContractCaller {
        sender: String,
        contract_caller: String,
        /// `(optional uint)` — burn height at which the allowance expires;
        /// `None` means it never expires.
        until_burn_ht: Option<u128>,
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

/// The `btc-lockup` sub-tuple from `register-for-bond` events.
#[derive(Debug, Clone)]
pub struct BtcLockup {
    /// `"l1"` for a Bitcoin L1 lockup, `"l2"` for an sBTC lockup.
    pub lockup_type: String,
    /// The proven L1 outputs for an `"l1"` lockup; `None` for `"l2"`.
    pub txs: Option<Vec<BtcLockupTx>>,
}

/// One entry in the `txs` list of a `register-for-bond` `btc-lockup` tuple.
#[derive(Debug, Clone)]
pub struct BtcLockupTx {
    /// Reversed (big-endian) txid as a `0x`-prefixed hex string.
    pub txid: String,
    pub output_index: u128,
}
