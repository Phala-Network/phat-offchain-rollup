// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "./Interfaces.sol";
import "./PhatRollupReceiver.sol";

// Uncomment this line to use console.log
// import "hardhat/console.sol";


/// A Phat Contract Rollup Anchor with a built-in request queue
///
/// Call `pushMessage(data)` to push the raw message to the Phat Contract. It returns the request
/// id, which can be used to link the response to the request later.
///
/// On the Phat Contract side, when some requests are processed, it should send an action
/// `ACTION_SET_QUEUE_HEAD` to removed the finished requests, and increment the queue lock
/// (very important!).
///
/// Storage layout:
///
/// - `<lockKey>`: `uint` - the version of the queue lock
/// - `<prefix>/_head`: `uint` - index of the first element
/// - `<prefix>/_tail`: `uint` - index of the next element to push to the queue
/// - `<prefix/<n>`: `bytes` - the `n`-th message; `n` is encoded as uint32
contract PhatRollupAnchor is IPhatRollupAnchor, ReentrancyGuard, Ownable {
    bytes4 constant ROLLUP_RECEIVED = 0x43a53d89;
    // function genReceiverSelector() public pure returns (bytes4) {
    //     return bytes4(keccak256("onPhatRollupReceived(address,bytes)"));
    // }
    // function testConvert(bytes calldata inputData) public view returns (uint256) {
    //     return toUint256(inputData, 0);
    // }

    // Constants aligned with the Phat Contract rollup queue implementation.
    bytes constant KEY_HEAD = "_head";
    bytes constant KEY_TAIL = "_tail";

    event MessageQueued(uint256 idx, bytes data);
    event MessageProcessedTo(uint256);

    uint8 constant ACTION_REPLY = 0;
    uint8 constant ACTION_SET_QUEUE_HEAD = 1;
    
    address submitter;
    address actionCallback;
    mapping (bytes => bytes) kvStore;
    bytes queuePrefix;

    constructor(address submitter_, address actionCallback_, bytes memory queuePrefix_) {
        // require(actionCallback_.isContract(), "bad callback");
        submitter = submitter_;
        actionCallback = actionCallback_;
        queuePrefix = queuePrefix_;
    }
    
    /// Triggers a rollup transaction with `eq` conditoin check on uint256 values
    ///
    /// - actions: Starts with one byte to define the action type and followed by the parameter of
    ///     the actions. Supported actions: ACTION_SYS, ACTION_CALLBACK
    function rollupU256CondEq(
        bytes[] calldata condKeys,
        bytes[] calldata condValues,
        bytes[] calldata updateKeys,
        bytes[] calldata updateValues,
        bytes[] calldata actions
    ) public nonReentrant() returns (bool) {
        require(msg.sender == submitter, "bad submitter");
        require(condKeys.length == condValues.length, "bad cond len");
        require(updateKeys.length == updateValues.length, "bad update len");
        
        // check cond
        for (uint i = 0; i < condKeys.length; i++) {
            uint32 value = toUint32Strict(kvStore[condKeys[i]]);
            uint32 expected = toUint32Strict(condValues[i]);
            if (value != expected) {
                revert("cond not met");
            }
        }
        
        // apply updates
        for (uint i = 0; i < updateKeys.length; i++) {
            kvStore[updateKeys[i]] = updateValues[i];
        }
        
        // apply actions
        for (uint i = 0; i < actions.length; i++) {
            handleAction(actions[i]);
        }

        return true;
    }

    function handleAction(bytes calldata action) private {
        uint8 actionType = uint8(action[0]);
        if (actionType == ACTION_REPLY) {
            require(checkAndCallReceiver(action[1:]), "action failed");
        } else if (actionType == ACTION_SET_QUEUE_HEAD) {
            require(action.length >= 5, "ACTION_SET_QUEUE_HEAD cannot decode");
            uint32 targetIdx = abi.decode(action[1:], (uint32));
            popTo(targetIdx);
        } else {
            revert("unsupported action");
        }
    }
    
    function checkAndCallReceiver(bytes calldata action) private returns(bool) {
        bytes4 retval = PhatRollupReceiver(actionCallback)
            .onPhatRollupReceived(address(this), action);
        return (retval == ROLLUP_RECEIVED);
    }

    function getStorage(bytes memory key) public view returns(bytes memory) {
        return kvStore[key];
    }

    function toUint32Strict(bytes memory _bytes) public pure returns (uint32) {
        if (_bytes.length == 0) {
            return 0;
        }
        require(_bytes.length == 32, "toUint32Strict_outOfBounds");
        uint32 v = abi.decode(_bytes, (uint32));
        return v;
    }

    // Queue functions

    /// Pushes a request to the queue waiting for the Phat Contract to process
    ///
    /// Returns the index of the reqeust.
    function pushMessage(bytes memory data) public onlyOwner() returns (uint32) {
        uint32 tail = queueGetUint(KEY_TAIL);
        bytes memory itemKey = abi.encode(tail);
        queueSetBytes(itemKey, data);
        queueSetUint(KEY_TAIL, tail + 1);
        emit MessageQueued(tail, data);
        return tail;
    }

    function popTo(uint32 targetIdx) internal {
        uint32 curTail = queueGetUint(KEY_TAIL);
        require(targetIdx <= curTail, "invalid pop target");
        for (uint32 i = queueGetUint(KEY_HEAD); i < targetIdx; i++) {
            queueRemoveItem(i);
        }
        queueSetUint(KEY_HEAD, targetIdx);
        emit MessageProcessedTo(targetIdx);
    }


    /// Returns the prefix of the queue related keys
    ///
    /// The queue is persisted in the rollup kv store with all its keys prefixed. This function
    /// returns the prefix.
    function queueGetPrefix() public view returns (bytes memory) {
        return queuePrefix;
    }

    /// Returns the raw bytes value stored in the queue kv store
    function queueGetBytes(bytes memory key) public view returns (bytes memory) {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        return kvStore[storageKey];
    }

    /// Returns the uint32 repr of the data stored in the queue kv store
    function queueGetUint(bytes memory key) public view returns (uint32) {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        return toUint32Strict(kvStore[storageKey]);
    }

    /// Stores a raw bytes value to the queue kv store
    function queueSetBytes(bytes memory key, bytes memory value) internal {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        kvStore[storageKey] = value;
    }

    /// Stores a uint32 value to the queue kv store
    function queueSetUint(bytes memory key, uint32 value) internal {
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        kvStore[storageKey] = abi.encode(value);
    }

    /// Removes a queue item
    function queueRemoveItem(uint32 idx) internal {
        bytes memory key = abi.encode(idx);
        bytes memory storageKey = bytes.concat(queuePrefix, key);
        delete kvStore[storageKey];
    }
}
