import { ethers } from "hardhat";

async function main() {
  const Anchor = await ethers.getContractFactory("PhatRollupAnchor");
  const TestOracle = await ethers.getContractFactory("TestOracle");

  const [deployer] = await ethers.getSigners();

  console.log('Deploying...');
  const submitter = deployer.address;  // When deploy for real e2e test, change it to the real submitter wallet.
  const oracle = await TestOracle.deploy();
  const anchor = await Anchor.deploy(submitter, oracle.address, "0x712f"); // "q/"
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
  await oracle.connect(deployer).setAnchor(anchor.address);
  await oracle.connect(deployer).request("bitcoin/usd");
  await oracle.connect(deployer).registerFeed(0, "polkadot/usd");
  console.log('Done');  
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
