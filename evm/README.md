# Phat Contract Solidity SDK

This package contains implementation designed for integration with Solidity based Smart Contract with Phat Contracts.

## Installation

We have tested with hardhat and Foundry. Truffle should be fine because it can import dependencies from the node_modules folder.

> [!WARNING]
> We recommend not using `forge install` if you are using Foundry, as it always uses the latest commit from the main branch. This can lead to version control issues and potential conflicts with dependencies. Instead, we suggest using `npm` or `yarn` as examples here.

### npm

```shell
npm install --save-dev @phala/solidity
```

### yarn

```shell
yarn add --dev @phala/solidity
```

### Setup remapping for Foundry

If you don't have the `remappings.txt`, you need to export remapping.

```shell
forge remappings > remappings.txt
```

And then added these two lines.

```txt
@openzeppelin/=node_modules/@openzeppelin/
@phala/solidity/=node_modules/@phala/solidity/
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
