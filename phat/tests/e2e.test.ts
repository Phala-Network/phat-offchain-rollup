import { SampleOracle } from '@/typings/SampleOracle';
import { EvmTransactor } from '@/typings/EvmTransactor';
import * as PhalaSdk from '@phala/sdk';
import type { KeyringPair } from '@polkadot/keyring/types';
import { Contract, ContractType } from 'devphase';

import 'dotenv/config';
import { LocalScheduler } from '@/typings/LocalScheduler';

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
    let schedulerFactory : LocalScheduler.Factory;
    let scheduler : LocalScheduler.Contract;

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
        schedulerFactory = await this.devPhase.getFactory(
            ContractType.InkCode,
            './artifacts/local_scheduler/local_scheduler.contract'
        );
        
        await oracleFactory.deploy();
        await evmTxFactory.deploy();
        await schedulerFactory.deploy();
        
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
            scheduler = await schedulerFactory.instantiate('default', []);
            console.log('SampleOracle deployed at', oracle.address.toString());
            console.log('EvmTransactor deployed at', evmTx.address.toString());
            console.log('LocalScheduler deployed at', scheduler.address.toString());

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

        it.skip('can send evm transaction', async function() {
            this.timeout(1000*30_000);

            await delay(3*1000);
            const response = await evmTx.query.testPollWithKey(certAlice, {}, rollupKey as any);

            console.log('result', response.result.toHuman());
            if (response.output) {
                console.log('resp', response.output.toHuman());
            } else {
                await delay(1000*1000);
            }
            // expect(response.output.toHuman()).to.be.equal(false);

            // await delay(10000000);
        });

        it('can run via scheduler', async function() {
            this.timeout(180_000);

            // addJob(EvmTransactor.poll())
            await scheduler.tx.addJob({}, 'job1', '* * * * *', evmTx.address, '0x1e44dfc6' as any)
                .signAndSend(alice, {nonce: -1});
            await delay(6000);

            const resJobs = await scheduler.query.getNumJobs(certAlice, {})
            expect(resJobs.result.isOk).to.be.true;
            expect(resJobs.output.toNumber()).to.be.equal(1);

            // The job is not scheduled before the first `poll()`
            const resSchedule1 = await scheduler.query.getJobSchedule(certAlice, {}, 0);
            expect(resSchedule1.output.isNone).to.be.true;

            // Poll it
            const resPoll1 = await scheduler.query.poll(certAlice, {});
            expect(resPoll1.result.isOk).to.be.true;
            // Then we get the schedule
            const resSchedule2 = await scheduler.query.getJobSchedule(certAlice, {}, 0);
            expect(resSchedule2.result.isOk).to.be.true;
            expect(resSchedule2.output.isSome).to.be.true;
            // Schedule details
            const [nextScheudled, job0] = resSchedule2.output.unwrap()
            expect(nextScheudled.toNumber()).to.be.greaterThan(0);
            expect(job0.toHuman()).to.deep.equal({
                name: 'job1',
                cronExpr: '* * * * *',
                target: evmTx.address.toString(),
                call: '0x1e44dfc6',
                enabled: true,
            });

            // Wait until triggered
            const scheduled = nextScheudled.toNumber();
            const now = Date.now();
            if (scheduled > now + 100) {
                await delay(scheduled - now + 100);
            }

            // Poll again, and trigger the scheduled action
            const resPoll2 = await scheduler.query.poll(certAlice, {});
            expect(resPoll2.result.isOk).to.be.true;
            console.log('poll2 result:', resPoll2.result.toHuman());

            // Schedule should be updated
            const resSchedule3 = await scheduler.query.getJobSchedule(certAlice, {}, 0);
            expect(resSchedule3.result.isOk).to.be.true;
            expect(resSchedule3.output.isSome).to.be.true;
            // Schedule details
            const [nextScheudled3, _job0] = resSchedule3.output.unwrap();
            expect(nextScheudled3.toNumber()).to.be.greaterThan(scheduled);
        });
    });

});