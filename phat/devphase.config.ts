import { ProjectConfigOptions } from 'devphase';
import { join } from 'path';
import { spawn } from 'child_process';

console.log('PWD', )

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
        const init = spawn('bash', ['tmp/scripts/init-blockchain.sh'], { stdio: 'inherit' });
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
    stack: {
        node: {
            port: 9944,
            binary: rel('tmp/phala-dev-stack/bin/node'),
            workingDir: rel('tmp/phala-dev-stack/.data/node'),
            envs: {},
            args: {
                '--dev': true,
                '--ws-port': '{{stack.node.port}}',
                '--ws-external': true,
                '--unsafe-ws-external': true,
                '--rpc-methods': 'Unsafe',
                '--block-millisecs': 1000,
            },
            timeout: 10000,
        },
        pruntime: {
            port: 8000, // server port
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
            suMnemonic: '//Ferdie', // super user mnemonic
            binary: rel('tmp/phala-dev-stack/bin/pherry'),
            workingDir: rel('tmp/phala-dev-stack/.data/pherry'),
            envs: {},
            args: {
                '--no-wait': true,
                '--mnemonic': '{{stack.pherry.suMnemonic}}',
                '--inject-key': '0000000000000000000000000000000000000000000000000000000000000001',
                '--substrate-ws-endpoint': 'ws://localhost:{{stack.node.port}}',
                '--pruntime-endpoint': 'http://localhost:{{stack.pruntime.port}}',
                '--dev-wait-block-ms': 1000,
                '--attestation-provider': 'none',
            },
            timeout: 2000,
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
        customEnvSetup: initChain,
    },
    /**
     * Custom mocha configuration
     */
    mocha: {}
};

export default config;