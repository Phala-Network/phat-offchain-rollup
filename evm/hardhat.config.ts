import "dotenv/config";
import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";

const config: HardhatUserConfig = {
  solidity: "0.8.17",
  networks: {
    goerli: {
      url: process.env['GOERLI_API'],
      accounts: [process.env['GOERLI_SK']!],
      chainId: 5,
    },
    moonbase: {
      url: 'https://rpc.api.moonbase.moonbeam.network',
      accounts: [process.env['GOERLI_SK']!],
      chainId: 1287,
    }
  },
  etherscan: {
    apiKey: process.env['ETHERSCAN_KEY'],
  },
};

export default config;
