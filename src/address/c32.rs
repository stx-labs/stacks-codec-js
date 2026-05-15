//! C32 encode/decode used to live here as a hand-tuned copy. The actual
//! algorithm now comes from `stacks_common::address::c32`; this module is a
//! thin re-export layer so the rest of the bindings keep their existing import
//! paths.
//!
//! Returning `(u8, [u8; 20])` from `c32_address_decode` matches the historical
//! API — upstream returns `Vec<u8>`, but every caller in this crate consumes
//! the bytes as a 20-byte array, so we normalize at the boundary.

pub fn c32_address(version: u8, data: &[u8]) -> Result<String, String> {
    stacks_common::address::c32::c32_address(version, data).map_err(|e| format!("{}", e))
}

pub fn c32_address_decode(c32_address_str: &str) -> Result<(u8, [u8; 20]), String> {
    let (version, bytes) = stacks_common::address::c32::c32_address_decode(c32_address_str)
        .map_err(|e| format!("{}", e))?;
    let bytes: [u8; 20] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| format!("c32 address decoded to {} bytes, expected 20", bytes.len()))?;
    Ok((version, bytes))
}
