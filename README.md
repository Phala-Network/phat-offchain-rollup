# phat-offchain-rollup
Phat Contract Offchain Rollup implementation


## Phat Contract side

### Misc

- [ ] Refactor experimantal code as contracts
    - [x] Switch to OpenBrush's `ink-env` with the advanced unit test kits

### Contract

- [ ] SimpleScheduler
    - [x] [Design](https://hackmd.io/vl7oVbUlQmW8a_rcxhk9JQ)
    - [ ] Query `poll()`
        - call all the ready targets
        - should check health (trigger exactly one health worker)
    - [ ] Tx `register(config, address, calldata)` owner-only (direct call, stateless)
    - [ ] Tx `delete(id)` owner-only
    - [ ] log the triggered events
- [ ] RollupTransactor
    - [x] Account management: generate secret key & reveal public key
    - [x] Tx `config(rpc, rollup_handler, anchor)` by owner
    - [x] Query `poll()`
        - get `Result<RollupResult, Vec<u8>>` response
        - submit tx to `RollupResult.target_id`
            - use the latest nonce
            - fire and forget
    - [x] enum RollupTarget
        - EVM(chain, address)
        - Pallet(chain)
    - [x] Raw tx submit
    - [ ] Gas efficiency submit
            - for gas efficiency, save the recent submitted tx to local storage (with timeout) to avoid redundant submission in a short period
- [ ] TestOracle
    - [x] Minimum implementation
    - [ ] Real-time fetch price
    - [ ] Refactor to strip SDK logic

### SDK

- [ ] Locks
    - [x] Experimental lock tree (tx_read, tx_write)
    - [ ] Correct implementation
- [x] struct RollupTx
    - [x] Condition
    - [x] Updates
    - [x] Actions
- [x] struct RollupResult
    - [x] RollupTx
    - [x] RollupTarget
    - [ ] (opt) signature of RollupTx
- [ ] RollupReadClient
    - [x] Read from EVM
    - [ ] Cross validation
- [x] RollupWriteClient
- [ ] struct RollupManager (in ink! storage)
    - [ ] Config RollupTarget (chain, address)
    - [ ] (opt) offchain attestation
- [ ] (low) Cross-platform Rollup
    - [x] Basic codec abstraction (`platform::Platform`)
    - [ ] State reading abstraction
    - [ ] RollupTx serialization

## Development Notes

- `abi.decode()` doesn't have any error handling currently. When it failed, the transaction will get revereted silently, which is hard to debug. So it's always a good habit to verify the raw input to `decode()` beforehand.
