//! Base58check helpers. The full implementation lives in
//! `stacks_common::address::b58`; this module is a thin re-export layer so
//! the rest of the bindings keep their existing import paths.

pub use stacks_common::address::b58::{check_encode_slice, from_check};
