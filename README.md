# phat-stateful-rollup
Phat Contract Stateful Rollup implementation


## Phat Contract side


- Contract
    - Scheduler (dummy)
        - query: `poll()`
            - call all the ready targets
            - should check health (trigger exactly one health worker)
        - tx: `register(config, address, calldata)` owner-only (direct call, stateless)
        - tx: `delete(id)` owner-only
        - log: the trigger events
    - RollupTransactor
        - tx: `add_target(type) -> id`
        - query: `target_info(id) -> TargetInfo`
            - address of the managed wallet
        - query: `execute(address, calldata)`
            - got `Result<RollupResult, Vec<u8>>` response
            - submit tx to `RollupResult.target_id`
                - use the latest nonce
                - fire and forget
        - enum RollupTarget
            - EVM(chain, address)
            - Pallet(chain)
    - TestOracle
- Library
    - OffchainRollup
        - struct RollupManager
            - offchain attestation
            - RollupTarget
        - Locks
            - lock tree
        - struct RollupTx
            - claim_read(lock)
            - claim_write(lock)
        - struct RollupResult
            - RollupTx
            - signature of RollupTx
            - RollupTarget
