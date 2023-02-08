import { ProjectConfigOptions } from 'devphase';

const config : ProjectConfigOptions = {
    stack: {
        blockTime: 500,
        version: 'nightly-2023-01-30',
        node: {
            port: 39944,
            binary: '{{directories.stacks}}/{{stack.version}}/phala-node',
            workingDir: '{{directories.stacks}}/.data/node',
            envs: {},
            args: {
                '--dev': true,
                '--port': 33333,
                '--ws-port': '{{stack.node.port}}',
                '--rpc-port': 39933,
                '--ws-external': true,
                '--unsafe-ws-external': true,
                '--rpc-methods': 'Unsafe',
                '--block-millisecs': '{{stack.blockTime}}',
            },
            timeout: 10000,
        },
        pruntime: {
            port: 38000, // server port
            binary: '{{directories.stacks}}/{{stack.version}}/pruntime',
            workingDir: '{{directories.stacks}}/.data/pruntime',
            envs: {
                'RUST_LOG': 'debug,runtime=trace'
            },
            args: {
                '--allow-cors': true,
                '--cores': 0,
                '--address': '0.0.0.0',
                '--port': '{{stack.pruntime.port}}',
            },
            timeout: 2000,
        },
        pherry: {
            gkMnemonic: '//Ferdie', // super user mnemonic
            binary: '{{directories.stacks}}/{{stack.version}}/pherry',
            workingDir: '{{directories.stacks}}/.data/pherry',
            envs: {},
            args: {
                '--no-wait': true,
                '--mnemonic': '{{stack.pherry.gkMnemonic}}',
                '--inject-key': '0000000000000000000000000000000000000000000000000000000000000001',
                '--substrate-ws-endpoint': 'ws://localhost:{{stack.node.port}}',
                '--pruntime-endpoint': 'http://localhost:{{stack.pruntime.port}}',
                '--dev-wait-block-ms': '{{stack.blockTime}}',
                '--attestation-provider': 'none',
            },
            timeout: 5000,
        }
    },
    /**
     * Networks configuration
     * Default network is local and it can be changed using CLI argument
     */
    networks: {
	local: {
	    nodeUrl: 'ws://localhost:{{stack.node.port}}',
	    workerUrl: 'http://localhost:{{stack.pruntime.port}}',
	    blockTime: 500,
	}
    },
    /**
     * Testing configuration
     */
     testing: {
        mocha: {}, // custom mocha configuration
        spawnStack: true, // spawn runtime stack? or assume there is running one
        stackLogOutput: true, // if specifed pipes output of all stack component to file (by default it is ignored)
        envSetup: { // environment setup
            setup: {
                // custom setup procedure callback; (devPhase) => Promise<void>
                custom: undefined,
                timeout: 120 * 1000,
            },
            teardown: {
                // custom teardown procedure callback ; (devPhase) => Promise<void>
                custom: undefined,
                timeout: 10 * 1000,
            }
        },
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
