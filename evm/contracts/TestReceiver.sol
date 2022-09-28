// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./PhatRollupReceiver.sol";

contract TestReceiver is PhatRollupReceiver {
    address[] recvFroms;
    bytes[] recvActions;

    event MsgReceived(address, bytes);

    function onPhatRollupReceived(address from, bytes calldata action)
        public override returns(bytes4)
    {
        recvFroms.push(from);
        recvActions.push(action);
        emit MsgReceived(from, action);
        return ROLLUP_RECEIVED;
    }

    function getRecvLength() public view returns (uint) {
        return recvFroms.length;
    }

    function getRecv(uint i) public view returns (address, bytes memory) {
        return (recvFroms[i], recvActions[i]);
    }
}
