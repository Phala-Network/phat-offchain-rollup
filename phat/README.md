# Phat Contract Implementation

## Run a local test environment

1. Put the Phala binaries inside `tmp/phala-dev-stack/bin`, including `node` (renamed from `phala-node`), `pruntime` and `pherry`.

2. Install yarn v1 and node v16, and install the dependencies

    ```bash
    yarn
    ```

3. Start the local stack

    ```bash
    yarn devphase stack
    ```

4. Init the testnet (currently by [this script](https://github.com/shelvenzhou/phala-blockchain-setup))

    ```bash
    # edit .env file
    yarn
    node src/setup-logserver.js
    ```

To configure the local test stack, please check [devphase.config.ts](./devphase.config.ts).

## Run E2E test