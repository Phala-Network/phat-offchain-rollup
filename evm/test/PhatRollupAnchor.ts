import { time, loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import { assert } from "console";

describe("RollupAnchor", function () {
  async function deployFixture() {
    // Contracts are deployed using the first signer/account by default
    const [owner, otherAccount] = await ethers.getSigners();

    const Anchor = await ethers.getContractFactory("PhatRollupAnchor");
    const TestReceiver = await ethers.getContractFactory("TestReceiver");
    const receiver = await TestReceiver.deploy();
    const anchor = await Anchor.deploy(otherAccount.address, receiver.address);

    return { anchor, receiver, owner, otherAccount };
  }

  // describe("Deployment", function () {
  //   it("Should set the right unlockTime", async function () {
  //     const { anchor, receiver } = await loadFixture(deployFixture);
  //   });

  //   it("Should get the constant calcuated", async function () {
  //     const { anchor } = await loadFixture(deployFixture);
  //     expect(await anchor.genReceiverSelector()).to.equal("0x43a53d89");
  //   })

  //   it("Should convert bytes to uint256", async function () {
  //     const { anchor } = await loadFixture(deployFixture);
  //     expect(await anchor.testConvert('0x')).to.equal('0');
  //     expect(await anchor.testConvert('0x0000000000000000000000000000000000000000000000000000000000000000')).to.equal('0');
  //     expect(await anchor.testConvert('0x0000000000000000000000000000000000000000000000000000000000000001')).to.equal('1');
  //   });
  // });

  describe("Rollup", function () {
    it("Should not forward from random sender", async function () {
      const { anchor, owner } = await loadFixture(deployFixture);
      await expect(
        anchor.connect(owner).rollupU256CondEq(
          // cond
          [], [],
          // updates
          [], [],
          // actions
          ['0xDEADBEEF'],
        )
      ).to.be.revertedWith('bad caller');
    });

    it("Should not allow invalid input arrays", async function () {
      const { anchor, otherAccount } = await loadFixture(deployFixture);

      await expect(
        anchor.connect(otherAccount).rollupU256CondEq(
          // cond
          ['0x01'], [],
          // updates
          [], [],
          // actions
          ['0xDEADBEEF'],
        )
      ).to.be.revertedWith('bad cond len');

      await expect(
        anchor.connect(otherAccount).rollupU256CondEq(
          // cond
          [], [],
          // updates
          ['0x'], [],
          // actions
          ['0xDEADBEEF'],
        )
      ).to.be.revertedWith('bad update len');
    });

    it("Should forward actions", async function () {
      const { anchor, receiver, otherAccount } = await loadFixture(deployFixture);

      await expect(
        anchor.connect(otherAccount).rollupU256CondEq(
          // cond
          ['0x01'],
          ['0x0000000000000000000000000000000000000000000000000000000000000000'],
          // updates
          ['0x01'],
          ['0x0000000000000000000000000000000000000000000000000000000000000001'],
          // actions
          ['0xDEADBEEF'],
        )
      ).not.to.be.reverted;

      expect(await receiver.getRecvLength()).to.be.equals('1');
      expect(await receiver.getRecv(0)).to.be.eql([anchor.address, '0xdeadbeef']);
      expect(await anchor.getStorage('0x01')).to.be.equals('0x0000000000000000000000000000000000000000000000000000000000000001');
    });
  });

  describe("OptimisticLock", function () {
    it("Should reject conflicting transaction", async function () {
      const { anchor, receiver, otherAccount } = await loadFixture(deployFixture);
      // Rollup from v0 to v1
      await expect(
        anchor.connect(otherAccount).rollupU256CondEq(
          // cond
          ['0x01'],
          ['0x0000000000000000000000000000000000000000000000000000000000000000'],
          // updates
          ['0x01'],
          ['0x0000000000000000000000000000000000000000000000000000000000000001'],
          // actions
          ['0xDEADBEEF'],
        )
      ).not.to.be.reverted;
      expect(await anchor.getStorage('0x01')).to.be.equals('0x0000000000000000000000000000000000000000000000000000000000000001');
      // Rollup to v1 again
      await expect(
        anchor.connect(otherAccount).rollupU256CondEq(
          // cond
          ['0x01'],
          ['0x0000000000000000000000000000000000000000000000000000000000000000'],
          // updates
          ['0x01'],
          ['0x0000000000000000000000000000000000000000000000000000000000000001'],
          // actions
          ['0xDEADBEEF'],
        )
      ).to.be.revertedWith('cond not met');
    });
  });
});
