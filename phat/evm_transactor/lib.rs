#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod evm_transator {
    use alloc::{string::String, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use phat_offchain_rollup::{RollupHandler, RollupHandlerForwarder, RollupTx};
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

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigurated,
        KeyRetired,
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
            let rollup = match rollup {
                Some(v) => v,
                None => return Ok(()),
            };

            #[cfg(feature = "std")]
            println!("RollupTx: {:#?}", rollup);

            // Connect to Ethereum RPC
            let contract = AnchorTxClient::connect(rpc, anchor.clone().into())?;
            #[cfg(feature = "std")]
            println!("submitting rollup tx");

            // Submit to EVM
            let pair = Self::key_pair();
            let tx_id = contract.submit_rollup(rollup.tx, pair)?;
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

    use pink_web3::contract::{Contract, Options};
    use pink_web3::transports::{resolve_ready, PinkHttp};
    use pink_web3::types::H160;
    use pink_web3::{
        api::{Eth, Namespace},
        keys::pink::KeyPair,
    };

    /// The client to submit transaction to the Evm anchor contract
    struct AnchorTxClient {
        contract: Contract<PinkHttp>,
    }

    impl AnchorTxClient {
        fn connect(rpc: &String, address: H160) -> Result<AnchorTxClient> {
            let eth = Eth::new(PinkHttp::new(rpc));
            let contract = Contract::from_json(eth, address, include_bytes!("res/anchor.abi.json"))
                .or(Err(Error::BadAbi))?;

            Ok(AnchorTxClient { contract })
        }

        fn submit_rollup(&self, tx: RollupTx, pair: KeyPair) -> Result<primitive_types::H256> {
            use ethabi::Token;
            use pink_web3::signing::Key;

            // Prepare rollupU256CondEq params
            let (cond_keys, cond_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = tx
                .conds
                .into_iter()
                .map(|cond| {
                    let phat_offchain_rollup::Cond::Eq(k, v) = cond;
                    (k.into(), v.map(Into::into).unwrap_or_default())
                })
                .unzip();
            let (update_keys, update_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = tx
                .updates
                .into_iter()
                .map(|(k, v)| (k.into(), v.map(Into::into).unwrap_or_default()))
                .unzip();
            let actions: Vec<Vec<u8>> = tx.actions.into_iter().map(Into::into).collect();
            let params = (
                Token::Array(cond_keys.into_iter().map(Token::Bytes).collect()),
                Token::Array(cond_values.into_iter().map(Token::Bytes).collect()),
                Token::Array(update_keys.into_iter().map(Token::Bytes).collect()),
                Token::Array(update_values.into_iter().map(Token::Bytes).collect()),
                Token::Array(actions.into_iter().map(Token::Bytes).collect()),
            );

            // Estiamte gas before submission
            let gas = resolve_ready(self.contract.estimate_gas(
                "rollupU256CondEq",
                params.clone(),
                pair.address(),
                Options::default(),
            ))
            .expect("FIXME: failed to estiamte gas");

            // Actually submit the tx (no guarantee for success)
            let tx_id = resolve_ready(self.contract.signed_call(
                "rollupU256CondEq",
                params,
                Options::with(|opt| opt.gas = Some(gas)),
                pair,
            ))
            .expect("FIXME: submit failed");
            Ok(tx_id)
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use hex_literal::hex;

        use ink::ToAccountId;
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[ink::test]
        fn it_works() {
            pink_extension_runtime::mock_ext::mock_all_ext();

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
            let rpc =
                "https://eth-goerli.g.alchemy.com/v2/68LXpUy0t0sLfZT2U-iYY5xh5OA8L6RV".to_string();
            let anchor_addr: H160 = hex!("b3083F961C729f1007a6A1265Ae6b97dC2Cc16f2").into();
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
