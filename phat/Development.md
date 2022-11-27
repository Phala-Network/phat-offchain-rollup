# Phat Contract Implementation

## Prepare the environment

1. Install yarn v1 and node v16, and install the dependencies

    ```bash
    yarn
    ```

2. Try to start the local stack. devphase will download the selected prebuilt binaries from Github at the first time.

    ```bash
    yarn devphase stack
    ```

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

## Run E2E test

Simply run:

```bash
yarn devphase test
```

The tests are written in TypeScript at `./tests/*.test.ts`. The logs are output to `./logs/{date}`
directory.
