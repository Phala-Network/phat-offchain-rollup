#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod sub0_factory {
    use alloc::{string::String, vec, vec::Vec};
    use ink::storage::traits::StorageLayout;
    use pink_extension::ResultExt;
    use scale::{Decode, Encode};

    #[ink(storage)]
    pub struct Sub0Factory {
        owner: AccountId,
        config: Option<Config>,
        deployments: Vec<Vec<u8>>,
        num_deployed: u32,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    struct Config {
        /// The RPC endpoint of the target blockchain
        rpc: String,
        /// The rollup anchor pallet id on the target blockchain
        pallet_id: u8,
        /// Key for submiting rollup transaction
        submit_key: [u8; 32],
        /// Code hash of the SubPriceFeed contract
        price_feed_code: Hash,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct Deployment {
        name: String,
        owner: AccountId,
        contract_id: AccountId,
        created_at: u64,
        expired_at: u64,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigured,
        InvalidKeyLength,
        FailedToDeployContract,
        FailedToConfigContract,
        FailedToTransferOwnership,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl Sub0Factory {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                owner: Self::env().caller(),
                config: Option::default(),
                deployments: vec![],
                num_deployed: 0,
            }
        }

        /// Configures the contract, only by the contract owner
        #[ink(message)]
        pub fn config(
            &mut self,
            rpc: String,
            pallet_id: u8,
            submit_key: Vec<u8>,
            price_feed_code: Hash,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config {
                rpc,
                pallet_id,
                submit_key: submit_key.try_into().or(Err(Error::InvalidKeyLength))?,
                price_feed_code,
            });
            Ok(())
        }

        /// Returns the current public config
        #[ink(message)]
        pub fn get_config(&self) -> Result<(u8, Hash)> {
            let config = self.ensure_configured()?.clone();
            Ok((config.pallet_id, config.price_feed_code))
        }

        /// Deploys a SubPriceFeed contract
        #[ink(message)]
        pub fn deploy_price_feed(
            &mut self,
            name: String,
            token0: String,
            token1: String,
        ) -> Result<AccountId> {
            use ink::ToAccountId;

            let config = self.ensure_configured()?.clone();
            let caller = self.env().caller();
            let mut deployed = sub_price_feed::SubPriceFeedRef::default()
                .code_hash(config.price_feed_code)
                .endowment(0)
                .salt_bytes(self.num_deployed.encode())
                .instantiate();

            deployed
                .config(
                    config.rpc.clone(),
                    config.pallet_id,
                    config.submit_key.to_vec(),
                    token0,
                    token1,
                )
                .log_err("failed to config SubPriceFeed")
                .or(Err(Error::FailedToConfigContract))?;
            deployed
                .transfer_ownership(caller)
                .log_err("failed to transfer ownership")
                .or(Err(Error::FailedToTransferOwnership))?;

            let created_at = self.env().block_timestamp();
            let expired_at = created_at + 3_600_000; // one hour
            self.deployments.push(
                Deployment {
                    name,
                    owner: caller,
                    contract_id: deployed.to_account_id(),
                    created_at,
                    expired_at,
                }
                .encode(),
            );

            self.num_deployed += 1;
            Ok(deployed.to_account_id())
        }

        /// Returns all the deployments
        #[ink(message)]
        pub fn get_deployments(&self) -> Result<Vec<Deployment>> {
            let deployments = self
                .deployments
                .iter()
                .map(|d| Decode::decode(&mut &d[..]).expect("canonical data; qed."))
                .collect();
            Ok(deployments)
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }

        /// Returns the config reference or raise the error `NotConfigured`
        fn ensure_configured(&self) -> Result<&Config> {
            self.config.as_ref().ok_or(Error::NotConfigured)
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use ink_env::call::FromAccountId;
        use sub_price_feed::SubPriceFeedRef;

        #[ink::test]
        fn it_works() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            // Mock code hash of contracts
            let hash1 = ink::primitives::Hash::try_from([10u8; 32]).unwrap();
            let hash2 = ink::primitives::Hash::try_from([20u8; 32]).unwrap();
            ink_env::test::register_contract::<Sub0Factory>(hash1.as_ref());
            ink_env::test::register_contract::<sub_price_feed::SubPriceFeed>(hash2.as_ref());
            let alice = AccountId::from([1u8; 32]);

            // Deploy the factory
            let mut factory = crate::sub0_factory::Sub0FactoryRef::default()
                .code_hash(hash1)
                .endowment(0)
                .salt_bytes([0u8; 0])
                .instantiate();

            factory
                .config(
                    "http://127.0.0.1:39933".to_string(),
                    100,
                    [1u8; 32].to_vec(),
                    hash2.clone(),
                )
                .expect("failed to config factory");

            // Deploy a new price feed
            let addr = factory
                .deploy_price_feed(
                    "myfeed".to_string(),
                    "polkadot".to_string(),
                    "usd".to_string(),
                )
                .expect("failed to deploy feed");

            // Can lookup the deployments
            let deployments = factory.get_deployments().unwrap();
            assert_eq!(
                deployments,
                vec![Deployment {
                    name: "myfeed".to_string(),
                    owner: alice.clone(),
                    contract_id: addr,
                    created_at: 0,
                    expired_at: 3600000,
                }]
            );

            // The ownership of the deployed price feed has been transferred from the factory
            // to Alice
            let feed: SubPriceFeedRef = FromAccountId::from_account_id(addr);
            assert_eq!(feed.owner(), alice);
        }
    }
}
