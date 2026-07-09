import type {
  DecodedPostConditionsResult,
  DecodedTxResult,
  DecodedNakamotoBlockResult,
  DecodedStacksBlockResult,
  ClarityValue,
  ClarityValueAbstract,
  PoxEvent,
  SignerMessage,
} from './index.js';

export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): DecodedTxResult;

/**
 * Decode a Nakamoto block (Stacks 3.x+).
 * The input should be the raw binary block data as returned by /v3/blocks/{block_id} endpoint.
 * @param arg - Hex string or Buffer containing the raw block data
 */
export function decodeNakamotoBlock(arg: string | Buffer): DecodedNakamotoBlockResult;

/**
 * Decode a Stacks 2.x block.
 * The input should be the raw binary block data as returned by /v2/blocks/{block_id} endpoint.
 * @param arg - Hex string or Buffer containing the raw block data
 */
export function decodeStacksBlock(arg: string | Buffer): DecodedStacksBlockResult;

export function decodeClarityValueToRepr(arg: string | Buffer): string;

export function decodeClarityValueToTypeName(arg: string | Buffer): string;

export function decodeClarityValue<T extends ClarityValue = ClarityValue>(arg: string | Buffer): T;

/**
 *
 * @param arg
 * @param deep - If not true, then the deserialized objects will only contain the
 * properties `hex, repr, type, type_id`. And nested types like Tuple, List, Response, etc will
 * not contain decoded children.
 * TODO: fix the clarity result type definition to be more accurate.
 */
export function decodeClarityValueList(
  arg: string | Buffer,
  deep?: false | undefined
): ClarityValueAbstract[];

/**
 *
 * @param arg
 * @param deep - If not true, then the deserialized objects will only contain the
 * properties `hex, repr, type, type_id`. And nested types like Tuple, List, Response, etc will
 * not contain decoded children.
 * TODO: fix the clarity result type definition to be more accurate.
 */
export function decodeClarityValueList(arg: string | Buffer, deep: true): ClarityValue[];

export function decodePostConditions(arg: string | Buffer): DecodedPostConditionsResult;

export function stacksToBitcoinAddress(stackAddress: string): string;

export function bitcoinToStacksAddress(bitcoinAddress: string): string;

export function isValidStacksAddress(address: string): boolean;

export function decodeStacksAddress(address: string): [version: number, hash160: string];

export function decodeClarityValueToPrincipal(clarityValue: string | Buffer): string;

export function stacksAddressFromParts(version: number, hash160: string | Buffer): string;

export function memoToString(memo: string | Buffer): string;

/**
 * Decode a serialized Clarity value representing a PoX synthetic event.
 *
 * The native runtime sniffs the Clarity value's shape and routes it to the
 * appropriate per-version decoder:
 *
 * - A flat tuple with a `topic` ASCII field → pox-5 event (returns a
 *   {@link Pox5Event}).
 * - A `Response(Ok({ stacker, locked, ..., name, data }))` tuple →
 *   pox-2 / pox-3 / pox-4 event (returns a {@link Pox4Event}).
 * - Anything else → `null`. This includes pox-2/3/4 `Response(Err _)`
 *   payloads from failed stacking calls.
 *
 * Narrow the result on `event.name` (a string-literal field present on every
 * variant). The pox-4 and pox-5 name sets don't overlap, so a single switch
 * over `name` is enough to discriminate.
 *
 * @param arg - Hex string or Buffer containing the serialized Clarity value
 * @param network - The Stacks network type (used only by the pox-4 decoder
 *   when encoding pox-addr bytes into a BTC address; ignored by pox-5).
 * @returns The decoded PoX event, or `null` if the Clarity value isn't a
 *   recognized event shape.
 */
export function decodePoxSyntheticEvent(
  arg: string | Buffer,
  network: 'mainnet' | 'testnet' | 'devnet' | 'mocknet'
): PoxEvent | null;

/**
 * Decode a StackerDB signer message from its consensus wire format, as stored
 * in the `.signers-*` StackerDB contracts / emitted over the signer network.
 *
 * The block-related messages consumers index are fully decoded
 * (`block_proposal`, `block_response`, `block_pushed`, `block_pre_commit`).
 * The epoch-2.5 `mock_*` messages and `state_machine_update` are recognized
 * but surfaced as an `unsupported` shape carrying only their discriminant.
 *
 * Discriminate on `message.type_name` (or the numeric `type_id`).
 *
 * @param arg - Hex string or Buffer containing the serialized signer message
 */
export function decodeSignerMessage(arg: string | Buffer): SignerMessage;
