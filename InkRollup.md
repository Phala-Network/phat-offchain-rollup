# Deploy a Phat-ink! Oracle with Offchain Rollup

This repo has implemented a Phat Contract serving as a data source of an Ink! smart-contract oracle. It can:

- Fetch price data from CoinGecko.
- Push-mode oracle: Create and config a price feed on the Ink! smart-contract side, and receive price quotes
  constantly streamed from the Phat Contract.
- Pull-mode oracle: Send individual requests from the Ink! smart-contract side, and receive responses from the Phat
  Contract.

## Architecture

(WIP)

- Phat Contracts
- Offchain Rollup clients
- Ink! smart-contracts

## Deploy

### Ink! contracts

The precompiled contract can be found at:
```
./ink/artifacts/test_oracle/test_oracle.contract
```

> If you want to build a fresh contract instead, you can compile it.

The source can be found at `ink/contracts/test_oracle` folder in this repo.
```bash
cd ink/contracts/test_oracle
```

Compile the Ink! smart contract
```bash
cargo contract build
```

You can choose to deploy the contract on a local node.
In this case you can install :
 - [Swanky node](https://github.com/AstarNetwork/swanky-node). The easiest method of installation is by downloading and executing a precompiled binary from the [Release Page](https://github.com/AstarNetwork/swanky-node/releases)
 - [Substrate node](https://github.com/paritytech/substrate-contracts-node.git) with pallet-contracts.

Or alternatively, you can deploy it to a public blockchain (e.g. Shibuya/Shiden/Astar) depending on
the network you have configured.


### Phat Contract

If you just want to run a unit test, now you can refer to the [InkPriceFeed unit test docs](./phat/contracts/ink_price_feed/README.md).
Otherwise, follow the instructions below if you would like to deploy a real Phat Contract on a live
chain. Here let's assume the deployment target is the Phala PoC-5 live testnet.

> PoC-5 Network parmeters:
>
> - Phat Contract UI: <https://phat.phala.network>
> - Substrate RPC: `wss://poc5.phala.network/ws`
> - PRuntime endpoint: `https://poc5.phala.network/tee-api-1`

You will need to deploy `InkPriceFeed` contract on the testnet. Enter [Phat UI](https://phat.phala.network).
Get some test coin by `Get Test-PHA` if you don't have. Then you can click `+ Upload` to deploy a
contract. 

The precompiled contract can be found at:

```
./phat/artifacts/ink_price_feed/ink_price_feed.contract
```

> If you want to build a fresh contract instead:

The phat contract is at `phat/contracts/ink_price_feed` folder in this repo.
```bash
cd phat/contracts/ink_price_feed
```

Compile the Phat contract
```bash
cargo contract build
```


After a successful deployment, the Phat UI should bring you to the contract page. Now you need to configure
the contract by sending a `config()` transaction with the arguments below:

- `rpc`: The Substrate RPC for Phat Contract to send transaction. It must be a http endpoint.
- `pallet_id`: The pallet id for Phat Contract to send transaction. 70 for Shibuya, 7 for swanky node.
- `call_id`: The call id for Phat Contract to send transaction. 6 in many cases.
- `contract id`: The anchor Ink! contract you deployed on substrate node, with "0x".
- `sender_key`: The secp256k1 private key you used to pay the transaction fees,  with "0x".

>Next you will have to authorise the phat contract to send the messages to ink! smart contract

Call the method `get_attest_address` and `get_ecdsa_public_key` to get the public keys used by the phat contract.

In the Ink! smart contract side, use the Contracts-UI or Polkadot.js to grant the phat contract as  attestor
- Use the method `registerAttestor` to set the attest_address and the ecdsa_public_key
- Use the method `accessControl::grantRole` to set only the attest_address
- Use the method `metaTransaction::registerEcdsaPublicKey` to set only the ecdsa_public_key

Once configured, you can call the following query methods in ink! smart contract:
- `createTradingPair`: Create a trading to get the price between two tokens
- `requestPrice`: Send a message to receive the latest price from the Phat Contract
- `getTradingPair`: Display the trading pair with the latest price received

You can call the following query methods in phat contract:

- `feed_price_from_coingecko()`: Fetch the latest price of your token0/token1 trading pair, and submit it to the
    Ink! smart contract contracts. You will get `FeedReceived` message on Ink smart contract side.
- `anser_price()`: Read one request from the Ink smart contract side, and answer it with the price quote. 
    You will get `FeedReceived` message on Ink smart contract side.
