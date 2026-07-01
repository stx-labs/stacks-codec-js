//! Decoder for synthetic events emitted by the PoX-5 Clarity contract.
//!
//! PoX-5 events come from `(print { topic: "...", ... })` calls in the
//! contract source, so each event arrives as a flat Clarity tuple with a
//! `topic` ASCII string field plus event-specific data. This is structurally
//! different from PoX-4, where the Stacks node synthesizes a
//! `Response(Ok({ name, data, ... }))` tuple per stacking method call.
//!
//! - [`types`] — `Pox5EventName`, `Pox5SyntheticEvent`, `Pox5EventData`, and
//!   the two sub-tuple structs (`StxRewardsInfo`, `BondRewardsInfo`).
//! - [`decode`] — sniffs `topic` and produces a typed [`types::Pox5SyntheticEvent`].
//! - [`neon_encoder`] — emits the JS object for a decoded event.
//!
//! Inputs that aren't PoX-5 events (no `topic` field on the top-level tuple)
//! yield `Ok(None)` from `decode::decode_pox5_synthetic_event`, which lets
//! the parent module fall back to PoX-4.

pub mod decode;
pub mod neon_encoder;
pub mod types;
