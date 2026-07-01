//! Decoder for synthetic events emitted by the PoX-4 Clarity contract.
//!
//! - [`types`] — `PoxEventName`, `PoxEventBase`, `PoxSyntheticEvent`,
//!   `PoxEventData` (the pox-4 event shapes).
//! - [`decode`] — walks a `clarity::vm::types::Value` and produces a
//!   [`types::PoxSyntheticEvent`].
//! - [`neon_encoder`] — emits the JS object for a decoded event.
//!
//! Inputs that aren't PoX-4 events (e.g. `ResponseErr`) yield `Ok(None)`
//! from `decode::decode_pox_synthetic_event`; the parent module's Neon
//! entry point turns that into a JS `null`.

pub mod decode;
pub mod neon_encoder;
pub mod types;
