// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

interface IPhatRollupAnchor {
    function pushMessage(bytes memory data) external returns (uint32);
}
