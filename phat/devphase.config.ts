import { ConfigOption } from 'devphase';
import { join } from 'path';

console.log('PWD', )

function rel(p: string): string {
    return join(process.cwd(), p);
}

const config : ConfigOption = {
    stack: {
        node: {
            binary: rel('tmp/phala-dev-stack/bin/node'),
            workingDir: rel('tmp/phala-dev-stack/.data/node'),
            envs: {},
            args: {
                '--dev': true,
                '--ws-external': true,
                '--unsafe-ws-external': true,
                '--rpc-methods': 'Unsafe',
                '--block-millisecs': 2000,
            },
            timeout: 10000,
        },
        pruntime: {
            binary: rel('tmp/phala-dev-stack/bin/pruntime'),
            workingDir: rel('tmp/phala-dev-stack/.data/pruntime'),
            envs: {},
            args: {
                '--allow-cors': true,
                '--cores': 0,
                '--address': '0.0.0.0',
                '--port': 8000,
            },
            timeout: 2000,
        },
        pherry: {
            binary: rel('tmp/phala-dev-stack/bin/pherry'),
            workingDir: rel('tmp/phala-dev-stack/.data/pherry'),
            envs: {},
            args: {
                '--no-wait': true,
                '--mnemonic': '//Alice',
                '--inject-key': '0000000000000000000000000000000000000000000000000000000000000001',
                '--substrate-ws-endpoint': 'ws://localhost:9944',
                '--pruntime-endpoint': 'http://localhost:8000',
                '--dev-wait-block-ms': 1000,
                '--attestation-provider': 'none',
            },
            timeout: 2000,
        }
    },
    /**
     * Custom mocha configuration
     */
    mocha: {}
};

export default config;