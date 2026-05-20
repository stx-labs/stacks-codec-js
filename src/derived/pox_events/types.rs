//! Shared types for PoX synthetic-event decoding.
//!
//! Types specific to a single PoX-contract version live under that version's
//! sub-module (e.g. `pox4::types`). This module holds the bits that every
//! PoX version reuses.

/// Network type used for BTC-address encoding inside PoX events.
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
