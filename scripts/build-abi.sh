#!/bin/bash

cat ./evm/artifacts/contracts/PhatQueuedAnchor.sol/PhatQueuedAnchor.json | jq -r '.abi | tostring' > ./phat/res/sample_oracle/anchor.abi.json
