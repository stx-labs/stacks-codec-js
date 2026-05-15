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

The migration is incremental: each module under `src/` was moved from
"hand-copy of upstream code" to "thin façade that delegates to upstream".
This file documents the pattern, hazards, and design decisions that came up
along the way.

**Status**

- **Phase 1 (complete):** Every consensus-decoding path delegates to upstream
  `StacksMessageCodec` / Clarity parsers first; local types and `convert_*`
  functions bridged the gap to the existing Neon encoders.
- **Phase 2 (complete):** Shadow types and `convert_*` bridges have been
  removed module-by-module. The Neon encoders now operate directly on upstream
  types via **Option A — zero-cost newtype wrappers**:
  `neon_util::Encode<'a, T>` (`#[repr(transparent)]`, holds `&'a T`) so we can
  implement `NeonJsSerialize` for upstream types without violating the orphan
  rule. JS-facing object shapes and TypeScript entry points are unchanged.
  The only deliberate exception is the Stacks 2.x block header (see the
  `stacks_block` row below).

**Phase 2 completed in this repo (snapshot for handoff)**

| Area | What changed |
|------|----------------|
| `src/neon_util.rs` | `Encode<'a, T>` plus `Encode::new`; encoders use `Encode(&value).neon_js_serialize(...)`. |
| `src/clarity_value/` | Neon entry points parse `clarity::vm::types::Value` directly (`deserialize_read` / codec). `neon_encoder.rs` walks the upstream tree; `repr_string` / `type_signature_string` match historical JS output. The legacy `types.rs` / `deserialize.rs` have been deleted — no local shadow tree remains. |
| `src/post_condition/` | `deserialize.rs` re-exports upstream post-condition types; `deserialize_post_condition` wraps `consensus_deserialize`. `neon_encoder.rs` implements `NeonJsSerialize` for `Encode<'_, …>`. |
| `src/stacks_tx/` | `deserialize.rs` is a thin re-export + `deserialize_transaction`. `neon_encoder.rs` rewritten for upstream payloads, auth, multisigs, coinbase fan-out, microblocks, post-condition buffer re-serialization. `mod.rs` hashes txid and calls `Encode(&tx)`. **Note:** `clarity_version` on versioned smart-contract payloads uses an explicit map from `clarity::vm::ClarityVersion` to the **1-based wire byte** (`Clarity2` → `2`), not `as u8` on the enum. |
| `src/address/` | `neon_encoder.rs` shares `NeonJsSerialize` for upstream `StacksAddress` / principal types. `decode_clarity_value_to_principal_inner` parses principal-prefixed inputs through upstream `clarity::vm::types::PrincipalData::consensus_deserialize`; the legacy buffer-prefixed (`0x02 || version || hash160`) shorthand is still handled manually, since it isn't a real Clarity buffer encoding. |
| `src/stacks_block/` | Nakamoto path: `deserialize_nakamoto_block` builds upstream `NakamotoBlock` (header via codec, tx vector read loosely — same rationale as Phase 1: no duplicate-txid / zero-tx / merkle checks from upstream block parsers). Stacks 2.x: **local shadow** `StacksBlockHeader` still used so **any 80-byte VRF blob** is accepted (upstream validates curve points). `neon_encoder.rs`: `Encode` wrappers; `BitVec<4000>` exposes wire-identical `data` hex and an MSB-first `bits` array (historical JS behavior; differs from upstream `BitVec::get` LSB order). |
| `src/pox_events/` | **Migrated:** `decode_pox_synthetic_event` walks `clarity::vm::types::Value`; `decode_pox_event` deserializes with `<Value as StacksMessageCodec>::consensus_deserialize`. PoX logic is still bespoke; only the Clarity representation is upstream. |

**Tests:** `cargo test --lib` and `npm test` (Jest) are green (41 + 41 tests respectively). `stacks_tx` post-condition unit tests expect `TransactionPostConditionMode::Originator` where the wire byte is `0x03`, matching `stackslib`.

**Residual intentional locals (not bugs)**

1. **`stacks_block` Stacks 2.x header** — read field-by-field rather than via
   upstream `StacksBlockHeader::consensus_deserialize` so any 80-byte VRF blob
   is accepted (upstream validates curve points; this crate historically did
   not). Block-body tx vectors are also read loosely (no duplicate-txid /
   zero-tx / merkle checks).
2. **`address::decode_clarity_value_to_principal_inner` buffer arm** — accepts
   the legacy `0x02 || version || hash160` shorthand that some callers use.
   This is **not** a real Clarity buffer (which carries a u32 length prefix)
   and therefore cannot go through `Value::deserialize_read`.

A `grep` for `convert_`, hand-rolled `XxxYy::deserialize` impls, or shadow
structs outside these two exceptions should now come up clean.

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
**Phase 2 (see the status table at the top) is that pass, in progress:** the
`Encode<'_, T>` wrapper replaces maintaining parallel local `struct` trees for
modules that are done; the paragraphs below predate that pass.

> **Documentation note:** The sections titled **Reference module: …** describe
> **Phase 1** in detail (façades, `convert_*`, local shadow types). They remain
> useful for hazards and rationale. For **what the code does now**, start with
> the **Phase 2 completed in this repo (snapshot for handoff)** table above.

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
- `src/address/neon_encoder.rs` — shared `NeonJsSerialize` for upstream
  `StacksAddress` / `PrincipalData`, reused by `stacks_tx` and
  `post_condition`.
- `src/address/mod.rs` — `decode_stacks_address`, `stacks_to_bitcoin_address`,
  `bitcoin_to_stacks_address`, etc. kept their behavior.
  `decode_clarity_value_to_principal_inner` now parses principal-prefixed
  inputs through upstream `clarity::vm::types::PrincipalData::consensus_deserialize`
  and only keeps the legacy buffer-prefixed shorthand (`0x02 || version ||
  hash160`) as a hand-rolled arm — see the **Residual intentional locals**
  note at the top.

Net: ~1000 LOC of hand-copied algorithm code replaced by ~100 LOC of
delegation.

## Reference module: `src/clarity_value/`

Done (Phase 2). The module now consists of just two files:

- `src/clarity_value/mod.rs` — the four Neon entry points
  (`decode_clarity_value`, `decode_clarity_value_type_name`,
  `decode_clarity_value_to_repr`, `decode_clarity_value_array`) call
  `clarity::vm::types::Value::deserialize_read(r, None, false)` directly and
  hand the upstream value to the encoder.
- `src/clarity_value/neon_encoder.rs` — `decode_clarity_val` walks
  `clarity::vm::types::Value` directly. `repr_string` and
  `type_signature_string` are kept here as free functions because their
  output is part of the JS contract and intentionally differs from upstream
  `fmt::Display` (`repr` uses the historical Stacks API format; the type
  signature format uses `(string-utf8 N)` where `N` is `chars * 4`).

The legacy `clarity_value/deserialize.rs` and `clarity_value/types.rs`
(local `TypePrefix`, `Value`, `ClarityValue`, `StandardPrincipalData`,
`QualifiedContractIdentifier`, `ClarityName`, `ContractName`, plus the
`convert_value` / `convert_clarity_value` bridge) have been deleted. Nothing
in the crate imports them anymore.

**Bytes-capture nuance**: The original deserializer captured each nested
value's raw byte slice from the input cursor. The encoder preserves this by
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
  - `convert_post_condition` is exposed `pub(crate)` so the `stacks_tx`
    façade reuses it for the post-conditions vector inside a transaction.
  - The transient `StacksAddress::deserialize` helper that lived here for
    `stacks_tx`'s benefit was deleted in the same pass that migrated
    `stacks_tx`; the local `StacksAddress` struct is now constructed solely
    from `convert_address(upstream)` calls.
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

## Reference module: `src/stacks_tx/`

Done. The biggest module so far (~1700 LOC across `deserialize.rs` +
`neon_encoder.rs`) was migrated to a thin façade over upstream's
`<StacksTransaction as StacksMessageCodec>::consensus_deserialize`:

- `src/stacks_tx/deserialize.rs`:
  - `StacksTransaction::deserialize` is now ~6 lines: it calls upstream's
    `consensus_deserialize` and runs `convert_transaction` to lower the
    upstream value tree into the local types so the Neon encoder doesn't
    need to change.
  - Local types (`StacksTransaction`, `TransactionAuth`,
    `TransactionSpendingCondition`, the *two* multisig hash-mode flavours
    rolled into one local enum, `TransactionPayload`, `PrincipalData`,
    `StandardPrincipalData`, `QualifiedContractIdentifier`,
    `TransactionContractCall`, `TransactionSmartContract`,
    `TransactionTenureChange`, `StacksMicroblockHeader`, `VRFProof`,
    `CoinbasePayload`, `TokenTransferMemo`, `MessageSignature`,
    `Secp256k1PublicKey`, `StacksPublicKeyBuffer`, `StacksString`,
    `BlockHeaderHash`, `Sha512Trunc256Sum`, `ClarityVersion`,
    `TenureChangeCause`, etc.) are all kept verbatim. Their `from_u8` /
    discriminant helpers are kept too because `neon_encoder.rs` and the
    `stacks_block` module still rely on them.
  - All hand-rolled `XxxYy::deserialize` impls were removed and replaced by
    a tree of straight-line `convert_*` functions, one per local type.
  - `convert_post_condition` from `post_condition::deserialize` is reused
    directly for the `post_conditions: Vec<TransactionPostCondition>` field.
- `src/stacks_tx/neon_encoder.rs` — unchanged.
- `src/stacks_tx/mod.rs` — unchanged. The public Neon entry point
  `decode_transaction` keeps its current TypeScript-facing shape exactly.

**Hazards encountered and how they were handled**:

1. **`post_conditions_serialized: Vec<u8>`** — the JS-facing
   `post_conditions_buffer` field. Upstream doesn't materialize this. The
   façade re-serializes the `(mode, len, [post_conditions])` triple via the
   canonical `StacksMessageCodec::consensus_serialize` to produce a
   byte-identical buffer, since wire encoding is deterministic.
2. **`OrderIndependentMultisig` vs `Multisig`** — upstream splits SIP-040
   non-sequential multisig into its own variant
   (`OrderIndependentMultisigSpendingCondition`, hash modes `0x05` / `0x07`),
   while the local enum lumps them as `MultisigHashMode::P2SHNonSequential`
   / `P2WSHNonSequential` inside the regular `Multisig` variant. The
   converter folds upstream's `OrderIndependentMultisig` arm back into the
   local `Multisig` variant, mapping the hash-mode bytes the obvious way.
3. **`TransactionPayload::SmartContract(_, Option<ClarityVersion>)`** —
   upstream collapses `TxPayloadSmartContract` (id 1) and
   `TxPayloadVersionedSmartContract` (id 6) into a single variant tagged by
   the optional Clarity version. The converter fans them back out so the JS
   `type_id` discriminator stays 1 vs 6.
4. **`TransactionPayload::Coinbase(_, Option<PrincipalData>, Option<VRFProof>)`**
   — upstream collapses `Coinbase` (id 4), `CoinbaseToAltRecipient` (id 5),
   and `NakamotoCoinbase` (id 8) into one variant discriminated by which of
   the two optional fields are populated. The converter unfolds them based
   on `(recipient_opt, vrf_opt)` so all three JS shapes survive.
5. **`ClarityVersion::Clarity6`** — local enum has variants `Clarity1..6`,
   upstream only `Clarity1..5`. The local `Clarity6` variant is now reserved
   for a future upstream release; the wire-format decoder cannot produce it
   today, and will start producing it automatically once upstream adds the
   variant and we bump the pin.
6. **`Secp256k1PublicKey` framing** — upstream stores parsed keys as
   `LibSecp256k1PublicKey` and exposes `to_bytes_compressed()` (always 33
   bytes, even when the wire-format flag says "uncompressed"). The local
   `Secp256k1PublicKey { key: StacksPublicKeyBuffer([u8; 33]), compressed:
   bool }` matches this exactly, so the converter just calls
   `to_bytes_compressed()` and copies the `compressed()` flag through.
7. **`TenureChangeCause` lacks `PartialEq` upstream** — upstream
   intentionally hides it; the converter uses an exhaustive `match` on the
   variants instead.
8. **`StacksMicroblockHeader::serialized_bytes`** — upstream doesn't carry
   the raw on-wire byte slice; the converter recovers it via
   `<UpstreamStacksMicroblockHeader as StacksMessageCodec>::serialize_to_vec`,
   again leveraging deterministic wire encoding.
9. **`ContractName` → `ClarityName`** — upstream's
   `TransactionContractCall.contract_name` and
   `TransactionSmartContract.name` are `ContractName` (a stricter
   guarded-string than `ClarityName`). The Neon encoder only ever calls
   `as_str()` on them, so the converter round-trips via `to_string()` into
   the looser local `ClarityName` without losing information.

All five Rust unit tests in `stacks_tx::deserialize::tests` pass (including
`test_decode_bug` and the four post-condition shape tests), as do all 41
Jest suites end-to-end — most importantly `tx-decode`, `tx-decode-2.1`,
`tx-decode-3.0`, and `nakamoto-block`, which exercise full transaction
decode for every payload variant in the wire format.

## Reference module: `src/stacks_block/`

*(Phase 1 write-up; **Phase 2** replaced the Nakamoto local structs — see the
snapshot table at the top.)*

Still accurate today:

- **Stacks 2.x header:** read field-by-field (not upstream
  `StacksBlockHeader::consensus_deserialize`) so any **80-byte VRF blob** is
  accepted; local `block_hash()` hashing remains unconditional (no genesis
  short-circuit like upstream).
- **Block body:** length-prefixed tx vector parsed with upstream
  `StacksTransaction::consensus_deserialize` only — **not** upstream’s full
  `StacksBlock` / `NakamotoBlock` parser (avoids zero-tx / merkle / duplicate
  txid rules that this crate never enforced).
- **Phase 2:** Nakamoto path returns upstream `NakamotoBlock` /
  `NakamotoBlockHeader`; txs are upstream `Vec<StacksTransaction>` (no
  `convert_transaction`). Neon uses `Encode<'_, _>`; upstream `BitVec<4000>`
  is encoded with wire-true `data` hex and an **MSB-first** `bits` array.

At last doc update, `cargo test --lib` and `npm test` were green, including
`nakamoto-block` against known mainnet hashes.
## Future cleanup pass

Phase 2 is complete. `post_condition`, `stacks_tx`, `stacks_block` (Nakamoto
path), `pox_events`, `clarity_value`, and `address` all emit from upstream
types via `Encode<'_, T>`. The only remaining hand-rolled consensus parsing
matches the **Residual intentional locals** list at the top:

- `stacks_block`: Stacks 2.x header (permissive 80-byte VRF) and the loose
  block-body tx vector.
- `address`: the legacy `0x02 || version || hash160` shorthand inside
  `decode_clarity_value_to_principal_inner` (not a real Clarity buffer).

A `grep` for `convert_`, hand-rolled `XxxYy::deserialize` impls, or shadow
structs outside those two exceptions comes up clean today.

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
