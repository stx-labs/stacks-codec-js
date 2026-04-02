# Stacks Codec JS

`@stacks/codec` is a Node.js [native addon](https://nodejs.org/api/addons.html) library written in
Rust, which provides functions for decoding binary/wire formats used in the Stacks blockchain.
Features include Clarity values, transactions, post-conditions, Stacks and Bitcoin addresses, PoX
synthetic event parsing, and more.

Various ASM/SIMD optimizations are used in areas which are prone to causing CPU bottlenecks when
used in hot paths, e.g. decoding raw Clarity values on the fly.

_This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon)._

### Runtime Support
* Node.js v20+

### Platform Support

* Linux (glibc) x86_64
* Linux (glibc) aarch64
* Linux (musl) x86_64
* Linux (musl) aarch64
* MacOS x86_64
* MacOS aarch64 (Apple Silicon M1)
* Windows x86_64

## Installation and Usage

```shell
npm install @stacks/codec
```

### Decoding serialized Clarity values
_Example Clarity value:_
<details>
<summary>Expand</summary>

```clar
(tuple 
  (active true) 
  (address 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) 
  (alias "Alice") 
  (balance u2000) 
  (ping (ok 250)) 
  (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) 
  (tags (list u"contributor" u"og \u{e2ad90}" u"clarity"))
)
```
</details>

#### Decode serialized Clarity value to repr string
```ts
import * as assert from 'node:assert';
import { decodeClarityValueToRepr } from '@stacks/codec';

// Serialized hex string of the example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const hex = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';

const reprStr = decodeClarityValueToRepr(hex);

assert.strictEqual(
  reprStr, 
  `(tuple (active true) (address 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) (alias "Alice") (balance u2000) (ping (ok 250)) (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) (tags (list u"contributor" u"og \\u{e2ad90}" u"clarity")))`
);
```

#### Decode serialized Clarity value to object
```ts
import * as assert from 'node:assert';
import { decodeClarityValue } from '@stacks/codec';

// Serialized hex string of the example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const hex = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';

// Decode into JSON object
const decoded = decodeClarityValue(hex);
```

Decoded Clarity value output result:

<details>
<summary>Expand</summary>

```ts
// Result object
assert.deepStrictEqual(decoded, {
  repr: `(tuple (active true) (address 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) (alias "Alice") (balance u2000) (ping (ok 250)) (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) (tags (list u"contributor" u"og \\u{e2ad90}" u"clarity")))`,
  hex: '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479',
  type_id: 12,
  data: {
    active: { repr: 'true', hex: '0x03', type_id: 3, value: true },
    address: {
      repr: "'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW",
      hex: '0x0516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b',
      type_id: 5,
      address_version: 22,
      address_hash_bytes: '0x142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b',
      address: 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW'
    },
    alias: {
      repr: '"Alice"',
      hex: '0x0d00000005416c696365',
      type_id: 13,
      data: 'Alice'
    },
    balance: {
      repr: 'u2000',
      hex: '0x01000000000000000000000000000007d0',
      type_id: 1,
      value: '2000'
    },
    ping: {
      repr: '(ok 250)',
      hex: '0x0700000000000000000000000000000000fa',
      type_id: 7,
      value: {
        repr: '250',
        hex: '0x00000000000000000000000000000000fa',
        type_id: 0,
        value: '250'
      }
    },
    public_key: {
      repr: '(some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)',
      hex: '0x0a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58',
      type_id: 10,
      value: {
        repr: '0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58',
        hex: '0x020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58',
        type_id: 2,
        buffer: '0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58'
      }
    },
    tags: {
      repr: '(list u"contributor" u"og \\u{e2ad90}" u"clarity")',
      hex: '0x0b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479',
      type_id: 11,
      list: [
        {
          repr: 'u"contributor"',
          hex: '0x0e0000000b636f6e7472696275746f72',
          type_id: 14,
          data: 'contributor'
        },
        {
          repr: 'u"og \\u{e2ad90}"',
          hex: '0x0e000000066f6720e2ad90',
          type_id: 14,
          data: 'og ⭐'
        },
        {
          repr: 'u"clarity"',
          hex: '0x0e00000007636c6172697479',
          type_id: 14,
          data: 'clarity'
        }
      ]
    }
  }
});
```
</details>

#### Decode serialized Clarity value to typed object

```ts
import * as assert from 'node:assert';
import { 
  decodeClarityValue,
  ClarityTypeID,
  ClarityValueTuple,
  ClarityValuePrincipalStandard,
  ClarityValueStringAscii,
  ClarityValueInt,
  ClarityValueBoolTrue,
  ClarityValueResponseOk,
  ClarityValueBuffer,
  ClarityValueList,
  ClarityValueStringUtf8,
  ClarityValueOptionalSome,
  ClarityValueUInt,
} from '@stacks/codec';

// Serialized hex string of the example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const hex = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';

// Provide a typed view of the decoded value
const decoded = decodeClarityValue<ClarityValueTuple<{
  active: ClarityValueBoolTrue;
  address: ClarityValuePrincipalStandard;
  alias: ClarityValueStringAscii;
  balance: ClarityValueUInt;
  ping: ClarityValueResponseOk<ClarityValueInt>;
  public_key: ClarityValueOptionalSome<ClarityValueBuffer>;
  tags: ClarityValueList<ClarityValueStringUtf8>;
}>>(hex);

assert.deepStrictEqual(decoded.data.tags.list.map(v => v.data), ['contributor', 'og ⭐', 'clarity']);
```

The annotated types are _not_ automatically checked at runtime, so type checks are needed for error handling:

<details>
<summary>Expand</summary>

```ts
assert.strictEqual(decoded.type_id, ClarityTypeID.Tuple);
assert.strictEqual(decoded.data.active.type_id, ClarityTypeID.BoolTrue);
assert.strictEqual(decoded.data.address.type_id, ClarityTypeID.PrincipalStandard);
assert.strictEqual(decoded.data.alias.type_id, ClarityTypeID.StringAscii);
assert.strictEqual(decoded.data.balance.type_id, ClarityTypeID.UInt);
assert.strictEqual(decoded.data.ping.type_id, ClarityTypeID.ResponseOk);
assert.strictEqual(decoded.data.ping.value.type_id, ClarityTypeID.Int);
assert.strictEqual(decoded.data.public_key.type_id, ClarityTypeID.OptionalSome);
assert.strictEqual(decoded.data.public_key.value.type_id, ClarityTypeID.Buffer);
assert.strictEqual(decoded.data.tags.type_id, ClarityTypeID.List);
assert.strictEqual(decoded.data.tags.list[0].type_id, ClarityTypeID.StringUtf8);

// Now we can safely access typed properties
assert.strictEqual(decoded.data.active.value, true);
assert.strictEqual(decoded.data.address.address, 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW');
assert.strictEqual(decoded.data.alias.data, 'Alice');
assert.strictEqual(decoded.data.balance.value, '2000');
assert.strictEqual(decoded.data.ping.value.value, '250');
assert.strictEqual(decoded.data.public_key.value.buffer, '0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58');
assert.deepStrictEqual(decoded.data.tags.list.map(v => v.data), ['contributor', 'og ⭐', 'clarity']);
```
</details>

### Decoding principals

#### Principal from serialized Clarity value

```ts
import * as assert from 'node:assert';
import { decodeClarityValueToPrincipal } from '@stacks/codec';

// Serialized hex string of an example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const standardPrincipal = decodeClarityValueToPrincipal('0x0516a13dce8114be0f707f94470a2e5e86eb402f2923');
assert.strictEqual(principal, 'SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM');

const contractPrincipal = decodeClarityValueToPrincipal('0x0616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c616273');
assert.strictEqual(contractPrincipal, 'SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C.bitcoin-monkeys-labs');
```

#### Stacks address from parts

```ts
import * as assert from 'node:assert';
import { stacksAddressFromParts } from '@stacks/codec';

const stacksAddressData = {
  version: 26,
  // Serialized hex string (0x-prefix optional, Buffer / Uint8Array also accepted)
  hash160: '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6'
};

const stacksAddress = stacksAddressFromParts(stacksAddressData.version, stacksAddressData.hash160);
assert.strictEqual(stacksAddress, 'ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
```

#### Stacks address to parts

```ts
import * as assert from 'node:assert';
import { decodeStacksAddress } from '@stacks/codec';

const [version, hash160] = decodeStacksAddress('ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
assert.strictEqual(version, 26);
assert.strictEqual(hash160, '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6');
```

### Decoding transactions

```ts
import * as assert from 'node:assert';
import { 
  decodeTransaction,
  AnchorModeID,
  TransactionVersion,
  TxPayloadTypeID,
  TxSpendingConditionSingleSigHashMode,
  TxPublicKeyEncoding,
  ClarityTypeID,
  PostConditionModeID,
  PostConditionAuthFlag,
  PostConditionAssetInfoID,
  PostConditionNonfungibleConditionCodeID,
  PostConditionNonFungibleConditionName
} from '@stacks/codec';

// Serialized hex string (0x-prefix optional, Buffer / Uint8Array also accepted)
const rawTx = '0000000001040089f5fd1f719e4449c980de38e3504be6770a2698000000000000014500000000000001f400008b510c9e20dc22040953d9d7eabf2038008fa4d89a5a6cb78bb9d513e75cd0df3924af9ce3b5f185705bc2f6ba3071710ec6a8803ed6da4addc40a05a01ee0f503020000000102021689f5fd1f719e4449c980de38e3504be6770a269816a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627314626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba1002162bcf9762d5b90bc36dc1b4759b1727690f92ddd30e6d61726b6574706c6163652d76340a6c6973742d6173736574000000040616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba010000000000000000000000000c84588001000000000000000000000000000000c8';

const decoded = decodeTransaction(rawTx);
```

Decoded transaction output result:
<details>
<summary>Expand</summary>

```ts
assert.deepStrictEqual(decoded, {
  tx_id: "0x49bcdba540d5c486f6f7a71de639a128a65f0378d7571e20c75a61661dd5b469",
  chain_id: 1,
  version: TransactionVersion.Mainnet,
  anchor_mode: AnchorModeID.Any,
  auth: {
    type_id: PostConditionAuthFlag.Standard,
    origin_condition: {
      hash_mode: TxSpendingConditionSingleSigHashMode.P2PKH,
      key_encoding: TxPublicKeyEncoding.Compressed,
      nonce: "325",
      signature: "0x008b510c9e20dc22040953d9d7eabf2038008fa4d89a5a6cb78bb9d513e75cd0df3924af9ce3b5f185705bc2f6ba3071710ec6a8803ed6da4addc40a05a01ee0f5",
      signer: {
        address: "SP24ZBZ8ZE6F48JE9G3F3HRTG9FK7E2H6K2QZ3Q1K",
        address_hash_bytes: "0x89f5fd1f719e4449c980de38e3504be6770a2698",
        address_version: 22
      },
      tx_fee: "500"
    }
  },
  payload: {
    type_id: TxPayloadTypeID.ContractCall,
    address: "SPNWZ5V2TPWGQGVDR6T7B6RQ4XMGZ4PXTEE0VQ0S",
    address_hash_bytes: "0x2bcf9762d5b90bc36dc1b4759b1727690f92ddd3",
    address_version: 22,
    contract_name: "marketplace-v4",
    function_name: "list-asset",
    function_args: [
      {
        type_id: ClarityTypeID.PrincipalContract,
        hex: "0x0616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c616273",
        repr: "'SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C.bitcoin-monkeys-labs"
      },
      {
        type_id: ClarityTypeID.UInt,
        hex: "0x01000000000000000000000000000008ba",
        repr: "u2234"
      },
      {
        type_id: ClarityTypeID.UInt,
        hex: "0x010000000000000000000000000c845880",
        repr: "u210000000"
      },
      {
        type_id: ClarityTypeID.UInt,
        hex: "0x01000000000000000000000000000000c8",
        repr: "u200"
      }
    ],
    function_args_buffer: "0x000000040616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba010000000000000000000000000c84588001000000000000000000000000000000c8"
  },
  post_condition_mode: PostConditionModeID.Deny,
  post_conditions: [
    {
      condition_code: PostConditionNonfungibleConditionCodeID.Sent,
      condition_name: PostConditionNonFungibleConditionName.Sent,
      asset: {
        asset_name: "bitcoin-monkeys-labs",
        contract_address: "SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C",
        contract_name: "bitcoin-monkeys-labs"
      },
      asset_info_id: PostConditionAssetInfoID.NonfungibleAsset,
      asset_value: {
        hex: "0x01000000000000000000000000000008ba",
        repr: "u2234",
        type_id: ClarityTypeID.UInt
      },
      principal: {
        address: "SP24ZBZ8ZE6F48JE9G3F3HRTG9FK7E2H6K2QZ3Q1K",
        address_hash_bytes: "0x89f5fd1f719e4449c980de38e3504be6770a2698",
        address_version: 22,
        type_id: ClarityTypeID.Buffer
      }
    }
  ],
  post_conditions_buffer: "0x020000000102021689f5fd1f719e4449c980de38e3504be6770a269816a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627314626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba10"
});
```
</details>

### Decoding PoX synthetic events

Decode serialized Clarity values from PoX contract log events into structured PoX synthetic event objects. This is useful for parsing stacking, delegation, and unlocking events emitted by the PoX contract.

```ts
import { decodePoxSyntheticEvent, PoxEventName } from '@stacks/codec';

// Serialized hex string of a PoX synthetic event Clarity value (ResponseOk wrapping a Tuple)
const rawClarityHex = '0x0700...';

const event = decodePoxSyntheticEvent(rawClarityHex, 'mainnet');

if (event !== null) {
  console.log(event.name);                      // e.g. 'stack-stx'
  console.log(event.stacker);                   // e.g. 'SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7'
  console.log(event.locked);                    // String-quoted u128, e.g. '1000000'
  console.log(event.balance);                   // String-quoted u128, e.g. '500000'
  console.log(event.burnchain_unlock_height);   // String-quoted u128, e.g. '150000'
  console.log(event.pox_addr);                  // BTC address string or null
  console.log(event.pox_addr_raw);              // Hex-encoded raw PoX address or null
  console.log(event.data);                      // Event-specific fields (varies by event type)
}
```

Supported event types: `handle-unlock`, `stack-stx`, `stack-increase`, `stack-extend`, `delegate-stx`, `delegate-stack-stx`, `delegate-stack-increase`, `delegate-stack-extend`, `stack-aggregation-commit`, `stack-aggregation-commit-indexed`, `stack-aggregation-increase`, `revoke-delegate-stx`.

Returns `null` when the Clarity value is a `ResponseErr` (indicating a non-event). Bitcoin address encoding supports P2PKH, P2SH, P2WPKH (segwit v0), P2WSH (segwit v0), and P2TR (taproot/segwit v1) address formats.

## Project Layout

The directory structure of this project is:

<pre>
/
├── Cargo.toml    # The Cargo <a href="https://doc.rust-lang.org/cargo/reference/manifest.html">manifest file</a>
├── package.json  # The npm <a href="https://docs.npmjs.com/cli/v7/configuring-npm/package-json">manifest file</a>
├── native/       # The <a href="https://nodejs.org/api/addons.html">Node addon</a> modules built by this project, these are <a href="https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries">dynamically-linked shared objects</a>
├── src/**/*.rs   # Directory containing the Rust source code for the project
|── index.ts      # Typescript definitions for the js interface exposed by the Node addon
|── loader.js     # Script to determine which addon file to load based on the executing target platform
|── loader.d.ts   # Type defintions for the functions exported by the Node addon
|── builder.js    # Script to build the native Node addon for the executing host platform
├── tests/*.ts    # Js/ts unit tests, primarily testing the Neon (rust<->nodejs) interop layer
└── perf-tests/   # Scripts to run performance benchmarks used by commands in package.json
</pre>

The Rust source code inside the `src/**/deserialize.rs` files are responsible for deserializing the Stacks blockchain wire/binary formats defined in [SIP-005](https://github.com/stacksgov/sips/blob/main/sips/sip-005/sip-005-blocks-and-transactions.md). 

## NPM Library Bundling

The Node addon modules for all supported platforms are compiled by [CI](.github/workflows/build.yml) and bundled inside the npm package. The native binary files are small enough that the bundled npm package is an acceptable ~20 MB in size. 