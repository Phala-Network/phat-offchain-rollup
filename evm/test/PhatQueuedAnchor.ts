import { defaultAbiCoder } from "@ethersproject/abi";
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { utils } from "ethers";
import { ethers } from "hardhat";

describe("QueuedAnchor", function () {
  async function deployFixture() {
    const [owner, submitter] = await ethers.getSigners();

    const Anchor = await ethers.getContractFactory("PhatQueuedAnchor");
    const TestReceiver = await ethers.getContractFactory("TestReceiver");
    const receiver = await TestReceiver.deploy();
    const anchor = await Anchor.deploy(submitter.address, receiver.address, "0x71"); // Q

    return { anchor, receiver, owner, submitter };
  }

  describe("Rollup", function () {
    it("Can process requests", async function () {
        const { anchor, receiver, owner, submitter } = await loadFixture(deployFixture);
        const pushTx = await anchor.connect(owner).pushRequest('0xdecaffee');
        expect(pushTx).not.to.be.reverted;
        expect(pushTx).to
            .emit(anchor, 'RequestQueued')
            .withArgs(0, '0xdecaffee');
        // Simulate a rollup
        const rollupTx = await anchor.connect(submitter).rollupU256CondEq(
                // cond (global=1)
                ['0x00'],
                [uint(1)],
                // updates (global=2)
                ['0x00'],
                [uint(2)],
                // actions 
                [
                    // Callback: req 00 responded with 0xDEADBEEF
                    '0x01' + '0000000000000000000000000000000000000000000000000000000000000000' + 'DEADBEEF',
                    // Custom: queue processed to 1
                    utils.hexConcat(['0x02', '0x00', defaultAbiCoder.encode(['uint'], [1])]),
                ],
            )
        expect(rollupTx).not.to.be.reverted;
        expect(rollupTx).to
            .emit(anchor, 'RequestProcessedTo')
            .withArgs(1);

        // Check queue processed
        expect(await receiver.getRecvLength()).to.be.equals('1');
        expect(await receiver.getRecv(0)).to.be.eql([anchor.address, '0x0000000000000000000000000000000000000000000000000000000000000000deadbeef']);
        // end
        expect(await anchor.getUint(hex('end'))).to.be.equals(1);
        // start
        expect(await anchor.getUint(hex('start'))).to.be.equals(1);
    })
  });
});

function uint(i: number): string {
  return defaultAbiCoder.encode(['uint'], [i])
}

function hex(str: string): string {
    return utils.hexlify(utils.toUtf8Bytes(str));
}
