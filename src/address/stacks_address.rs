//! C32 version-byte constants for the Stacks address space.
//!
//! The address struct itself now comes from
//! `stacks_common::types::chainstate::StacksAddress`; this module only retains
//! the four version-byte constants that the b58 ↔ c32 conversion logic in
//! `super::mod` switches on.

pub const C32_ADDRESS_VERSION_MAINNET_SINGLESIG: u8 = 22; // P
pub const C32_ADDRESS_VERSION_MAINNET_MULTISIG: u8 = 20; // M
pub const C32_ADDRESS_VERSION_TESTNET_SINGLESIG: u8 = 26; // T
pub const C32_ADDRESS_VERSION_TESTNET_MULTISIG: u8 = 21; // N
