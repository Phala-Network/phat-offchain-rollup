// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./PhatRollupAnchor.sol";

contract TestReceiver is PhatRollupAnchor {
    bytes[] recvActions;

    event MsgReceived(bytes);

    constructor(address attestor) {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(PhatRollupAnchor.ATTESTOR_ROLE, attestor);
    }

    function pushMessage(bytes memory data) public {
        _pushMessage(data);
    }

    function _onMessageReceived(bytes calldata action) internal override {
        recvActions.push(action);
        emit MsgReceived(action);
    }

    function getRecvLength() public view returns (uint) {
        return recvActions.length;
    }

    function getRecv(uint i) public view returns (bytes memory) {
        return recvActions[i];
    }
}
