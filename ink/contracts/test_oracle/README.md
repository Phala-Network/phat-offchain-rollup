# Test Oracle

Implements a simple oracle to get/display the price of trading pairs. It uses the crate `phat_rollup_anchor_ink`.
It supports:
 - create a trading pair with an id and the token names. The name must match with the API id from CoinGecko. By example: `polkadot`, `astar`, `pha`, `usd`. Only an address granted as `MANAGER` can do it.
 - configure the attestor authorized to send the prices. Only an address granted as `MANAGER` can do it.
 - send a request to get the price of a given trading pair. Only an address granted as `MANAGER` can do it.
 - handle the messages to feed the trading pair. Only an address granted as `ATTESTOR` can do it.
 - display the trading pair with this id.
 - allow meta transactions to separate the attestor and the payer.
 - managed the roles and grant an address as `ADMIN`, `MANAGER` or `ATTESTOR`. Only the admin can do it.

By default, the contract owner is granted as `ADMIN` and `MANAGER` but it is not granted as `ATTESTOR`.

## Build

To build the contract:

```bash
cargo contract build
```

## Run e2e tests

Before you can run the test, you have to install a Substrate node with pallet-contracts. By default, `e2e tests` require that you install `substrate-contracts-node`. You do not need to run it in the background since the node is started for each test independently. To install the latest version:
```bash
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git
```

If you want to run any other node with pallet-contracts you need to change `CONTRACTS_NODE` environment variable:
```bash
export CONTRACTS_NODE="YOUR_CONTRACTS_NODE_PATH"
```

And finally execute the following command to start e2e tests execution.
```bash
cargo test --features e2e-tests
```
