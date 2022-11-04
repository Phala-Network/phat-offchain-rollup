import { SampleOracle } from '@/typings/SampleOracle';
import { EvmTransactor } from '@/typings/EvmTransactor';
import * as PhalaSdk from '@phala/sdk';
import type { KeyringPair } from '@polkadot/keyring/types';
import { Contract, ContractType } from 'devphase';

import 'dotenv/config';

async function delay(ms: number): Promise<void> {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

describe('Full Test', () => {
    const rpc = process.env.RPC;
    const anchorAddr = '0x' + process.env.ANCHOR_ADDR;
    const rollupKey = '0x' + process.env.PRIVKEY;

    let oracleFactory : SampleOracle.Factory;
    let oracle : SampleOracle.Contract;
    let evmTxFactory : EvmTransactor.Factory;
    let evmTx : EvmTransactor.Contract;

    let alice : KeyringPair;
    let certAlice : PhalaSdk.CertificateData;

    before(async function() {
        oracleFactory = await this.devPhase.getFactory(
            ContractType.InkCode,
            './artifacts/sample_oracle/sample_oracle.contract'
        );
        evmTxFactory = await this.devPhase.getFactory(
            ContractType.InkCode,
            './artifacts/evm_transactor/evm_transactor.contract'
        );
        
        await oracleFactory.deploy();
        await evmTxFactory.deploy();
        
        alice = this.devPhase.accounts.alice;
        certAlice = await PhalaSdk.signCertificate({
            api: this.api,
            pair: alice,
        });
        console.log('Signer:', alice.address.toString());
    });

    describe('the full stack', () => {
        before(async function() {
            this.timeout(30_000);

            // Deploy contract
            oracle = await oracleFactory.instantiate('default', []);
            evmTx = await evmTxFactory.instantiate('default', []);
            console.log('SampleOracle deployed at', oracle.address.toString());
            console.log('EvmTransactor deployed at', oracle.address.toString());

            // Check owner
            const oracleOwner = await oracle.query.owner(certAlice, {});
            expect(oracleOwner.result.isOk).to.be.true;
            expect(oracleOwner.output.toString()).to.be.equal(alice.address.toString());

            const evmTxOwner = await evmTx.query.owner(certAlice, {});
            expect(evmTxOwner.result.isOk).to.be.true;
            expect(evmTxOwner.output.toString()).to.be.equal(alice.address.toString());

            // Config the oracle
            const configOracle = await oracle.tx
                .config({}, rpc, anchorAddr)
                .signAndSend(alice, {nonce: -1});
            console.log('Oracle configured', configOracle.toHuman());

            const configEvmTx = await evmTx.tx
                .config({}, rpc, oracle.address, anchorAddr)
                .signAndSend(alice, {nonce: -1});
            console.log('EvmTransactor configured', configEvmTx.toHuman());
        });

        it('can run pure RollupHandler', async function() {
            this.timeout(1000*30_000);

            await delay(3*1000);
            const response = await oracle.query['rollupHandler::handleRollup'](certAlice, {});

            console.log('result', response.result.toHuman());
            if (response.output) {
                console.log('resp', response.output.toHuman());
            } else {
                await delay(1000*1000);
            }
            // expect(response.output.toHuman()).to.be.equal(false);

            // await delay(10000000);
        });

        it('can send evm transaction', async function() {
            this.timeout(1000*30_000);

            await delay(3*1000);
            const response = await evmTx.query.testPollWithKey(certAlice, {}, rollupKey);

            console.log('result', response.result.toHuman());
            if (response.output) {
                console.log('resp', response.output.toHuman());
            } else {
                await delay(1000*1000);
            }
            // expect(response.output.toHuman()).to.be.equal(false);

            // await delay(10000000);
        });
    });

});