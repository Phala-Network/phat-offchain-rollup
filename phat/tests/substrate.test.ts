import { SubPriceFeed } from '@/typings/SubPriceFeed';
import * as PhalaSdk from '@phala/sdk';
import { ApiPromise } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';
import { ContractType } from 'devphase';

import 'dotenv/config';
import { LocalScheduler } from '@/typings/LocalScheduler';

async function delay(ms: number): Promise<void> {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

describe('Full Test', () => {
    const httpRpc: string = "http://127.0.0.1:39933";
    const secretBob: string = "0x398f0c28f98885e046333d4a41c19cee4c37368a9832c6502f6cfd182e2aef89";

    let priceFeedFactory : SubPriceFeed.Factory;
    let priceFeed : SubPriceFeed.Contract;

    let api: ApiPromise;
    let alice : KeyringPair;
    let certAlice : PhalaSdk.CertificateData;
    const txConf = { gasLimit: "10000000000000", storageDepositLimit: null };

    before(async function() {
        priceFeedFactory = await this.devPhase.getFactory(
            ContractType.InkCode,
            './artifacts/sub_price_feed/sub_price_feed.contract'
        );
        await priceFeedFactory.deploy();
        
        api = this.api;
        alice = this.devPhase.accounts.alice;
        certAlice = await PhalaSdk.signCertificate({
            api,
            pair: alice,
        });
        console.log('Signer:', alice.address.toString());
    });

    describe('SubPriceFeed', () => {
        before(async function() {
            this.timeout(30_000);
            // Deploy contract
            priceFeed = await priceFeedFactory.instantiate('default', [], {transferToCluster: 1e12});
            console.log('SubPriceFeed deployed at', priceFeed.address.toString());
        });

        it('should has correct owners', async function() {
            const feedOwner = await priceFeed.query.owner(certAlice, {});
            expect(feedOwner.result.isOk).to.be.true;
            expect(feedOwner.output.toString()).to.be.equal(alice.address.toString());
        });

        it('should be configurable', async function() {
            // Config the oracle
            const feedConfig = await priceFeed.tx
                .config(txConf, httpRpc, 100, secretBob as any, 'polkadot', 'usd')
                .signAndSend(alice, {nonce: -1});
            console.log('Feed configured', feedConfig.toHuman());
            await delay(3*1000);

            // Init the rollup on the blockchain
            const init = await priceFeed.query.maybeInitRollup(certAlice, {});
            expect(init.result.isOk).to.be.true;
            expect(init.output.isOk).to.be.true;
            expect(init.output.asOk.isSome).to.be.true;
        });

        it('can submit tx', async function() {
            this.timeout(1000*30_000);

            const feed = await priceFeed.query.feedPrice(certAlice, {});
            expect(feed.result.isOk).to.be.true;
            expect(feed.output.isOk).to.be.true;
            expect(feed.output.asOk.isSome).to.be.true;
            await delay(3*1000);

            // The response should be received on the blockchain
            const receivedPrice = await api.query.phatOracle.priceFeeds.entries(alice.address);
            expect(receivedPrice.length).to.be.equal(1);

            // To keep the blockchain running for a longer time, uncomment the following
            // await delay(3000*1000);
        });

    });

});