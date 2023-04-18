// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./PhatRollupAnchor.sol";

contract TestOracle is PhatRollupAnchor, Ownable {
    event PriceReceived(uint reqId, string pair, uint256 price);
    event FeedReceived(uint feedId, string pair,  uint256 price);
    event ErrorReceived(uint reqId, string pair,  uint256 errno);

    uint constant TYPE_RESPONSE = 0;
    uint constant TYPE_FEED = 1;
    uint constant TYPE_ERROR = 2;

    mapping (uint => string) feeds;
    mapping (uint => string) requests;
    uint nextRequest = 1;

    constructor(address phatAttestor) {
        _grantRole(PhatRollupAnchor.ATTESTOR_ROLE, phatAttestor);
    }

    function request(string calldata tradingPair) public {
        // assemble the request
        uint id = nextRequest;
        requests[id] = tradingPair;
        _pushMessage(abi.encode(id, tradingPair));
        nextRequest += 1;
    }

    function registerFeed(uint id, string calldata name) public onlyOwner() {
        feeds[id] = name;
    }

    function _onMessageReceived(bytes calldata action) internal override {
        require(action.length == 32 * 3, "cannot parse action");
        (uint respType, uint id, uint256 data) = abi.decode(action, (uint, uint, uint256));
        if (respType == TYPE_RESPONSE) {
            emit PriceReceived(id, requests[id], data);
            delete requests[id];
        } else if (respType == TYPE_FEED) {
            emit FeedReceived(id, feeds[id], data);
        } else if (respType == TYPE_ERROR) {
            emit ErrorReceived(id, requests[id], data);
            delete requests[id];
        }
    }
}
