//! Generic Neon-object setter helpers used by every per-PoX-version JS
//! encoder.
//!
//! All `u128` values are emitted as JS strings (JS numbers lose precision
//! past 2^53). Optional fields are emitted as `null` rather than omitted,
//! so downstreams can rely on a stable JS object shape.

use neon::prelude::*;

pub fn set_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: &str,
) -> NeonResult<()> {
    let val = cx.string(value);
    obj.set(cx, key, val)?;
    Ok(())
}

pub fn set_bool<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: bool,
) -> NeonResult<()> {
    let val = cx.boolean(value);
    obj.set(cx, key, val)?;
    Ok(())
}

pub fn set_u128_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: u128,
) -> NeonResult<()> {
    let val = cx.string(value.to_string());
    obj.set(cx, key, val)?;
    Ok(())
}

pub fn set_optional_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: Option<&str>,
) -> NeonResult<()> {
    match value {
        Some(s) => {
            let val = cx.string(s);
            obj.set(cx, key, val)?;
        }
        None => {
            let val = cx.null();
            obj.set(cx, key, val)?;
        }
    }
    Ok(())
}

pub fn set_optional_u128_string<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    value: Option<u128>,
) -> NeonResult<()> {
    match value {
        Some(v) => {
            let val = cx.string(v.to_string());
            obj.set(cx, key, val)?;
        }
        None => {
            let val = cx.null();
            obj.set(cx, key, val)?;
        }
    }
    Ok(())
}

/// Emit a `Vec<u128>` as a JS array of string-quoted numbers.
pub fn set_u128_array<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    key: &str,
    items: &[u128],
) -> NeonResult<()> {
    let arr = JsArray::new(cx, items.len());
    for (i, n) in items.iter().enumerate() {
        let s = cx.string(n.to_string());
        arr.set(cx, i as u32, s)?;
    }
    obj.set(cx, key, arr)?;
    Ok(())
}
