import { ethers } from "hardhat";

async function main() {
  const Anchor = await ethers.getContractFactory("PhatQueuedAnchor");
  const TestOracle = await ethers.getContractFactory("TestOracle");

  const [deployer] = await ethers.getSigners();

  console.log('Deploying...');
  const oracle = await TestOracle.deploy();
  const anchor = await Anchor.deploy(deployer.address, oracle.address, "0x71"); // Q
  await Promise.all([
    oracle.deployed(),
    anchor.deployed(),
  ])
  console.log('Deployed', {
    anchor: anchor.address,
    oracle: oracle.address,
  });

  console.log('Configuring...');
  await anchor.connect(deployer).transferOwnership(oracle.address);
  await oracle.connect(deployer).setQueuedAnchor(anchor.address);
  await oracle.connect(deployer).request("btc/usdt");
  console.log('Done');  
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
