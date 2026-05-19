//! Hex helpers used by the JS bindings.
//!
//! Historically this module wrapped a SIMD-accelerated `hex-simd` crate for
//! throughput, but those versions were yanked from crates.io. Upstream
//! `stacks-core` standardizes on the `hex` crate, so we do the same: the
//! perf delta is negligible at typical message sizes and the dep tree is
//! simpler.
//!
//! Notable difference from `hex::encode`: `encode_hex` here adds a `0x` prefix,
//! since every consumer of these helpers in the bindings expects that prefix
//! (it matches the JS-facing JSON shape).

#[derive(Debug)]
pub struct DecodeError(pub hex::FromHexError);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl From<hex::FromHexError> for DecodeError {
    fn from(e: hex::FromHexError) -> Self {
        DecodeError(e)
    }
}

pub fn decode_hex<T: AsRef<[u8]>>(data: T) -> Result<Box<[u8]>, DecodeError> {
    let data_ref = data.as_ref();
    let data_len = data_ref.len();
    if data_len == 0 {
        return Ok(Box::new([0u8; 0]));
    }
    let payload = if data_len >= 2 && data_ref[0] == b'0' && data_ref[1] == b'x' {
        &data_ref[2..]
    } else {
        data_ref
    };
    Ok(hex::decode(payload)?.into_boxed_slice())
}

pub fn encode_hex(data: &[u8]) -> Box<str> {
    let mut out = String::with_capacity(2 + data.len() * 2);
    out.push_str("0x");
    out.push_str(&hex::encode(data));
    out.into_boxed_str()
}

pub fn encode_hex_no_prefix(data: &[u8]) -> Box<str> {
    hex::encode(data).into_boxed_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode() {
        let input = b"hello world";
        let hex_str = encode_hex(input);
        let repr = hex_str.to_string();
        assert_eq!(repr, "0x68656c6c6f20776f726c64");
    }

    #[test]
    fn test_hex_decode_with_prefix() {
        let decoded = decode_hex("0x68656c6c6f").unwrap();
        assert_eq!(&*decoded, b"hello");
    }

    #[test]
    fn test_hex_decode_without_prefix() {
        let decoded = decode_hex("68656c6c6f").unwrap();
        assert_eq!(&*decoded, b"hello");
    }
}
