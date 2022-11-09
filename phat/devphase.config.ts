import { ProjectConfigOptions } from 'devphase';
import { join } from 'path';
import { spawn } from 'child_process';

function rel(p: string): string {
    return join(process.cwd(), p);
}

async function initChain(devphase: any): Promise<void> {
    console.log('######################## Initializing blockchain ########################');
    // Necessary to run; copied from devphase `defaultSetupenv()`
    devphase.mainClusterId = devphase.options.clusterId;
    await devphase.prepareWorker(devphase.options.workerUrl);
    // Run our custom init script
    return new Promise((resolve) => {
        const init = spawn(
            'bash',
            ['tmp/scripts/init-blockchain.sh', devphase.options.nodeUrl, devphase.options.workerUrl],
            { stdio: 'inherit' }
        );
        // function onData(data: Buffer) {
        //     console.log('[INIT]', data.toString());
        // }
        // init.stdout.on('data', onData);
        // init.stderr.on('data', onData);
        init.on('exit', code => {
            console.log('initChain script exited with code', code);
            resolve();
        });
    })
}

const config : ProjectConfigOptions = {
    directories: {
        logs: './tmp/phala-dev-stack/logs'
    },
    stack: {
        node: {
            port: 39944,
            binary: rel('tmp/phala-dev-stack/bin/node'),
            workingDir: rel('tmp/phala-dev-stack/.data/node'),
            envs: {},
            args: {
                '--dev': true,
                '--port': 33333,
                '--ws-port': '{{stack.node.port}}',
                '--ws-external': true,
                '--unsafe-ws-external': true,
                '--rpc-methods': 'Unsafe',
                '--block-millisecs': 1000,
            },
            timeout: 10000,
        },
        pruntime: {
            port: 38000, // server port
            binary: rel('tmp/phala-dev-stack/bin/pruntime'),
            workingDir: rel('tmp/phala-dev-stack/.data/pruntime'),
            envs: {
                'RUST_LOG': 'info,runtime=trace'
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
            binary: rel('tmp/phala-dev-stack/bin/pherry'),
            workingDir: rel('tmp/phala-dev-stack/.data/pherry'),
            envs: {},
            args: {
                '--no-wait': true,
                '--mnemonic': '{{stack.pherry.gkMnemonic}}',
                '--inject-key': '0000000000000000000000000000000000000000000000000000000000000001',
                '--substrate-ws-endpoint': 'ws://localhost:{{stack.node.port}}',
                '--pruntime-endpoint': 'http://localhost:{{stack.pruntime.port}}',
                '--dev-wait-block-ms': 1000,
                '--attestation-provider': 'none',
            },
            timeout: 5000,
        }
    },
    /**
     * Configuration options of DevPhase instance used in testing
     */
    devPhaseOptions: {
        nodeUrl: 'ws://localhost:{{stack.node.port}}',
        workerUrl: 'http://localhost:{{stack.pruntime.port}}',
        accountsMnemonic: '', // default account
        accountsPaths: {
            alice: '//Alice',
            bob: '//Bob',
            charlie: '//Charlie',
            dave: '//Dave',
            eve: '//Eve',
            ferdie: '//Ferdie',
        },
        sudoAccount: 'alice',
        ss58Prefix: 30,
        clusterId: '0x0000000000000000000000000000000000000000000000000000000000000000',
    },
    /**
     * Testing configuration
     */
     testing: {
        mocha: {}, // custom mocha configuration
        envSetup: { // environment setup
            setup: {
                custom: initChain, // custom setup procedure callback; (devPhase) => Promise<void>
                timeout: 60 * 1000,
            },
            teardown: {
                custom: undefined, // custom teardown procedure callback ; (devPhase) => Promise<void>
                timeout: 10 * 1000,
            }
        },
        blockTime: 500, // overrides block time specified in node (and pherry) component
        stackLogOutput : true, // if specifed pipes output of all stack component to file (by default it is ignored)
    },
};

export default config;