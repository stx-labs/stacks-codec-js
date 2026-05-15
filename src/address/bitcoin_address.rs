//! Bitcoin address parsing, delegated to upstream's canonical implementation.
//!
//! Upstream exposes `LegacyBitcoinAddress` in `stackslib::burnchains::bitcoin::address`
//! and the legacy-version byte constants there. We re-shape the result into the
//! local `BitcoinAddress` struct so the rest of the bindings keep working with
//! the same field layout. Once the bindings switch to upstream's types
//! directly, this façade module can be deleted.

use blockstack_lib::burnchains::bitcoin::address::{
    legacy_address_type_to_version_byte, LegacyBitcoinAddress, LegacyBitcoinAddressType,
};
use blockstack_lib::burnchains::bitcoin::BitcoinNetworkType as UpstreamBitcoinNetworkType;

pub use blockstack_lib::burnchains::bitcoin::address::{
    ADDRESS_VERSION_MAINNET_MULTISIG, ADDRESS_VERSION_MAINNET_SINGLESIG,
    ADDRESS_VERSION_TESTNET_MULTISIG, ADDRESS_VERSION_TESTNET_SINGLESIG,
};

pub enum BitcoinAddressType {
    PublicKeyHash,
    ScriptHash,
}

pub enum BitcoinNetworkType {
    Mainnet,
    Testnet,
    #[allow(dead_code)]
    Regtest,
}

pub struct BitcoinAddress {
    pub addrtype: BitcoinAddressType,
    pub network_id: BitcoinNetworkType,
    pub hash160_bytes: [u8; 20],
}

impl From<LegacyBitcoinAddressType> for BitcoinAddressType {
    fn from(t: LegacyBitcoinAddressType) -> Self {
        match t {
            LegacyBitcoinAddressType::PublicKeyHash => BitcoinAddressType::PublicKeyHash,
            LegacyBitcoinAddressType::ScriptHash => BitcoinAddressType::ScriptHash,
        }
    }
}

impl From<&BitcoinAddressType> for LegacyBitcoinAddressType {
    fn from(t: &BitcoinAddressType) -> Self {
        match t {
            BitcoinAddressType::PublicKeyHash => LegacyBitcoinAddressType::PublicKeyHash,
            BitcoinAddressType::ScriptHash => LegacyBitcoinAddressType::ScriptHash,
        }
    }
}

impl From<UpstreamBitcoinNetworkType> for BitcoinNetworkType {
    fn from(n: UpstreamBitcoinNetworkType) -> Self {
        match n {
            UpstreamBitcoinNetworkType::Mainnet => BitcoinNetworkType::Mainnet,
            UpstreamBitcoinNetworkType::Testnet => BitcoinNetworkType::Testnet,
            UpstreamBitcoinNetworkType::Regtest => BitcoinNetworkType::Regtest,
        }
    }
}

impl From<&BitcoinNetworkType> for UpstreamBitcoinNetworkType {
    fn from(n: &BitcoinNetworkType) -> Self {
        match n {
            BitcoinNetworkType::Mainnet => UpstreamBitcoinNetworkType::Mainnet,
            BitcoinNetworkType::Testnet => UpstreamBitcoinNetworkType::Testnet,
            BitcoinNetworkType::Regtest => UpstreamBitcoinNetworkType::Regtest,
        }
    }
}

pub fn from_b58(addrb58: &str) -> Result<BitcoinAddress, String> {
    let legacy = LegacyBitcoinAddress::from_b58(addrb58).map_err(|e| format!("{:?}", e))?;
    Ok(BitcoinAddress {
        addrtype: legacy.addrtype.into(),
        network_id: legacy.network_id.into(),
        hash160_bytes: legacy.bytes.0,
    })
}

pub fn address_type_to_version_byte(
    addrtype: &BitcoinAddressType,
    network_id: &BitcoinNetworkType,
) -> u8 {
    legacy_address_type_to_version_byte(addrtype.into(), network_id.into())
}
