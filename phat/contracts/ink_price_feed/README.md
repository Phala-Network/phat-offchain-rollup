# InkPriceFeed

Implements a simple price feed with Ink! Offchain Rollup. It supports streaming a price feed and
answering individual requests from the Ink! contract side.


## Build

To build the contract:

```bash
cargo contract build
```

## Run Integration tests

### Deploy the ink! smart contract `test_oracle`

Before you can run the tests, you need to have an ink! smart contract deployed in a Substrate node with pallet-contracts.

#### Use the default ink! smart contract 

You can use the default smart contract deployed on Shibuya (`a4MAezQGvh8czvrBih66JyxbvvmM45SStDg4BoVauGvMPYm`).

#### Or deploy your own ink! smart contract

You can build the smart contract 
```bash
cd ../../ink/contracts/test_oracle
cargo contract build
```
And use Contracts-UI or Polkadot.js to deploy your contract and interact with it.
You will have to configure `alice` as attestor.

### Add trading pairs and push some requests

Use Contracts-UI or Polkadot.js to interact with your smart contract deployed on local node or Shibuya.
You can create a new trading pair and request a price feed by the Phat Contract.

In Shibuya, there are already 3 trading pairs defined in the contracts `a4MAezQGvh8czvrBih66JyxbvvmM45SStDg4BoVauGvMPYm`.
 - id 11 for the pair `polkadot`/`usd`
 - id 12 for `astar`/`usd`
 - id 13 for `pha`/`usd`

If you want to create another request for the trading pair with the id 12 
```bash
cargo contract call --contract a4MAezQGvh8czvrBih66JyxbvvmM45SStDg4BoVauGvMPYm --message request_price --args 12 --url wss://rpc.shibuya.astar.network --suri "bottom drive obey lake curtain smoke basket hold race lonely fit walk"  ../../../ink/artifacts/test_oracle/test_oracle.wasm
```

### Run the integration tests

Copy `.env_localhost` or `.env_shibuya` as `.env` if you haven't done it before. 
It tells the Phat Contract how to connect to ink! smart contract you just created.

And finally execute the following command to start integration tests execution.

```bash
cargo test  -- --ignored --test-threads=1
```

### Parallel in Integration Tests

The flag `--test-threads=1` is necessary because by default [Rust unit tests run in parallel](https://doc.rust-lang.org/book/ch11-02-running-tests.html).
There may have a few tests trying to send out transactions at the same time, resulting
conflicting nonce values.
The solution is to add `--test-threads=1`. So the unit test framework knows that you don't want
parallel execution.

### Enable Meta-Tx

Meta transaction allows the Phat Contract to submit rollup tx with attest key signature while using
arbitrary account to pay the gas fee. To enable meta tx in the unit test, change the `.env` file
and specify `SENDER_KEY`.

The Meta-Tx works fine in these cases

|                                             | Ink! Smart Contract deployed on local node | Ink! Smart Contract deployed on testnet        |
|---------------------------------------------|--------------------------------------------|------------------------------------------------|
| Phat contract running on local (cargo test) | Meta-Tx works                              | Meta-Tx doesn't work (signature doesn't match) |
| Phat contract deployed on testnet           | Never tried                                | Meta-Tx works                                  |

