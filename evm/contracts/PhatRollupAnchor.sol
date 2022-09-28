// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "./PhatRollupReceiver.sol";

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract PhatRollupAnchor is ReentrancyGuard {
    bytes4 constant ROLLUP_RECEIVED = 0x43a53d89;
    // function genReceiverSelector() public pure returns (bytes4) {
    //     return bytes4(keccak256("onPhatRollupReceived(address,bytes)"));
    // }
    // function testConvert(bytes calldata inputData) public view returns (uint256) {
    //     return toUint256(inputData, 0);
    // }

    uint8 constant ACTION_SYS = 0;
    uint8 constant ACTION_CALLBACK = 1;
    uint8 constant ACTION_CUSTOM = 2;
    
    address caller;
    address actionCallback;
    mapping (bytes => bytes) phatStorage;

    constructor(address caller_, address actionCallback_) {
        // require(actionCallback_.isContract(), "bad callback");
        caller = caller_;
        actionCallback = actionCallback_;
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
        require(msg.sender == caller, "bad caller");
        require(condKeys.length == condValues.length, "bad cond len");
        require(updateKeys.length == updateValues.length, "bad update len");
        
        // check cond
        for (uint i = 0; i < condKeys.length; i++) {
            uint256 value = toUint256Strict(phatStorage[condKeys[i]], 0);
            uint256 expected = toUint256Strict(condValues[i], 0);
            if (value != expected) {
                revert("cond not met");
            }
        }
        
        // apply actions
        for (uint i = 0; i < actions.length; i++) {
            handleAction(actions[i]);
        }
        
        // apply updates
        for (uint i = 0; i < updateKeys.length; i++) {
            phatStorage[updateKeys[i]] = updateValues[i];
        }

        return true;
    }

    function handleAction(bytes calldata action) private {
        uint8 actionType = uint8(action[0]);
        if (actionType == ACTION_SYS) {
            // pass
        } else if (actionType == ACTION_CALLBACK) {
            require(checkAndCallReceiver(action[1:]), "action failed");
        } else if (actionType == ACTION_CUSTOM) {
            handleCustomAction(action[1:]);
        } else {
            revert("unsupported action");
        }
    }

    /// Handles a custom action defined in a child contract
    ///
    /// Override it in the child class if you want to implement any special custom actions. Revert
    /// if you want to interrupt the transaction.
    function handleCustomAction(bytes calldata action) internal virtual {}
    
    function checkAndCallReceiver(bytes calldata action) private returns(bool) {
        bytes4 retval = PhatRollupReceiver(actionCallback)
            .onPhatRollupReceived(address(this), action);
        return (retval == ROLLUP_RECEIVED);
    }

    function getStorage(bytes memory key) public view returns(bytes memory) {
        return phatStorage[key];
    }

    function toUint256Strict(bytes memory _bytes, uint256 _start) public pure returns (uint256) {
        if (_bytes.length == 0) {
            return 0;
        }
        require(_bytes.length == _start + 32, "toUint256_outOfBounds");
        uint256 tempUint;
        assembly {
            tempUint := mload(add(add(_bytes, 0x20), _start))
        }
        return tempUint;
    }
}
