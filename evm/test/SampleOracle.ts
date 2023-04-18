import { defaultAbiCoder } from "@ethersproject/abi";
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { BigNumber, utils } from "ethers";
import { ethers } from "hardhat";

const TYPE_RESPONSE = 0;
const TYPE_FEED = 1;
const TYPE_ERROR = 2;

describe("SampleOracle", function () {
  async function deployFixture() {
    const [owner, attestor] = await ethers.getSigners();

    const TestOracle = await ethers.getContractFactory("TestOracle");
    const oracle = await TestOracle.deploy(attestor.address);

    return { oracle, owner, attestor };
  }

  describe("Oracle", function () {
    it("Can receive price", async function () {
      const { oracle, owner, attestor } = await loadFixture(deployFixture);

      // Send a request
      const reqTx = await oracle.connect(owner).request("btc/usdt");
      await expect(reqTx).not.to.be.reverted;
      await expect(reqTx).to.emit(oracle, 'MessageQueued');

      // Simulate a rollup to respond
      const btcPrice = BigNumber.from(10).pow(18).mul(19500);
      const rollupTx = await oracle.connect(attestor).rollupU256CondEq(
              // cond (global=0)
              ['0x00'], [uint(0)],
              // updates (global=1)
              ['0x00'], [uint(1)],
              // actions 
              [
                  // Callback: (RESPONSE, rid: 0, price: 19500)
                  utils.hexConcat([
                    '0x00',
                    defaultAbiCoder.encode(['uint', 'uint', 'uint256'], [TYPE_RESPONSE, 1, btcPrice])
                  ]),
                  // Queue processed to 1
                  utils.hexConcat(['0x01', uint(1)]),
              ],
          )
      await expect(rollupTx).not.to.be.reverted;
      await expect(rollupTx).to
          .emit(oracle, 'MessageProcessedTo')
          .withArgs(1);
      await expect(rollupTx).to
          .emit(oracle, 'PriceReceived')
          .withArgs(1, 'btc/usdt', btcPrice);
    })

    it("Can receive feed", async function () {
      const { oracle, owner, attestor } = await loadFixture(deployFixture);

      const reqTx = await oracle.connect(owner).registerFeed(0, "btc/usdt");
      await expect(reqTx).not.to.be.reverted;

      // Simulate a rollup to respond
      const btcPrice = BigNumber.from(10).pow(18).mul(19510);
      const rollupTx = await oracle.connect(attestor).rollupU256CondEq(
              // cond (global=0)
              ['0x00'], [uint(0)],
              // updates (global=1)
              ['0x00'], [uint(1)],
              // actions 
              [
                  // Callback: (FEED, feedId: 0, price: 19510)
                  utils.hexConcat([
                    '0x00',
                    defaultAbiCoder.encode(['uint', 'uint', 'uint256'], [TYPE_FEED, 0, btcPrice])
                  ]),
              ],
          )
      await expect(rollupTx).not.to.be.reverted;
      await expect(rollupTx).to
          .emit(oracle, 'FeedReceived')
          .withArgs(0, 'btc/usdt', btcPrice);
    })

    it("Can process error", async function () {
      const { oracle, owner, attestor } = await loadFixture(deployFixture);

      const reqTx = await oracle.connect(owner).request("btc/usdt");
      await expect(reqTx).not.to.be.reverted;
      await expect(reqTx).to.emit(oracle, 'MessageQueued');

      // Simulate a rollup to respond
      const rollupTx = await oracle.connect(attestor).rollupU256CondEq(
              // cond (global=0)
              ['0x00'], [uint(0)],
              // updates (global=1)
              ['0x00'], [uint(1)],
              // actions 
              [
                  // Callback: (ERROR, rid: 0, error: 19510)
                  utils.hexConcat([
                    '0x00',
                    defaultAbiCoder.encode(['uint', 'uint', 'uint256'], [TYPE_ERROR, 1, 256])
                  ]),
              ],
          )
      await expect(rollupTx).not.to.be.reverted;
      await expect(rollupTx).to
          .emit(oracle, 'ErrorReceived')
          .withArgs(1, 'btc/usdt', 256);
    })

  });
});

function uint(i: number): string {
  return defaultAbiCoder.encode(['uint32'], [i])
}