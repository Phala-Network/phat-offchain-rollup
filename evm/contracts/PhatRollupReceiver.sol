// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

abstract contract PhatRollupReceiver {
    // bytes4(keccak256("onPhatRollupReceived(address,bytes)"))
    bytes4 constant ROLLUP_RECEIVED = 0x43a53d89;
    function onPhatRollupReceived(address _from, bytes calldata _action)
        public virtual returns(bytes4);
}
