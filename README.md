# Phat Offchain Rollup

Phat Offchain Rollup is an SDK designed to simplify the process of connecting Phat Contracts to a wide range of blockchains. Its primary focus is on providing transactional and atomic cross-blockchain operations for seamless integration and interaction.

## Table of Contents

- [The Challenge](#the-challenge)
- [Key Benefits](#key-benefits)
- [Getting Started](#getting-started)
    - [Add the Cargo Dependency](#add-the-cargo-dependency)
    - [Use the Rollup Client](#use-the-rollup-client)
    - [Request-Response Programming Model](#request-response-programming-model)
- [Integration](#integration)
    - [Deploy Offchain Rollup Anchor](#deploy-offchain-rollup-anchor)
    - [Integrate with Your Contract](#integrate-with-your-contract)
- [Examples and Use Cases](#examples-and-use-cases)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [License](#license)

## The Challenge

Developing Phat Contracts that interact with blockchains can be a common but challenging task, especially when it comes to handling concurrency issues in off-chain programs. Without a proper synchronization mechanism, Phat Contract instances may end up conflicting with each other.

Consider a real-world scenario: a smart contract distributes computation tasks to Phat Contract workers. These workers compete with each other when claiming tasks from the blockchain. Ideally, each task should only be claimed once. However, without coordination among the workers, they might send transactions simultaneously to claim the same task, resulting in inconsistent smart contract states.

Consistent state management is crucial when developing a Phat Contract that communicates with a blockchain. Developers need a reliable and **transactional** way to perform operations, where read and write tasks are combined into a single unit and executed atomically on the blockchain. This approach aligns with the **ACID** principle found in transactional database management systems.

Offchain Rollup is here to simplify Phat Contract development by providing a stable, ACID-compliant connection to various blockchains. This eliminates concurrency issues and enables a request-response programming model for seamless interaction between Phat Contracts and on-chain smart contracts.

## Key Benefits

- An on-chain gateway contract that enables seamless connectivity to Phat Contracts
- Reliable kv-store on blockchains for durable state management
    - Transactional (ACID) on-chain kv-store designed for stateful Phat Contracts
    - ACID-compliant read, write, and contract call operations for consistent data handling
- Request-response programming model that simplifies interactions between Phat Contracts and on-chain smart contracts
- Compatibility with EVM, Substrate, and ink!-powered blockchains for enhanced flexibility

## Getting Started

![](./images/offchain-rollup-arch.png)

To successfully use the Phat Offchain Rollup, follow these three steps illustrated in the diagram:

1. Integrate the rollup client within the Phat Contract to establish a connection with the anchor.
2. Deploy the Anchor contract onto the target blockchain.
3. Integrate the anchor with your consumer contract (the smart contract that interacts with the Phat Contract).

The rest of this section will focus on the Rollup Client integration and provide a detailed explanation on how to utilize the Phat Contract. Deployment of the anchor contract and its integration with the consumer contract will be covered in the [Integration](#integration) section later.

### Add the Cargo Dependency

> If you are not familiar with developing Phat Contracts, you can start with the [Setup Wiki Page](https://wiki.phala.network/en-us/build/stateless/setup/) for more information.

Update the `Cargo.toml` file in your Phat Contract project to include the `phat-offchain-rollup` dependency under the `[dependencies]` section:

```toml
phat_offchain_rollup = { git = "https://github.com/Phala-Network/phat-offchain-rollup.git", branch = "main", default-features = false, features = ["evm", "substrate"] }
```

Then, append the `std` feature to the `[features]` section as shown:

```toml
[features]
default = ["std"]
std = [
    "phat_offchain_rollup/std",
]
```

The `features` attribute allows you to control the rollup target chain activation. By default, all target chains are disabled to minimize the binary size. Enable them manually using the `features` attribute. The supported features include:

- `evm`: enables the client to connect to the EVM rollup anchor contracts.
- `substrate`: allows the client to connect to the Substrate rollup anchor pallet.
- `ink`: (currently a work in progress).

Additionally, the `logging` feature can be utilized to display internal logs, including internet access, to the logger. This can be helpful for debugging purposes.

### Using the Rollup Client

> In this section, we will use the `EvmRollupClient` as an example. It's straightforward to replace it with Substrate or ink! rollup clients, if needed.

To work with Offchain Rollup, follow these steps:

#### 1. Create an offchain rollup client:

```rust
let rpc = "http://localhost:8545";
let anchor_addr: H160 = hex!["e7f1725E7734CE288F8367e1Bb143E90bb3F0512"].into();
let client = EvmRollupClient::new(rpc, anchor_addr)
    .expect("failed to create rollup client");
```

The parameters represent:

- `rpc`: The JSON-RPC endpoint of the target EVM compatible chain. It must be HTTPS or HTTP (for testing only).
- `anchor_addr`: The deployed anchor contract address.

#### 2. Access the core functionalities:

Read values from the KV store:

```rust
let key = b"some-key";
let value: Vec<u8> = client.session.get(key)
    .expect("failed to get value");
```

Write values to the KV store:

```rust
let key = b"some-key";
let value = b"some-value".to_vec();
client.session.put(key, value);
```

Remove an entry in the KV store:

```rust
let key = b"some-key";
client.session.delete(key);
```

Note that read operations may fail due to network issues when accessing the remote RPC endpoint. Write operations are temporarily saved to the rollup client in memory and will not be applied to the blockchain until committed.

#### 3. Commit changes and submit the rollup transaction:

```rust
let maybe_submittable = client
    .commit()
    .expect("failed to commit");
if let Some(submittable) = maybe_submittable {
    let tx_id = submittable
        .submit(pair)
        .expect("failed to submit rollup tx");
}
```

Upon a successful submission, the client will broadcast the transaction and return the `tx_id` for future reference. Note that submitting a transaction doesn't guarantee that the transaction will be included in the blockchain.

### Request-Response Programming Model

A common use case of offchain rollup is to establish a stable Request-Response connection between the Phat Contract and the blockchain. The anchor contract enables developers to push arbitrary messages to the request queue. For instance, in the EVM Rollup Anchor contract, this can be done as follows:

```solidity
uint id = 1000;
string tradingPair = "polkadot/usd";
bytes message = abi.encode(id, tradingPair);
IPhatRollupAnchor(anchor).pushMessage(message);
```

On the Phat Contract side, the rollup client can connect to the anchor, check the queue, and potentially reply to the requests.

```rust
// Get a request if available
if let Some(raw_req) = client
    .session()
    .pop()
    .expect("failed to read queue") {
    // let action: Vec<u8> = ... Create your response based on the raw_req ...
    client.action(Action::Reply(action));
}
```

The rollup client provides features to handle requests:

- `client.session().pop()`: Returns an unprocessed request from the queue. Otherwise, returns `None`.
- `client.action(Action::Reply(action))`: Adds a reply action to send an arbitrary `Vec<u8>` data blob back to the anchor contract.

The `Reply` actions should be paired with the `pop()`. Once a reply is committed and submitted to the target blockchain, the anchor contract will pop the pending request accordingly in an ACID way. If the Phat Contract fails in this process, developers can retry execution multiple times until successful.

Note that the error handling in the sample code above is simplified. In real-world scenarios, developers should carefully handle both retry-able and non-retry-able errors. For instance, retries might help with network problems, but not with decoding an invalid request.

Finally, the consumer contract can be configured to receive responses as shown below.

```solidity
function onPhatRollupReceived(address _from, bytes calldata action)
    public override returns(bytes4)
{
    // Always check the sender. Otherwise, you can be gamed by hackers.
    require(msg.sender == anchor, "bad caller");
    // Utilize `action` here.
}
```

## Integration

To build an end-to-end project with offchain rollup, follow these steps to deploy the **Offchain Rollup Anchor** contract or pallet to the target blockchain and integrate it with the **Consumer Contract**. The rollup anchor is provided in this repository, while the consumer contract refers to the dApp that communicates with the Phat Contract.

### Deploy Offchain Rollup Anchor

To deploy the EVM rollup anchor, follow these steps:

1. Deploy the Phat Contract with a pre-generated ECDSA key pair (called submission key)
    - Sample code: [EvmPriceFeed](./phat/contracts/evm_price_feed/lib.rs)
2. (FIXIT) Deploy the contract: [PhatRollupAnchor](./evm/contracts/PhatRollupAnchor.sol) with the following parameters
    - `PhatRollupAnchor()`
    - `attestor`: The `H160` address of the submission key
    - `actionCallback`: The address of the consumer contract to receive the response
3. Transfer the ownership of `PhatRollupAnchor` to the consumer contract by calling `anchor.transferOwnership(consumerContract)`

Find a reference script [here](./evm/scripts/deploy-test.ts).

The Substrate pallet and ink! anchor deployment docs are currently under development (TODO).

### Integrate with Your Contract

Detailed instructions for consumer contract integration are coming soon (TODO). In the meantime, please refer to provided examples:

- For EVM: Sample consumer contract [TestOracle](./evm/contracts/TestOracle.sol)
- For ink! (WIP)
- For Substrate: Sample consumer pallet [phat-oracle-pallet](https://github.com/Phala-Network/phala-blockchain/blob/master/pallets/offchain-rollup/src/oracle.rs)

### Integration Resources

- EVM
    - [Phat-EVM Oracle Sample](./phat/contracts/evm_price_feed/README.md)
    - [pink-web3](https://docs.rs/pink-web3): A web3 client for calling EVM chain JSON-RPC and handling EVM ABI codec
- ink! (WIP)
- Substrate
    - [Phat-Substrate Oracle Sample](./phat/contracts/sub_price_feed)
    - [pink-subrpc](https://docs.rs/pink-subrpc/): A Substrate JSON-RPC client similar to Subxt, supporting HTTP(s)-only

Through these steps and resources, developers can seamlessly integrate Offchain Rollup with their projects and create powerful Phat Contracts that interact with various blockchains efficiently and securely.

## Technical Details

For an in-depth explanation of the project's technical aspects, please refer to the [Technical Details](./TechnicalDetails.md) page.

## Examples and Use Cases

Explore various examples and use cases of Phat Offchain Rollup in action:

- [Phat-EVM Oracle on Offchain Rollup](./EvmRollup.md)
- Phat-ink Oracle on Offchain Rollup (WIP)
- Phat-Substrate Oracle on Offchain Rollup (Documentation WIP)

## API Reference

Find API documentation for key components of the Phat Offchain Rollup SDK below:

- Phat Offchain Rollup API (WIP)
- [Pink-KV-Session](https://docs.rs/pink-kv-session/)
- EVM [PhatRollupAnchor](./evm/contracts/PhatRollupAnchor.sol)
- ink! Anchor Contract (WIP)
- Substrate [Offchain Rollup Anchor Pallet](https://github.com/Phala-Network/phala-blockchain/blob/master/pallets/offchain-rollup/src/anchor.rs)

As more features and integrations are developed, this README file will be updated accordingly to provide comprehensive documentation and resources for developers to make the most of the Phat Offchain Rollup SDK.

## Contributing

See [Development Notes](./DevNotes.md).

## License

The project is released under the Apache-2.0 license. This open-source license allows for the free use, modification, distribution, and commercial implementation of the software, while also ensuring that future contributions maintain the same level of open access. For more information about the specifics of the Apache-2.0 license, please visit [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).
