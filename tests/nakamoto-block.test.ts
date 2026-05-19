import * as fs from 'fs';
import * as path from 'path';
import { decodeNakamotoBlock, decodeStacksBlock } from '../index';

describe('Nakamoto block decoding', () => {
  it('should decode a Nakamoto block', () => {
    const blockBuffer = fs.readFileSync(path.join(__dirname, 'fixtures/nakamoto-block.bin'));
    const result = decodeNakamotoBlock(blockBuffer);

    expect(result).toHaveProperty('block_id');
    expect(result).toHaveProperty('header');
    expect(result).toHaveProperty('txs');

    expect(result.header.version).toBe(0);
    expect(result.header.chain_length).toBe('557923');
    expect(result.header.burn_spent).toBe('403018706956');
    expect(result.header.consensus_hash).toBe('0xe86587f4ed4ca465b87649ace9341d9fdfd113ba');
    expect(result.header.parent_block_id).toBe(
      '0x8de0fa074023b893f73c8491ab5c93bb3f5af4bd5f0449578b99b508cca61595'
    );
    expect(result.header.tx_merkle_root).toBe(
      '0x080d35f6c5c02929a00fca1cc6f00a1c3828d905eb61e002ffd4e48f1ecef29d'
    );
    expect(result.header.state_index_root).toBe(
      '0xbf5ed8f745df2629d0d971fe9667f75a352a5dea4c8a0e451dcaa72b375d28fc'
    );

    expect(result.header.pox_treatment).toBeDefined();
    expect(result.header.pox_treatment.len).toBe(3891);
    expect(result.header.pox_treatment.data).toHaveLength(976);

    expect(result.txs).toHaveLength(1);

    // Computed hashes should be hex strings (with 0x prefix)
    expect(result.header.block_hash).toMatch(/^0x[0-9a-f]{64}$/);
    expect(result.header.index_block_hash).toMatch(/^0x[0-9a-f]{64}$/);
    expect(result.block_id).toMatch(/^0x[0-9a-f]{64}$/);

    // block_id should equal index_block_hash
    expect(result.block_id).toBe(result.header.index_block_hash);
  });

  it('should handle invalid block data gracefully', () => {
    expect(() => {
      decodeNakamotoBlock('deadbeef');
    }).toThrow();
  });
});

describe('Stacks 2.x block decoding', () => {
  // `decodeStacksBlock` now delegates to upstream
  // `<StacksBlock as StacksMessageCodec>::consensus_deserialize`, which
  // enforces the same checks the Stacks node enforces at the wire layer:
  // VRF proofs must be valid Edwards curve points, the tx vector must be
  // non-empty, transaction ids must be unique, anchor modes must be
  // `OnChainOnly` or `Any`, and the header's `tx_merkle_root` must match
  // the merkle root of the txs. Happy-path coverage for 2.x blocks requires
  // a real mainnet block fixture; until such a fixture lands in
  // `tests/fixtures/`, the tests below pin the strict-rejection behavior so
  // any future regression to the old permissive parser is caught.

  // Matches stacks-common `VRFProof::empty()`: an all-`0x01` 80-byte proof
  // that decodes to a valid (non-low-order) curve point.
  const validVrfProofHex = '01'.repeat(80);

  function buildHeader(opts: { proof: string; total_work_work?: string }): string {
    return [
      '00', // version
      '0000000000000001', // total_work.burn
      opts.total_work_work ?? '0000000000000001', // total_work.work
      opts.proof, // VRF proof (80 bytes)
      '11'.repeat(32), // parent_block
      '22'.repeat(32), // parent_microblock
      '0000', // parent_microblock_sequence
      '33'.repeat(32), // tx_merkle_root
      '44'.repeat(32), // state_index_root
      '55'.repeat(20), // microblock_pubkey_hash
    ].join('');
  }

  it('rejects blocks whose VRF proof is not a valid curve point', () => {
    const blockHex =
      buildHeader({ proof: '00'.repeat(80) }) + '00000000'; // 0 txs
    expect(() => decodeStacksBlock(blockHex)).toThrow(/VRF proof|curve|consensus/i);
  });

  it('rejects zero-transaction blocks even when the VRF proof is valid', () => {
    const blockHex = buildHeader({ proof: validVrfProofHex }) + '00000000';
    expect(() => decodeStacksBlock(blockHex)).toThrow(/zero transactions/i);
  });

  it('should handle invalid block data gracefully', () => {
    expect(() => {
      decodeStacksBlock('deadbeef');
    }).toThrow();
  });
});
