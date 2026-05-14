import * as bindings from './loader.js';
export * from './loader.js';
export const StacksNativeEncodingBindings = bindings;
export default StacksNativeEncodingBindings;

export type TxPostCondition = PostConditionStx | PostConditionFungible | PostConditionNonfungible;

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

export interface PostConditionAssetInfo {
  contract_address: string;
  contract_name: string;
  asset_name: string;
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
// PoX Synthetic Event Types
// ============================================================================

export enum PoxEventName {
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

export interface PoxEventBase {
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

export interface PoxEventHandleUnlock extends PoxEventBase {
  name: PoxEventName.HandleUnlock;
  data: {
    /** String-quoted unsigned integer */
    first_cycle_locked: string;
    /** String-quoted unsigned integer */
    first_unlocked_cycle: string;
  };
}

export interface PoxEventStackStx extends PoxEventBase {
  name: PoxEventName.StackStx;
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

export interface PoxEventStackIncrease extends PoxEventBase {
  name: PoxEventName.StackIncrease;
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

export interface PoxEventStackExtend extends PoxEventBase {
  name: PoxEventName.StackExtend;
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

export interface PoxEventDelegateStx extends PoxEventBase {
  name: PoxEventName.DelegateStx;
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

export interface PoxEventDelegateStackStx extends PoxEventBase {
  name: PoxEventName.DelegateStackStx;
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

export interface PoxEventDelegateStackIncrease extends PoxEventBase {
  name: PoxEventName.DelegateStackIncrease;
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

export interface PoxEventDelegateStackExtend extends PoxEventBase {
  name: PoxEventName.DelegateStackExtend;
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

export interface PoxEventStackAggregationCommit extends PoxEventBase {
  name: PoxEventName.StackAggregationCommit;
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

export interface PoxEventStackAggregationCommitIndexed extends PoxEventBase {
  name: PoxEventName.StackAggregationCommitIndexed;
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

export interface PoxEventStackAggregationIncrease extends PoxEventBase {
  name: PoxEventName.StackAggregationIncrease;
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

export interface PoxEventRevokeDelegateStx extends PoxEventBase {
  name: PoxEventName.RevokeDelegateStx;
  data: {
    delegate_to: string;
    /** String-quoted unsigned integer or null */
    end_cycle_id: string | null;
    /** String-quoted unsigned integer or null */
    start_cycle_id: string | null;
  };
}

export type DecodedPoxSyntheticEvent =
  | PoxEventHandleUnlock
  | PoxEventStackStx
  | PoxEventStackIncrease
  | PoxEventStackExtend
  | PoxEventDelegateStx
  | PoxEventDelegateStackStx
  | PoxEventDelegateStackIncrease
  | PoxEventDelegateStackExtend
  | PoxEventStackAggregationCommit
  | PoxEventStackAggregationCommitIndexed
  | PoxEventStackAggregationIncrease
  | PoxEventRevokeDelegateStx;
