# Phat Offchain Rollup

Phat Offchain Rollup is a SDK that enables Phat Contracts to easily connect to various blockchains. It is specifically designed to offer transactional and atomic cross blockchain operations.

## Table of Contents

- [Problem](#problem)
- [Features](#features)
- [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [Installation](#installation)
- [Usage](#usage)
    - [Deploy Offchain Rollup Anchor](#deploy-offchain-rollup-anchor)
    - [Create an Offchain Rollup Client](#create-an-offchain-rollup-client)
    - [Read and Write to the Blockchain](#read-and-write-to-the-blockchain)
    - [Request-Response Programming Model](#request-response-programming-model)
- [Examples and Use Cases](#examples-and-use-cases)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [License](#license)

## Problem

It's common to develop Phat Contracts that interact with the blokchain. However, running offchain programs are sometimes challenging due to it's concurrent execution nature. Without a synchronizing mechanism, the Phat Contract instances may conflict with each other.

Imagine a smart contract to distribute computation tasks to Phat Contract workers. The workers may compete with each other when claiming tasks from the blockchain. Each task is supposed to be claimed only once. However, without the coordination between the workers, multiple workers may try to send the transactions to claim the same task simultaneously and overriding each other, resulting the inconsistent smart contract states.

This is a typical state consistency scenario in distributed systems. When developing a Phat Contract talking to a blockchain, it's desired to have a reliable way to perform **transactional operations**, which means the read and write operations should be combined as a single unit, and finally executed atomically on the blockchain, obeying the same **ACID** principle in a transactional DBMS.

Offchain Rollup aims to simplify the development of Phat Contracts by providing a stable ACID connection to various blockchains, handling concurrency issues, and enabling a request-response programming model for easy interaction between Phat Contracts and on-chain smart contracts.

## Features

- A gateway deployed on-chain to allow connection to Phat Contract
- Store states reliablely on blockchains
- Transactional (ACID) on-chain kv-store for stateful Phat Contract
- Transactional (ACID) read, write and contract calls
- Request-response programming model for easy interaction with on-chain smart contracts
- Support EVM, Substrate, and ink! compatible blockchains

## Getting Started

### Prerequisites

TODO: List prerequisites, such as specific versions of Rust, Substrate, EVM, etc.

### Installation

TODO: Describe the installation process step by step, including commands and required tools.

## Usage

### Deploy Offchain Rollup Anchor

TODO: Explain how to deploy the Offchain Rollup Anchor to the target blockchain both for EVM and non-EVM environments.

### Create an Offchain Rollup Client

TODO: Describe how to include the SDK in an offchain ink! contract project and create a client that points to the deployed anchor contract.

### Read and Write to the Blockchain

TODO: Show examples of using the Offchain Rollup Client to perform read and write operations on the blockchain.

### Commit Transactions to the Blockchain

TODO: Explain how to call the commit function in the client to submit a transaction to the target blockchain.

### Request-Response Programming Model

TODO: Explain how to use the request-response programming model to create Phat Contracts that interact with on-chain smart contracts.

## Technical Details

Please refer to [this page](./technical-details.md)

## Examples and Use Cases

- [Phat-EVM oracle on offchain rollup](./EvmRollup.md)
- Phat-ink oracle on offchain rollup (WIP)
- Phat-Substrate oracle on offchain rollup (Doc WIP)

## API Reference

*TODO: Provide a detailed API reference for the Offchain Rollup SDK, including available functions, parameters, return values, and error handling.*

## Contributing

TODO: Provide guidelines for contributing to the Offchain Rollup project, including coding standards, testing, and submitting pull requests.

## License

TODO: Specify the license used for the Offchain Rollup project.


## [Development Notes](./DevNotes.md)

Highlighted TODOs:

- Simple scheduler
- Optimization: Batch read from rollup anchor