// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./Interfaces.sol";
import "./PhatRollupReceiver.sol";

contract TestOracle is PhatRollupReceiver, Ownable {
    event PriceReceived(uint reqId, string pair, uint256 price);
    event FeedReceived(uint feedId, string pair,  uint256 price);

    uint constant TYPE_RESPONSE = 0;
    uint constant TYPE_FEED = 1;

    address anchor = address(0);
    mapping (uint => string) feeds;
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

    function registerFeed(uint id, string calldata name) public onlyOwner() {
        feeds[id] = name;
    }

    function onPhatRollupReceived(address /*_from*/, bytes calldata action)
        public override returns(bytes4)
    {
        // Always check the sender. Otherwise you can get fooled.
        require(msg.sender == anchor, "bad caller");

        require(action.length == 32 * 3, "cannot parse action");
        (uint respType, uint id, uint256 price) = abi.decode(action, (uint, uint, uint256));
        if (respType == TYPE_RESPONSE) {
            emit PriceReceived(id, requests[id], price);
            delete requests[id];
        } else if (respType == TYPE_FEED) {
            emit FeedReceived(id, feeds[id], price);
        }
        return ROLLUP_RECEIVED;
    }
}
