import { defaultAbiCoder } from "@ethersproject/abi";
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { BigNumber, utils } from "ethers";
import { ethers } from "hardhat";

describe("SampleOracle", function () {
  async function deployFixture() {
    const [owner, submitter] = await ethers.getSigners();

    const Anchor = await ethers.getContractFactory("PhatQueuedAnchor");
    const TestOracle = await ethers.getContractFactory("TestOracle");
    const oracle = await TestOracle.deploy();
    const anchor = await Anchor.deploy(submitter.address, oracle.address, "0x71"); // Q

    // Set receiver as the owner of the anchor because the receiver will push requests.
    await expect(anchor.connect(owner).transferOwnership(oracle.address)).not.to.be.reverted;
    await expect(oracle.connect(owner).setQueuedAnchor(anchor.address)).not.to.be.reverted;

    return { anchor, oracle, owner, submitter };
  }

  describe("Oracle", function () {
    it("Can receive price", async function () {
        const { anchor, oracle, owner, submitter } = await loadFixture(deployFixture);

        // Send a request
        const reqTx = await oracle.connect(owner).request("btc/usdt");
        expect(reqTx).not.to.be.reverted;
        expect(reqTx).to.emit(anchor, 'RequestQueued');

        // Simulate a rollup to respond
        const btcPrice = BigNumber.from(10).pow(18).mul(19500);
        const rollupTx = await anchor.connect(submitter).rollupU256CondEq(
                // cond (global=1)
                ['0x00'],
                [uint(1)],
                // updates (global=2)
                ['0x00'],
                [uint(2)],
                // actions 
                [
                    // Callback: (rid: 0, price: 19500)
                    utils.hexConcat([
                      '0x01',
                      defaultAbiCoder.encode(['uint', 'uint256'], [0, btcPrice])
                    ]),
                    // Custom: queue processed to 1
                    utils.hexConcat(['0x02', '0x00', uint(1)]),
                ],
            )
        expect(rollupTx).not.to.be.reverted;
        expect(rollupTx).to
            .emit(anchor, 'RequestProcessedTo')
            .withArgs(1);
        expect(rollupTx).to
            .emit(oracle, 'PriceReceived')
            .withArgs(0, 'btc/usdt', btcPrice);
    })
  });
});

function uint(i: number): string {
  return defaultAbiCoder.encode(['uint'], [i])
}