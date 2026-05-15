# Migrating to upstream `stacks-core` wire-format crates

## Why this document exists

Historically `stacks-encoding-native-js` carried a hand-derived copy of every
type and decoder needed to parse Stacks consensus-encoded data (transactions,
blocks, Clarity values, post-conditions, addresses). That copy drifted out of
sync every time upstream `stacks-network/stacks-core` changed the wire format
(SIP-040 Originator post-conditions, SIP-034 tenure-extend dimensions,
non-sequential multisig hash modes, Nakamoto coinbase, Clarity 3/4/5/6, etc.),
which made shipping consensus updates a copy-paste-and-pray exercise.

This crate now depends directly on `stackslib`, `clarity`, `stacks-common`, and
`stacks-codec` as `git` dependencies pinned to a specific `stacks-core`
revision (see `Cargo.toml`). The pin can be bumped at any time using
[`scripts/update-stacks-core.sh`](../scripts/update-stacks-core.sh).

The migration is incremental: each module under `src/` is being moved from
"hand-copy of upstream code" to "thin façade that delegates to upstream".
This file documents the pattern and tracks progress.

## The pattern

For each module, keep:

- The public neon-binding functions (entry points exported in `src/lib.rs`).
- The `neon_encoder.rs` files that map parsed Rust types into the
  JS-facing object shape declared by the TypeScript interface in `index.ts`.

Replace:

- Local `deserialize.rs` implementations with calls into upstream's
  `StacksMessageCodec::consensus_deserialize` (or equivalent).
- Local algorithm code (c32, b58, hashes, address parsing, etc.) with calls
  into upstream's canonical implementations.

When a local type has fields that don't exist on the upstream type (e.g. the
existing `StacksTransaction::post_conditions_serialized: Vec<u8>` field, which
captures the raw post-conditions byte span for the JS output), compute the
extra fields explicitly during decode — typically by re-serializing the parsed
upstream value, or by capturing cursor offsets around the relevant
`consensus_deserialize` call.

A future, separate refactor pass can then delete the local type definitions
entirely and rewrite `neon_encoder.rs` to operate on upstream types directly.
The current "façade" phase is intentionally low-risk so that this can land
behind small, reviewable diffs.

## Reference module: `src/address/`

This is the worked example. Before:

- `src/address/c32.rs` (592 LOC) — hand-tuned crockford-32 codec.
- `src/address/b58.rs` (353 LOC) — base58check + SHA256 checksum helpers.
- `src/address/bitcoin_address.rs` (95 LOC) — Bitcoin legacy address parsing.
- `src/address/stacks_address.rs` (89 LOC) — `StacksAddress` struct +
  `AddressHashMode` enum.

After:

- `src/address/c32.rs` — 16 LOC. Wraps
  `stacks_common::address::c32::{c32_address, c32_address_decode}`, normalizing
  the error type to `String` and reshaping the byte vec to `[u8; 20]`.
- `src/address/b58.rs` — 5 LOC. Re-exports `stacks_common::address::b58`.
- `src/address/bitcoin_address.rs` — 75 LOC of `From` impls between the local
  `BitcoinAddress` struct/enums and `blockstack_lib::burnchains::bitcoin::address::LegacyBitcoinAddress`.
- `src/address/stacks_address.rs` — unchanged.
- `src/address/mod.rs` — unchanged (and the `decode_stacks_address`,
  `stacks_to_bitcoin_address`, etc. neon functions kept their behavior).

Net: ~1000 LOC of hand-copied algorithm code replaced by ~100 LOC of
delegation. All 42 unit tests still pass.

## Reference module: `src/clarity_value/`

Done. The recursive `ClarityValue::deserialize` was replaced with a façade:

- `src/clarity_value/deserialize.rs`:
  - `ClarityValue::deserialize` now calls
    `clarity::vm::types::Value::deserialize_read(r, None, false)` and walks
    the resulting upstream tree into the local `Value` / `ClarityValue` types
    via `convert_value` / `convert_clarity_value`.
  - `TypePrefix` is kept locally because `address`, `post_condition` and
    `stacks_tx` still import it via this path (their own migrations will
    drop these imports). The values match upstream's
    `clarity_types::types::serialization::TypePrefix` exactly.
  - The smaller helpers (`ClarityName::deserialize`,
    `ContractName::deserialize`, `StandardPrincipalData::deserialize`) are
    kept for the same reason; they will be removed when their last call
    sites in `stacks_tx`, `post_condition`, and `address` are migrated.
- `src/clarity_value/types.rs` — unchanged. The local `ClarityValue` /
  `Value` / `StandardPrincipalData` / `QualifiedContractIdentifier` types
  remain as façades over their upstream equivalents, with the `repr_string`
  and `type_signature` formatters preserved bit-for-bit so the JS-facing
  output is byte-identical.
- `src/clarity_value/neon_encoder.rs` — unchanged; still operates on the
  local `Value` enum.
- `src/clarity_value/mod.rs` — unchanged. The four neon entry points
  (`decode_clarity_value`, `decode_clarity_value_type_name`,
  `decode_clarity_value_to_repr`, `decode_clarity_value_array`) now exercise
  upstream's parser without any API changes.

**Bytes-capture nuance**: The original deserializer captured each nested
value's raw byte slice from the input cursor. We preserve this by
re-serializing each nested upstream value via
`<Value as StacksMessageCodec>::serialize_to_vec` (the trait method, which
returns `Vec<u8>` — the inherent `Value::serialize_to_vec` shadows it with a
`Result`-returning version, so it must be disambiguated). Clarity wire
encoding is canonical and deterministic, so the round-tripped bytes match
the original input. The top-level value's bytes are still taken directly
from the cursor positions before/after `deserialize_read` to skip one
serialization round-trip on the hot path.

The `clarity-value-to-json`, `clarity-value-to-repr`, and
`clarity-value-list-decode` Jest suites pass unchanged.

## Reference module: `src/post_condition/`

Done. The hand-rolled `TransactionPostCondition::deserialize` (and its
private `PostConditionPrincipal::deserialize` / `AssetInfo::deserialize`
helpers) was replaced with a single delegation to upstream's canonical
[`StacksMessageCodec`] impl in `stackslib`:

- `src/post_condition/deserialize.rs`:
  - `TransactionPostCondition::deserialize` now calls
    `<blockstack_lib::chainstate::stacks::TransactionPostCondition as
    StacksMessageCodec>::consensus_deserialize(fd)` and runs
    `convert_post_condition` to lower the result into the local enum tree.
  - The local enums (`TransactionPostCondition`, `PostConditionPrincipal`,
    `AssetInfo`, `AssetInfoID`, `PostConditionPrincipalID`,
    `FungibleConditionCode`, `NonfungibleConditionCode`) are kept verbatim
    so the Neon encoder doesn't need to change. `From<UpstreamX>` impls are
    provided for the two condition-code enums to keep the converter
    straight-line.
  - For the Nonfungible variant's asset value, we reuse
    `crate::clarity_value::deserialize::convert_clarity_value(_, true)`,
    which already knows how to capture per-node `serialized_bytes` for the
    Neon encoder.
  - The `StacksAddress::deserialize` impl stays put — it's still called by
    `stacks_tx::deserialize`. It will move (or vanish entirely) when that
    module is migrated next.
- `src/post_condition/neon_encoder.rs` — unchanged.
- `src/post_condition/mod.rs` — unchanged. The public Neon entry point
  `decode_tx_post_conditions` keeps its current shape.

**Bytes-capture nuance**: Same as `clarity_value` — Clarity wire encoding is
canonical, so the Nonfungible asset value's per-node `hex` field stays
byte-identical because it's recovered via `serialize_to_vec` round-trip.

The `post-conditions` and `tx-decode*` Jest suites pass unchanged, as does
the `post_condition::tests::test_decode_samples` regression test that runs
the gzipped corpus of real on-chain post-conditions through the new path.

To preserve this code path's reuse, the helper
`crate::clarity_value::deserialize::convert_clarity_value` was promoted from
private to `pub(crate)`.

## Remaining modules

### `src/stacks_tx/` — ~1700 LOC, the biggest

**Upstream targets**:

- `blockstack_lib::chainstate::stacks::StacksTransaction`,
  `TransactionPayload`, `TransactionAuth`, `TransactionSpendingCondition`,
  `MultisigSpendingCondition`, `SinglesigSpendingCondition`,
  `OrderIndependentMultisigSpendingCondition`,
  `TransactionContractCall`, `TransactionSmartContract`,
  `TenureChangePayload`, `TokenTransferMemo`, `CoinbasePayload`,
  `StacksMicroblockHeader`, `TransactionAuthField`, `TransactionAuthFieldID`,
  `TransactionPublicKeyEncoding`.

**Hazards**:

- Local code tracks `post_conditions_serialized: Vec<u8>` (raw bytes of the
  post-conditions section, needed for the JS `post_conditions_buffer` field
  in the TypeScript output). Upstream doesn't expose this. Easiest fix:
  capture the cursor `position()` before and after calling
  `Vec::<TransactionPostCondition>::consensus_deserialize`, then slice the
  original input.
- Local `AddressHashMode` is a single enum with six variants (P2PKH,
  P2SH, P2SHNonSequential, P2WPKH, P2WSH, P2WSHNonSequential). Upstream
  splits these across three enums (`SinglesigHashMode`, `MultisigHashMode`,
  `OrderIndependentMultisigHashMode`). The JS-facing TypeScript already uses
  the split form (`TxSpendingConditionSingleSigHashMode` /
  `TxSpendingConditionMultiSigHashMode`), so this actually simplifies the
  `neon_encoder`.
- `TransactionPayload::SmartContract(s, version_opt)` collapses what the
  TypeScript exposes as two distinct shapes (`TxPayloadSmartContract` and
  `TxPayloadVersionedSmartContract`); branch on `version_opt.is_some()` in the
  encoder.
- `TransactionPayload::Coinbase(payload, recipient_opt, vrf_opt)` collapses
  `TxPayloadCoinbase`, `TxPayloadCoinbaseToAltRecipient`, and
  `TxPayloadNakamotoCoinbase`; branch on `(recipient_opt, vrf_opt)`.

**Suggested approach**: This is best done as a single self-contained PR.
Replace `src/stacks_tx/deserialize.rs` with a tiny adapter that calls
`StacksTransaction::consensus_deserialize` and then constructs the local
struct fields (mostly `From` impls). Keep `src/stacks_tx/neon_encoder.rs`
unchanged.

### `src/stacks_block/` — ~640 LOC

**Upstream targets**:

- `blockstack_lib::chainstate::stacks::{StacksBlock, StacksBlockHeader}`.
- `blockstack_lib::chainstate::nakamoto::{NakamotoBlock, NakamotoBlockHeader}`.

**Hazards**:

- Local code computes `block_hash` and `index_block_hash` as derived fields.
  Upstream exposes these via `StacksBlockHeader::block_hash()` /
  `NakamotoBlockHeader::block_id()` etc.
- The Nakamoto header's `pox_treatment` `BitVec` lives in
  `stacks_common::bitvec::BitVec`. Local exposes both `data` and `bits`
  fields; upstream exposes the bitvec directly so the encoder will need to
  iterate the bits explicitly.

**Suggested approach**: Smaller than `stacks_tx`. Same pattern: decode
upstream, wrap, leave encoder untouched.

## Workflow for a follow-up session

1. Pick one module from the list above.
2. Read its `deserialize.rs` and `neon_encoder.rs` to understand which fields
   the JS-facing output depends on.
3. Write a `convert_from_upstream` (or `From<&upstream_type>`) function that
   produces the local type, populating any extra fields (offsets, cached raw
   bytes, etc.) explicitly.
4. Rewrite the public neon entry point to:
   - Decode using upstream's `consensus_deserialize`.
   - Convert to the local type.
   - Call the unchanged `neon_encoder`.
5. Delete the now-orphan code from the local `deserialize.rs` (keep only the
   `struct` / `enum` declarations).
6. Run `cargo test --lib` and `npm run build:dev && npm test` to validate.

When all five modules are done, a second pass can delete the local
`struct`/`enum` types entirely and rewrite the `neon_encoder.rs` files
against upstream types directly.

## CI considerations

`stackslib` transitively requires `rusqlite` with a bundled SQLite C library,
which means every CI target needs a working C compiler. The existing matrix
already provides one for every supported target:

- **`linux-x64-musl` / `linux-arm64-musl`**: `x86_64-linux-musl-gcc` /
  `aarch64-linux-musl-gcc` from `musl.cc` are downloaded during the workflow
  and exposed via the standard `CC_<target>` / `CARGO_TARGET_<TARGET>_LINKER`
  env vars. `rusqlite`'s build script picks these up automatically.
- **`linux-x64-glibc` / `linux-arm64-glibc`**: built inside the `rust` Docker
  image, which ships with `gcc`.
- **`macos-x64` / `macos-arm64`**: `clang` is installed on every GitHub-hosted
  macOS runner.
- **`win32-x64`**: MSVC `cl.exe` from the GitHub-hosted Windows runner.

Build wall-clock time will increase noticeably (~+1–3 min per target) because
we now compile ~150 additional crates from upstream and the bundled SQLite C
sources, but no workflow changes are required. The cargo cache action
(`Swatinem/rust-cache`) should keep incremental rebuilds fast.

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
