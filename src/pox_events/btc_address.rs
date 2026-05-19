use std::convert::TryFrom;

use crate::address::check_encode_slice;

use super::types::StacksNetwork;

/// Converts a PoX address (version byte + hashbytes) into a Bitcoin address string.
///
/// Version mapping:
/// - 0 → P2PKH: base58check with BTC version 0x00 mainnet / 0x6f testnet
/// - 1,2,3 → P2SH variants: base58check with BTC version 0x05 mainnet / 0xc4 testnet
/// - 4 → P2WPKH: bech32 segwit v0
/// - 5 → P2WSH: bech32 segwit v0
/// - 6 → P2TR: bech32m segwit v1
pub fn pox_address_to_btc_address(
    version: u8,
    hashbytes: &[u8],
    network: StacksNetwork,
) -> Result<String, String> {
    match version {
        // P2PKH
        0 => {
            let btc_version: u8 = if network.is_mainnet() { 0x00 } else { 0x6f };
            let mut data = vec![btc_version];
            data.extend_from_slice(hashbytes);
            Ok(check_encode_slice(&data))
        }
        // P2SH, P2SH-P2WPKH, P2SH-P2WSH
        1..=3 => {
            let btc_version: u8 = if network.is_mainnet() { 0x05 } else { 0xc4 };
            let mut data = vec![btc_version];
            data.extend_from_slice(hashbytes);
            Ok(check_encode_slice(&data))
        }
        // P2WPKH (segwit v0, bech32)
        4 => {
            let hrp = segwit_hrp(network);
            encode_segwit(hrp, 0, hashbytes)
        }
        // P2WSH (segwit v0, bech32)
        5 => {
            let hrp = segwit_hrp(network);
            encode_segwit(hrp, 0, hashbytes)
        }
        // P2TR (segwit v1, bech32m)
        6 => {
            let hrp = segwit_hrp(network);
            encode_segwit(hrp, 1, hashbytes)
        }
        _ => Err(format!("Unknown PoX address version: {}", version)),
    }
}

fn segwit_hrp(network: StacksNetwork) -> bech32::Hrp {
    if network.is_mainnet() {
        bech32::Hrp::parse_unchecked("bc")
    } else {
        // testnet, devnet, mocknet all use regtest HRP for bech32
        bech32::Hrp::parse_unchecked("bcrt")
    }
}

fn encode_segwit(hrp: bech32::Hrp, witness_version: u8, data: &[u8]) -> Result<String, String> {
    let fe32_version = bech32::Fe32::try_from(witness_version)
        .map_err(|e| format!("Invalid witness version: {}", e))?;
    let encoded = bech32::segwit::encode(hrp, fe32_version, data)
        .map_err(|e| format!("Bech32 encoding error: {}", e))?;
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::decode_hex;

    #[test]
    fn test_p2pkh_mainnet() {
        // version 0, 20-byte hash
        let hash = decode_hex("f8917303bfa8ef24f292e8fa1419b20460ba064d").unwrap();
        let addr = pox_address_to_btc_address(0, &hash, StacksNetwork::Mainnet).unwrap();
        assert_eq!(addr, "1PfJpZsjreyVrqeoAfabrRwwjQyoSQMmHH");
    }

    #[test]
    fn test_p2pkh_testnet() {
        let hash = decode_hex("f8917303bfa8ef24f292e8fa1419b20460ba064d").unwrap();
        let addr = pox_address_to_btc_address(0, &hash, StacksNetwork::Testnet).unwrap();
        // testnet P2PKH starts with 'm' or 'n'
        assert!(addr.starts_with('m') || addr.starts_with('n'));
    }

    #[test]
    fn test_p2sh_mainnet() {
        let hash = decode_hex("f8917303bfa8ef24f292e8fa1419b20460ba064d").unwrap();
        let addr = pox_address_to_btc_address(1, &hash, StacksNetwork::Mainnet).unwrap();
        // P2SH mainnet starts with '3'
        assert!(addr.starts_with('3'));
    }

    #[test]
    fn test_p2sh_testnet() {
        let hash = decode_hex("f8917303bfa8ef24f292e8fa1419b20460ba064d").unwrap();
        let addr = pox_address_to_btc_address(1, &hash, StacksNetwork::Testnet).unwrap();
        // P2SH testnet starts with '2'
        assert!(addr.starts_with('2'));
    }

    #[test]
    fn test_p2wpkh_mainnet() {
        // version 4 → segwit v0, 20-byte witness program
        let hash = decode_hex("751e76e8199196d454941c45d1b3a323f1433bd6").unwrap();
        let addr = pox_address_to_btc_address(4, &hash, StacksNetwork::Mainnet).unwrap();
        assert!(addr.starts_with("bc1q"));
    }

    #[test]
    fn test_p2wsh_mainnet() {
        // version 5 → segwit v0, 32-byte witness program
        let hash =
            decode_hex("1863143c14c5166804bd19203356da136c985678cd4d27a1b8c6329604903262").unwrap();
        let addr = pox_address_to_btc_address(5, &hash, StacksNetwork::Mainnet).unwrap();
        assert!(addr.starts_with("bc1q"));
    }

    #[test]
    fn test_p2tr_mainnet() {
        // version 6 → segwit v1, 32-byte witness program
        let hash =
            decode_hex("a60869f0dbcf1dc659c9cecbee090449d6a21c3d5c31a381c39af694d10c8b3e").unwrap();
        let addr = pox_address_to_btc_address(6, &hash, StacksNetwork::Mainnet).unwrap();
        assert!(addr.starts_with("bc1p"));
    }

    #[test]
    fn test_p2wpkh_testnet() {
        let hash = decode_hex("751e76e8199196d454941c45d1b3a323f1433bd6").unwrap();
        let addr = pox_address_to_btc_address(4, &hash, StacksNetwork::Testnet).unwrap();
        assert!(addr.starts_with("bcrt1q"));
    }

    #[test]
    fn test_p2tr_testnet() {
        let hash =
            decode_hex("a60869f0dbcf1dc659c9cecbee090449d6a21c3d5c31a381c39af694d10c8b3e").unwrap();
        let addr = pox_address_to_btc_address(6, &hash, StacksNetwork::Testnet).unwrap();
        assert!(addr.starts_with("bcrt1p"));
    }

    #[test]
    fn test_unknown_version() {
        let hash = decode_hex("f8917303bfa8ef24f292e8fa1419b20460ba064d").unwrap();
        let result = pox_address_to_btc_address(7, &hash, StacksNetwork::Mainnet);
        assert!(result.is_err());
    }
}
