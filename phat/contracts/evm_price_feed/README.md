# EvmPriceFeed

Implements a simple price feed with EVM Offchain Rollup. It supports streaming a price feed and
answering individual requests from the EVM side.

## Run a full unit tests

To run the unit tests:

```bash
cargo test
```

The contract logs to `env_logger`. To run with logger enabled:

```
RUST_LOG=info RUST_BACKTRACE=1 cargo test -- --nocapture
```

Some tests involving EVM testnet setup are marked as `ignored`. To run the full tests:

1. Start a local hardhat testnet and deploy the EVM

    ```bash
    cd evm
    # Run `yarn` to init the repo if you haven't done it before
    # Run the two command below in separate windows / panes
    npx hardhat node
    npx hardhat run --network localhost ./scripts/deploy-test.ts
    ```

2. Copy `.env_sample` as `.env` if you haven't done it before. It tells the Phat Contract to
   connect to the local EVM testnet you just created.

3. Run the full e2e test:

    ```bash
    cargo test -- --ignored
    ```

Note that after running `deploy-test.ts`, it will push only one oracle reqeust to the rollup queue.
After a successful `answer_price_request` test run, the queue will become empty. To push a new
request, run:

```bash
npx hardhat run --network localhost ./scripts/testnet-push-request.ts
```
