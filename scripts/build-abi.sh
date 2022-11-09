#!/bin/bash

cat ./evm/artifacts/contracts/PhatQueuedAnchor.sol/PhatQueuedAnchor.json | jq -r '.abi | tostring' > ./phat/rollup/res/anchor.abi.json
