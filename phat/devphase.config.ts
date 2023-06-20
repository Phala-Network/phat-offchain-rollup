import { ProjectConfigOptions, StackSetupMode } from '@devphase/service';

const config : ProjectConfigOptions = {
    /*
     * Project directories
     */
    directories: {
        artifacts: 'artifacts',
        contracts: 'contracts',
        logs: 'logs',
        scripts: 'scripts',
        stacks: 'stacks',
        tests: 'tests',
        typings: 'typings'
    },
    /*
     * Stack configuration
     * {
     *     [componentName : string]: {
     *          binary: string, // path to binary
     *          workingDir: string, // working directory as above
     *          evns: {
     *              [name: string]: string,
     *          },
     *          args: {
     *              [name: string]: string,
     *          },
     *          timeout: number // start up timeout
     *     }
     * }
     */
    stack: {
        version: 'latest', // version which you want to pull from official repository (tag name) or "latest" one
        blockTime: 6000, // default block time for direct stack running (may be overriden in testing mode)
        setupOptions: {
            mode: StackSetupMode.None,
            workerUrl: 'http://localhost:{{stack.pruntime.port}}'
        },
        node: {
            port: 9944, // ws port
            binary: '{{directories.stacks}}/{{stack.version}}/phala-node',
            workingDir: '{{directories.stacks}}/.data/node',
            dataDir: '{{directories.stacks}}/.data/node',
            envs: {},
            args: {
                '--dev': true,
                '--rpc-methods': 'Unsafe',
                '--ws-port': '{{stack.node.port}}',
                '--block-millisecs': '{{stack.blockTime}}', // override at runtime
            },
            timeout: 10000,
        },
        pruntime: {
            port: 8000, // server port
            binary: '{{directories.stacks}}/{{stack.version}}/pruntime',
            workingDir: '{{directories.stacks}}/.data/pruntime',
            dataDir: '{{directories.stacks}}/.data/pruntime',
            envs: {},
            args: {
                '--allow-cors': true,
                '--cores': 0,
                '--port': '{{stack.pruntime.port}}'
            },
            timeout: 2000,
        },
        pherry: {
            gkMnemonic: '//Alice', // gate keeper mnemonic
            binary: '{{directories.stacks}}/{{stack.version}}/pherry',
            workingDir: '{{directories.stacks}}/.data/pherry',
            dataDir: '{{directories.stacks}}/.data/pherry',
            envs: {},
            args: {
                '--no-wait': true,
                '--mnemonic': '{{stack.pherry.gkMnemonic}}',
                '--inject-key': '0000000000000000000000000000000000000000000000000000000000000001',
                '--substrate-ws-endpoint': 'ws://localhost:{{stack.node.port}}',
                '--pruntime-endpoint': 'http://localhost:{{stack.pruntime.port}}',
                '--dev-wait-block-ms': '{{stack.blockTime}}', // override at runtime
            },
            timeout: 2000,
        }
    },
    /**
     * Testing configuration
     */
    testing: {
        mocha: {}, // custom mocha configuration
        blockTime: 100, // block time override for spawning local testnet
        stackSetupConfig: { // environment setup
            setup: {
                custom: undefined, // custom setup procedure callback; (devPhase) => Promise<void>
                timeout: 120 * 1000,
            },
            teardown: {
                custom: undefined, // custom teardown procedure callback ; (devPhase) => Promise<void>
                timeout: 10 * 1000,
            }
        },
        stackLogOutput: false, // display stack output in console
    },
    /**
     * Networks configuration
     * Default network is local and it can be changed using CLI argument
     */
    networks: {
        local: {
            nodeUrl: 'ws://localhost:{{stack.node.port}}',
            nodeApiOptions: {},
            workerUrl: 'http://localhost:{{stack.pruntime.port}}',
            defaultClusterId: '0x0000000000000000000000000000000000000000000000000000000000000000', // set default cluster ID for further actions
            blockTime: 6000, // network block time (may be overriden in testing mode)
        },
        poc5: {
            nodeUrl: 'wss://poc5.phala.network/ws',
            workerUrl: 'https://poc5.phala.network/tee-api-1',
            defaultClusterId: '0x0000000000000000000000000000000000000000000000000000000000000001',
        }
    },
    /**
     * Accounts fallback configuration
     * It is overriden by values saved in ./accounts.json
     */
    accountsConfig: {
        keyrings: {
            alice: '//Alice', // string (in case of mnemonic) or account keyring JSON
            bob: '//Bob',
            charlie: '//Charlie',
            dave: '//Dave',
            eve: '//Eve',
            ferdie: '//Ferdie'
        },
        suAccount: 'alice'
    }
};

export default config;