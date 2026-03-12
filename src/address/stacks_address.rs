use std::convert::TryFrom;

use super::c32::c32_address_decode;

pub const C32_ADDRESS_VERSION_MAINNET_SINGLESIG: u8 = 22; // P
pub const C32_ADDRESS_VERSION_MAINNET_MULTISIG: u8 = 20; // M
pub const C32_ADDRESS_VERSION_TESTNET_SINGLESIG: u8 = 26; // T
pub const C32_ADDRESS_VERSION_TESTNET_MULTISIG: u8 = 21; // N

#[derive(Debug, PartialEq)]
pub struct StacksAddress {
    pub version: u8,
    pub hash160_bytes: [u8; 20],
}

impl StacksAddress {
    pub fn new(version: u8, hash160_bytes: [u8; 20]) -> Self {
        Self {
            version,
            hash160_bytes,
        }
    }

    pub fn from_string(s: &str) -> Result<StacksAddress, String> {
        let (version, bytes) = match c32_address_decode(s) {
            Ok((v, b)) => (v, b),
            Err(e) => {
                return Err(format!("Error decoding c32 address: {}", e));
            }
        };

        Ok(StacksAddress {
            version: version,
            hash160_bytes: bytes,
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum AddressHashMode {
    // serialization modes for public keys to addresses.
    // We support four different modes due to legacy compatibility with Stacks v1 addresses:
    SerializeP2PKH = 0x00, // hash160(public-key), same as bitcoin's p2pkh
    SerializeP2SH = 0x01,  // hash160(multisig-redeem-script), same as bitcoin's multisig p2sh
    SerializeP2SHNonSequential = 0x05, // hash160(multisig-redeem-script), same as bitcoin's multisig p2sh (non-sequential signing)
    SerializeP2WPKH = 0x02, // hash160(segwit-program-00(p2pkh)), same as bitcoin's p2sh-p2wpkh
    SerializeP2WSH = 0x03,  // hash160(segwit-program-00(public-keys)), same as bitcoin's p2sh-p2wsh
    SerializeP2WSHNonSequential = 0x07, // hash160(segwit-program-00(public-keys)), same as bitcoin's p2sh-p2wsh (non-sequential signing)
}

impl AddressHashMode {
    pub fn to_version_mainnet(&self) -> u8 {
        match *self {
            AddressHashMode::SerializeP2PKH => C32_ADDRESS_VERSION_MAINNET_SINGLESIG,
            _ => C32_ADDRESS_VERSION_MAINNET_MULTISIG,
        }
    }

    pub fn to_version_testnet(&self) -> u8 {
        match *self {
            AddressHashMode::SerializeP2PKH => C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
            _ => C32_ADDRESS_VERSION_TESTNET_MULTISIG,
        }
    }
}

/// Given the u8 of an AddressHashMode, deduce the AddressHashNode
impl TryFrom<u8> for AddressHashMode {
    type Error = String;

    fn try_from(value: u8) -> Result<AddressHashMode, Self::Error> {
        match value {
            x if x == AddressHashMode::SerializeP2PKH as u8 => Ok(AddressHashMode::SerializeP2PKH),
            x if x == AddressHashMode::SerializeP2SH as u8 => Ok(AddressHashMode::SerializeP2SH),
            x if x == AddressHashMode::SerializeP2SHNonSequential as u8 => {
                Ok(AddressHashMode::SerializeP2SHNonSequential)
            }
            x if x == AddressHashMode::SerializeP2WPKH as u8 => {
                Ok(AddressHashMode::SerializeP2WPKH)
            }
            x if x == AddressHashMode::SerializeP2WSH as u8 => Ok(AddressHashMode::SerializeP2WSH),
            x if x == AddressHashMode::SerializeP2WSHNonSequential as u8 => {
                Ok(AddressHashMode::SerializeP2WSHNonSequential)
            }
            _ => Err(format!("Invalid version {}", value)),
        }
    }
}
