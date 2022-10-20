#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod evm_transator {
    use alloc::string::String;
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use phat_offchain_rollup::{
        clients::evm::write::AnchorTxClient, RollupHandler, RollupHandlerForwarder,
    };
    use primitive_types::H160;
    use scale::{Decode, Encode};

    #[ink(storage)]
    pub struct EvmTransactor {
        owner: AccountId,
        config: Option<Config>,
        /// If the key is banned forever, prevent the use of it in the future.
        retired: bool,
    }

    #[derive(Encode, Decode, Debug, PackedLayout, SpreadLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    struct Config {
        rollup_handler: AccountId,
        rpc: String,
        anchor: [u8; 20],
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigurated,
        KeyRetired,
        KeyNotRetiredYet,
        UpstreamFailed,
        BadAbi,
        FailedToGetStorage,
        FailedToDecodeStorage,
        FailedToEstimateGas,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl EvmTransactor {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                owner: Self::env().caller(),
                config: None,
                retired: false,
            }
        }

        /// Configures the transactor
        #[ink(message)]
        pub fn config(
            &mut self,
            rpc: String,
            rollup_handler: AccountId,
            anchor: H160,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config {
                rpc,
                rollup_handler,
                anchor: anchor.into(),
            });
            Ok(())
        }

        /// Returns the wallet address the transactor used to submit transactions
        #[ink(message)]
        pub fn wallet(&self) -> H160 {
            use pink_web3::signing::Key;
            Self::key_pair().address()
        }

        /// Retires the wallet to allow the owner to extract the key
        #[ink(message)]
        pub fn retire_wallet(&mut self) -> Result<()> {
            self.ensure_owner()?;
            self.retired = true;
            Ok(())
        }

        /// Extracts the retired secret key
        #[ink(message)]
        pub fn get_retired_secret_key(&self) -> Result<[u8; 32]> {
            self.ensure_owner()?;
            if !self.retired {
                return Err(Error::KeyNotRetiredYet);
            }
            // TODO: reveal the priv key when pink-web3 supports.
            // TODO: should we allow to renonce the key extraction?
            Ok([0u8; 32])
        }

        /// Called by a scheduler periodically
        #[ink(message)]
        pub fn poll(&self) -> Result<()> {
            if self.retired {
                return Err(Error::KeyRetired);
            }
            let Config {
                rollup_handler,
                rpc,
                anchor,
            } = self.config.as_ref().ok_or(Error::NotConfigurated)?;

            // Adhoc way to call another contract implementing a trait definition
            use ink_env::{call::FromAccountId, CallFlags};
            use ink_lang::codegen::TraitCallBuilder;
            use pink_extension::PinkEnvironment;
            let rollup_handler =
                RollupHandlerForwarder::<PinkEnvironment>::from_account_id(*rollup_handler);

            let result = rollup_handler
                .call()
                .handle_rollup()
                .call_flags(CallFlags::default())
                .fire()
                .expect("rollup upstream failed");
            let rollup = result.or(Err(Error::UpstreamFailed))?;

            // Only submit the tx if the results is not None
            let rollup = match rollup {
                Some(v) => v,
                None => return Ok(()),
            };

            #[cfg(feature = "std")]
            println!("RollupTx: {:#?}", rollup);

            // Connect to Ethereum RPC
            //
            // Note that currently we ignore `rollup.target` configuration because it's already
            // configured in the transactor.
            let contract = AnchorTxClient::connect(rpc, anchor.clone().into())
                .expect("FIXME: failed to connect to anchor");
            #[cfg(feature = "std")]
            println!("submitting rollup tx");

            // Submit to EVM
            let pair = Self::key_pair();
            let tx_id = contract
                .submit_rollup(rollup.tx, pair)
                .expect("FIXME: failed to submit rollup tx");
            #[cfg(feature = "std")]
            println!("submitted: {:?}", tx_id);

            // TODO: prevent redundant poll in a short period?
            Ok(())
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }

        /// Derives the key pair on the fly
        fn key_pair() -> pink_web3::keys::pink::KeyPair {
            pink_web3::keys::pink::KeyPair::derive_keypair(b"rollup-transactor")
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use ink::ToAccountId;
        use ink_lang as ink;

        fn consts() -> (String, Vec<u8>, H160, H160) {
            use std::env;
            dotenvy::dotenv().ok();
            /*
             Deployed {
                anchor: '0xb3083F961C729f1007a6A1265Ae6b97dC2Cc16f2',
                oracle: '0x8Bf50F8d0B62017c9B83341CB936797f6B6235dd'
            }
            */
            let rpc = env::var("RPC").unwrap();
            let key = hex::decode(env::var("PRIVKEY").unwrap()).expect("hex decode failed");
            let pubkey: [u8; 20] = hex::decode(env::var("PUBKEY").expect("env not found"))
                .expect("hex decode failed")
                .try_into()
                .expect("invald length");
            let pubkey: H160 = pubkey.into();
            let anchor_addr: [u8; 20] =
                hex::decode(env::var("ANCHOR_ADDR").expect("env not found"))
                    .expect("hex decode failed")
                    .try_into()
                    .expect("invald length");
            let anchor_addr: H160 = anchor_addr.into();
            (rpc, key, pubkey, anchor_addr)
        }

        #[ink::test]
        fn wallet_works() {
            pink_extension_runtime::mock_ext::mock_all_ext();
            let (_rpc, key, pubkey, _anchor_addr) = consts();
            pink_extension::chain_extension::mock::mock_derive_sr25519_key(move |_| key.clone());

            let hash1 = ink_env::Hash::try_from([10u8; 32]).unwrap();
            ink_env::test::register_contract::<EvmTransactor>(hash1.as_ref());

            // Deploy Transactor
            let mut transactor = EvmTransactorRef::default()
                .code_hash(hash1.clone())
                .endowment(0)
                .salt_bytes([0u8; 0])
                .instantiate()
                .expect("failed to deploy EvmTransactor");

            // Can reproduce the wallet address
            assert_eq!(transactor.wallet(), pubkey);

            // TODO: enable the test below when the adv test framework supports

            // Even the owner cannot read the secret key before retiring
            // assert_eq!(transactor.get_retired_secret_key(), Err(Error::KeyNotRetiredYet));

            // Owner can retire the wallet
            transactor.retire_wallet().unwrap();
            let sk = transactor.get_retired_secret_key().unwrap();
            assert_eq!(sk, [0u8; 32]);

            // Others cannot get the secret key
        }

        #[ink::test]
        fn it_works() {
            pink_extension_runtime::mock_ext::mock_all_ext();
            let (rpc, key, _pubkey, anchor_addr) = consts();
            pink_extension::chain_extension::mock::mock_derive_sr25519_key(move |_| key.clone());

            // Register contracts
            let hash1 = ink_env::Hash::try_from([10u8; 32]).unwrap();
            let hash2 = ink_env::Hash::try_from([20u8; 32]).unwrap();
            ink_env::test::register_contract::<EvmTransactor>(hash1.as_ref());
            ink_env::test::register_contract::<sample_oracle::SampleOracle>(hash2.as_ref());

            // Deploy Transactor
            let mut transactor = EvmTransactorRef::default()
                .code_hash(hash1.clone())
                .endowment(0)
                .salt_bytes([0u8; 0])
                .instantiate()
                .expect("failed to deploy EvmTransactor");

            // Deploy Oracle
            let mut oracle = ::sample_oracle::SampleOracleRef::default()
                .code_hash(hash2.clone())
                .endowment(0)
                .salt_bytes([0u8; 0])
                .instantiate()
                .expect("failed to deploy SampleOracle");

            // println!(
            //     "Contract deployed (transactor = {:?}, oracle = {:?})",
            //     transactor.to_account_id(),
            //     oracle.to_account_id(),
            // );

            // Setup transactor
            transactor
                .config(rpc.clone(), oracle.to_account_id(), anchor_addr)
                .unwrap();

            // Setup oracle
            oracle.config(rpc, anchor_addr).unwrap();

            // Call transactor
            transactor.poll().unwrap();
        }
    }
}
