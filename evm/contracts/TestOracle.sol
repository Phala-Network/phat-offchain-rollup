// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./Interfaces.sol";
import "./PhatRollupReceiver.sol";

contract TestOracle is PhatRollupReceiver, Ownable {
    event PriceReceived(uint reqid, string pair, uint256 price);

    address queuedAnchor = address(0);
    mapping (uint => string) requests;
    uint nextRequest = 0;

    function setQueuedAnchor(address queuedAnchor_) public onlyOwner() {
        queuedAnchor = queuedAnchor_;
    }

    function request(string calldata tradingPair) public {
        require(queuedAnchor != address(0), "anchor not configured");
        // assemble the request
        uint id = nextRequest;
        requests[id] = tradingPair;
        IPhatQueuedAnchor(queuedAnchor).pushRequest(abi.encode(id, tradingPair));
        nextRequest += 1;
    }

    function onPhatRollupReceived(address _from, bytes calldata action)
        public override returns(bytes4)
    {
        // Always check the sender. Otherwise you can get fooled.
        require(msg.sender == queuedAnchor, "bad caller");

        (uint id, uint256 price) = abi.decode(action, (uint, uint256));
        emit PriceReceived(id, requests[id], price);
        delete requests[id];
        return ROLLUP_RECEIVED;
    }
}
