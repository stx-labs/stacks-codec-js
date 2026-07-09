import * as fs from 'fs';
import * as path from 'path';
import { decodeSignerMessage, SignerMessageTypeID } from '../index';

describe('Signer message decoding', () => {
  it('decodes a block pre-commit message', () => {
    // Wire format: type-prefix byte (7) followed by the 32-byte signer
    // signature hash.
    const hex = '07' + '01'.repeat(32);
    const result = decodeSignerMessage(hex);

    expect(result.type_id).toBe(SignerMessageTypeID.BlockPreCommit);
    expect(result.type_name).toBe('block_pre_commit');
    if (result.type_name !== 'block_pre_commit') throw new Error('wrong variant');
    expect(result.block_pre_commit.signer_signature_hash).toBe('0x' + '01'.repeat(32));
  });

  it('decodes an accepted block response', () => {
    // Fixture generated from libsigner:
    //   SignerMessage::BlockResponse(BlockResponse::accepted(
    //     Sha512Trunc256Sum([0x02; 32]), MessageSignature::empty(),
    //     1_700_000_000, 1_700_000_001))
    const hex = fs
      .readFileSync(
        path.join(__dirname, 'fixtures/signer-message-block-response-accepted.hex'),
        'utf8'
      )
      .trim();
    const result = decodeSignerMessage(hex);

    expect(result.type_id).toBe(SignerMessageTypeID.BlockResponse);
    expect(result.type_name).toBe('block_response');
    if (result.type_name !== 'block_response') throw new Error('wrong variant');

    const response = result.block_response;
    expect(response.response_type).toBe('accepted');
    if (response.response_type !== 'accepted') throw new Error('expected accepted');

    expect(response.signer_signature_hash).toBe('0x' + '02'.repeat(32));
    expect(response.signature).toBe('0x' + '00'.repeat(65));
    expect(response.metadata.server_version).toContain('stacks-signer');
    expect(response.response_data.tenure_extend_timestamp).toBe('1700000000');
    expect(response.response_data.tenure_extend_read_count_timestamp).toBe('1700000001');
    expect(response.response_data.reject_reason.reject_reason_name).toBe('not_rejected');
    expect(response.response_data.failed_txid).toBeNull();
  });
});
