import { ClarityVersion, decodeTransaction, TxPayloadTypeID } from '../index.js';

test('stacks 4.1 - decode tx - versioned smart contract Clarity 7', () => {
  // The Clarity 3 vector from tx-decode-3.0.test.ts with the payload's
  // clarity-version byte patched from 0x03 to 0x07. The signature no longer
  // matches, which decoding does not check.
  const tx = '8080000000040005572d04565d56f67e84ad7e20deedd8e7bba2fd00000000000000000000000000000bb800010c3287ab0587cf952022e079d261baeacc533d8c92b754136271450eb121ba4c0849f3c47f5a6f967dd1088e8242a9f9ed0454bfadaa19c71d25c8812e47160a030200000000060707636f756e7465720000004e28646566696e652d646174612d76617220636f756e742075696e74207530290a28646566696e652d726561642d6f6e6c7920286765742d636f756e742920287661722d67657420636f756e742929';
  const decoded = decodeTransaction(tx);

  expect(decoded.tx_id).toBe('0x00e3af90b60567e7b8c79f6d9042e000031fcf84b6c96c96f6a6ecbac0b71740');
  expect(decoded.payload).toEqual({
    "type_id": TxPayloadTypeID.VersionedSmartContract,
    "clarity_version": ClarityVersion.Clarity7,
    "contract_name": "counter",
    "code_body": "(define-data-var count uint u0)\n(define-read-only (get-count) (var-get count))"
  });
  expect(ClarityVersion.Clarity7).toBe(7);
});
