// Usage:
// ADDR=<your-evm-queued-anchor-address> npx hardhat run scripts/check-anchor.ts --network goerli

import { ethers } from "hardhat";

async function main() {
  const Anchor = await ethers.getContractFactory("PhatQueuedAnchor");
//   const TestOracle = await ethers.getContractFactory("TestOracle");
//   const [deployer] = await ethers.getSigners();

  const anchorAddr = process.env.ADDR ?? '?';
  console.log(anchorAddr, process.argv);

  const anchor = Anchor.attach(anchorAddr);
  const start = await anchor.getUint(hex('start'));
  const end = await anchor.getUint(hex('end'));

  console.log({start, end});
}

function hex(str: string): string {
    return ethers.utils.hexlify(ethers.utils.toUtf8Bytes(str));
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
