import * as bindings from './loader.js';
export * from './loader.js';
export const StacksNativeEncodingBindings = bindings;
export default StacksNativeEncodingBindings;

export type TxPostCondition =
  | PostConditionStx
  | PostConditionFungible
  | PostConditionNonfungible
  | PostConditionStaking
  | PostConditionPox;

export interface DecodedPostConditionsResult {
  post_condition_mode: PostConditionModeID;
  post_conditions: TxPostCondition[];
}

export interface DecodedTxResult {
  /** Hex encoded string of the serialized transaction */
  tx_id: string;
  version: TransactionVersion;
  chain_id: number;
  auth: TxAuthStandard | TxAuthSponsored;
  anchor_mode: AnchorModeID;
  post_condition_mode: PostConditionModeID;
  post_conditions: TxPostCondition[];
  /** Hex string */
  post_conditions_buffer: string;
  payload:
    | TxPayloadTokenTransfer
    | TxPayloadSmartContract
    | TxPayloadContractCall
    | TxPayloadPoisonMicroblock
    | TxPayloadCoinbase
    | TxPayloadCoinbaseToAltRecipient
    | TxPayloadVersionedSmartContract
    | TxPayloadTenureChange
    | TxPayloadNakamotoCoinbase;
}

export enum PostConditionAssetInfoID {
  STX = 0,
  FungibleAsset = 1,
  NonfungibleAsset = 2,
  Staking = 3,
  Pox = 4,
}

export interface PostConditionStx {
  asset_info_id: PostConditionAssetInfoID.STX;
  principal: PostConditionPrincipal;
  condition_code: PostConditionFungibleConditionCodeID;
  condition_name: PostConditionFungibleConditionCodeName;
  amount: string;
}

export interface PostConditionFungible {
  asset_info_id: PostConditionAssetInfoID.FungibleAsset;
  principal: PostConditionPrincipal;
  asset: PostConditionAssetInfo;
  condition_code: PostConditionFungibleConditionCodeID;
  condition_name: PostConditionFungibleConditionCodeName;
  amount: string;
}

export interface PostConditionNonfungible {
  asset_info_id: PostConditionAssetInfoID.NonfungibleAsset;
  principal: PostConditionPrincipal;
  asset: PostConditionAssetInfo;
  asset_value: ClarityValueAbstract;
  condition_code: PostConditionNonfungibleConditionCodeID;
  condition_name: PostConditionNonFungibleConditionName;
}

/**
 * Constrains how much STX a principal may stake (lock for PoX) during the
 * transaction. Only valid in Stacks epoch 4.0 and later.
 */
export interface PostConditionStaking {
  asset_info_id: PostConditionAssetInfoID.Staking;
  principal: PostConditionPrincipal;
  condition_code: PostConditionFungibleConditionCodeID;
  condition_name: PostConditionFungibleConditionCodeName;
  amount: string;
}

/**
 * Constrains whether a principal may perform a position-altering PoX
 * operation (`unstake`, `unstake-sbtc`, `update-bond-registration`,
 * `announce-l1-early-exit`) during the transaction. Only valid in Stacks
 * epoch 4.0 and later.
 */
export interface PostConditionPox {
  asset_info_id: PostConditionAssetInfoID.Pox;
  principal: PostConditionPrincipal;
  condition_code: PostConditionPoxConditionCodeID;
  condition_name: PostConditionPoxConditionCodeName;
}

export interface PostConditionAssetInfo {
  contract_address: string;
  contract_name: string;
  asset_name: string;
}

export enum PostConditionPoxConditionCodeID {
  NotPerformed = 0x30,
  MaybePerformed = 0x31,
  Performed = 0x32,
}

export enum PostConditionPoxConditionCodeName {
  NotPerformed = 'not_performed',
  MaybePerformed = 'maybe_performed',
  Performed = 'performed',
}

export enum PostConditionNonfungibleConditionCodeID {
  Sent = 0x10,
  NotSent = 0x11,
  MaybeSent = 0x12,
}

export enum PostConditionNonFungibleConditionName {
  Sent = 'sent',
  NotSent = 'not_sent',
  MaybeSent = 'maybe_sent',
}

export enum PostConditionFungibleConditionCodeID {
  SentEq = 0x01,
  SentGt = 0x02,
  SentGe = 0x03,
  SentLt = 0x04,
  SentLe = 0x05,
}

export enum PostConditionFungibleConditionCodeName {
  SentEq = 'sent_equal_to',
  SentGt = 'sent_greater_than',
  SentGe = 'sent_greater_than_or_equal_to',
  SentLt = 'sent_less_than',
  SentLe = 'sent_less_than_or_equal_to',
}

export enum PostConditionPrincipalTypeID {
  /** A STX post-condition, which pertains to the origin account's STX. */
  Origin = 0x01,
  /** A Fungible token post-condition, which pertains to one of the origin account's fungible tokens. */
  Standard = 0x02,
  /** A Non-fungible token post-condition, which pertains to one of the origin account's non-fungible tokens. */
  Contract = 0x03,
}

export type PostConditionPrincipal =
  | PostConditionPrincipalOrigin
  | PostConditionPrincipalStandard
  | PostConditionPrincipalContract;

export interface PostConditionPrincipalOrigin {
  type_id: PostConditionPrincipalTypeID.Origin;
}

export interface PostConditionPrincipalStandard {
  type_id: PostConditionPrincipalTypeID.Standard;
  address_version: number;
  /** Hex string */
  address_hash_bytes: string;
  address: string;
}

export interface PostConditionPrincipalContract {
  type_id: PostConditionPrincipalTypeID.Contract;
  address_version: number;
  /** Hex string */
  address_hash_bytes: string;
  address: string;
  contract_name: string;
}

export interface TxPayloadTokenTransfer {
  type_id: TxPayloadTypeID.TokenTransfer;
  recipient: PrincipalStandardData | PrincipalContractData;
  amount: string;
  /** Hex encoded string of the 34-bytes */
  memo_hex: string;
}

export enum PrincipalTypeID {
  Standard = 5,
  Contract = 6,
}

export interface PrincipalStandardData {
  type_id: PrincipalTypeID.Standard;
  address_version: number;
  /** Hex string */
  address_hash_bytes: string;
  address: string;
}

export interface PrincipalContractData {
  type_id: PrincipalTypeID.Contract;
  contract_name: string;
  address_version: number;
  /** Hex string */
  address_hash_bytes: string;
  address: string;
}

export interface TxPayloadSmartContract {
  type_id: TxPayloadTypeID.SmartContract;
  contract_name: string;
  code_body: string;
}

export interface TxPayloadContractCall {
  type_id: TxPayloadTypeID.ContractCall;
  address_version: number;
  /** Hex string */
  address_hash_bytes: string;
  address: string;
  contract_name: string;
  function_name: string;
  function_args: ClarityValueAbstract[];
  /** Hex string */
  function_args_buffer: string;
}

export interface TxPayloadPoisonMicroblock {
  type_id: TxPayloadTypeID.PoisonMicroblock;
  microblock_header_1: TxMicroblockHeader;
  microblock_header_2: TxMicroblockHeader;
}

export interface TxPayloadCoinbase {
  type_id: TxPayloadTypeID.Coinbase;
  /** Hex string */
  payload_buffer: string;
}

export interface TxPayloadCoinbaseToAltRecipient {
  type_id: TxPayloadTypeID.CoinbaseToAltRecipient;
  /** Hex string */
  payload_buffer: string;
  recipient: PrincipalStandardData | PrincipalContractData;
}

export interface TxPayloadNakamotoCoinbase {
  type_id: TxPayloadTypeID.NakamotoCoinbase;
  /** Hex string */
  payload_buffer: string;
  /** Optional, null if not specified */
  recipient: PrincipalStandardData | PrincipalContractData | null;
  /** Hex string */
  vrf_proof: string;
}

export interface TxPayloadVersionedSmartContract {
  type_id: TxPayloadTypeID.VersionedSmartContract;
  clarity_version: ClarityVersion;
  contract_name: string;
  code_body: string;
}

export interface TxPayloadTenureChange {
  type_id: TxPayloadTypeID.TenureChange;
  /** Consensus hash of this tenure.  Corresponds to the sortition in which the miner of this
   * block was chosen. It may be the case that this miner's tenure gets _extended_ across
   * subsequent sortitions; if this happens, then this `consensus_hash` value _remains the same_
   * as the sortition in which the winning block-commit was mined. */
  tenure_consensus_hash: string;
  /** Consensus hash of the previous tenure. Corresponds to the sortition of the previous winning block-commit. */
  prev_tenure_consensus_hash: string;
  /** Current consensus hash on the underlying burnchain. Corresponds to the last-seen sortition. */
  burn_view_consensus_hash: string;
  /** The StacksBlockId of the last block from the previous tenure */
  previous_tenure_end: string;
  /** The number of blocks produced since the last sortition-linked tenure */
  previous_tenure_blocks: number;
  /** Cause of change in mining tenure. Depending on cause, tenure can be ended or extended. */
  cause: TenureChangeCause;
  /** (Hex string) The ECDSA public key hash of the current tenure */
  pubkey_hash: string;
}

export enum TenureChangeCause {
  /** A valid winning block-commit */
  BlockFound = 0,
  /** The next burnchain block is taking too long, so extend the runtime budget */
  Extended = 1,
  /** SIP-034: extend specific dimensions - runtime */
  ExtendedRuntime = 2,
  /** SIP-034: extend specific dimensions - read count */
  ExtendedReadCount = 3,
  /** SIP-034: extend specific dimensions - read length */
  ExtendedReadLength = 4,
  /** SIP-034: extend specific dimensions - write count */
  ExtendedWriteCount = 5,
  /** SIP-034: extend specific dimensions - write length */
  ExtendedWriteLength = 6,
}

export enum TxPayloadTypeID {
  TokenTransfer = 0,
  SmartContract = 1,
  ContractCall = 2,
  PoisonMicroblock = 3,
  Coinbase = 4,
  CoinbaseToAltRecipient = 5,
  VersionedSmartContract = 6,
  TenureChange = 7,
  NakamotoCoinbase = 8,
}

export enum PostConditionAuthFlag {
  Standard = 0x04,
  Sponsored = 0x05,
}

export interface TxAuthStandard {
  type_id: PostConditionAuthFlag.Standard;
  origin_condition: DecodedTxSpendingConditionSingleSig | DecodedTxSpendingConditionMultiSig;
}

export interface TxAuthSponsored {
  type_id: PostConditionAuthFlag.Sponsored;
  origin_condition: DecodedTxSpendingConditionSingleSig | DecodedTxSpendingConditionMultiSig;
  sponsor_condition: DecodedTxSpendingConditionSingleSig | DecodedTxSpendingConditionMultiSig;
}

export enum TxSpendingConditionSingleSigHashMode {
  /** hash160(public-key), same as bitcoin's p2pkh */
  P2PKH = 0x00,
  /** hash160(segwit-program-00(p2pkh)), same as bitcoin's p2sh-p2wpkh */
  P2WPKH = 0x02,
}

export enum TxSpendingConditionMultiSigHashMode {
  /** hash160(multisig-redeem-script), same as bitcoin's multisig p2sh */
  P2SH = 0x01,
  /** hash160(multisig-redeem-script), same as bitcoin's multisig p2sh (non-sequential signing) */
  P2SHNonSequential = 0x05,
  /** hash160(segwit-program-00(public-keys)), same as bitcoin's p2sh-p2wsh */
  P2WSH = 0x03,
  /** hash160(segwit-program-00(public-keys)), same as bitcoin's p2sh-p2wsh (non-sequential signing) */
  P2WSHNonSequential = 0x07,
}

export enum ClarityVersion {
  Clarity1 = 1,
  Clarity2 = 2,
  Clarity3 = 3,
  Clarity4 = 4,
  Clarity5 = 5,
  Clarity6 = 6,
}

export interface DecodedTxSpendingConditionSingleSig {
  hash_mode: TxSpendingConditionSingleSigHashMode;
  signer: DecodedStacksAddress;
  nonce: string;
  tx_fee: string;
  /** A 1-byte public key encoding field to indicate whether or not the public key should be compressed before hashing. */
  key_encoding: TxPublicKeyEncoding;
  signature: string;
}

export interface DecodedTxSpendingConditionMultiSig {
  hash_mode: TxSpendingConditionMultiSigHashMode;
  signer: DecodedStacksAddress;
  nonce: string;
  tx_fee: string;
  fields: (TxAuthFieldPublicKey | TxAuthFieldSignature)[];
  signatures_required: number;
}

export enum TxAuthFieldTypeID {
  /** The next 33 bytes are a compressed secp256k1 public key. If the field ID is 0x00, the key will be loaded as a compressed secp256k1 public key. */
  PublicKeyCompressed = 0x00,
  /** The next 33 bytes are a compressed secp256k1 public key. If it is 0x01, then the key will be loaded as an uncompressed secp256k1 public key. */
  PublicKeyUncompressed = 0x01,
  /** The next 65 bytes are a recoverable secp256k1 ECDSA signature. If the field ID is 0x02, then the recovered public key will be loaded as a compressed public key. */
  SignatureCompressed = 0x02,
  /** The next 65 bytes are a recoverable secp256k1 ECDSA signature. If it is 0x03, then the recovered public key will be loaded as an uncompressed public key. */
  SignatureUncompressed = 0x03,
}

export interface TxAuthFieldPublicKey {
  type_id: TxAuthFieldTypeID.PublicKeyCompressed | TxAuthFieldTypeID.PublicKeyUncompressed;
  /** Hex encoded public key bytes. */
  public_key: string;
}

export interface TxAuthFieldSignature {
  type_id: TxAuthFieldTypeID.SignatureCompressed | TxAuthFieldTypeID.SignatureUncompressed;
  /** Hex encoded signatures bytes. */
  signature: string;
}

export interface TxMicroblockHeader {
  /** Hex string */
  buffer: string;
  version: number;
  sequence: number;
  /** Hex string */
  prev_block: string;
  /** Hex string */
  tx_merkle_root: string;
  /** Hex string */
  signature: string;
}

export enum TxPublicKeyEncoding {
  Compressed = 0x00,
  Uncompressed = 0x01,
}

export interface DecodedStacksAddress {
  address_version: number;
  /** Hex-encoded string of the hash160 signer address. */
  address_hash_bytes: string;
  address: string;
}

export enum TransactionVersion {
  Mainnet = 0x00,
  Testnet = 0x80,
}

export enum AnchorModeID {
  /** The transaction MUST be included in an anchored block. */
  OnChainOnly = 1,
  /** The transaction MUST be included in a microblock. */
  OffChainOnly = 2,
  /** The leader can choose where to include the transaction. */
  Any = 3,
}

export enum PostConditionModeID {
  /** This transaction may affect other assets not listed in the post-conditions. */
  Allow = 0x01,
  /** This transaction may NOT affect other assets besides those listed in the post-conditions. */
  Deny = 0x02,
  /** Deny for the transaction origin; allow for everyone else (SIP-040). */
  Originator = 0x03,
}

export interface ClarityValueCommon {
  /** Clarity repr value */
  repr: string;
  /** Hex encoded string of the serialized Clarity value */
  hex: string;
}

export interface ClarityValueAbstract extends ClarityValueCommon {
  type_id: number;
}

export enum ClarityTypeID {
  Int = 0,
  UInt = 1,
  Buffer = 2,
  BoolTrue = 3,
  BoolFalse = 4,
  PrincipalStandard = 5,
  PrincipalContract = 6,
  ResponseOk = 7,
  ResponseError = 8,
  OptionalNone = 9,
  OptionalSome = 10,
  List = 11,
  Tuple = 12,
  StringAscii = 13,
  StringUtf8 = 14,
}

export interface ClarityValueInt extends ClarityValueCommon {
  type_id: ClarityTypeID.Int;
  /** String-quoted signed integer */
  value: string;
}

export interface ClarityValueUInt extends ClarityValueCommon {
  type_id: ClarityTypeID.UInt;
  /** String-quoted unsigned integer */
  value: string;
}

export interface ClarityValueBoolTrue extends ClarityValueCommon {
  type_id: ClarityTypeID.BoolTrue;
  value: true;
}

export interface ClarityValueBoolFalse extends ClarityValueCommon {
  type_id: ClarityTypeID.BoolFalse;
  value: false;
}

export interface ClarityValueBuffer extends ClarityValueCommon {
  type_id: ClarityTypeID.Buffer;
  /** Hex string */
  buffer: string;
}

export interface ClarityValueList<T extends ClarityValue = ClarityValue>
  extends ClarityValueCommon {
  type_id: ClarityTypeID.List;
  list: T[];
}

export interface ClarityValueStringAscii extends ClarityValueCommon {
  type_id: ClarityTypeID.StringAscii;
  data: string;
}

export interface ClarityValueStringUtf8 extends ClarityValueCommon {
  type_id: ClarityTypeID.StringUtf8;
  data: string;
}

export interface ClarityValuePrincipalStandard extends ClarityValueCommon {
  type_id: ClarityTypeID.PrincipalStandard;
  address: string;
  address_version: number;
  /** Hex string */
  address_hash_bytes: string;
}

export interface ClarityValuePrincipalContract extends ClarityValueCommon {
  type_id: ClarityTypeID.PrincipalContract;
  address: string;
  address_version: number;
  /** Hex string */
  address_hash_bytes: string;
  contract_name: string;
}

export type ClarityTupleData<T extends ClarityValue = ClarityValue> = { [key: string]: T };

export interface ClarityValueTuple<T extends ClarityTupleData = ClarityTupleData>
  extends ClarityValueCommon {
  type_id: ClarityTypeID.Tuple;
  data: T;
}

export interface ClarityValueOptionalSome<T extends ClarityValue = ClarityValue>
  extends ClarityValueCommon {
  type_id: ClarityTypeID.OptionalSome;
  value: T;
}

export interface ClarityValueOptionalNone extends ClarityValueCommon {
  type_id: ClarityTypeID.OptionalNone;
}

export interface ClarityValueResponseOk<T extends ClarityValue = ClarityValue>
  extends ClarityValueCommon {
  type_id: ClarityTypeID.ResponseOk;
  value: T;
}

export interface ClarityValueResponseError<T extends ClarityValue = ClarityValue>
  extends ClarityValueCommon {
  type_id: ClarityTypeID.ResponseError;
  value: T;
}

export type ClarityValue =
  | ClarityValueInt
  | ClarityValueUInt
  | ClarityValueBoolTrue
  | ClarityValueBoolFalse
  | ClarityValueBuffer
  | ClarityValueList
  | ClarityValueStringAscii
  | ClarityValueStringUtf8
  | ClarityValuePrincipalStandard
  | ClarityValuePrincipalContract
  | ClarityValueTuple
  | ClarityValueOptionalSome
  | ClarityValueOptionalNone
  | ClarityValueResponseOk
  | ClarityValueResponseError;

export type ClarityValueOptional<T extends ClarityValue = ClarityValue> =
  | ClarityValueOptionalSome<T>
  | ClarityValueOptionalNone;
export type ClarityValueBool = ClarityValueBoolTrue | ClarityValueBoolFalse;
export type ClarityValueResponse<
  TOk extends ClarityValue = ClarityValue,
  TError extends ClarityValue = ClarityValue
> = ClarityValueResponseOk<TOk> | ClarityValueResponseError<TError>;

/**
 * Type for commonly used `(optional bool)`
 */
export type ClarityValueOptionalBool = ClarityValueOptional<ClarityValueBool>;

/**
 * Type for commonly used `(optional uint)`
 */
export type ClarityValueOptionalUInt = ClarityValueOptional<ClarityValueUInt>;

// ============================================================================
// Nakamoto Block Types (Stacks 3.x+)
// ============================================================================

export interface DecodedNakamotoBlockResult {
  /** Hex encoded string of the block ID (index block hash) */
  block_id: string;
  header: NakamotoBlockHeader;
  txs: DecodedTxResult[];
}

export interface NakamotoBlockHeader {
  version: number;
  /** String-quoted unsigned integer - total blocks preceding this one */
  chain_length: string;
  /** String-quoted unsigned integer - total BTC spent in sortition */
  burn_spent: string;
  /** Hex string (20 bytes) - consensus hash of the burnchain block */
  consensus_hash: string;
  /** Hex string (32 bytes) - parent block ID */
  parent_block_id: string;
  /** Hex string (32 bytes) - merkle root of transactions */
  tx_merkle_root: string;
  /** Hex string (32 bytes) - MARF trie root hash */
  state_index_root: string;
  /** String-quoted unsigned integer - Unix timestamp */
  timestamp: string;
  /** Hex string (65 bytes) - miner's ECDSA signature */
  miner_signature: string;
  /** Array of hex strings (65 bytes each) - signer signatures */
  signer_signature: string[];
  /** PoX treatment bitvec */
  pox_treatment: BitVec;
  /** Hex string (32 bytes) - computed block hash */
  block_hash: string;
  /** Hex string (32 bytes) - computed index block hash */
  index_block_hash: string;
}

export interface BitVec {
  /** Number of bits */
  len: number;
  /** Hex encoded data bytes */
  data: string;
  /** Array of boolean values for each bit */
  bits: boolean[];
}

// ============================================================================
// Stacks 2.x Block Types
// ============================================================================

export interface DecodedStacksBlockResult {
  /** Hex encoded string of the block hash */
  block_hash: string;
  header: StacksBlockHeader;
  txs: DecodedTxResult[];
}

export interface StacksBlockHeader {
  version: number;
  total_work: StacksWorkScore;
  /** Hex string (80 bytes) - VRF proof */
  proof: string;
  /** Hex string (32 bytes) - parent block hash */
  parent_block: string;
  /** Hex string (32 bytes) - parent microblock hash */
  parent_microblock: string;
  /** Parent microblock sequence number */
  parent_microblock_sequence: number;
  /** Hex string (32 bytes) - merkle root of transactions */
  tx_merkle_root: string;
  /** Hex string (32 bytes) - MARF trie root hash */
  state_index_root: string;
  /** Hex string (20 bytes) - hash160 of microblock public key */
  microblock_pubkey_hash: string;
  /** Hex string (32 bytes) - computed block hash */
  block_hash: string;
}

export interface StacksWorkScore {
  /** String-quoted unsigned integer - burn amount */
  burn: string;
  /** String-quoted unsigned integer - work score */
  work: string;
}

// ============================================================================
// PoX Synthetic Event Types — pox-2 / pox-3 / pox-4
//
// These describe synthetic events the Stacks node emits for the older PoX
// contracts (pox-2 through pox-4). The wire shape is always
// `Response(Ok({ stacker, locked, ..., name, data }))` — the node
// synthesizes the wrapper from contract-call return values.
//
// PoX-5 changed the model: events are produced by `(print { topic, ... })`
// calls inside the contract itself, so they have a different shape. Those
// types live in the next section below.
// ============================================================================

export enum Pox4EventName {
  HandleUnlock = 'handle-unlock',
  StackStx = 'stack-stx',
  StackIncrease = 'stack-increase',
  StackExtend = 'stack-extend',
  DelegateStx = 'delegate-stx',
  DelegateStackStx = 'delegate-stack-stx',
  DelegateStackIncrease = 'delegate-stack-increase',
  DelegateStackExtend = 'delegate-stack-extend',
  StackAggregationCommit = 'stack-aggregation-commit',
  StackAggregationCommitIndexed = 'stack-aggregation-commit-indexed',
  StackAggregationIncrease = 'stack-aggregation-increase',
  RevokeDelegateStx = 'revoke-delegate-stx',
}

export interface Pox4EventBase {
  /**
   * Discriminant identifying the source PoX contract version. For events
   * decoded from pox-2 / pox-3 / pox-4 this is always `'pox4'` — pox-5
   * events come back as {@link Pox5Event} instead.
   */
  pox_version: 'pox4';
  stacker: string;
  /** String-quoted unsigned integer */
  locked: string;
  /** String-quoted unsigned integer */
  balance: string;
  /** String-quoted unsigned integer */
  burnchain_unlock_height: string;
  pox_addr: string | null;
  pox_addr_raw: string | null;
}

export interface Pox4EventHandleUnlock extends Pox4EventBase {
  name: Pox4EventName.HandleUnlock;
  data: {
    /** String-quoted unsigned integer */
    first_cycle_locked: string;
    /** String-quoted unsigned integer */
    first_unlocked_cycle: string;
  };
}

export interface Pox4EventStackStx extends Pox4EventBase {
  name: Pox4EventName.StackStx;
  data: {
    /** String-quoted unsigned integer */
    lock_amount: string;
    /** String-quoted unsigned integer */
    lock_period: string;
    /** String-quoted unsigned integer */
    start_burn_height: string;
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
    /** Hex string or null */
    signer_key: string | null;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventStackIncrease extends Pox4EventBase {
  name: Pox4EventName.StackIncrease;
  data: {
    /** String-quoted unsigned integer */
    increase_by: string;
    /** String-quoted unsigned integer */
    total_locked: string;
    /** Hex string or null */
    signer_key: string | null;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventStackExtend extends Pox4EventBase {
  name: Pox4EventName.StackExtend;
  data: {
    /** String-quoted unsigned integer */
    extend_count: string;
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
    /** Hex string or null */
    signer_key: string | null;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventDelegateStx extends Pox4EventBase {
  name: Pox4EventName.DelegateStx;
  data: {
    /** String-quoted unsigned integer */
    amount_ustx: string;
    delegate_to: string;
    /** String-quoted unsigned integer or null */
    unlock_burn_height: string | null;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventDelegateStackStx extends Pox4EventBase {
  name: Pox4EventName.DelegateStackStx;
  data: {
    /** String-quoted unsigned integer */
    lock_amount: string;
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
    /** String-quoted unsigned integer */
    start_burn_height: string;
    /** String-quoted unsigned integer */
    lock_period: string;
    delegator: string;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventDelegateStackIncrease extends Pox4EventBase {
  name: Pox4EventName.DelegateStackIncrease;
  data: {
    /** String-quoted unsigned integer */
    increase_by: string;
    /** String-quoted unsigned integer */
    total_locked: string;
    delegator: string;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventDelegateStackExtend extends Pox4EventBase {
  name: Pox4EventName.DelegateStackExtend;
  data: {
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
    /** String-quoted unsigned integer */
    extend_count: string;
    delegator: string;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventStackAggregationCommit extends Pox4EventBase {
  name: Pox4EventName.StackAggregationCommit;
  data: {
    /** String-quoted unsigned integer */
    reward_cycle: string;
    /** String-quoted unsigned integer */
    amount_ustx: string;
    /** Hex string or null */
    signer_key: string | null;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventStackAggregationCommitIndexed extends Pox4EventBase {
  name: Pox4EventName.StackAggregationCommitIndexed;
  data: {
    /** String-quoted unsigned integer */
    reward_cycle: string;
    /** String-quoted unsigned integer */
    amount_ustx: string;
    /** Hex string or null */
    signer_key: string | null;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventStackAggregationIncrease extends Pox4EventBase {
  name: Pox4EventName.StackAggregationIncrease;
  data: {
    /** String-quoted unsigned integer */
    reward_cycle: string;
    /** String-quoted unsigned integer */
    amount_ustx: string;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export interface Pox4EventRevokeDelegateStx extends Pox4EventBase {
  name: Pox4EventName.RevokeDelegateStx;
  data: {
    delegate_to: string;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export type Pox4Event =
  | Pox4EventHandleUnlock
  | Pox4EventStackStx
  | Pox4EventStackIncrease
  | Pox4EventStackExtend
  | Pox4EventDelegateStx
  | Pox4EventDelegateStackStx
  | Pox4EventDelegateStackIncrease
  | Pox4EventDelegateStackExtend
  | Pox4EventStackAggregationCommit
  | Pox4EventStackAggregationCommitIndexed
  | Pox4EventStackAggregationIncrease
  | Pox4EventRevokeDelegateStx;

// ============================================================================
// PoX Synthetic Event Types — pox-5
//
// PoX-5 events are emitted by explicit `(print { topic: "...", ... })` calls
// in the contract source, so each event arrives as a flat Clarity tuple with
// a `topic` ASCII string plus event-specific data. This is structurally
// different from pox-2/3/4, where the Stacks node synthesizes a
// `Response(Ok({ stacker, locked, ..., name, data }))` per stacking call.
//
// On the JS side every pox-5 event has the same outer shape
// `{ name: string, data: { ... } }`; the per-event `data` payloads are
// modeled below.
// ============================================================================

export enum Pox5EventName {
  SetBondAdmin = 'set-bond-admin',
  SetupBond = 'setup-bond',
  AddToAllowlist = 'add-to-allowlist',
  RegisterForBond = 'register-for-bond',
  UpdateBondRegistration = 'update-bond-registration',
  RegisterSigner = 'register-signer',
  Stake = 'stake',
  StakeUpdate = 'stake-update',
  AnnounceL1EarlyExit = 'announce-l1-early-exit',
  UnstakeSbtc = 'unstake-sbtc',
  Unstake = 'unstake',
  CalculateRewards = 'calculate-rewards',
  BondDistribution = 'bond-distribution',
  ClaimRewards = 'claim-rewards',
  ClaimStakerRewardsForSigner = 'claim-staker-rewards-for-signer',
  GrantSignerKey = 'grant-signer-key',
  RevokeSignerGrant = 'revoke-signer-grant',
  DisallowContractCaller = 'disallow-contract-caller',
  AllowContractCaller = 'allow-contract-caller',
}

export interface Pox5EventBase {
  /**
   * Discriminant identifying the source PoX contract version. For events
   * decoded from pox-5 this is always `'pox5'` — earlier-contract events
   * come back as {@link Pox4Event} instead.
   */
  pox_version: 'pox5';
}

export interface Pox5EventSetBondAdmin extends Pox5EventBase {
  name: Pox5EventName.SetBondAdmin;
  data: {
    /** c32 principal of the previous bond admin. */
    old_admin: string;
    /** c32 principal of the new bond admin. */
    new_admin: string;
  };
}

export interface Pox5EventSetupBond extends Pox5EventBase {
  name: Pox5EventName.SetupBond;
  data: {
    /** String-quoted unsigned integer */
    bond_index: string;
    /** String-quoted unsigned integer (basis points) */
    target_rate: string;
    /** String-quoted unsigned integer */
    stx_value_ratio: string;
    /** String-quoted unsigned integer */
    min_ustx_ratio: string;
    /**
     * `(buff 683)` hex string. Bitcoin script subscript guarding the
     * early-exit (`OP_ELSE`) branch of the L1 lockup (e.g. `<pubkey>
     * OP_CHECKSIG`, or an M-of-N `CHECKMULTISIG` template).
     */
    early_unlock_bytes: string;
    /** String-quoted unsigned integer */
    first_reward_cycle: string;
    /** String-quoted unsigned integer */
    bond_start_height: string;
    /** String-quoted unsigned integer */
    unlock_cycle: string;
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
  };
}

export interface Pox5EventAddToAllowlist extends Pox5EventBase {
  name: Pox5EventName.AddToAllowlist;
  data: {
    /** c32 principal of the staker being added to a bond's allowlist. */
    staker: string;
    /** String-quoted unsigned integer */
    max_sats: string;
    /** String-quoted unsigned integer */
    bond_index: string;
  };
}

/** One proven L1 output in a `register-for-bond` `btc_lockup`. */
export interface Pox5BtcLockupTx {
  /** Reversed (big-endian) txid as a `0x`-prefixed hex string. */
  txid: string;
  /** String-quoted unsigned integer */
  output_index: string;
}

/** The `btc_lockup` sub-object of a `register-for-bond` event. */
export interface Pox5BtcLockup {
  /** `'l1'` for a Bitcoin L1 lockup, `'l2'` for an sBTC lockup. */
  type: string;
  /**
   * The proven L1 outputs for an `'l1'` lockup, or `null` for an `'l2'`
   * (sBTC) lockup.
   */
  txs: Pox5BtcLockupTx[] | null;
}

export interface Pox5EventRegisterForBond extends Pox5EventBase {
  name: Pox5EventName.RegisterForBond;
  data: {
    /** c32 principal */
    signer: string;
    /** c32 principal */
    staker: string;
    /** String-quoted unsigned integer */
    amount_ustx: string;
    /** String-quoted unsigned integer */
    sats_total: string;
    /** String-quoted unsigned integer */
    bond_index: string;
    /** String-quoted unsigned integer */
    first_reward_cycle: string;
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
    /** String-quoted unsigned integer */
    unlock_cycle: string;
    /** True if the participant proved an L1 BTC lockup; false if they locked sBTC. */
    is_l1_lock: boolean;
    /** How the BTC was locked (L1 proof outputs vs. sBTC). */
    btc_lockup: Pox5BtcLockup;
  };
}

export interface Pox5EventUpdateBondRegistration extends Pox5EventBase {
  name: Pox5EventName.UpdateBondRegistration;
  data: {
    /** c32 principal */
    staker: string;
    /** c32 principal of the new signer */
    signer: string;
    /** c32 principal of the previous signer */
    old_signer: string;
    /** String-quoted unsigned integer */
    bond_index: string;
    /** String-quoted unsigned integer */
    amount_ustx: string;
    /** String-quoted unsigned integer */
    amount_sats: string;
    /** String-quoted unsigned integer */
    first_reward_cycle: string;
    /** String-quoted unsigned integer */
    num_cycles: string;
    /** True if the participant's stake is an L1 BTC lockup. */
    is_l1_lock: boolean;
  };
}

export interface Pox5EventRegisterSigner extends Pox5EventBase {
  name: Pox5EventName.RegisterSigner;
  data: {
    /** c32 principal */
    signer: string;
    /** `(buff 33)` hex string — compressed secp256k1 public key. */
    signer_key: string;
  };
}

export interface Pox5EventStake extends Pox5EventBase {
  name: Pox5EventName.Stake;
  data: {
    /** c32 principal */
    signer: string;
    /** c32 principal */
    staker: string;
    /** String-quoted unsigned integer */
    amount_ustx: string;
    /** String-quoted unsigned integer */
    num_cycles: string;
    /** String-quoted unsigned integer */
    first_reward_cycle: string;
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
    /** String-quoted unsigned integer */
    unlock_cycle: string;
  };
}

export interface Pox5EventStakeUpdate extends Pox5EventBase {
  name: Pox5EventName.StakeUpdate;
  data: {
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
    /** c32 principal */
    staker: string;
    /** c32 principal of the new signer */
    signer: string;
    /** c32 principal of the previous signer */
    old_signer: string;
    /** String-quoted unsigned integer (the previous unlock cycle before extension) */
    prev_unlock_height: string;
    /** String-quoted unsigned integer */
    unlock_cycle: string;
    /** String-quoted unsigned integer */
    num_cycles: string;
    /** String-quoted unsigned integer — total locked amount after this update */
    amount_ustx: string;
    /** String-quoted unsigned integer */
    amount_increase: string;
    /** String-quoted unsigned integer */
    cycles_to_extend: string;
  };
}

export interface Pox5EventAnnounceL1EarlyExit extends Pox5EventBase {
  name: Pox5EventName.AnnounceL1EarlyExit;
  data: {
    /** c32 principal */
    staker: string;
    /** c32 principal */
    signer: string;
    /** String-quoted unsigned integer */
    bond_index: string;
    /** String-quoted unsigned integer */
    amount_sats_released: string;
  };
}

export interface Pox5EventUnstakeSbtc extends Pox5EventBase {
  name: Pox5EventName.UnstakeSbtc;
  data: {
    /** c32 principal */
    staker: string;
    /** c32 principal */
    signer: string;
    /** String-quoted unsigned integer */
    bond_index: string;
    /** String-quoted unsigned integer */
    amount_withdrawn_sats: string;
    /** String-quoted unsigned integer — sBTC shares remaining after withdrawal */
    new_amount_sats: string;
  };
}

export interface Pox5EventUnstake extends Pox5EventBase {
  name: Pox5EventName.Unstake;
  data: {
    /** c32 principal */
    staker: string;
    /** c32 principal */
    signer: string;
    /** String-quoted unsigned integer */
    amount_ustx: string;
    /** String-quoted unsigned integer */
    first_reward_cycle: string;
    /** String-quoted unsigned integer */
    unlock_cycle: string;
    /** String-quoted unsigned integer */
    unlock_burn_height: string;
  };
}

/**
 * Logged once per `calculate-rewards` call, after all per-bond distributions
 * have been folded and the STX reward cycle accounting has been committed.
 */
export interface Pox5EventCalculateRewards extends Pox5EventBase {
  name: Pox5EventName.CalculateRewards;
  data: {
    /** Array of string-quoted unsigned integers (bond indices being settled) */
    bond_periods: string[];
    /** String-quoted unsigned integer */
    calculation_height: string;
    /** String-quoted unsigned integer — total new rewards accrued since the last calculation. */
    gross_accrued_rewards: string;
    /** String-quoted unsigned integer — portion of `gross_accrued_rewards` paid out to bonds. */
    total_bond_rewards: string;
    /** String-quoted unsigned integer — amount added to the reserve this calculation. */
    reserve_deposit: string;
    /** String-quoted unsigned integer — reserve balance after `reserve_deposit` was applied. */
    reserve_balance: string;
    /** String-quoted unsigned integer */
    stx_cycle: string;
    /** String-quoted unsigned integer — rewards allocated to STX stakers for the cycle. */
    total_stx_staker_rewards: string;
    /** String-quoted unsigned integer */
    cycle_staked_ustx: string;
    /** String-quoted unsigned integer — per-uSTX rewards accrued this calculation (zero when no STX is staked). */
    accrued_rewards_per_ustx: string;
    /** String-quoted unsigned integer — running per-uSTX reward total for the cycle after this calculation. */
    cumulative_rewards_per_ustx: string;
  };
}

export interface Pox5EventBondDistribution extends Pox5EventBase {
  name: Pox5EventName.BondDistribution;
  data: {
    /** String-quoted unsigned integer */
    bond_index: string;
    /** String-quoted unsigned integer */
    target_yield: string;
    /** String-quoted unsigned integer — rewards earned by this bond this calculation. */
    bond_rewards: string;
    /** String-quoted unsigned integer */
    bond_staked_sats: string;
    /** String-quoted unsigned integer — per-sat rewards accrued this calculation. */
    accrued_rewards_per_sat: string;
    /** String-quoted unsigned integer — running per-sat reward total for the bond after this calculation. */
    cumulative_rewards_per_sat: string;
  };
}

/** Sub-tuple emitted under `stx_rewards` in `claim-rewards` events. */
export interface Pox5ClaimRewardsInfo {
  /** String-quoted unsigned integer */
  earned: string;
  /** String-quoted unsigned integer */
  rewards_per_token: string;
}

/** One entry in the `bond_rewards` list of a `claim-rewards` event. */
export interface Pox5BondRewardsInfo extends Pox5ClaimRewardsInfo {
  /** String-quoted unsigned integer */
  bond_index: string;
}

export interface Pox5EventClaimRewards extends Pox5EventBase {
  name: Pox5EventName.ClaimRewards;
  data: {
    /** c32 principal of the signer manager that claimed. */
    signer_manager: string;
    /** String-quoted unsigned integer */
    reward_cycle: string;
    stx_rewards: Pox5ClaimRewardsInfo;
    bond_rewards: Pox5BondRewardsInfo[];
    /** String-quoted unsigned integer */
    bond_totals: string;
    /** String-quoted unsigned integer */
    total_rewards: string;
  };
}

export interface Pox5EventClaimStakerRewardsForSigner extends Pox5EventBase {
  name: Pox5EventName.ClaimStakerRewardsForSigner;
  data: {
    /** c32 principal of the signer manager. */
    signer_manager: string;
    /** c32 principal of the staker. */
    staker: string;
    /** String-quoted unsigned integer */
    reward_cycle: string;
    /**
     * String-quoted unsigned integer for bond rewards, or `null` for
     * STX-only staking rewards.
     */
    bond_index: string | null;
    /** String-quoted unsigned integer */
    rewards_claimed: string;
  };
}

export interface Pox5EventGrantSignerKey extends Pox5EventBase {
  name: Pox5EventName.GrantSignerKey;
  data: {
    /** `(buff 33)` hex string — compressed secp256k1 public key. */
    signer_key: string;
    /** c32 principal of the signer manager. */
    signer_manager: string;
    /** String-quoted unsigned integer */
    auth_id: string;
  };
}

export interface Pox5EventRevokeSignerGrant extends Pox5EventBase {
  name: Pox5EventName.RevokeSignerGrant;
  data: {
    /** `(buff 33)` hex string — compressed secp256k1 public key. */
    signer_key: string;
    /** c32 principal of the signer manager. */
    signer_manager: string;
  };
}

export interface Pox5EventDisallowContractCaller extends Pox5EventBase {
  name: Pox5EventName.DisallowContractCaller;
  data: {
    /** c32 principal of the tx-sender that revoked the allowance. */
    sender: string;
    /** c32 principal of the contract-caller whose allowance was removed. */
    contract_caller: string;
  };
}

export interface Pox5EventAllowContractCaller extends Pox5EventBase {
  name: Pox5EventName.AllowContractCaller;
  data: {
    /** c32 principal of the tx-sender that granted the allowance. */
    sender: string;
    /** c32 principal of the allowed contract-caller. */
    contract_caller: string;
    /**
     * String-quoted unsigned integer burn height at which the allowance
     * expires, or `null` if it never expires.
     */
    until_burn_ht: string | null;
  };
}

export type Pox5Event =
  | Pox5EventSetBondAdmin
  | Pox5EventSetupBond
  | Pox5EventAddToAllowlist
  | Pox5EventRegisterForBond
  | Pox5EventUpdateBondRegistration
  | Pox5EventRegisterSigner
  | Pox5EventStake
  | Pox5EventStakeUpdate
  | Pox5EventAnnounceL1EarlyExit
  | Pox5EventUnstakeSbtc
  | Pox5EventUnstake
  | Pox5EventCalculateRewards
  | Pox5EventBondDistribution
  | Pox5EventClaimRewards
  | Pox5EventClaimStakerRewardsForSigner
  | Pox5EventGrantSignerKey
  | Pox5EventRevokeSignerGrant
  | Pox5EventDisallowContractCaller
  | Pox5EventAllowContractCaller;

// ============================================================================
// PoX Synthetic Event — combined union
// ============================================================================

/**
 * Any decoded PoX synthetic event, regardless of the source contract version.
 *
 * Two discriminants are available; use whichever fits the call site:
 *
 * - `event.pox_version` — `'pox4'` or `'pox5'`. Use this when you only care
 *   which contract family the event came from (e.g. routing to a per-version
 *   handler).
 * - `event.name` — the specific event-name string literal. The pox-4 and
 *   pox-5 name sets don't overlap, so a single switch on `name` narrows
 *   all the way down to the per-event interface.
 */
export type PoxEvent = Pox4Event | Pox5Event;

// ============================================================================
// Signer messages (libsigner `SignerMessage`)
// ============================================================================

export enum SignerMessageTypeID {
  BlockProposal = 0,
  BlockResponse = 1,
  BlockPushed = 2,
  MockProposal = 3,
  MockSignature = 4,
  MockBlock = 5,
  StateMachineUpdate = 6,
  BlockPreCommit = 7,
}

export interface SignerMessageBase {
  type_id: SignerMessageTypeID;
  type_name: string;
}

/** Signer message metadata appended to block responses. */
export interface SignerMessageMetadata {
  server_version: string;
}

/** Block proposal from a miner for signers to observe and sign. */
export interface SignerMessageBlockProposal extends SignerMessageBase {
  type_id: SignerMessageTypeID.BlockProposal;
  type_name: 'block_proposal';
  block_proposal: {
    block: DecodedNakamotoBlockResult;
    /** String-quoted unsigned integer */
    burn_height: string;
    /** String-quoted unsigned integer */
    reward_cycle: string;
    block_proposal_data: BlockProposalData;
  };
}

export interface BlockProposalData {
  version: number;
  server_version: string;
  miner_diagnostic_data: MinerDiagnosticData | null;
  /** Hex string of any trailing bytes from a future version (empty `0x` if none). */
  unknown_bytes: string;
}

export interface MinerDiagnosticData {
  /** String-quoted unsigned integer */
  burnchain_tip_height: string;
  /** Hex string (20 bytes) */
  burnchain_tip_consensus_hash: string;
  /** Hex string (32 bytes) */
  burnchain_tip_header_hash: string;
  /** String-quoted unsigned integer */
  tenure_extend_time_stamp: string;
  /** String-quoted unsigned integer */
  read_count_extend_timestamp: string;
  mining_reason_id: number;
  mining_reason_name: 'block_found' | 'extended' | 'read_count_extend';
}

/** Block response (accept or reject) from a signer. */
export interface SignerMessageBlockResponse extends SignerMessageBase {
  type_id: SignerMessageTypeID.BlockResponse;
  type_name: 'block_response';
  block_response: BlockResponseAccepted | BlockResponseRejected;
}

export interface BlockResponseAccepted {
  response_type: 'accepted';
  /** Hex string (32 bytes) */
  signer_signature_hash: string;
  /** Hex string (65 bytes) */
  signature: string;
  metadata: SignerMessageMetadata;
  response_data: BlockResponseData;
}

export interface BlockResponseRejected {
  response_type: 'rejected';
  reason: string;
  reason_code: RejectCode;
  /** Hex string (32 bytes) */
  signer_signature_hash: string;
  /** Hex string (65 bytes) */
  signature: string;
  chain_id: number;
  metadata: SignerMessageMetadata;
  response_data: BlockResponseData;
}

export interface BlockResponseData {
  version: number;
  /** String-quoted unsigned integer */
  tenure_extend_timestamp: string;
  reject_reason: RejectReason;
  /** String-quoted unsigned integer */
  tenure_extend_read_count_timestamp: string;
  /** Hex string (32 bytes), or `null` if no transaction caused a failure. */
  failed_txid: string | null;
  /** Hex string of any trailing bytes from a future version (empty `0x` if none). */
  unknown_bytes: string;
}

/** The reason code on a `rejected` block response (`reason_code` field). */
export interface RejectCode {
  reject_code_name:
    | 'validation_failed'
    | 'no_sortition_view'
    | 'connectivity_issues'
    | 'rejected_in_prior_round'
    | 'sortition_view_mismatch'
    | 'testing_directive';
  /** Present when `reject_code_name === 'validation_failed'`. */
  validate_reject_code?: number;
  validate_reject_code_name?: string;
  /** Present when `reject_code_name === 'connectivity_issues'`. */
  message?: string;
}

/** The versioned rejection detail carried in `response_data.reject_reason`. */
export interface RejectReason {
  reject_reason_name: string;
  /** Present when `reject_reason_name === 'validation_failed'`. */
  validate_reject_code?: number;
  validate_reject_code_name?: string;
  /** Present when `reject_reason_name === 'connectivity_issues'`. */
  message?: string;
  /** Present when `reject_reason_name === 'consensus_hash_mismatch'`. */
  expected_consensus_hash?: string;
  actual_consensus_hash?: string;
  /** Present when `reject_reason_name === 'unknown'`. */
  unknown_code?: number;
}

/** A block pushed from a miner to the signer set. */
export interface SignerMessageBlockPushed extends SignerMessageBase {
  type_id: SignerMessageTypeID.BlockPushed;
  type_name: 'block_pushed';
  block_pushed: DecodedNakamotoBlockResult;
}

/** A pre-commit message from a signer for other signers to observe. */
export interface SignerMessageBlockPreCommit extends SignerMessageBase {
  type_id: SignerMessageTypeID.BlockPreCommit;
  type_name: 'block_pre_commit';
  block_pre_commit: {
    /** Hex string (32 bytes) */
    signer_signature_hash: string;
  };
}

/**
 * Recognized-but-unsupported signer messages: the epoch-2.5 `mock_*` types and
 * `state_machine_update`. Only the discriminant is decoded.
 */
export interface SignerMessageUnsupported extends SignerMessageBase {
  type_id:
    | SignerMessageTypeID.MockProposal
    | SignerMessageTypeID.MockSignature
    | SignerMessageTypeID.MockBlock
    | SignerMessageTypeID.StateMachineUpdate;
  type_name: 'mock_proposal' | 'mock_signature' | 'mock_block' | 'state_machine_update';
  unsupported: true;
}

export type SignerMessage =
  | SignerMessageBlockProposal
  | SignerMessageBlockResponse
  | SignerMessageBlockPushed
  | SignerMessageBlockPreCommit
  | SignerMessageUnsupported;
