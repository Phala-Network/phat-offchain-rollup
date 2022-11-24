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
  const lockKey = await anchor.getLockKey();
  const prefix = await anchor.getPrefix();
  const start = await anchor.getUint(hex('start'));
  const end = await anchor.getUint(hex('end'));
  const globalLock = await anchor.getStorage('0x00');
  // const rawStart = await anchor.getStorage('0x71000000000000000000000000000000000000000000000000000000000000007374617274');
  // const rawEnd = await anchor.getStorage('0x7100000000000000000000000000000000000000000000000000000000000000656e64');
  const items = [
    await anchor.getBytes('0x0000000000000000000000000000000000000000000000000000000000000003'),
    await anchor.getBytes('0x0000000000000000000000000000000000000000000000000000000000000004'),
    await anchor.getBytes('0x0000000000000000000000000000000000000000000000000000000000000005'),
  ];

//   const rawStorage = await Promise.all([
//     ethers.provider.getStorageAt(anchorAddr, 0),
//     ethers.provider.getStorageAt(anchorAddr, 1),
//     ethers.provider.getStorageAt(anchorAddr, 2),
//     ethers.provider.getStorageAt(anchorAddr, 3),
//     ethers.provider.getStorageAt(anchorAddr, 4),
//   ]);

  console.log({lockKey, prefix, start, end, globalLock, items});
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
