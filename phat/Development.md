# Phat Contract Implementation

## Prepare the environment

1. Initialize git submodules, and install yarn v1, node v18, and the dependencies:

    ```bash
    git submodule update --init
    yarn
    (cd setup; yarn)
    ```

2. Update rust and cargo-contract (>= 3.2.0):

    ```bash
    rustup update
    cargo install --force --locked cargo-contract
    ```

3. Try to start the local stack. devphase will download the selected prebuilt binaries from Github at the first time.

    ```bash
    yarn devphase stack
    ```

If you haven't installed node.js, it's suggested to install it via [nvm](https://github.com/nvm-sh/nvm#install--update-script). 
It's also suggested to install `yarn` (classical) by `npm install --global yarn` ([docs](https://classic.yarnpkg.com/lang/en/docs/install)).

## Launch a standalone local test stack for custom testing

1. start the local stack.

    ```bash
    yarn devphase stack
    ```

2. Init the testnet (currently by [this script](https://github.com/shelvenzhou/phala-blockchain-setup))

    ```bash
    # edit .env file
    yarn
    node src/setup-drivers.js
    ```

3. You can also dump the contract log from the log server driver with the same scripts:

    ```bash
    node src/dump-logs.js
    ```

To configure the local test stack, please check [devphase.config.ts](./devphase.config.ts).

## Compile contracts

```bash
yarn devphase contract compile
```

You can also specify which the contract to build by adding the contract name. The name should be
in snake case, consistent with the directory names under `contracts/`.

## Run E2E test

Simply run:

```bash
yarn devphase contract test
```

The tests are written in TypeScript at `./tests/*.test.ts`. The logs are output to `./logs/{date}`
directory.

