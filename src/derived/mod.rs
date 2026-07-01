//! Local decoders that produce Stacks-API-shaped output rather than
//! forwarding upstream consensus types.
//!
//! Each child module derives a JS-facing object from a raw input (Clarity
//! value or byte buffer). The output shape is a Stacks-API contract — not a
//! consensus wire type — which is why these modules live outside
//! `crate::upstream`.
//!
//! - [`memo`] — decode Stacks transaction memos into printable strings,
//!   using a baked-in unicode-printability table.
//! - [`pox_events`] — walk Clarity values into PoX synthetic-event shapes
//!   (`StackStx`, `DelegateStx`, etc.).

pub mod memo;
pub mod pox_events;
