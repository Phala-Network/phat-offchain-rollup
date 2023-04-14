import "dotenv/config";
import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "hardhat-gas-reporter";

const config: HardhatUserConfig = {
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
  solidity: {
    version: "0.8.9",
    settings: {
      optimizer: {
        enabled: !!process.env.OPTIMIZE,
        runs: 300,
      }
    },
  },
  gasReporter: {
    currency: 'USD',
    gasPrice: 25,
    enabled: !!process.env.REPORT_GAS,
    coinmarketcap: process.env.COIN_MARKETCAP_API_KEY || "",
  }
};

export default config;
