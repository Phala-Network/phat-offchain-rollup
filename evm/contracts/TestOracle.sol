// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./Interfaces.sol";
import "./PhatRollupReceiver.sol";

contract TestOracle is PhatRollupReceiver, Ownable {
    event PriceReceived(uint reqid, string pair, uint256 price);

    address anchor = address(0);
    mapping (uint => string) requests;
    uint nextRequest = 0;

    function setAnchor(address anchor_) public onlyOwner() {
        anchor = anchor_;
    }

    function request(string calldata tradingPair) public {
        require(anchor != address(0), "anchor not configured");
        // assemble the request
        uint id = nextRequest;
        requests[id] = tradingPair;
        IPhatRollupAnchor(anchor).pushMessage(abi.encode(id, tradingPair));
        nextRequest += 1;
    }

    function onPhatRollupReceived(address /*_from*/, bytes calldata action)
        public override returns(bytes4)
    {
        // Always check the sender. Otherwise you can get fooled.
        require(msg.sender == anchor, "bad caller");

        (uint id, uint256 price) = abi.decode(action, (uint, uint256));
        emit PriceReceived(id, requests[id], price);
        delete requests[id];
        return ROLLUP_RECEIVED;
    }
}
