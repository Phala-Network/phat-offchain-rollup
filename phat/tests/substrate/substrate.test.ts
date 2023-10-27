import { SubPriceFeed } from '@/typings/SubPriceFeed';
import { Sub0Factory } from '@/typings/Sub0Factory'
import * as PhalaSdk from '@phala/sdk';
import { ApiPromise } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';
import { ContractType } from '@devphase/service';

import 'dotenv/config';

async function delay(ms: number): Promise<void> {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

describe('Substrate Offchain Rollup', () => {
    const secretBob: string = "0x398f0c28f98885e046333d4a41c19cee4c37368a9832c6502f6cfd182e2aef89";
    const defaultDelay: number = 10_000;
    const defaultTimeout: number = 120_000;

    let priceFeedFactory: SubPriceFeed.Factory;
    let priceFeed: SubPriceFeed.Contract;
    let priceFeedCodeHash: string;
    let sub0Factory: Sub0Factory.Factory;
    let sub0: Sub0Factory.Contract;

    let api: ApiPromise;
    let httpRpc: string;
    let alice : KeyringPair;
    let certAlice : PhalaSdk.CertificateData;
    const txConf = { gasLimit: "10000000000000", storageDepositLimit: null };

    before(async function() {
        httpRpc = this.devPhase.networkConfig.nodeUrl;
        priceFeedFactory = await this.devPhase.getFactory('sub_price_feed', {
            contractType: ContractType.InkCode,
        });
        sub0Factory = await this.devPhase.getFactory('sub0_factory', {
            contractType: ContractType.InkCode,
        });
        priceFeedCodeHash = priceFeedFactory.metadata.source.hash;

        await priceFeedFactory.deploy();
        await sub0Factory.deploy();
        expect(priceFeedCodeHash.startsWith('0x')).to.be.true;
        
        api = this.api;
        alice = this.devPhase.accounts.alice;
        certAlice = await PhalaSdk.signCertificate({ pair: alice });
        console.log('Signer:', alice.address.toString());
        console.log('PriceFeed code:', priceFeedCodeHash)
    });

    describe('SubPriceFeed', () => {
        before(async function() {
            this.timeout(defaultTimeout);

            // Deploy contract
            priceFeed = await priceFeedFactory.instantiate('default', [], {transferToCluster: 1e12});
            console.log('SubPriceFeed deployed at', priceFeed.address.toString());
        });

        it('should have correct owners', async function() {
            const feedOwner = await priceFeed.query.owner({ cert: certAlice }, {});
            expect(feedOwner.result.isOk).to.be.true;
            expect(feedOwner.output.asOk.toString()).to.be.equal(alice.address.toString());
        });

        it('can be configured', async function() {
            this.timeout(defaultTimeout);

            // Config the oracle
            const feedConfig = await priceFeed.tx
                .config(txConf, httpRpc, 100, secretBob as any, 'polkadot', 'usd')
                .signAndSend(alice, {nonce: -1});
            console.log('Feed configured', feedConfig.toHuman());
            await delay(defaultDelay);

            // Init the rollup on the blockchain
            const init = await priceFeed.query.maybeInitRollup({ cert: certAlice }, {});
            console.log('Result: ', init.result.toHuman())
            console.log('Output: ', init.output.toHuman())
            expect(init.result.isOk).to.be.true;
            expect(init.output.isOk).to.be.true;
            expect(init.output.asOk.isOk).to.be.true;
        });

        it('can submit tx', async function() {
            this.timeout(defaultTimeout);

            const feed = await priceFeed.query.feedPrice({ cert: certAlice }, {});
            expect(feed.result.isOk).to.be.true;
            expect(feed.output.isOk).to.be.true;
            expect(feed.output.asOk.isOk).to.be.true;
            await delay(defaultDelay);

            // The response should be received on the blockchain
            const receivedPrice = await api.query.phatOracle.priceFeeds.entries(alice.address);
            expect(receivedPrice.length).to.be.equal(1);
        });
    });

    describe('Sub0Factory', () => {
        before(async function() {
            this.timeout(defaultTimeout);

            // Deploy contract
            sub0 = await sub0Factory.instantiate('default', [], {transferToCluster: 1e12});
            console.log('Sub0Factory deployed at', sub0.address.toString());
        });

        it('should have correct owners', async function() {
            const sub0Owner = await sub0.query.owner({ cert: certAlice }, {});
            expect(sub0Owner.result.isOk).to.be.true;
            expect(sub0Owner.output.asOk.toString()).to.be.equal(alice.address.toString());
        });

        it('can be configured', async function() {
            this.timeout(defaultTimeout);

            // Config the oracle
            const sub0Config = await sub0.tx
                .config(txConf, httpRpc, 100, secretBob as any, priceFeedCodeHash)
                .signAndSend(alice, {nonce: -1});
            console.log('Sub0Factory configured', sub0Config.toHuman());
            await delay(4*1000);

            const config = await sub0.query.getConfig({ cert: certAlice }, {})
            expect(config.result.isOk).to.be.true;
            expect(config.output.isOk).to.be.true;
            expect(config.output.asOk.asOk.length).to.be.equal(2);
        });

        let priceFeed1: SubPriceFeed.Contract;
        it('can deploy price feeds', async function() {
            this.timeout(defaultTimeout);

            let deploy = await api.tx.utility.batchAll([
                sub0.tx.deployPriceFeed(txConf, 'feed1', 'polkadot', 'usd'),
                sub0.tx.deployPriceFeed(txConf, 'feed2', 'bitcoin', 'usd'),
            ]).signAndSend(alice, {nonce: -1});

            console.log('PriceFeed1&2 deployed', deploy.toHuman());
            await delay(defaultDelay);

            let deployments = await sub0.query.getDeployments({ cert: certAlice }, {});
            expect(deployments.result.isOk).to.be.true;
            expect(deployments.output.asOk.asOk.length).to.be.equal(2);

            // Get the address in hex, and attach to it.
            //
            // Note that `contractId.toString()` returns and SS58 encoded address by default, but
            // Polkadot.js cannot parse it to H256.
            let feed1Addr = deployments.output.asOk.asOk[1].contractId.toHex();
            priceFeed1 = await priceFeedFactory.attach(feed1Addr);
        });

        it('can trigger a rollup', async function() {
            this.timeout(defaultTimeout);

            // Init the rollup on the blockchain
            const init = await priceFeed1.query.maybeInitRollup({ cert: certAlice }, {});
            expect(init.result.isOk).to.be.true;
            expect(init.output.isOk).to.be.true;
            expect(init.output.asOk.isOk).to.be.true;
            await delay(defaultDelay);

            // Trigger a rollup
            const feed = await priceFeed1.query.feedPrice({ cert: certAlice }, {});
            expect(feed.result.isOk).to.be.true;
            expect(feed.output.isOk).to.be.true;
            expect(feed.output.asOk.isOk).to.be.true;
            await delay(defaultDelay);

            // The response should be received on the blockchain
            const receivedPrice = await api.query.phatOracle.priceFeeds.entries(alice.address);
            expect(receivedPrice.length).to.be.equal(2);  // 2 in totoal: 1 existing & 1 more
        });

    });

    // // To keep the blockchain running after the test, remove the "skip" in the following test
    // after('keeps running', async function() {
    //     this.timeout(1000 * 30_000);
    //     await delay(1000 * 30_000);
    // });
});
