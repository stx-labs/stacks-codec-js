let targetName = `${process.platform}-${process.arch}`;
if (process.platform === 'linux') {
  const libc = require('detect-libc').familySync() || 'glibc';
  targetName += `-${libc}`;
}
const addon = require(`./native/${targetName}.node`);
module.exports = addon;
// Explicit named exports so Node's CJS static analysis (and `export * from` in ESM) can see them.
module.exports.getVersion = addon.getVersion;
module.exports.decodeTransaction = addon.decodeTransaction;
module.exports.decodeNakamotoBlock = addon.decodeNakamotoBlock;
module.exports.decodeStacksBlock = addon.decodeStacksBlock;
module.exports.decodeClarityValueToRepr = addon.decodeClarityValueToRepr;
module.exports.decodeClarityValueToTypeName = addon.decodeClarityValueToTypeName;
module.exports.decodeClarityValue = addon.decodeClarityValue;
module.exports.decodeClarityValueList = addon.decodeClarityValueList;
module.exports.decodePostConditions = addon.decodePostConditions;
module.exports.stacksToBitcoinAddress = addon.stacksToBitcoinAddress;
module.exports.bitcoinToStacksAddress = addon.bitcoinToStacksAddress;
module.exports.isValidStacksAddress = addon.isValidStacksAddress;
module.exports.decodeStacksAddress = addon.decodeStacksAddress;
module.exports.decodeClarityValueToPrincipal = addon.decodeClarityValueToPrincipal;
module.exports.stacksAddressFromParts = addon.stacksAddressFromParts;
module.exports.memoToString = addon.memoToString;
module.exports.decodePoxSyntheticEvent = addon.decodePoxSyntheticEvent;
module.exports.decodeSignerMessage = addon.decodeSignerMessage;
module.exports.startProfiler = addon.startProfiler;
module.exports.stopProfiler = addon.stopProfiler;
module.exports.createProfiler = addon.createProfiler;
module.exports.perfTestC32Encode = addon.perfTestC32Encode;
