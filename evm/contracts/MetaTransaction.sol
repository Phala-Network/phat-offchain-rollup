// SPDX-License-Identifier: MIT
// OpenZeppelin Contracts (last updated v4.8.0) (metatx/MinimalForwarder.sol)
// Modified by Phala Network, 2023

pragma solidity ^0.8.9;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import "@openzeppelin/contracts/utils/Context.sol";

contract MetaTxReceiver is EIP712, Context {
    using ECDSA for bytes32;

    struct ForwardRequest {
        address from;
        uint256 nonce;
        bytes data;
    }

    bytes32 private constant _TYPEHASH =
        keccak256("ForwardRequest(address from,uint256 nonce,bytes data)");

    mapping(address => uint256) private _nonces;

    constructor() EIP712("PhatRollupMetaTxReceiver", "0.0.1") {}

    // View functions for signer

    function metaTxGetNonce(address from) public view returns (uint256) {
        return _nonces[from];
    }

    function metaTxPrepare(address from, bytes calldata data) public view returns (ForwardRequest memory, bytes32) {
        return metaTxPrepareWithNonce(from, data, _nonces[from]);
    }

    function metaTxPrepareWithNonce(address from, bytes calldata data, uint256 nonce) public view returns (ForwardRequest memory, bytes32) {
        require(nonce >= _nonces[from], "nonce too low");
        ForwardRequest memory req = ForwardRequest(from, nonce, data);
        bytes32 hash = _hashTypedDataV4(
            keccak256(abi.encode(_TYPEHASH, from, nonce, keccak256(data)))
        );
        return (req, hash);
    }

    // Verification functions

    function metaTxVerify(ForwardRequest calldata req, bytes calldata signature) public view returns (bool) {
        address signer = _hashTypedDataV4(
            keccak256(abi.encode(_TYPEHASH, req.from, req.nonce, keccak256(req.data)))
        ).recover(signature);
        return _nonces[req.from] == req.nonce && signer == req.from;
    }

    modifier useMetaTx(
        ForwardRequest calldata req,
        bytes calldata signature
    ) {
        require(metaTxVerify(req, signature), "MetaTxReceiver: signature does not match request");
        _nonces[req.from] = req.nonce + 1;
        _;
    }
}
