// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "./MetaTransaction.sol";

// Uncomment this line to use console.log
// import "hardhat/console.sol";

/// Adds the Offchain Rollup functionalities to your contract
///
/// Phat Offchain Rollup Anchor implements the rollup functions to allow your contract to
/// integrate. It implements the basic kv-store, the rollup transaction handling, and also allow
/// you to interact with the Phat Contract in a request-response style.
///
/// ## Solidity Usage
///
/// ```solidity
/// contract ConsumerContract is PhatRollupAnchor {
///     constructor(address attestor) {
///         _grantRole(PhatRollupAnchor.ATTESTOR_ROLE, attestor);
///     }
///     function _onMessageReceived(bytes calldata action) internal override {
///         emit MsgReceived(action);
///     }
/// }
/// ```
///
/// Inherit this abstract contract in your consumer contract. To allow the Phat Contract to connect
/// to your consumer contract properly, you will need to specify `attestor`, an address generated
/// and controlled by the Phat Contract as its credential.
///
/// Add a attestor by `_grantRole()` as above. The attestors are controlled by OpenZeppelin's
/// `AccessControl` library. It allows to add and remove members to the role. You should have at
/// least one `attestor` to receive response from Phat Contract.
///
/// Then you should implement `_onMessageReceived()` to receive response. The parameter `action` is
/// the raw data provided by the Phat Contract. Usually it's encoded meaningful data in some
/// predefined schema (e.g. `abi.encode()`).
///
/// Call `_pushMessage(data)` to push the raw message to the Phat Contract. It returns the request
/// id, which can be used to link the response to the request later.
///
/// ## Phat Contract Usage
///
/// On the Phat Contract side, when some requests are processed, it should send an action
/// `ACTION_SET_QUEUE_HEAD` to removed the finished requests.
///
/// ## Storage layout
///
/// - `<lockKey>`: `uint` - the version of the queue lock
/// - `<prefix>/_head`: `uint` - index of the first element
/// - `<prefix>/_tail`: `uint` - index of the next element to push to the queue
/// - `<prefix/<n>`: `bytes` - the `n`-th message; `n` is encoded as uint32
abstract contract PhatRollupAnchor is ReentrancyGuard, MetaTxReceiver, AccessControl {
    // Constants aligned with the Phat Contract rollup queue implementation.
    bytes constant QUEUE_PREFIX = "q/";
    bytes constant KEY_HEAD = "_head";
    bytes constant KEY_TAIL = "_tail";

    // Only submission from attestor is allowed.
    bytes32 public constant ATTESTOR_ROLE = keccak256("ATTESTOR_ROLE");

    event MetaTxDecoded();
    event MessageQueued(uint256 idx, bytes data);
    event MessageProcessedTo(uint256);

    uint8 constant ACTION_REPLY = 0;
    uint8 constant ACTION_SET_QUEUE_HEAD = 1;
    uint8 constant ACTION_GRANT_ATTESTOR = 10;
    uint8 constant ACTION_REVOKE_ATTESTOR = 11;
    
    mapping (bytes => bytes) kvStore;

    /// Triggers a rollup transaction with `eq` conditoin check on uint256 values
    ///
    /// - actions: Starts with one byte to define the action type and followed by the parameter of
    ///     the actions. Supported actions: ACTION_REPLY, ACTION_SET_QUEUE_HEAD
    ///
    /// Note that calling from `address(this)` is allowed to make parameters a calldata. Don't
    /// abuse it.
    function rollupU256CondEq(
        bytes[] calldata condKeys,
        bytes[] calldata condValues,
        bytes[] calldata updateKeys,
        bytes[] calldata updateValues,
        bytes[] calldata actions
    ) public returns (bool) {
        // Allow meta tx to call itself
        require(msg.sender == address(this) || hasRole(ATTESTOR_ROLE, msg.sender), "bad attestor");
        return _rollupU256CondEqInternal(condKeys, condValues, updateKeys, updateValues, actions);
    }

    function metaTxRollupU256CondEq(
        ForwardRequest calldata req,
        bytes calldata signature
    ) public useMetaTx(req, signature) returns (bool) {
        require(hasRole(ATTESTOR_ROLE, req.from), "bad attestor");
        (
            bytes[] memory condKeys,
            bytes[] memory condValues,
            bytes[] memory updateKeys,
            bytes[] memory updateValues,
            bytes[] memory actions
            ) = abi.decode(req.data, (bytes[], bytes[], bytes[], bytes[], bytes[]));
        emit MetaTxDecoded();
        // Self-call to move memory bytes to calldata
        return this.rollupU256CondEq(condKeys, condValues, updateKeys, updateValues, actions);
    }

    function _rollupU256CondEqInternal(
        bytes[] calldata condKeys,
        bytes[] calldata condValues,
        bytes[] calldata updateKeys,
        bytes[] calldata updateValues,
        bytes[] calldata actions
    ) internal nonReentrant() returns (bool) {
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
            _onMessageReceived(action[1:]);
        } else if (actionType == ACTION_SET_QUEUE_HEAD) {
            require(action.length >= 1 + 32, "ACTION_SET_QUEUE_HEAD cannot decode");
            uint32 targetIdx = abi.decode(action[1:], (uint32));
            _popTo(targetIdx);
        } else if (actionType == ACTION_GRANT_ATTESTOR) {
            require(action.length >= 1 + 20, "ACTION_GRANT_ATTESTOR cannot decode");
            address attestor = abi.decode(action[1:], (address));
            _grantRole(ATTESTOR_ROLE, attestor);
        } else if (actionType == ACTION_REVOKE_ATTESTOR) {
            require(action.length >= 1 + 20, "ACTION_REVOKE_ATTESTOR cannot decode");
            address attestor = abi.decode(action[1:], (address));
            _revokeRole(ATTESTOR_ROLE, attestor);
        } else {
            revert("unsupported action");
        }
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
    function _pushMessage(bytes memory data) internal returns (uint32) {
        uint32 tail = queueGetUint(KEY_TAIL);
        bytes memory itemKey = abi.encode(tail);
        queueSetBytes(itemKey, data);
        queueSetUint(KEY_TAIL, tail + 1);
        emit MessageQueued(tail, data);
        return tail;
    }

    function _popTo(uint32 targetIdx) internal {
        uint32 curTail = queueGetUint(KEY_TAIL);
        require(targetIdx <= curTail, "invalid pop target");
        for (uint32 i = queueGetUint(KEY_HEAD); i < targetIdx; i++) {
            queueRemoveItem(i);
        }
        queueSetUint(KEY_HEAD, targetIdx);
        emit MessageProcessedTo(targetIdx);
    }

    /// The handler to be called when a message is received from a Phat Contract
    ///
    /// Reverting in this function resulting the revert of the offchain rollup transaction.
    function _onMessageReceived(bytes calldata action) internal virtual;

    /// Returns the prefix of the queue related keys
    ///
    /// The queue is persisted in the rollup kv store with all its keys prefixed. This function
    /// returns the prefix.
    function queueGetPrefix() public pure returns (bytes memory) {
        return QUEUE_PREFIX;
    }

    /// Returns the raw bytes value stored in the queue kv store
    function queueGetBytes(bytes memory key) public view returns (bytes memory) {
        bytes memory storageKey = bytes.concat(QUEUE_PREFIX, key);
        return kvStore[storageKey];
    }

    /// Returns the uint32 repr of the data stored in the queue kv store
    function queueGetUint(bytes memory key) public view returns (uint32) {
        bytes memory storageKey = bytes.concat(QUEUE_PREFIX, key);
        return toUint32Strict(kvStore[storageKey]);
    }

    /// Stores a raw bytes value to the queue kv store
    function queueSetBytes(bytes memory key, bytes memory value) internal {
        bytes memory storageKey = bytes.concat(QUEUE_PREFIX, key);
        kvStore[storageKey] = value;
    }

    /// Stores a uint32 value to the queue kv store
    function queueSetUint(bytes memory key, uint32 value) internal {
        bytes memory storageKey = bytes.concat(QUEUE_PREFIX, key);
        kvStore[storageKey] = abi.encode(value);
    }

    /// Removes a queue item
    function queueRemoveItem(uint32 idx) internal {
        bytes memory key = abi.encode(idx);
        bytes memory storageKey = bytes.concat(QUEUE_PREFIX, key);
        delete kvStore[storageKey];
    }
}
