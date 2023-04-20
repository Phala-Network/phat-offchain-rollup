import { ethers } from "hardhat";

async function main() {
  const TestOracle = await ethers.getContractFactory("TestOracle");

  const [deployer] = await ethers.getSigners();

  console.log('Deploying...');
  const attestor = deployer.address;  // When deploy for real e2e test, change it to the real attestor wallet.
  const oracle = await TestOracle.deploy(attestor);
  await oracle.deployed();
  console.log('Deployed', {
    oracle: oracle.address,
  });

  console.log('Configuring...');
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
