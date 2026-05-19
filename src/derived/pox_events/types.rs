/// Network type for BTC address encoding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StacksNetwork {
    Mainnet,
    Testnet,
    Devnet,
    Mocknet,
}

impl StacksNetwork {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "mainnet" => Ok(StacksNetwork::Mainnet),
            "testnet" => Ok(StacksNetwork::Testnet),
            "devnet" => Ok(StacksNetwork::Devnet),
            "mocknet" => Ok(StacksNetwork::Mocknet),
            _ => Err(format!("Unknown network: {}", s)),
        }
    }

    pub fn is_mainnet(&self) -> bool {
        matches!(self, StacksNetwork::Mainnet)
    }
}

/// All PoX synthetic event names emitted by the Clarity contracts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PoxEventName {
    HandleUnlock,
    StackStx,
    StackIncrease,
    StackExtend,
    DelegateStx,
    DelegateStackStx,
    DelegateStackIncrease,
    DelegateStackExtend,
    StackAggregationCommit,
    StackAggregationCommitIndexed,
    StackAggregationIncrease,
    RevokeDelegateStx,
}

impl PoxEventName {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "handle-unlock" => Some(PoxEventName::HandleUnlock),
            "stack-stx" => Some(PoxEventName::StackStx),
            "stack-increase" => Some(PoxEventName::StackIncrease),
            "stack-extend" => Some(PoxEventName::StackExtend),
            "delegate-stx" => Some(PoxEventName::DelegateStx),
            "delegate-stack-stx" => Some(PoxEventName::DelegateStackStx),
            "delegate-stack-increase" => Some(PoxEventName::DelegateStackIncrease),
            "delegate-stack-extend" => Some(PoxEventName::DelegateStackExtend),
            "stack-aggregation-commit" => Some(PoxEventName::StackAggregationCommit),
            "stack-aggregation-commit-indexed" => Some(PoxEventName::StackAggregationCommitIndexed),
            "stack-aggregation-increase" => Some(PoxEventName::StackAggregationIncrease),
            "revoke-delegate-stx" => Some(PoxEventName::RevokeDelegateStx),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PoxEventName::HandleUnlock => "handle-unlock",
            PoxEventName::StackStx => "stack-stx",
            PoxEventName::StackIncrease => "stack-increase",
            PoxEventName::StackExtend => "stack-extend",
            PoxEventName::DelegateStx => "delegate-stx",
            PoxEventName::DelegateStackStx => "delegate-stack-stx",
            PoxEventName::DelegateStackIncrease => "delegate-stack-increase",
            PoxEventName::DelegateStackExtend => "delegate-stack-extend",
            PoxEventName::StackAggregationCommit => "stack-aggregation-commit",
            PoxEventName::StackAggregationCommitIndexed => "stack-aggregation-commit-indexed",
            PoxEventName::StackAggregationIncrease => "stack-aggregation-increase",
            PoxEventName::RevokeDelegateStx => "revoke-delegate-stx",
        }
    }
}

/// Base fields common to all PoX synthetic events.
#[derive(Debug, Clone)]
pub struct PoxEventBase {
    pub stacker: String,
    pub locked: u128,
    pub balance: u128,
    pub burnchain_unlock_height: u128,
    pub pox_addr: Option<String>,
    pub pox_addr_raw: Option<String>,
}

/// A fully decoded PoX synthetic event.
#[derive(Debug, Clone)]
pub struct PoxSyntheticEvent {
    pub base: PoxEventBase,
    pub name: PoxEventName,
    pub data: PoxEventData,
}

/// Event-specific data payload.
#[derive(Debug, Clone)]
pub enum PoxEventData {
    HandleUnlock {
        first_cycle_locked: u128,
        first_unlocked_cycle: u128,
    },
    StackStx {
        lock_amount: u128,
        lock_period: u128,
        start_burn_height: u128,
        unlock_burn_height: u128,
        signer_key: Option<String>,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    StackIncrease {
        increase_by: u128,
        total_locked: u128,
        signer_key: Option<String>,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    StackExtend {
        extend_count: u128,
        unlock_burn_height: u128,
        signer_key: Option<String>,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    DelegateStx {
        amount_ustx: u128,
        delegate_to: String,
        unlock_burn_height: Option<u128>,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    DelegateStackStx {
        lock_amount: u128,
        unlock_burn_height: u128,
        start_burn_height: u128,
        lock_period: u128,
        delegator: String,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    DelegateStackIncrease {
        increase_by: u128,
        total_locked: u128,
        delegator: String,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    DelegateStackExtend {
        unlock_burn_height: u128,
        extend_count: u128,
        delegator: String,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    StackAggregationCommit {
        reward_cycle: u128,
        amount_ustx: u128,
        signer_key: Option<String>,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    StackAggregationCommitIndexed {
        reward_cycle: u128,
        amount_ustx: u128,
        signer_key: Option<String>,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    StackAggregationIncrease {
        reward_cycle: u128,
        amount_ustx: u128,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
    RevokeDelegateStx {
        delegate_to: String,
        end_cycle_id: Option<u128>,
        start_cycle_id: Option<u128>,
    },
}
