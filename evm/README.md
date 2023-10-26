# Phat Contract Solidity SDK

This package contains implementation designed for integration with Solidity based Smart Contract with Phat Contracts.

## Installation

### npm

```shell
npm install --save-dev @phala/solidity
```

### yarn

```shell
yarn add --dev @phala/solidity
```

## Usage

```solidity
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@phala/solidity/contracts/PhatRollupAnchor.sol";

contract OracleConsumerContract is PhatRollupAnchor, Ownable {
    event ResponseReceived(uint reqId, string reqData, uint256 value);
    event ErrorReceived(uint reqId, string reqData, uint256 errno);

    uint constant TYPE_RESPONSE = 0;
    uint constant TYPE_ERROR = 2;

    mapping(uint => string) requests;
    uint nextRequest = 1;

    constructor(address phatAttestor) {
        _grantRole(PhatRollupAnchor.ATTESTOR_ROLE, phatAttestor);
    }

    function setAttestor(address phatAttestor) public {
        _grantRole(PhatRollupAnchor.ATTESTOR_ROLE, phatAttestor);
    }

    function request(string calldata reqData) public {
        // assemble the request
        uint id = nextRequest;
        requests[id] = reqData;
        _pushMessage(abi.encode(id, reqData));
        nextRequest += 1;
    }

    function _onMessageReceived(bytes calldata action) internal override {
        // Optional to check length of action
        // require(action.length == 32 * 3, "cannot parse action");
        (uint respType, uint id, uint256 data) = abi.decode(
            action,
            (uint, uint, uint256)
        );
        if (respType == TYPE_RESPONSE) {
            emit ResponseReceived(id, requests[id], data);
            delete requests[id];
        } else if (respType == TYPE_ERROR) {
            emit ErrorReceived(id, requests[id], data);
            delete requests[id];
        }
    }
}
```
