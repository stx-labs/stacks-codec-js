use lazy_static::lazy_static;
use neon::prelude::*;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

use crate::util::neon::arg_as_bytes;

mod unicode_printable;

fn memo_normalize<T: AsRef<[u8]>>(input: T) -> String {
    let memo_str = String::from_utf8_lossy(input.as_ref());
    let mut result_str: String = String::with_capacity(memo_str.len());
    for g in memo_str.graphemes(true) {
        let chars: Vec<char> = g.chars().collect();
        // If char length is greater than one, assume printable grapheme cluster
        if chars.len() == 1 {
            if unicode_printable::is_printable(chars[0]) {
                result_str.push(chars[0]);
            } else {
                result_str.push(' ');
            }
        } else {
            result_str.push_str(g);
        }
    }
    lazy_static! {
        // Match one or more spans of `�` (unicode replacement character) and/or `\s` (whitespace)
        static ref UNICODE_REPLACEMENT_RE: Regex = Regex::new(r"(\u{FFFD}|\s)+").unwrap();
    }
    let memo_no_unknown = UNICODE_REPLACEMENT_RE.replace_all(&result_str, " ");
    let memo_no_invalid = memo_no_unknown.trim();
    memo_no_invalid.to_string()
}

pub fn memo_to_string(mut cx: FunctionContext) -> JsResult<JsString> {
    let normalized = arg_as_bytes(&mut cx, 0, |input_bytes| Ok(memo_normalize(input_bytes)))
        .or_else(|e| cx.throw_error(e))?;
    let str_result = cx.string(normalized);
    Ok(str_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::hex;

    #[test]
    fn test_memo_decode_whitespace() {
        let input = "hello   world";
        let output1 = memo_normalize(input);
        let expected1 = "hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_unknown_unicode() {
        let input = "hello�world  test part1   goodbye�world  test part2     ";
        let output1 = memo_normalize(input);
        let expected1 = "hello world test part1 goodbye world test part2";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_misc_btc_coinbase() {
        let input = hex::decode_hex("037e180b04956b4e68627463706f6f6c2f3266646575fabe6d6df77973b452568eb2f43593285804dad9d7ef057eada5ff9f2a1634ec43f514b1020000008e9b20aa0ebfd204924b040000000000").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "~ kNhbtcpool/2fdeu mm ys RV 5 (X ~ * 4 C K";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_misc_btc_coinbase_2() {
        let input = hex::decode_hex("037c180b2cfabe6d6d5e0eb001a2eaea9c5e39b7f54edd5c23eb6e684dab1995191f664658064ba7dc10000000f09f909f092f4632506f6f6c2f6500000000000000000000000000000000000000000000000000000000000000000000000500f3fa0200").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "| , mm^ ^9 N \\# nhM fFX K 🐟 /F2Pool/e";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_grapheme_extended() {
        let input = "👩‍👩‍👧‍👦 hello world";
        let output1 = memo_normalize(input);
        let expected1 = "👩‍👩‍👧‍👦 hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_unicode() {
        let input = hex::decode_hex("f09f87b3f09f87b12068656c6c6f20776f726c64").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "🇳🇱 hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_padded_start() {
        let input = hex::decode_hex("00000000000068656c6c6f20776f726c64").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_padded_end() {
        let input = hex::decode_hex("68656c6c6f20776f726c64000000000000").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_padded_middle() {
        let input =
            hex::decode_hex("68656c6c6f20776f726c6400000000000068656c6c6f20776f726c64").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "hello world hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_unicode_scalar() {
        let input = "hello worldy̆ test";
        let output1 = memo_normalize(input);
        let expected1 = "hello worldy̆ test";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_zero_width_joiner() {
        let input = "👨\u{200D}👩";
        let output1 = memo_normalize(input);
        let expected1 = "👨‍👩";
        assert_eq!(output1, expected1);
    }
}
