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

  // Fixtures generated from libsigner:
  //   SignerMessage::BlockResponse(BlockResponse::rejected(
  //     Sha512Trunc256Sum([0x03; 32]), <reason>,
  //     &StacksPrivateKey::from_seed(&[0x01]), false,
  //     1_700_000_000, 1_700_000_001))
  it.each([
    ['proposal-too-old', 'proposal_too_old', 18],
    ['reward-cycle-retired', 'reward_cycle_retired', 19],
  ])('decodes a rejected block response (%s)', (fixture, reasonName, wireCode) => {
    const hex = fs
      .readFileSync(
        path.join(__dirname, `fixtures/signer-message-block-response-rejected-${fixture}.hex`),
        'utf8'
      )
      .trim();
    // In the response data, the reject reason code byte sits between the two
    // u64 tenure-extend timestamps; check the fixture really carries the new
    // wire code rather than trusting the decoder alone.
    expect(hex).toContain(`000000006553f100${wireCode.toString(16)}000000006553f101`);

    const result = decodeSignerMessage(hex);

    expect(result.type_id).toBe(SignerMessageTypeID.BlockResponse);
    if (result.type_name !== 'block_response') throw new Error('wrong variant');

    const response = result.block_response;
    expect(response.response_type).toBe('rejected');
    if (response.response_type !== 'rejected') throw new Error('expected rejected');

    expect(response.signer_signature_hash).toBe('0x' + '03'.repeat(32));
    expect(response.chain_id).toBe(0x80000000);
    // libsigner maps reasons without a dedicated RejectCode to SortitionViewMismatch.
    expect(response.reason_code.reject_code_name).toBe('sortition_view_mismatch');
    expect(response.response_data.tenure_extend_timestamp).toBe('1700000000');
    expect(response.response_data.tenure_extend_read_count_timestamp).toBe('1700000001');
    expect(response.response_data.reject_reason).toEqual({ reject_reason_name: reasonName });
    expect(response.response_data.failed_txid).toBeNull();
  });
});
