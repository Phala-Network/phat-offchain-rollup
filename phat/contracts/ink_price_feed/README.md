# InkPriceFeed

Implements a simple price feed with Ink! Offchain Rollup. It supports streaming a price feed and
answering individual requests from the Ink! contract side.


## Build

To build the contract:

```bash
cargo contract build
```

## Run e2e tests

### Deploy the ink! smart contract `test_oracle`

Before you can run the tests, you need to have an ink! smart contract deployed in a Substrate node with pallet-contracts.

#### Use the default ink! smart contract 

You can use the default smart contract deployed on Shibuya.

#### Or deploy your own ink! smart contract

You can build the smart contract 
```bash
cd ../../ink/contracts/test_oracle
cargo contract build
```
And use Contracts-UI or Polkadot.js to deploy your contract and interact with it.
You will have to configure `alice` as attestor.

### Run the e2e tests
And finally execute the following command to start e2e tests execution.

```bash
cargo test  -- --ignored --test-threads=1
```

### Parallel in Unit Tests

The flag `--test-threads=1` is necessary because by default [Rust unit tests run in parallel](https://doc.rust-lang.org/book/ch11-02-running-tests.html).
There may have a few tests trying to sending out transactions at the same time, resulting
conflicting nonce values.
The solution is to add `--test-threads=1`. So the unit test framework knows that you don't want
parallel execution.

### Push More Requests

After running `deploy-test.ts`, it will push only one oracle request to the rollup queue.
After a successful `answer_price_request` test run, the queue will become empty. To push a new
request, run:

```bash
TODO
```

### Enable Meta-Tx

Meta transaction allows the Phat Contract to submit rollup tx with attest key signature while using
arbitrary account to pay the gas fee. To enable meta tx in the unit test, change the `.env` file
and specify `SENDER_KEY`.

The Meta-Tx works fine in these cases

|                                             | Ink! Smart Contract deployed on local node | Ink! Smart Contract deployed on testnet        |
|---------------------------------------------|------------------------------------------|------------------------------------------------|
| Phat contract running on local (cargo test) | Meta-Tx works                            | Meta-Tx doesn't work (signature doesn't match) |
| Phat contract deployed on testnet           | Never try                                |  Meta-Tx works        |

