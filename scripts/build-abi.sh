#!/bin/bash

cat ./evm/artifacts/contracts/PhatRollupAnchor.sol/PhatRollupAnchor.json | jq -r '.abi | tostring' > ./phat/crates/rollup/res/anchor.abi.json
