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
    cargo test -- --ignored --test-threads=1
    ```

### Parallel in Unit Tests

The flag `--test-threads=1` is necessary because by default [Rust unit tests run in parallel](https://doc.rust-lang.org/book/ch11-02-running-tests.html).
There may have a few tests trying to sending out transactions at the same time, resulting
conflicting nonce values. In such case, you will get EVM erros like below:

```
RpcError { code: -32000, message: \"Nonce too low. Expected nonce to be 8 but got 7.\" }
```

The solution is to add `--test-threads=1`. So the unit test framework knows that you don't want
parallel execution.

### Push More Requests

After running `deploy-test.ts`, it will push only one oracle reqeust to the rollup queue.
After a successful `answer_price_request` test run, the queue will become empty. To push a new
request, run:

```bash
npx hardhat run --network localhost ./scripts/testnet-push-request.ts
```

### Enable Meta-Tx

Meta transaction allows the Phat Contract to submit rollup tx with attest key signature while using
arbitrary account to pay the gas fee. To enable meta tx in the unit test, change the `.env` file
and specify `SENDER_KEY`.
