// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./PhatRollupAnchor.sol";
import "./Interfaces.sol";

/// A Phat Contract Rollup Anchor with a built-in request queue
///
/// Call `pushRequest(data)` to push the raw message to the Phat Contract. It returns the request
/// id, which can be used to link the response to the request later.
///
/// On the Phat Contract side, when some requests are processed, it should send an action
/// `ACTION_QUEUE_PROCESSED_TO` to removed the finished requests, and increment the queue lock
/// (very important!).
///
/// Storage layout:
///
/// - `<lockKey>`: `uint` - the version of the queue lock
/// - `<prefix>/start`: `uint` - index of the first element
/// - `<prefix>/end`: `uint` - index of the next element to push to the queue
/// - `<prefix/<n>`: `bytes` - the `n`-th message
contract PhatQueuedAnchor is PhatRollupAnchor, IPhatQueuedAnchor, Ownable {
    event Configured(bytes queuePrefix, bytes lockKey);
    event RequestQueued(uint256 idx, bytes data);
    event RequestProcessedTo(uint256);

    bytes queuePrefix;
    bytes lockKey;

    uint8 constant ACTION_QUEUE_PROCESSED_TO = 0;

    constructor(address caller_, address actionCallback_, bytes memory queuePrefix_)
        PhatRollupAnchor(caller_, actionCallback_)
    {
        // TODO: Now we are using the global lock. Should switch to fine grained lock in the
        // future.
        lockKey = hex"00";
        queuePrefix = queuePrefix_;
        emit Configured(queuePrefix, lockKey);
    }

    function getLockKey() public view returns (bytes memory) {
        return lockKey;
    }

    function getPrefix() public view returns (bytes memory) {
        return queuePrefix;
    }

    function getUint(bytes memory key) public view returns (uint256) {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        return toUint256Strict(phatStorage[storageKey], 0);
    }

    function getBytes(bytes memory key) public view returns (bytes memory) {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        return phatStorage[storageKey];
    }

    function setUint(bytes memory key, uint256 value) internal {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        phatStorage[storageKey] = abi.encode(value);
    }

    function setBytes(bytes memory key, bytes memory value) internal {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        phatStorage[storageKey] = value;
    }

    function removeBytes(bytes memory key) internal {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        phatStorage[storageKey] = "";
    }

    function incLock() internal {
        uint256 v = toUint256Strict(phatStorage[lockKey], 0);
        phatStorage[lockKey] = abi.encode(v + 1);
    }

    /// Pushes a request to the queue waiting for the Phat Contract to process
    ///
    /// Returns the index of the reqeust.
    function pushRequest(bytes memory data) public onlyOwner() returns (uint256) {
        uint256 end = getUint("end");
        bytes memory itemKey = abi.encode(end);
        setBytes(itemKey, data);
        setUint("end", end + 1);
        incLock();
        emit RequestQueued(end, data);
        return end;
    }

    function popTo(uint256 end) internal {
        uint256 queueEnd = getUint("end");
        require(end <= queueEnd, "invalid queue end");
        for (uint256 i = getUint("start"); i < end; i++) {
            bytes memory itemKey = abi.encode(end);
            removeBytes(itemKey);
        }
        setUint("start", end);
        incLock();
        emit RequestProcessedTo(end);
    }

    // Handle queue related messages
    function handleCustomAction(bytes calldata action) internal override {
        require(action.length > 0, "invalid action");
        // processed to: [0] [u256 to]
        uint8 actionType = uint8(action[0]);
        if (actionType == ACTION_QUEUE_PROCESSED_TO) {
            require(action.length == 32 + 1, "bad queue arg size");
            uint256 end = abi.decode(action[1:], (uint256));
            popTo(end);
        } else {
            revert("unsupported action");
        }
    }
}