# Migrating to upstream `stacks-core` wire-format crates

## Why this document exists

Historically `stacks-encoding-native-js` carried a hand-derived copy of every
type and decoder needed to parse Stacks consensus-encoded data (transactions,
blocks, Clarity values, post-conditions, addresses). That copy drifted out of
sync every time upstream `stacks-network/stacks-core` changed the wire format
(SIP-040 Originator post-conditions, SIP-034 tenure-extend dimensions,
non-sequential multisig hash modes, Nakamoto coinbase, Clarity 3/4/5/6, etc.),
which made shipping consensus updates a copy-paste-and-pray exercise.

This crate now depends directly on `stackslib`, `clarity`, `stacks-common`,
and `stacks-codec` as `git` dependencies pinned to a specific `stacks-core`
revision (see `Cargo.toml`). The pin can be bumped at any time using
[`scripts/update-stacks-core.sh`](../scripts/update-stacks-core.sh).

The migration is complete — see the rest of this document for the current
state, behavior changes, CI implications, and pin-bump workflow. Earlier
phases (Phase 1 "façade + `convert_*`", Phase 2 "delete shadow types",
Phase 3 "strict consensus parsers") are recorded in the git history.

## Current state

All consensus-decoding paths delegate to upstream's canonical
`StacksMessageCodec` or Clarity parsers. The Neon encoders operate on
upstream types directly via zero-cost newtype wrappers
(`neon_util::Encode<'a, T>`, `#[repr(transparent)]`) so we can implement
`NeonJsSerialize` for upstream types without violating the orphan rule.
JS-facing object shapes and TypeScript entry points are unchanged.

| Module | What it does today |
|--------|--------------------|
| `src/neon_util.rs` | `Encode<'a, T>` wrapper; encoders call `Encode(&value).neon_js_serialize(...)`. |
| `src/clarity_value/` | Neon entry points call `clarity::vm::types::Value::deserialize_read` directly. `neon_encoder.rs` walks the upstream tree; `repr_string` / `type_signature_string` produce historical JS output. No local Value tree. |
| `src/post_condition/` | `deserialize.rs` re-exports upstream post-condition types; `deserialize_post_condition` wraps `consensus_deserialize`. `neon_encoder.rs` implements `NeonJsSerialize` for `Encode<'_, …>`. |
| `src/stacks_tx/` | `deserialize.rs` is a thin re-export + `deserialize_transaction`. `neon_encoder.rs` handles upstream payloads, auth, multisigs, coinbase fan-out, microblocks, post-condition buffer re-serialization. **Note:** `clarity_version` on versioned smart-contract payloads uses an explicit map from `clarity::vm::ClarityVersion` to the **1-based wire byte** (`Clarity2` → `2`), not `as u8` on the enum. |
| `src/address/` | `decode_clarity_value_to_principal_inner` calls upstream `PrincipalData::consensus_deserialize` directly. `bitcoin_address.rs` was deleted; `mod.rs` uses upstream `LegacyBitcoinAddress` and `legacy_address_type_to_version_byte`. The local `StacksAddress` struct and `AddressHashMode` enum were also deleted in favor of `stacks_common::types::chainstate::StacksAddress`. |
| `src/stacks_block/` | Both 2.x and Nakamoto paths delegate to upstream's full `<… as StacksMessageCodec>::consensus_deserialize`. All shadow types deleted. `BitVec<4000>` Neon encoder is custom (see "JS-contract divergences" below). |
| `src/pox_events/` | `decode_pox_synthetic_event` walks `clarity::vm::types::Value`; `decode_pox_event` deserializes with `<Value as StacksMessageCodec>::consensus_deserialize`. PoX-event logic itself is bespoke (no upstream equivalent — it's a Stacks-API shape). |

A `grep` for `convert_`, hand-rolled `XxxYy::deserialize` impls, or shadow
consensus structs comes up clean.

## Behavior changes vs. the legacy permissive parser

Switching to upstream's strict codecs introduced four observable changes
for JS callers:

1. `decodeStacksBlock` (2.x) now throws on invalid VRF proofs (must be a
   valid Edwards curve point), zero-transaction blocks, duplicate tx-ids,
   tx-Merkle-root mismatches, multiple-coinbase blocks, and transactions
   with `OffChainOnly` anchor mode.
2. `decodeNakamotoBlock` now throws on the equivalent Nakamoto-level
   structural violations (duplicate tx-ids, tenure-change rules, etc.).
3. `decodeClarityValueToPrincipal` now requires a real principal-prefixed
   Clarity value (`0x05` or `0x06`); the legacy `0x02 || version ||
   hash160` shorthand is no longer accepted.
4. `decodeStacksBlock` returns the `FIRST_STACKS_BLOCK_HASH` constant for
   blocks with `total_work.work == 0` (matching upstream's boot-block
   short-circuit); the legacy code always recomputed the hash
   unconditionally.

The JS API surface and JSON object shapes for valid inputs are otherwise
identical to what shipped on `main`. The `ClarityVersion` enum in
`index.ts` gained one additive variant (`Clarity6 = 6`).

## Intentional JS-contract divergences (won't fix)

A few bespoke pieces of code remain in the Neon encoders because their
output is part of the public JS contract and intentionally differs from
upstream:

- `BitVec<4000>` encoder emits an **MSB-first** `bits` array (upstream's
  `BitVec::get` is LSB-first). The `data` hex is wire-identical.
- Clarity `repr_string` / `type_signature_string` formatters match
  historical Stacks API output: keyword forms like `(list ...)`,
  single-quote prefix on principals, `(string-utf8 N)` where `N` is the
  encoded **byte count** (`chars * 4`), etc.
- The PoX synthetic-event walker is a Stacks-API shape, not a consensus
  type — there is no upstream encoder to delegate to.

## Tests

- `cargo test --lib`: 41 unit tests covering address conversions, post-condition
  decoding, transaction deserialization, Nakamoto block decoding, memo
  utilities, and the PoX event walker.
- `npm test` (Jest): 42 end-to-end tests for every public Neon entry point.
  The Stacks 2.x Jest tests exercise the strict-rejection path; happy-path
  coverage for 2.x blocks needs a real mainnet fixture (not yet captured
  in `tests/fixtures/`).

## CI considerations

`stackslib` transitively requires `rusqlite` with a bundled SQLite C library,
so every CI target needs a working C compiler. The existing matrix already
provides one:

- **`linux-x64-musl` / `linux-arm64-musl`**: `x86_64-linux-musl-gcc` /
  `aarch64-linux-musl-gcc` from `musl.cc` are downloaded during the
  workflow and exposed via `CC_<target>` / `CARGO_TARGET_<TARGET>_LINKER`
  env vars. `rusqlite`'s build script picks these up automatically.
- **`linux-x64-glibc` / `linux-arm64-glibc`**: built inside the `rust`
  Docker image, which ships with `gcc`.
- **`macos-x64` / `macos-arm64`**: `clang` ships with GitHub-hosted macOS
  runners.
- **`win32-x64`**: MSVC `cl.exe` ships with the Windows runner.

Build wall-clock time is ~1–3 min higher per target because we now compile
~150 additional crates plus the bundled SQLite C sources. No workflow
changes were required; the `Swatinem/rust-cache` action keeps incremental
rebuilds fast.

## Updating the upstream pin

```sh
scripts/update-stacks-core.sh                  # → latest develop HEAD
scripts/update-stacks-core.sh master           # → master branch tip
scripts/update-stacks-core.sh 3.4.0.0.2        # → a release tag
scripts/update-stacks-core.sh abc1234          # → an explicit SHA
```

The script rewrites every `rev = "..."` in `Cargo.toml` that points at
`stacks-network/stacks-core`, runs `cargo update`, then `cargo check`. Set
`SKIP_CHECK=1` to skip the verification step.
