use neon::prelude::*;
use std::io::Cursor;

use crate::hex::encode_hex;
use crate::neon_util::*;

use self::deserialize::{deserialize_nakamoto_block, StacksBlock};

pub mod deserialize;
mod neon_encoder;

/// Decode a Nakamoto block (Stacks 3.x+)
pub fn decode_nakamoto_block(mut cx: FunctionContext) -> JsResult<JsObject> {
    let block = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        let block = deserialize_nakamoto_block(&mut cursor)
            .or_else(|e| Err(format!("Failed to decode Nakamoto block: {:?}\n", &e)))?;
        Ok(block)
    })
    .or_else(|e| cx.throw_error(e))?;

    let block_obj = cx.empty_object();

    let block_id = cx.string(encode_hex(&block.header.block_id().0));
    block_obj.set(&mut cx, "block_id", block_id)?;

    Encode(&block).neon_js_serialize(&mut cx, &block_obj, &())?;
    Ok(block_obj)
}

/// Decode a Stacks 2.x block
pub fn decode_stacks_block(mut cx: FunctionContext) -> JsResult<JsObject> {
    let block = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        let block = StacksBlock::deserialize(&mut cursor)
            .or_else(|e| Err(format!("Failed to decode Stacks block: {:?}\n", &e)))?;
        Ok(block)
    })
    .or_else(|e| cx.throw_error(e))?;

    let block_obj = cx.empty_object();

    let block_hash = cx.string(encode_hex(&block.header.block_hash()));
    block_obj.set(&mut cx, "block_hash", block_hash)?;

    Encode(&block).neon_js_serialize(&mut cx, &block_obj, &())?;
    Ok(block_obj)
}
