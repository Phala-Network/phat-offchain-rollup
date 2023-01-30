import { ethers } from "hardhat";

async function main() {
  const TestOracle = await ethers.getContractFactory("TestOracle");
  const [deployer] = await ethers.getSigners();
  const oracle = await TestOracle.attach('0x5FbDB2315678afecb367f032d93F642f64180aa3');
  await Promise.all([
    oracle.deployed(),
  ])

  console.log('Pushing a request...');
  await oracle.connect(deployer).request("bitcoin/usd");
  console.log('Done');  
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
